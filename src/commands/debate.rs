//! Debate command - Multi-AI debate orchestration

use crate::orchestrator::DebateOrchestrator;

/// Run a debate between Claude, Codex, and Gemini
pub async fn run_debate(
    topic: String,
    rounds: usize,
    output: String,
    timeout: u64,
) -> anyhow::Result<()> {
    println!("GPT ENGAGE DEBATE");
    println!("Topic: {}", topic);
    println!();

    // Run the debate
    let result = DebateOrchestrator::run_debate(&topic, rounds, timeout).await?;

    // Output results based on format
    match output.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&result)?;
            println!("{}", json);
        }
        "markdown" => {
            print_markdown(&result)?;
        }
        "text" | _ => {
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
            println!("{}:", response.cli);
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
            println!("### {}", response.cli);
            println!();
            println!("{}", response.response);
            println!();
        }
    }

    Ok(())
}
