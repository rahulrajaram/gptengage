//! Status command - Show detected CLIs and active sessions

use crate::config::ConfigManager;
use crate::invokers::{ClaudeInvoker, CodexInvoker, GeminiInvoker, Invoker};
use crate::session::SessionManager;

/// Show status of detected CLIs and active sessions
pub async fn show_status() -> anyhow::Result<()> {
    println!("GPT Engage v{}", env!("CARGO_PKG_VERSION"));
    println!();

    // Detect available CLIs
    let claude = ClaudeInvoker::new();
    let codex = CodexInvoker::new();
    let gemini = GeminiInvoker::new();

    println!("Detected LLM CLIs:");
    if claude.is_available() {
        println!("  ✓ {} (Claude Code)", claude.name());
    } else {
        println!("  ✗ claude (not found in PATH)");
    }
    if codex.is_available() {
        println!("  ✓ {} (Codex CLI)", codex.name());
    } else {
        println!("  ✗ codex (not found in PATH)");
    }
    if gemini.is_available() {
        println!("  ✓ {} (Gemini CLI)", gemini.name());
    } else {
        println!("  ✗ gemini (not found in PATH)");
    }
    println!();

    // Show configuration
    let config = ConfigManager::new()?;
    println!("Configuration:");
    println!("  Default timeout: {}s", config.default_timeout);
    println!("  Default debate rounds: {}", config.default_debate_rounds);
    println!("  Config directory: {:?}", ConfigManager::get_config_dir()?);
    println!();

    // Show active sessions
    let session_manager = SessionManager::new()?;
    let sessions = session_manager.list_sessions().await?;

    println!("Active Sessions: {}", sessions.len());
    if sessions.is_empty() {
        println!("  (None)");
    } else {
        for session in sessions {
            let time_ago = format_time_ago(session.last_interaction);
            println!("  • {} ({}): {}", session.name, session.cli, time_ago);
            println!("    Topic: {}", session.topic);
        }
    }

    Ok(())
}

fn format_time_ago(time: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(time);

    if duration.num_seconds() < 60 {
        format!("{} seconds ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else {
        format!("{} days ago", duration.num_days())
    }
}
