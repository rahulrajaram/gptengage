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
        model: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut args: Vec<&str> = Vec::new();

        // Add model if specified
        // Example models: gemini-2.5-pro, gemini-2.0-flash
        if let Some(m) = model {
            args.push("--model");
            args.push(m);
        }

        // Add access mode flags
        match access_mode {
            AccessMode::ReadOnly => {
                args.extend_from_slice(&["--sandbox", "--include-directories", "."]);
            }
            AccessMode::WorkspaceWrite => {
                args.extend_from_slice(&[
                    "--sandbox",
                    "--include-directories",
                    ".",
                    "--approval-mode",
                    "auto_edit",
                ]);
            }
        };

        execute_command("gemini", &args, prompt, timeout).await
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
