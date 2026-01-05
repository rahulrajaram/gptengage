//! Codex CLI invoker

use super::base::{command_exists, execute_command};
use super::Invoker;
use async_trait::async_trait;

#[derive(Clone)]
pub struct CodexInvoker;

#[async_trait]
impl Invoker for CodexInvoker {
    async fn invoke(&self, prompt: &str, timeout: u64) -> anyhow::Result<String> {
        execute_command("codex", &["exec", "--full-auto"], prompt, timeout).await
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
