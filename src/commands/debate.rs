//! Debate command - Multi-AI debate orchestration

use crate::orchestrator::{AgentFile, DebateOrchestrator, Participant};

/// Debate configuration options
pub struct DebateOptions {
    pub topic: String,
    pub agent: Option<String>,
    pub instances: Option<usize>,
    pub participants: Option<String>,
    pub agent_file: Option<String>,
    pub rounds: usize,
    pub output: String,
    pub timeout: u64,
}

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
pub async fn run_debate(options: DebateOptions) -> anyhow::Result<()> {
    println!("GPT ENGAGE DEBATE");
    println!("Topic: {}", options.topic);

    // Parse participants from various sources
    let result = if let Some(agent_cli) = options.agent {
        // Multi-instance mode: create N instances of the same CLI
        let num_instances = options.instances.unwrap_or(3);

        // Validate CLI name
        let valid_clis = ["claude", "codex", "gemini"];
        if !valid_clis.contains(&agent_cli.to_lowercase().as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid CLI '{}'. Must be one of: claude, codex, gemini",
                agent_cli
            ));
        }

        println!(
            "Multi-instance mode: {} {} instance(s)",
            num_instances, agent_cli
        );
        println!("(Leveraging LLM nondeterminism and debate dynamics)");
        println!();

        // Create N participants with the same CLI
        let participants: Vec<Participant> = (0..num_instances)
            .map(|_| Participant::new(agent_cli.clone(), None))
            .collect();

        DebateOrchestrator::run_debate_with_participants(
            &options.topic,
            participants,
            options.rounds,
            options.timeout,
        )
        .await?
    } else if let Some(agent_file) = options.agent_file {
        // Load and validate agent file
        let agent_file = AgentFile::load(&agent_file)?;
        let participants = agent_file.to_participants();

        println!("Loaded {} agent(s) from file:", participants.len());
        for p in &participants {
            println!("  - {}", p.display_name());
        }
        println!();

        DebateOrchestrator::run_debate_with_participants(
            &options.topic,
            participants,
            options.rounds,
            options.timeout,
        )
        .await?
    } else if let Some(participants_str) = options.participants {
        let participants = parse_participants(&participants_str)?;
        println!("Participants:");
        for p in &participants {
            println!("  - {}", p.display_name());
        }
        println!();
        DebateOrchestrator::run_debate_with_participants(
            &options.topic,
            participants,
            options.rounds,
            options.timeout,
        )
        .await?
    } else {
        println!("Using default participants: Claude, Codex, Gemini");
        println!();
        DebateOrchestrator::run_debate(&options.topic, options.rounds, options.timeout).await?
    };

    // Output results based on format
    match options.output.as_str() {
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
