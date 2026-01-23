//! CLI argument parsing and command dispatching

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gptengage")]
#[command(
    about = "Multi-AI CLI Orchestrator - Debate & Invoke across Claude Code, Codex, and Gemini",
    long_about = "Multi-AI CLI Orchestrator - Debate & Invoke across Claude Code, Codex, and Gemini

gptengage enables orchestration of multiple AI CLI tools (Claude Code, Codex, Gemini)
for debates, code reviews, and interactive sessions.

QUICK START:
    gptengage status                    Check available CLIs
    gptengage invoke claude \"Hello\"     Simple invocation
    gptengage debate \"Topic here\"       Multi-AI debate

EXAMPLES:
    # Check which AI CLIs are available
    gptengage status

    # Invoke a single AI
    gptengage invoke claude \"Explain this code\" --context-file src/main.rs

    # Run a debate between all available AIs
    gptengage debate \"Should we use microservices?\" --rounds 5

    # Run a debate with specific personas
    gptengage debate \"Tech stack\" -p \"claude:CTO,codex:Architect\"

    # Generate agent definitions for programmatic use
    gptengage generate-agents --topic \"API design\" --roles \"Lead,PM\" -o agents.json
    gptengage debate \"API design\" --agent-file agents.json

For detailed help on any command, use: gptengage <command> --help"
)]
#[command(version)]
#[command(author)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a multi-AI debate (Claude + Codex + Gemini)
    ///
    /// TERMINOLOGY NOTE:
    ///   --agent       Selects a CLI/model (claude, codex, gemini), not an AI agent.
    ///                 Use with --instances to spawn multiple participants of the same CLI.
    ///   --agent-file  Structured participant definitions (JSON) for AI agents consuming
    ///                 this tool programmatically. Generate with 'generate-agents' command.
    ///
    /// Examples:
    ///   # Default debate (Claude, Codex, Gemini without personas)
    ///   gptengage debate "Should we migrate to microservices?"
    ///
    ///   # Multi-instance: 3 Claude instances (leverages nondeterminism)
    ///   gptengage debate "Code review strategy" --agent claude --instances 3
    ///
    ///   # With personas (human-friendly format)
    ///   gptengage debate "Tech stack decision" -p "claude:CTO,claude:Architect,codex:Engineer"
    ///
    ///   # With agent definition file (for programmatic/agent use)
    ///   gptengage debate "API design strategy" --agent-file agents.json
    ///
    ///   # 5 rounds with JSON output
    ///   gptengage debate "REST vs GraphQL" --rounds 5 --output json
    #[command(verbatim_doc_comment)]
    Debate {
        /// The topic to debate
        topic: String,

        /// Select a single CLI to use for all participants
        ///
        /// Values: claude, codex, gemini
        ///
        /// Use with --instances to create multi-instance debates where the same
        /// LLM debates itself (leveraging nondeterminism and debate dynamics).
        ///
        /// Examples:
        ///   --agent claude --instances 3 (3 Claude instances)
        ///   --agent gemini --instances 2 (2 Gemini instances)
        ///
        /// Cannot be used with --participants or --agent-file
        #[arg(long, conflicts_with_all = ["participants", "agent_file"], verbatim_doc_comment)]
        agent: Option<String>,

        /// Number of instances to spawn when using --agent
        ///
        /// Creates multiple independent instances of the same CLI. Each instance
        /// will produce different outputs due to LLM nondeterminism and respond
        /// to other participants' inputs during the debate.
        ///
        /// Default: 3 when --agent is specified
        ///
        /// Example: --agent claude --instances 5
        ///
        /// Requires --agent to be specified
        #[arg(long, requires = "agent", verbatim_doc_comment)]
        instances: Option<usize>,

        /// Participants in format: cli:persona,cli:persona
        ///
        /// Format: "cli:persona,cli:persona,..."
        /// Examples:
        ///   -p "claude:CEO,claude:Architect,codex:PM"
        ///   -p "claude:Security Expert,gemini:UX Designer"
        ///
        /// Cannot be used with --agent-file or --agent
        #[arg(long, short = 'p', conflicts_with_all = ["agent_file", "agent"], verbatim_doc_comment)]
        participants: Option<String>,

        /// Path to agent definition file (JSON) with full agent specifications
        ///
        /// Agent files require structured definitions with persona, instructions,
        /// expertise, and communication_style. Use 'generate-agents' to create.
        ///
        /// Example: --agent-file agents.json
        ///
        /// Cannot be used with --participants or --agent
        #[arg(long, conflicts_with_all = ["participants", "agent"], verbatim_doc_comment)]
        agent_file: Option<String>,

        /// Number of debate rounds (default: 3)
        #[arg(long, short = 'r', default_value = "3")]
        rounds: usize,

        /// Output format: text, json, markdown
        #[arg(long, short = 'o', default_value = "text")]
        output: String,

        /// Timeout per CLI invocation in seconds
        ///
        /// The CLI process is terminated if it exceeds this duration.
        /// Default: 120 seconds (2 minutes)
        #[arg(long, short = 't', default_value = "120", verbatim_doc_comment)]
        timeout: u64,

        /// Allow write access within the current directory (default: read-only)
        #[arg(long, verbatim_doc_comment)]
        write: bool,
    },

    /// Invoke a specific CLI with a prompt
    ///
    /// Examples:
    ///   # Simple invocation
    ///   gptengage invoke claude "Explain quantum computing"
    ///
    ///   # With session (maintains conversation history)
    ///   gptengage invoke claude "Review my auth code" --session auth-review
    ///   gptengage invoke claude "Fix the JWT bug" --session auth-review
    ///
    ///   # With context file
    ///   gptengage invoke claude "Review this code" --context-file src/main.rs
    ///
    ///   # With custom topic and timeout
    ///   gptengage invoke gemini "Complex analysis" --session analysis --topic "Performance Review" --timeout 180
    #[command(verbatim_doc_comment)]
    Invoke {
        /// Which CLI to invoke: claude, codex, or gemini
        cli: String,

        /// The prompt to send
        prompt: String,

        /// Session name for persistent conversation
        ///
        /// Sessions maintain full conversation history. Each turn is
        /// injected into subsequent prompts for context continuity.
        ///
        /// Example: --session my-session
        #[arg(long, short = 's', verbatim_doc_comment)]
        session: Option<String>,

        /// Session topic description
        ///
        /// Auto-generated from first prompt if omitted.
        /// Example: --topic "Code review session"
        #[arg(long, verbatim_doc_comment)]
        topic: Option<String>,

        /// File to include as context
        ///
        /// File contents are prepended to the prompt.
        /// Example: --context-file src/auth.rs
        #[arg(long, short = 'c', verbatim_doc_comment)]
        context_file: Option<String>,

        /// Timeout in seconds
        ///
        /// The CLI process is terminated if it exceeds this duration.
        /// Default: 120 seconds (2 minutes)
        #[arg(long, short = 't', default_value = "120", verbatim_doc_comment)]
        timeout: u64,

        /// Allow write access within the current directory (default: read-only)
        #[arg(long, verbatim_doc_comment)]
        write: bool,
    },

    /// Manage sessions
    #[command(subcommand)]
    Session(SessionCommands),

    /// Show status of detected CLIs and active sessions
    Status,

    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Generate agent definitions for debate participants
    ///
    /// Uses AI to create detailed agent definitions with personas, instructions,
    /// expertise areas, and communication styles. Validates all fields before saving.
    ///
    /// Examples:
    ///   # Generate 3 agents for a microservices debate
    ///   gptengage generate-agents \
    ///     --topic "Should we migrate to microservices?" \
    ///     --roles "CEO,Principal Architect,Product Manager" \
    ///     --output agents.json
    ///
    ///   # Use Codex instead of Claude for generation
    ///   gptengage generate-agents \
    ///     --topic "API design strategy" \
    ///     --roles "Backend Lead,Frontend Lead,DBA" \
    ///     --output api-agents.json \
    ///     --use-cli codex
    ///
    ///   # Then use the generated file in a debate
    ///   gptengage debate "Should we migrate to microservices?" --agent-file agents.json
    ///
    /// Generated file format (schema version 1.0):
    ///   {
    ///     "schema_version": "1.0",
    ///     "generated_by": "gptengage-claude",
    ///     "participants": [
    ///       {
    ///         "cli": "claude",
    ///         "persona": "CEO",
    ///         "instructions": "Focus on business impact, ROI, and strategic alignment...",
    ///         "expertise": ["business strategy", "finance", "leadership"],
    ///         "communication_style": "Executive - concise and action-oriented"
    ///       }
    ///     ]
    ///   }
    #[command(verbatim_doc_comment)]
    GenerateAgents {
        /// The debate topic (used to generate relevant agent personas)
        ///
        /// This helps the AI create contextually appropriate agent definitions.
        /// Example: --topic "Should we adopt Kubernetes?"
        #[arg(long, verbatim_doc_comment)]
        topic: String,

        /// Comma-separated list of roles to generate
        ///
        /// Format: "Role1,Role2,Role3"
        /// Examples:
        ///   --roles "CEO,CTO,CFO"
        ///   --roles "Senior Engineer,Product Manager,Designer"
        ///   --roles "Security Expert,Compliance Officer,Legal Counsel"
        #[arg(long, verbatim_doc_comment)]
        roles: String,

        /// Output file path for generated agent definitions
        ///
        /// File will contain validated JSON with schema version 1.0
        /// Example: --output agents.json
        #[arg(long, short = 'o', verbatim_doc_comment)]
        output: String,

        /// CLI to use for generation (default: claude)
        ///
        /// Available: claude, codex, gemini
        /// Example: --use-cli codex
        #[arg(long, default_value = "claude", verbatim_doc_comment)]
        use_cli: String,

        /// Timeout in seconds
        ///
        /// The CLI process is terminated if it exceeds this duration.
        /// Default: 120 seconds (2 minutes)
        #[arg(long, short = 't', default_value = "120", verbatim_doc_comment)]
        timeout: u64,

        /// Allow write access within the current directory (default: read-only)
        #[arg(long, verbatim_doc_comment)]
        write: bool,
    },
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
        use crate::invokers::AccessMode;

        match self.command {
            Commands::Debate {
                topic,
                agent,
                instances,
                participants,
                agent_file,
                rounds,
                output,
                timeout,
                write,
            } => {
                debate::run_debate(debate::DebateOptions {
                    topic,
                    agent,
                    instances,
                    participants,
                    agent_file,
                    rounds,
                    output,
                    timeout,
                    access_mode: AccessMode::from_write_flag(write),
                })
                .await
            }

            Commands::Invoke {
                cli,
                prompt,
                session,
                topic,
                context_file,
                timeout,
                write,
            } => {
                invoke::run_invoke(
                    cli,
                    prompt,
                    session,
                    topic,
                    context_file,
                    timeout,
                    AccessMode::from_write_flag(write),
                )
                .await
            }

            Commands::Session(session_cmd) => match session_cmd {
                SessionCommands::List => session::list_sessions().await,
                SessionCommands::Show { name } => session::show_session(name).await,
                SessionCommands::End { name, all } => session::end_session(name, all).await,
            },

            Commands::Status => status::show_status().await,

            Commands::Config(config_cmd) => match config_cmd {
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
            },

            Commands::GenerateAgents {
                topic,
                roles,
                output,
                use_cli,
                timeout,
                write,
            } => {
                generate_agents::run_generate_agents(
                    topic,
                    roles,
                    output,
                    use_cli,
                    timeout,
                    AccessMode::from_write_flag(write),
                )
                .await
            }
        }
    }
}
