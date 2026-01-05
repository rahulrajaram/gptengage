//! CLI Invokers - Execute external LLM CLIs

pub mod base;
pub mod claude;
pub mod codex;
pub mod gemini;

pub use base::*;
pub use claude::*;
pub use codex::*;
pub use gemini::*;

use async_trait::async_trait;

/// Trait for CLI invokers
#[async_trait]
pub trait Invoker: Send + Sync {
    /// Invoke the CLI with the given prompt
    async fn invoke(&self, prompt: &str, timeout: u64) -> anyhow::Result<String>;

    /// Get the CLI name
    fn name(&self) -> &str;

    /// Check if the CLI is available
    fn is_available(&self) -> bool;
}
