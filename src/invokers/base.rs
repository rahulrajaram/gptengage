//! Base invoker implementation with common utilities

use anyhow::Result;
use std::process::{Command, Stdio};

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
            // Kill the child process by PID to prevent resource leak
            if let Some(child_pid) = pid {
                #[cfg(unix)]
                {
                    unsafe {
                        libc::kill(child_pid as i32, libc::SIGTERM);
                    }
                }
                #[cfg(windows)]
                {
                    // Windows termination would require additional crate dependencies
                    // For now, just log the timeout
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
