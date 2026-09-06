//! Shared process supervision and observable transport outcomes.
use super::{AccessMode, InvocationOutcome, InvocationReport};
use anyhow::Result;
use std::process::{Command, Stdio};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

const CLAUDE_NESTING_ENV_VARS: &[&str] = &["CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT"];
const OUTPUT_LIMIT: usize = 16 * 1024 * 1024;

struct ProcessGroup(Option<u32>);
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.0 {
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
    }
}

async fn capture(mut stream: impl AsyncRead + Unpin) -> std::io::Result<(String, bool)> {
    let mut output = Vec::new();
    let mut truncated = false;
    let mut chunk = [0; 8192];
    loop {
        let count = stream.read(&mut chunk).await?;
        if count == 0 {
            break;
        }
        let retained = count.min(OUTPUT_LIMIT.saturating_sub(output.len()));
        output.extend_from_slice(&chunk[..retained]);
        truncated |= retained < count;
    }
    Ok((String::from_utf8_lossy(&output).into_owned(), truncated))
}

/// Execute with a typed process outcome. Provider completion/identity remain unknown.
pub async fn execute_command_report(
    cmd: &str,
    args: &[&str],
    input: &str,
    timeout: u64,
    report: InvocationReport,
) -> InvocationReport {
    let started = std::time::Instant::now();
    let mut command = tokio::process::Command::new(cmd);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    for var in CLAUDE_NESTING_ENV_VARS {
        command.env_remove(var);
    }
    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return InvocationReport {
                outcome: InvocationOutcome::SpawnFailed,
                diagnostic: Some(error.to_string()),
                ..report
            }
        }
    };
    let group = ProcessGroup(child.id());
    let mut stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let input = input.as_bytes().to_vec();
    // All pipe futures live within this scope, so cancellation cannot detach them.
    let execution = async {
        // Own stdin so completing the writer closes the pipe. ChildStdin::shutdown
        // alone does not drop the handle, and readers such as cat require EOF.
        let write = async move {
            stdin.write_all(&input).await?;
            stdin.shutdown().await
        };
        let (status, stdout, stderr, ()) =
            tokio::try_join!(child.wait(), capture(stdout), capture(stderr), write)?;
        Ok::<_, std::io::Error>((status, stdout, stderr))
    };
    let result = tokio::time::timeout(std::time::Duration::from_secs(timeout), execution).await;
    // Terminate remaining descendants on every exit, including successful parent exit.
    drop(group);
    let elapsed_ms = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
    match result {
        Ok(Ok((status, (stdout, out_cut), (stderr, err_cut)))) => InvocationReport {
            outcome: if !status.success() {
                InvocationOutcome::Failed
            } else if out_cut {
                InvocationOutcome::IoFailed
            } else {
                InvocationOutcome::Succeeded
            },
            stdout,
            stderr,
            exit_code: status.code(),
            elapsed_ms,
            diagnostic: if out_cut || err_cut {
                Some("Captured output truncated at 16 MiB per stream".into())
            } else if !status.success() {
                Some(format!("Command failed with status {status}"))
            } else {
                None
            },
            ..report
        },
        failed => {
            // kill() waits for the direct child; bound cleanup separately from execution.
            let cleanup =
                tokio::time::timeout(std::time::Duration::from_secs(1), child.kill()).await;
            let (outcome, reason) = match failed {
                Err(_) => (
                    InvocationOutcome::TimedOut,
                    format!("Command timed out after {timeout} seconds"),
                ),
                Ok(Err(error)) => (InvocationOutcome::IoFailed, error.to_string()),
                Ok(Ok(_)) => unreachable!(),
            };
            InvocationReport {
                outcome,
                elapsed_ms,
                diagnostic: Some(format!(
                    "{reason}; partial output unavailable{}",
                    if cleanup.is_err() {
                        "; cleanup deadline exceeded"
                    } else {
                        ""
                    }
                )),
                ..report
            }
        }
    }
}

pub async fn execute_command(
    cmd: &str,
    args: &[&str],
    input: &str,
    timeout: u64,
) -> Result<String> {
    execute_command_report(
        cmd,
        args,
        input,
        timeout,
        InvocationReport::unknown(cmd, None, AccessMode::ReadOnly),
    )
    .await
    .into_text()
}

/// Check if a command exists in PATH
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .ok()
        .is_some_and(|o| o.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_command_basic() {
        let result = execute_command("echo", &["hello world"], "", 5).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello world");
    }

    #[tokio::test]
    async fn test_execute_command_timeout() {
        let result = execute_command("sleep", &["30"], "", 1).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("timed out"),
            "expected timeout error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_claude_env_vars_not_inherited() {
        // Set the nesting env vars in our process
        std::env::set_var("CLAUDECODE", "1");
        std::env::set_var("CLAUDE_CODE_ENTRYPOINT", "cli");

        let result = execute_command("env", &[], "", 5).await;
        assert!(result.is_ok());
        let output = result.unwrap();

        // The child should NOT see these variables
        for line in output.lines() {
            assert!(
                !line.starts_with("CLAUDECODE="),
                "CLAUDECODE should be stripped from child env"
            );
            assert!(
                !line.starts_with("CLAUDE_CODE_ENTRYPOINT="),
                "CLAUDE_CODE_ENTRYPOINT should be stripped from child env"
            );
        }

        // Clean up
        std::env::remove_var("CLAUDECODE");
        std::env::remove_var("CLAUDE_CODE_ENTRYPOINT");
    }
    #[tokio::test]
    async fn report_keeps_exit_code_and_both_streams() {
        let report = execute_command_report(
            "sh",
            &["-c", "printf answer; printf diagnosis >&2; exit 7"],
            "",
            5,
            InvocationReport::unknown("fixture", None, AccessMode::ReadOnly),
        )
        .await;
        assert_eq!(report.outcome, InvocationOutcome::Failed);
        assert_eq!(report.exit_code, Some(7));
        assert_eq!(report.stdout, "answer");
        assert_eq!(report.stderr, "diagnosis");
        assert!(report.effective_model.is_none());
        assert!(report.completion.is_none());
        assert!(report.usage.is_none());
    }

    #[tokio::test]
    async fn report_distinguishes_spawn_failure() {
        let report = execute_command_report(
            "/nonexistent/gptengage-fixture",
            &[],
            "",
            5,
            InvocationReport::unknown("fixture", None, AccessMode::ReadOnly),
        )
        .await;
        assert_eq!(report.outcome, InvocationOutcome::SpawnFailed);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn report_timeout_stops_descendant_before_side_effect() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("survived");
        let path = marker.to_str().unwrap();
        let report = execute_command_report(
            "sh",
            &["-c", "(sleep 2; touch \"$1\") & wait", "fixture", path],
            "",
            1,
            InvocationReport::unknown("fixture", None, AccessMode::ReadOnly),
        )
        .await;
        assert_eq!(report.outcome, InvocationOutcome::TimedOut);
        tokio::time::sleep(std::time::Duration::from_millis(1300)).await;
        assert!(!marker.exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn early_stdin_close_is_bounded_and_visible() {
        let report = execute_command_report(
            "sh",
            &["-c", "exec 0<&-; sleep 10"],
            &"x".repeat(256 * 1024),
            1,
            InvocationReport::unknown("fixture", None, AccessMode::ReadOnly),
        )
        .await;
        assert!(matches!(
            report.outcome,
            InvocationOutcome::IoFailed | InvocationOutcome::TimedOut
        ));
    }
}
