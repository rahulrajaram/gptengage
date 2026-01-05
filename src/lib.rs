//! GPT Engage - Multi-AI CLI Orchestrator
//!
//! A standalone CLI tool that orchestrates multiple LLM CLIs (Claude Code, Codex, Gemini)
//! without modifying their configuration directories.

pub mod cli;
pub mod commands;
pub mod config;
pub mod invokers;
pub mod orchestrator;
pub mod session;
pub mod utils;

pub use cli::Cli;
pub use commands::*;
pub use config::*;
pub use invokers::*;
pub use session::*;

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
