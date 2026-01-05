//! CLI argument parsing and command dispatching

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gptengage")]
#[command(about = "Multi-AI CLI Orchestrator - Debate & Invoke across Claude Code, Codex, and Gemini")]
#[command(version)]
#[command(author)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a multi-AI debate (Claude + Codex + Gemini)
    Debate {
        /// The topic to debate
        topic: String,

        /// Number of debate rounds (default: 3)
        #[arg(long, default_value = "3")]
        rounds: usize,

        /// Output format: text, json, markdown
        #[arg(long, default_value = "text")]
        output: String,

        /// Timeout per CLI invocation in seconds
        #[arg(long, default_value = "120")]
        timeout: u64,
    },

    /// Invoke a specific CLI with a prompt
    Invoke {
        /// Which CLI to invoke: claude, codex, or gemini
        cli: String,

        /// The prompt to send
        prompt: String,

        /// Session name for persistent conversation
        #[arg(long)]
        session: Option<String>,

        /// Session topic description
        #[arg(long)]
        topic: Option<String>,

        /// File to include as context
        #[arg(long)]
        context_file: Option<String>,

        /// Timeout in seconds
        #[arg(long, default_value = "120")]
        timeout: u64,
    },

    /// Manage sessions
    #[command(subcommand)]
    Session(SessionCommands),

    /// Show status of detected CLIs and active sessions
    Status,

    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand)]
pub enum SessionCommands {
    /// List all active sessions
    List,

    /// Show session details and history
    Show {
        /// Session name
        name: String,
    },

    /// End a session
    End {
        /// Session name (or --all for all sessions)
        name: Option<String>,

        /// End all sessions
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Get a config value
    Get {
        /// Config key
        key: String,
    },

    /// Set a config value
    Set {
        /// Config key
        key: String,

        /// Config value
        value: String,
    },

    /// List all configuration
    List,
}

impl Cli {
    pub async fn execute(self) -> anyhow::Result<()> {
        use crate::commands::*;

        match self.command {
            Commands::Debate {
                topic,
                rounds,
                output,
                timeout,
            } => debate::run_debate(topic, rounds, output, timeout).await,

            Commands::Invoke {
                cli,
                prompt,
                session,
                topic,
                context_file,
                timeout,
            } => invoke::run_invoke(cli, prompt, session, topic, context_file, timeout).await,

            Commands::Session(session_cmd) => {
                match session_cmd {
                    SessionCommands::List => session::list_sessions().await,
                    SessionCommands::Show { name } => session::show_session(name).await,
                    SessionCommands::End { name, all } => session::end_session(name, all).await,
                }
            }

            Commands::Status => status::show_status().await,

            Commands::Config(config_cmd) => {
                match config_cmd {
                    ConfigCommands::Get { key } => {
                        let config = crate::config::ConfigManager::new()?;
                        match config.get(&key) {
                            Some(value) => println!("{}: {}", key, value),
                            None => println!("Config key '{}' not found", key),
                        }
                        Ok(())
                    }
                    ConfigCommands::Set { key, value } => {
                        let mut config = crate::config::ConfigManager::new()?;
                        config.set(&key, &value)?;
                        println!("Set {} = {}", key, value);
                        Ok(())
                    }
                    ConfigCommands::List => {
                        let config = crate::config::ConfigManager::new()?;
                        println!("Configuration:");
                        println!("  default_timeout: {}", config.default_timeout);
                        println!("  default_debate_rounds: {}", config.default_debate_rounds);
                        Ok(())
                    }
                }
            }
        }
    }
}
