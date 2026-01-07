//! Debate command - Multi-AI debate orchestration

use crate::orchestrator::{AgentFile, DebateOrchestrator, Participant};

/// Parse participants from format "cli:persona,cli:persona" or "cli,cli"
fn parse_participants(participants_str: &str) -> anyhow::Result<Vec<Participant>> {
    let mut participants = Vec::new();

    for part in participants_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let parts: Vec<&str> = part.split(':').collect();
        match parts.len() {
            1 => {
                // Just CLI name, no persona
                participants.push(Participant::new(parts[0].to_string(), None));
            }
            2 => {
                // CLI:persona format
                participants.push(Participant::new(
                    parts[0].to_string(),
                    Some(parts[1].to_string()),
                ));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid participant format '{}'. Expected 'cli' or 'cli:persona'",
                    part
                ));
            }
        }
    }

    if participants.is_empty() {
        return Err(anyhow::anyhow!("At least one participant is required"));
    }

    Ok(participants)
}

/// Run a debate between specified participants or default CLIs
pub async fn run_debate(
    topic: String,
    participants_str: Option<String>,
    agent_file_path: Option<String>,
    rounds: usize,
    output: String,
    timeout: u64,
) -> anyhow::Result<()> {
    println!("GPT ENGAGE DEBATE");
    println!("Topic: {}", topic);

    // Parse participants from various sources
    let result = if let Some(agent_file) = agent_file_path {
        // Load and validate agent file
        let agent_file = AgentFile::load(&agent_file)?;
        let participants = agent_file.to_participants();

        println!("Loaded {} agent(s) from file:", participants.len());
        for p in &participants {
            println!("  - {}", p.display_name());
        }
        println!();

        DebateOrchestrator::run_debate_with_participants(&topic, participants, rounds, timeout)
            .await?
    } else if let Some(participants_str) = participants_str {
        let participants = parse_participants(&participants_str)?;
        println!("Participants:");
        for p in &participants {
            println!("  - {}", p.display_name());
        }
        println!();
        DebateOrchestrator::run_debate_with_participants(&topic, participants, rounds, timeout)
            .await?
    } else {
        println!("Using default participants: Claude, Codex, Gemini");
        println!();
        DebateOrchestrator::run_debate(&topic, rounds, timeout).await?
    };

    // Output results based on format
    match output.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&result)?;
            println!("{}", json);
        }
        "markdown" => {
            print_markdown(&result)?;
        }
        _ => {
            print_text(&result)?;
        }
    }

    Ok(())
}

fn print_text(result: &crate::orchestrator::DebateResult) -> anyhow::Result<()> {
    for (round_num, responses) in result.rounds.iter().enumerate() {
        println!("ROUND {}", round_num + 1);
        println!("────────────────────────────────────────");

        for response in responses {
            println!("{}:", response.display_name());
            println!("{}", response.response);
            println!();
        }
    }

    println!("DEBATE COMPLETE");
    Ok(())
}

fn print_markdown(result: &crate::orchestrator::DebateResult) -> anyhow::Result<()> {
    println!("# {}", result.topic);
    println!();

    for (round_num, responses) in result.rounds.iter().enumerate() {
        println!("## Round {}", round_num + 1);
        println!();

        for response in responses {
            println!("### {}", response.display_name());
            println!();
            println!("{}", response.response);
            println!();
        }
    }

    Ok(())
}
