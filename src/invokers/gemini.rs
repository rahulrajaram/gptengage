//! Gemini CLI invoker

use super::base::{command_exists, execute_command};
use super::{AccessMode, Invoker};
use async_trait::async_trait;

#[derive(Clone)]
pub struct GeminiInvoker;

#[async_trait]
impl Invoker for GeminiInvoker {
    async fn invoke(
        &self,
        prompt: &str,
        timeout: u64,
        access_mode: AccessMode,
    ) -> anyhow::Result<String> {
        let args = match access_mode {
            AccessMode::ReadOnly => &["--sandbox", "--include-directories", "."][..],
            AccessMode::WorkspaceWrite => &[
                "--sandbox",
                "--include-directories",
                ".",
                "--approval-mode",
                "auto_edit",
            ][..],
        };

        execute_command("gemini", args, prompt, timeout).await
    }

    fn name(&self) -> &str {
        "gemini"
    }

    fn is_available(&self) -> bool {
        command_exists("gemini")
    }
}

impl Default for GeminiInvoker {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiInvoker {
    pub fn new() -> Self {
        GeminiInvoker
    }
}
