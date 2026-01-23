//! Codex CLI invoker

use super::base::{command_exists, execute_command};
use super::{AccessMode, Invoker};
use async_trait::async_trait;

#[derive(Clone)]
pub struct CodexInvoker;

#[async_trait]
impl Invoker for CodexInvoker {
    async fn invoke(
        &self,
        prompt: &str,
        timeout: u64,
        access_mode: AccessMode,
    ) -> anyhow::Result<String> {
        let args = match access_mode {
            AccessMode::ReadOnly => &["exec", "--sandbox", "read-only", "--cd", "."][..],
            AccessMode::WorkspaceWrite => {
                &["exec", "--sandbox", "workspace-write", "--cd", "."][..]
            }
        };

        execute_command("codex", args, prompt, timeout).await
    }

    fn name(&self) -> &str {
        "codex"
    }

    fn is_available(&self) -> bool {
        command_exists("codex")
    }
}

impl Default for CodexInvoker {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexInvoker {
    pub fn new() -> Self {
        CodexInvoker
    }
}
