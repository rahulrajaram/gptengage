//! Generate agent definitions command

use crate::invokers::{ClaudeInvoker, CodexInvoker, GeminiInvoker, Invoker};
use crate::orchestrator::{AgentDefinition, AgentFile};

/// Generate agent definitions for debate participants
pub async fn run_generate_agents(
    topic: String,
    roles: String,
    output_path: String,
    use_cli: String,
    timeout: u64,
) -> anyhow::Result<()> {
    println!("Generating agent definitions...");
    println!("Topic: {}", topic);
    println!("Roles: {}", roles);
    println!();

    // Parse roles
    let role_list: Vec<&str> = roles.split(',').map(|s| s.trim()).collect();
    if role_list.is_empty() {
        return Err(anyhow::anyhow!("No roles specified"));
    }

    // Select the invoker
    let invoker: Box<dyn Invoker> = match use_cli.to_lowercase().as_str() {
        "claude" => Box::new(ClaudeInvoker::new()),
        "codex" => Box::new(CodexInvoker::new()),
        "gemini" => Box::new(GeminiInvoker::new()),
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown CLI '{}'. Use 'claude', 'codex', or 'gemini'",
                use_cli
            ))
        }
    };

    if !invoker.is_available() {
        return Err(anyhow::anyhow!(
            "CLI '{}' is not available. Please ensure it's installed and in your PATH",
            use_cli
        ));
    }

    // Build the prompt for generating agent definitions
    let prompt = build_generation_prompt(&topic, &role_list);

    println!("Using {} to generate agent definitions...", use_cli);
    let response = invoker.invoke(&prompt, timeout).await?;

    // Parse the response as JSON
    let agent_definitions = parse_agent_response(&response, &role_list)?;

    // Create the agent file
    let agent_file = AgentFile {
        schema_version: "1.0".to_string(),
        generated_by: Some(format!("gptengage-{}", use_cli)),
        participants: agent_definitions,
    };

    // Validate all agents
    for (idx, agent) in agent_file.participants.iter().enumerate() {
        agent.validate().map_err(|e| {
            anyhow::anyhow!(
                "Generated agent {} ({}) failed validation: {}",
                idx + 1,
                agent.persona,
                e
            )
        })?;
    }

    // Write to file
    let json = serde_json::to_string_pretty(&agent_file)?;
    std::fs::write(&output_path, json)?;

    println!();
    println!(
        "✅ Generated {} agent definition(s)",
        agent_file.participants.len()
    );
    println!("📄 Saved to: {}", output_path);
    println!();
    println!("Agents:");
    for agent in &agent_file.participants {
        println!("  - {} ({})", agent.cli, agent.persona);
    }
    println!();
    println!(
        "Use with: gptengage debate \"{}\" --agent-file {}",
        topic, output_path
    );

    Ok(())
}

fn build_generation_prompt(topic: &str, roles: &[&str]) -> String {
    format!(
        r#"Generate detailed agent definitions for a debate on the following topic:

Topic: "{}"

Create exactly {} agent definition(s), one for each of these roles: {}

For each agent, provide a JSON object with these fields:
- "cli": The CLI to use (choose between "claude", "codex", or "gemini" - distribute evenly)
- "persona": The role name (e.g., "CEO", "Principal Architect")
- "instructions": Detailed instructions (2-4 sentences) on how this role should approach the debate, what to prioritize, and their communication style
- "expertise": Array of 3-5 expertise areas relevant to this role
- "communication_style": A brief description of how this role communicates (e.g., "Executive - concise and action-oriented")

IMPORTANT: Return ONLY a valid JSON array of agent objects, nothing else. No markdown, no explanations, just the JSON array.

Example format:
[
  {{
    "cli": "claude",
    "persona": "CEO",
    "instructions": "Focus on business impact, ROI, and strategic alignment. Be decisive but ask about risks. Prioritize quick wins and long-term sustainability. Keep responses under 3 paragraphs.",
    "expertise": ["business strategy", "finance", "leadership", "market analysis", "risk management"],
    "communication_style": "Executive - concise and action-oriented"
  }},
  ...
]

Generate the agent definitions now:"#,
        topic,
        roles.len(),
        roles.join(", ")
    )
}

fn parse_agent_response(
    response: &str,
    expected_roles: &[&str],
) -> anyhow::Result<Vec<AgentDefinition>> {
    // Try to extract JSON from the response
    let json_str = extract_json_array(response)?;

    // Parse the JSON
    let agents: Vec<AgentDefinition> = serde_json::from_str(&json_str).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse agent definitions from response. Error: {}\nResponse: {}",
            e,
            response
        )
    })?;

    // Validate count
    if agents.len() != expected_roles.len() {
        return Err(anyhow::anyhow!(
            "Expected {} agents, but got {}",
            expected_roles.len(),
            agents.len()
        ));
    }

    Ok(agents)
}

fn extract_json_array(text: &str) -> anyhow::Result<String> {
    // Find the first '[' and last ']' to extract the JSON array
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            if start < end {
                return Ok(text[start..=end].to_string());
            }
        }
    }

    Err(anyhow::anyhow!(
        "Could not find valid JSON array in response. Expected format: [{{...}}]"
    ))
}
