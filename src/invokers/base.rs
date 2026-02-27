//! Base invoker implementation with common utilities

use anyhow::Result;
use std::process::{Command, Stdio};

/// Environment variables that Claude Code sets to detect nesting.
/// We strip these so child processes (e.g. `claude -p`) don't think
/// they're running inside another Claude instance.
const CLAUDE_NESTING_ENV_VARS: &[&str] = &["CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT"];

/// Execute a command with timeout
pub async fn execute_command(
    cmd: &str,
    args: &[&str],
    input: &str,
    timeout: u64,
) -> Result<String> {
    let mut command = tokio::process::Command::new(cmd);
    command.args(args);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    // Strip Claude nesting env vars so child processes can invoke Claude CLI
    for var in CLAUDE_NESTING_ENV_VARS {
        command.env_remove(var);
    }

    // Create a new session so the child is a process group leader,
    // isolated from the parent's terminal. This also lets us kill
    // the entire process group on timeout.
    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    let mut child = command.spawn()?;

    // Save the PID before child is consumed by wait_with_output
    let pid = child.id();

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(input.as_bytes()).await?;
    }

    // Wait for completion with timeout
    let timeout_duration = std::time::Duration::from_secs(timeout);

    tokio::select! {
        result = child.wait_with_output() => {
            match result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                    if output.status.success() {
                        Ok(stdout)
                    } else {
                        Err(anyhow::anyhow!("Command failed: {}", stderr))
                    }
                }
                Err(e) => Err(e.into()),
            }
        }
        _ = tokio::time::sleep(timeout_duration) => {
            // Kill the entire process group (negative PID) to prevent
            // orphaned child processes (e.g. claude spawning node, etc.)
            if let Some(child_pid) = pid {
                #[cfg(unix)]
                {
                    let pgid = -(child_pid as i32);
                    unsafe {
                        libc::kill(pgid, libc::SIGTERM);
                        // Brief pause then SIGKILL to ensure cleanup
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        libc::kill(pgid, libc::SIGKILL);
                    }
                }
                #[cfg(windows)]
                {
                    let _ = child_pid;
                    // TODO(windows): Process termination is not implemented on Windows.
                    // The child process will NOT be killed on timeout, which may leave
                    // orphaned AI CLI processes. A proper fix requires either the
                    // `windows-sys` crate (TerminateProcess) or Tokio's Child::kill().
                }
            }
            Err(anyhow::anyhow!("Command timed out after {} seconds", timeout))
        }
    }
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
}
