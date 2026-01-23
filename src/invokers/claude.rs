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
    ) -> anyhow::Result<String> {
        let args = match access_mode {
            AccessMode::ReadOnly => &["-p", "--tools", "Read", "--allowed-tools", "Read"][..],
            AccessMode::WorkspaceWrite => {
                &["-p", "--tools", "Read,Edit", "--allowed-tools", "Read,Edit"][..]
            }
        };

        execute_command("claude", args, prompt, timeout).await
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
