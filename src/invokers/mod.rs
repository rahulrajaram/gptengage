//! CLI Invokers - Execute external LLM CLIs

pub mod base;
pub mod claude;
pub mod codex;
pub mod gemini;
pub mod plugin;

pub use base::*;
pub use claude::*;
pub use codex::*;
pub use gemini::*;
pub use plugin::*;

use crate::plugins::PluginManager;
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

/// Get an invoker by name.
///
/// Returns a built-in invoker for claude, codex, or gemini.
/// Falls back to checking plugins for custom CLIs.
pub fn get_invoker(name: &str) -> Option<Box<dyn Invoker>> {
    match name.to_lowercase().as_str() {
        "claude" => Some(Box::new(ClaudeInvoker::new())),
        "codex" => Some(Box::new(CodexInvoker::new())),
        "gemini" => Some(Box::new(GeminiInvoker::new())),
        _ => {
            // Check plugins
            let plugin_manager = PluginManager::new().ok()?;
            let config = plugin_manager.get_plugin(name)?.clone();
            Some(Box::new(PluginInvoker::new(config)))
        }
    }
}

/// Check if a CLI name is valid (built-in or plugin).
pub fn is_valid_cli(name: &str) -> bool {
    let builtin = ["claude", "codex", "gemini"];
    if builtin.contains(&name.to_lowercase().as_str()) {
        return true;
    }

    // Check plugins
    if let Ok(plugin_manager) = PluginManager::new() {
        return plugin_manager.has_plugin(name);
    }

    false
}
