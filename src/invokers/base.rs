//! Base invoker implementation with common utilities

use std::process::{Command, Stdio};
use anyhow::Result;

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

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(input.as_bytes()).await?;
    }

    // Wait for completion with timeout
    let timeout_duration = std::time::Duration::from_secs(timeout);
    match tokio::time::timeout(timeout_duration, async {
        child.wait_with_output().await
    }).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                Ok(stdout)
            } else {
                Err(anyhow::anyhow!("Command failed: {}", stderr))
            }
        }
        Ok(Err(e)) => Err(e.into()),
        Err(_) => {
            Err(anyhow::anyhow!("Command timed out after {} seconds", timeout))
        }
    }
}

/// Check if a command exists in PATH
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which").arg(cmd).output().ok().map_or(false, |o| o.status.success())
}
