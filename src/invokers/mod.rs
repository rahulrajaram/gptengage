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

/// Access mode for invoked CLIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// Read-only access within the current directory.
    ReadOnly,
    /// Write access within the current directory.
    WorkspaceWrite,
}

impl AccessMode {
    pub fn from_write_flag(write: bool) -> Self {
        if write {
            AccessMode::WorkspaceWrite
        } else {
            AccessMode::ReadOnly
        }
    }
}

/// Trait for CLI invokers
#[async_trait]
pub trait Invoker: Send + Sync {
    /// Invoke the CLI with the given prompt
    async fn invoke(
        &self,
        prompt: &str,
        timeout: u64,
        access_mode: AccessMode,
    ) -> anyhow::Result<String>;

    /// Get the CLI name
    fn name(&self) -> &str;

    /// Check if the CLI is available
    fn is_available(&self) -> bool;
}
