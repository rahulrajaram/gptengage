//! Debate orchestration - Run multi-round debates

use crate::invokers::{ClaudeInvoker, CodexInvoker, GeminiInvoker, Invoker};
use serde::{Deserialize, Serialize};
use tokio::task;

pub struct DebateOrchestrator;

/// Full agent definition with persona, instructions, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub cli: String,
    pub persona: String,
    pub instructions: String,
    #[serde(default)]
    pub expertise: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub communication_style: Option<String>,
}

impl AgentDefinition {
    /// Validate that required fields are non-empty
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.cli.trim().is_empty() {
            return Err(anyhow::anyhow!("Agent 'cli' field cannot be empty"));
        }
        if self.persona.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "Agent 'persona' field cannot be empty (required for agent definitions)"
            ));
        }
        if self.instructions.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "Agent 'instructions' field cannot be empty (required for agent definitions)"
            ));
        }
        if self.instructions.len() < 10 {
            return Err(anyhow::anyhow!(
                "Agent 'instructions' must be at least 10 characters (got {})",
                self.instructions.len()
            ));
        }
        Ok(())
    }

    /// Convert to a Participant with enriched prompt building
    pub fn to_participant(&self) -> Participant {
        Participant {
            cli: self.cli.clone(),
            persona: Some(self.persona.clone()),
            agent_definition: Some(self.clone()),
        }
    }
}

/// Agent file schema
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentFile {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_by: Option<String>,
    pub participants: Vec<AgentDefinition>,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

impl AgentFile {
    /// Load and validate an agent file
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read agent file '{}': {}", path, e))?;

        let agent_file: AgentFile = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse agent file '{}': {}", path, e))?;

        // Validate schema version
        if agent_file.schema_version != "1.0" {
            return Err(anyhow::anyhow!(
                "Unsupported schema version '{}'. Expected '1.0'",
                agent_file.schema_version
            ));
        }

        // Validate all participants
        if agent_file.participants.is_empty() {
            return Err(anyhow::anyhow!(
                "Agent file must contain at least one participant"
            ));
        }

        for (idx, agent) in agent_file.participants.iter().enumerate() {
            agent.validate().map_err(|e| {
                anyhow::anyhow!("Validation failed for participant {}: {}", idx + 1, e)
            })?;
        }

        Ok(agent_file)
    }

    /// Convert to a list of Participants
    pub fn to_participants(&self) -> Vec<Participant> {
        self.participants
            .iter()
            .map(|agent| agent.to_participant())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub cli: String,
    pub persona: Option<String>,
    #[serde(skip)]
    pub agent_definition: Option<AgentDefinition>,
}

impl Participant {
    pub fn new(cli: String, persona: Option<String>) -> Self {
        Self {
            cli,
            persona,
            agent_definition: None,
        }
    }

    pub fn display_name(&self) -> String {
        match &self.persona {
            Some(p) => format!("{} ({})", self.cli, p),
            None => self.cli.clone(),
        }
    }

    pub fn build_prompt_with_persona(&self, base_prompt: &str) -> String {
        // If we have a full agent definition, use rich context
        if let Some(agent_def) = &self.agent_definition {
            let mut context = String::from("[AGENT CONTEXT]\n");
            context.push_str(&format!("Role: {}\n", agent_def.persona));
            context.push_str(&format!("Instructions: {}\n", agent_def.instructions));

            if !agent_def.expertise.is_empty() {
                context.push_str(&format!("Expertise: {}\n", agent_def.expertise.join(", ")));
            }

            if let Some(style) = &agent_def.communication_style {
                context.push_str(&format!("Communication Style: {}\n", style));
            }

            context.push_str("[/AGENT CONTEXT]\n\n");
            context.push_str(base_prompt);
            return context;
        }

        // Fall back to simple persona context
        match &self.persona {
            Some(persona) => {
                format!(
                    "[ROLE CONTEXT]\nYou are participating in this debate as a {}. Respond from that perspective, drawing on the expertise, priorities, and viewpoints typical of this role.\n[/ROLE CONTEXT]\n\n{}",
                    persona, base_prompt
                )
            }
            None => base_prompt.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResponse {
    pub cli: String,
    pub persona: Option<String>,
    pub response: String,
}

impl RoundResponse {
    pub fn display_name(&self) -> String {
        match &self.persona {
            Some(p) => format!("{} ({})", self.cli, p),
            None => self.cli.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebateResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gptengage_version: Option<String>,
    pub topic: String,
    pub rounds: Vec<Vec<RoundResponse>>,
}

impl DebateOrchestrator {
    /// Run a debate with specific participants
    pub async fn run_debate_with_participants(
        topic: &str,
        participants: Vec<Participant>,
        num_rounds: usize,
        timeout: u64,
    ) -> anyhow::Result<DebateResult> {
        if participants.is_empty() {
            return Err(anyhow::anyhow!("At least one participant is required"));
        }

        let mut rounds: Vec<Vec<RoundResponse>> = Vec::new();

        for round in 1..=num_rounds {
            println!("Running round {} of {}...", round, num_rounds);

            // Build base context for this round
            let mut base_context = format!("Topic: {}\n\nRound {}\n\n", topic, round);

            if round > 1 {
                if let Some(prev_round) = rounds.last() {
                    base_context.push_str("Previous responses:\n");
                    for response in prev_round.iter() {
                        base_context.push_str(&format!(
                            "{}: {}\n\n",
                            response.display_name(),
                            response.response
                        ));
                    }
                }
            }

            base_context.push_str("Please provide your perspective on this topic.");

            // Spawn tasks for all participants in parallel
            let mut tasks = Vec::new();

            for participant in &participants {
                let participant_clone = participant.clone();
                let ctx = participant_clone.build_prompt_with_persona(&base_context);

                let task = task::spawn(async move {
                    let invoker: Box<dyn Invoker> =
                        match participant_clone.cli.to_lowercase().as_str() {
                            "claude" => Box::new(ClaudeInvoker::new()),
                            "codex" => Box::new(CodexInvoker::new()),
                            "gemini" => Box::new(GeminiInvoker::new()),
                            _ => {
                                eprintln!(
                                    "Unknown CLI '{}', skipping participant",
                                    participant_clone.cli
                                );
                                return None;
                            }
                        };

                    if !invoker.is_available() {
                        eprintln!(
                            "{} is not available, skipping",
                            participant_clone.display_name()
                        );
                        return None;
                    }

                    match invoker.invoke(&ctx, timeout).await {
                        Ok(response) => Some(RoundResponse {
                            cli: participant_clone.cli.clone(),
                            persona: participant_clone.persona.clone(),
                            response,
                        }),
                        Err(e) => {
                            eprintln!(
                                "{} invocation failed: {}",
                                participant_clone.display_name(),
                                e
                            );
                            None
                        }
                    }
                });

                tasks.push(task);
            }

            // Wait for all tasks to complete
            let results = futures::future::join_all(tasks).await;

            let round_responses: Vec<RoundResponse> =
                results.into_iter().flatten().flatten().collect();

            // Ensure at least one responder per round
            if round_responses.is_empty() {
                return Err(anyhow::anyhow!(
                    "No participants were able to respond in round {}. Please ensure their CLIs are installed and available.",
                    round
                ));
            }

            rounds.push(round_responses);
        }

        Ok(DebateResult {
            gptengage_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            topic: topic.to_string(),
            rounds,
        })
    }

    /// Run a debate with default participants (Claude, Codex, Gemini without personas)
    pub async fn run_debate(
        topic: &str,
        num_rounds: usize,
        timeout: u64,
    ) -> anyhow::Result<DebateResult> {
        let participants = vec![
            Participant::new("claude".to_string(), None),
            Participant::new("codex".to_string(), None),
            Participant::new("gemini".to_string(), None),
        ];

        Self::run_debate_with_participants(topic, participants, num_rounds, timeout).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_response_creation() {
        let response = RoundResponse {
            cli: "Claude".to_string(),
            persona: None,
            response: "This is Claude's perspective".to_string(),
        };

        assert_eq!(response.cli, "Claude");
        assert_eq!(response.persona, None);
        assert_eq!(response.response, "This is Claude's perspective");
        assert_eq!(response.display_name(), "Claude");
    }

    #[test]
    fn test_round_response_with_persona() {
        let response = RoundResponse {
            cli: "Claude".to_string(),
            persona: Some("CEO".to_string()),
            response: "From a CEO perspective...".to_string(),
        };

        assert_eq!(response.cli, "Claude");
        assert_eq!(response.persona, Some("CEO".to_string()));
        assert_eq!(response.display_name(), "Claude (CEO)");
    }

    #[test]
    fn test_round_response_serialization() {
        let response = RoundResponse {
            cli: "Codex".to_string(),
            persona: Some("Architect".to_string()),
            response: "This is Codex's perspective".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RoundResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.cli, "Codex");
        assert_eq!(deserialized.persona, Some("Architect".to_string()));
        assert_eq!(deserialized.response, "This is Codex's perspective");
    }

    #[test]
    fn test_debate_result_creation() {
        let result = DebateResult {
            gptengage_version: Some("0.1.0".to_string()),
            topic: "Should we use Rust?".to_string(),
            rounds: vec![vec![
                RoundResponse {
                    cli: "Claude".to_string(),
                    persona: None,
                    response: "Yes, Rust is great".to_string(),
                },
                RoundResponse {
                    cli: "Gemini".to_string(),
                    persona: None,
                    response: "Go is simpler".to_string(),
                },
            ]],
        };

        assert_eq!(result.topic, "Should we use Rust?");
        assert_eq!(result.rounds.len(), 1);
        assert_eq!(result.rounds[0].len(), 2);
        assert_eq!(result.rounds[0][0].cli, "Claude");
    }

    #[test]
    fn test_debate_result_multiple_rounds() {
        // Round 1
        let rounds = vec![
            vec![
                RoundResponse {
                    cli: "Claude".to_string(),
                    persona: None,
                    response: "Round 1: Claude's view".to_string(),
                },
                RoundResponse {
                    cli: "Codex".to_string(),
                    persona: None,
                    response: "Round 1: Codex's view".to_string(),
                },
            ],
            // Round 2
            vec![
                RoundResponse {
                    cli: "Claude".to_string(),
                    persona: None,
                    response: "Round 2: Claude's refined view".to_string(),
                },
                RoundResponse {
                    cli: "Codex".to_string(),
                    persona: None,
                    response: "Round 2: Codex's refined view".to_string(),
                },
            ],
        ];

        let result = DebateResult {
            gptengage_version: None,
            topic: "Test Topic".to_string(),
            rounds,
        };

        assert_eq!(result.rounds.len(), 2);
        assert_eq!(result.rounds[0].len(), 2);
        assert_eq!(result.rounds[1].len(), 2);
        assert!(result.rounds[1][0].response.contains("Round 2"));
    }

    #[test]
    fn test_debate_result_serialization() {
        let result = DebateResult {
            gptengage_version: Some("0.1.0".to_string()),
            topic: "Tabs vs Spaces".to_string(),
            rounds: vec![vec![
                RoundResponse {
                    cli: "Claude".to_string(),
                    persona: None,
                    response: "Tabs are consistent".to_string(),
                },
                RoundResponse {
                    cli: "Gemini".to_string(),
                    persona: None,
                    response: "Spaces are standard".to_string(),
                },
            ]],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: DebateResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.topic, "Tabs vs Spaces");
        assert_eq!(deserialized.rounds.len(), 1);
        assert_eq!(deserialized.rounds[0].len(), 2);
    }

    #[test]
    fn test_debate_result_empty_rounds() {
        let result = DebateResult {
            gptengage_version: None,
            topic: "Empty debate".to_string(),
            rounds: vec![],
        };

        assert_eq!(result.rounds.len(), 0);
        assert_eq!(result.topic, "Empty debate");
    }

    #[test]
    fn test_round_response_clone() {
        let response1 = RoundResponse {
            cli: "Claude".to_string(),
            persona: Some("CEO".to_string()),
            response: "Test response".to_string(),
        };

        let response2 = response1.clone();

        assert_eq!(response1.cli, response2.cli);
        assert_eq!(response1.persona, response2.persona);
        assert_eq!(response1.response, response2.response);
    }

    #[test]
    fn test_round_response_with_long_content() {
        let long_response = "a".repeat(10000);
        let response = RoundResponse {
            cli: "Claude".to_string(),
            persona: None,
            response: long_response.clone(),
        };

        assert_eq!(response.response.len(), 10000);

        // Verify serialization works with large content
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RoundResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.response.len(), 10000);
    }

    #[test]
    fn test_debate_result_with_special_chars() {
        let result = DebateResult {
            gptengage_version: Some("0.1.0".to_string()),
            topic: "Test with 特殊 characters & symbols! 🚀".to_string(),
            rounds: vec![vec![RoundResponse {
                cli: "Claude".to_string(),
                persona: None,
                response: "Response with unicode: émojis: 🎉".to_string(),
            }]],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: DebateResult = serde_json::from_str(&json).unwrap();

        assert!(deserialized.topic.contains("特殊"));
        assert!(deserialized.rounds[0][0].response.contains("🎉"));
    }

    #[test]
    fn test_participant_creation() {
        let p1 = Participant::new("claude".to_string(), None);
        assert_eq!(p1.cli, "claude");
        assert_eq!(p1.persona, None);
        assert_eq!(p1.display_name(), "claude");

        let p2 = Participant::new("claude".to_string(), Some("CEO".to_string()));
        assert_eq!(p2.cli, "claude");
        assert_eq!(p2.persona, Some("CEO".to_string()));
        assert_eq!(p2.display_name(), "claude (CEO)");
    }

    #[test]
    fn test_participant_prompt_building() {
        let base = "Discuss the topic";

        let p1 = Participant::new("claude".to_string(), None);
        let prompt1 = p1.build_prompt_with_persona(base);
        assert_eq!(prompt1, base);

        let p2 = Participant::new("claude".to_string(), Some("CEO".to_string()));
        let prompt2 = p2.build_prompt_with_persona(base);
        assert!(prompt2.contains("ROLE CONTEXT"));
        assert!(prompt2.contains("CEO"));
        assert!(prompt2.contains(base));
    }
}
