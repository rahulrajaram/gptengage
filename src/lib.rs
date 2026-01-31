//! GPT Engage - Multi-AI CLI Orchestrator
//!
//! A standalone CLI tool that orchestrates multiple LLM CLIs (Claude Code, Codex, Gemini)
//! without modifying their configuration directories.

pub mod cli;
pub mod commands;
pub mod config;
pub mod invokers;
pub mod orchestrator;
pub mod plugins;
pub mod session;
pub mod templates;
pub mod utils;

pub use cli::Cli;
pub use config::*;
pub use session::*;

// Re-export key types explicitly to avoid ambiguous glob re-exports
pub use commands::{debate, generate_agents, invoke, status};
pub use invokers::{
    get_invoker, is_valid_cli, AccessMode, ClaudeInvoker, CodexInvoker, GeminiInvoker, Invoker,
};

#[derive(Debug)]
pub struct GptEngage {
    pub config: ConfigManager,
    pub session_manager: SessionManager,
}

impl GptEngage {
    pub async fn new() -> anyhow::Result<Self> {
        let config = ConfigManager::new()?;
        let session_manager = SessionManager::new()?;

        Ok(Self {
            config,
            session_manager,
        })
    }
}
