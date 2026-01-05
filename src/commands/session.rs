//! Session management commands

use crate::session::SessionManager;

/// List all active sessions
pub async fn list_sessions() -> anyhow::Result<()> {
    let manager = SessionManager::new()?;
    let sessions = manager.list_sessions().await?;

    if sessions.is_empty() {
        println!("No active sessions.");
        return Ok(());
    }

    println!("┌──────────────┬────────┬─────────────────────────┬──────────────┐");
    println!("│ Session      │ CLI    │ Topic                   │ Last Used    │");
    println!("├──────────────┼────────┼─────────────────────────┼──────────────┤");

    for session in sessions {
        let time_ago = format_time_ago(session.last_interaction);
        // Use chars().take() for UTF-8 safe truncation
        let truncated_topic: String = session.topic.chars().take(23).collect();
        println!(
            "│ {:<12} │ {:<6} │ {:<23} │ {:<12} │",
            session.name, session.cli, truncated_topic, time_ago
        );
    }

    println!("└──────────────┴────────┴─────────────────────────┴──────────────┘");
    Ok(())
}

/// Show a specific session's history
pub async fn show_session(name: String) -> anyhow::Result<()> {
    let manager = SessionManager::new()?;
    let session = manager.load_session(&name).await?;

    println!("Session: {}", session.name);
    println!("CLI: {}", session.cli);
    println!("Topic: {}", session.topic);
    println!("Created: {}", session.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Last interaction: {}", session.last_interaction.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Turns: {}", session.turns.len());
    println!();
    println!("┌─────────────────────────────────────────────────────────────┐");

    for (idx, turn) in session.turns.iter().enumerate() {
        let role_str = if turn.role == "user" {
            "[You]".to_string()
        } else {
            format!("[{}]", session.cli)
        };

        println!("│ {}: {}                                    │", idx + 1, role_str);
        println!("│                                                             │");

        // Word wrap content
        for line in turn.content.lines() {
            let mut current = line.to_string();
            while current.len() > 58 {
                let (chunk, rest) = current.split_at(58);
                println!("│ {} │", chunk);
                current = rest.to_string();
            }
            println!("│ {:<57} │", current);
        }
        println!("│                                                             │");
    }

    println!("└─────────────────────────────────────────────────────────────┘");
    println!();
    println!("To continue this session, run:");
    println!("  gptengage invoke {} \"<your message>\" --session {}", session.cli, session.name);

    Ok(())
}

/// End a session
pub async fn end_session(name: Option<String>, all: bool) -> anyhow::Result<()> {
    let manager = SessionManager::new()?;

    if all {
        let sessions = manager.list_sessions().await?;
        if sessions.is_empty() {
            println!("No sessions to delete.");
            return Ok(());
        }

        for session in sessions {
            manager.delete_session(&session.name).await?;
            println!("✓ Deleted session: {}", session.name);
        }
        println!("\nAll sessions deleted.");
    } else if let Some(n) = name {
        manager.delete_session(&n).await?;
        println!("✓ Session '{}' deleted.", n);
    } else {
        return Err(anyhow::anyhow!("Specify a session name or use --all"));
    }

    Ok(())
}

fn format_time_ago(time: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(time);

    if duration.num_seconds() < 60 {
        format!("{}s ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else {
        format!("{}d ago", duration.num_days())
    }
}
