//! CLI Invokers - Execute external LLM CLIs

pub mod result;
pub use result::*;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
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
        model: Option<&str>,
    ) -> anyhow::Result<String>;

    /// Validate locally known model-selection constraints without launching a process.
    /// Passing validation proves forwarding support, not provider acceptance or identity.
    fn validate_model(&self, model: Option<&str>) -> anyhow::Result<()> {
        if model.is_some_and(|model| model.trim().is_empty()) {
            anyhow::bail!("Explicit model must not be empty");
        }
        Ok(())
    }

    /// Observable report; legacy implementors expose only text success/failure.
    async fn invoke_report(
        &self,
        prompt: &str,
        timeout: u64,
        access_mode: AccessMode,
        model: Option<&str>,
    ) -> InvocationReport {
        let report = InvocationReport::unknown(self.name(), model, access_mode);
        if let Err(error) = self.validate_model(model) {
            return InvocationReport {
                outcome: InvocationOutcome::Rejected,
                diagnostic: Some(error.to_string()),
                ..report
            };
        }
        match self.invoke(prompt, timeout, access_mode, model).await {
            Ok(stdout) => InvocationReport {
                outcome: InvocationOutcome::Succeeded,
                stdout,
                ..report
            },
            Err(error) => InvocationReport {
                outcome: InvocationOutcome::Failed,
                diagnostic: Some(error.to_string()),
                ..report
            },
        }
    }

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
