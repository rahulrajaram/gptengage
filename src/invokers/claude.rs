//! Claude Code CLI invoker

use super::base::{command_exists, execute_command};
use super::{AccessMode, Invoker};
use async_trait::async_trait;

#[derive(Clone)]
pub struct ClaudeInvoker;

#[async_trait]
impl Invoker for ClaudeInvoker {
    async fn invoke(
        &self,
        prompt: &str,
        timeout: u64,
        access_mode: AccessMode,
        model: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut args: Vec<&str> = vec!["-p"];

        // Add model if specified
        // Example models: claude-sonnet-4-20250514, claude-opus-4-20250514
        if let Some(m) = model {
            args.push("--model");
            args.push(m);
        }

        // Add access mode flags
        match access_mode {
            AccessMode::ReadOnly => {
                args.extend_from_slice(&["--tools", "Read", "--allowed-tools", "Read"]);
            }
            AccessMode::WorkspaceWrite => {
                args.extend_from_slice(&["--tools", "Read,Edit", "--allowed-tools", "Read,Edit"]);
            }
        };

        execute_command("claude", &args, prompt, timeout).await
    }

    fn name(&self) -> &str {
        "claude"
    }

    fn is_available(&self) -> bool {
        command_exists("claude")
    }
}

impl Default for ClaudeInvoker {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeInvoker {
    pub fn new() -> Self {
        ClaudeInvoker
    }
}
