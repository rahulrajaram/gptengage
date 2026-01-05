//! Debate orchestration - Run multi-round debates

use crate::invokers::{ClaudeInvoker, CodexInvoker, GeminiInvoker, Invoker};
use serde::{Deserialize, Serialize};
use tokio::task;

pub struct DebateOrchestrator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResponse {
    pub cli: String,
    pub response: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebateResult {
    pub topic: String,
    pub rounds: Vec<Vec<RoundResponse>>,
}

impl DebateOrchestrator {
    pub async fn run_debate(
        topic: &str,
        num_rounds: usize,
        timeout: u64,
    ) -> anyhow::Result<DebateResult> {
        let claude = ClaudeInvoker::new();
        let codex = CodexInvoker::new();
        let gemini = GeminiInvoker::new();

        let mut rounds: Vec<Vec<RoundResponse>> = Vec::new();

        for round in 1..=num_rounds {
            println!("Running round {} of {}...", round, num_rounds);

            // Build context for this round
            let mut context = format!("Topic: {}\n\nRound {}\n\n", topic, round);

            if round > 1 {
                if let Some(prev_round) = rounds.last() {
                    context.push_str("Previous responses:\n");
                    for response in prev_round.iter() {
                        context.push_str(&format!("{}: {}\n\n", response.cli, response.response));
                    }
                }
            }

            context.push_str("Please provide your perspective on this topic.");

            // Clone invokers for this round
            let claude_clone = claude.clone();
            let codex_clone = codex.clone();
            let gemini_clone = gemini.clone();

            // Run all three CLIs in parallel
            let claude_future = {
                let ctx = context.clone();
                task::spawn(async move {
                    if claude_clone.is_available() {
                        match claude_clone.invoke(&ctx, timeout).await {
                            Ok(response) => Some(RoundResponse {
                                cli: "Claude".to_string(),
                                response,
                            }),
                            Err(e) => {
                                eprintln!("Claude invocation failed: {}", e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
            };

            let codex_future = {
                let ctx = context.clone();
                task::spawn(async move {
                    if codex_clone.is_available() {
                        match codex_clone.invoke(&ctx, timeout).await {
                            Ok(response) => Some(RoundResponse {
                                cli: "Codex".to_string(),
                                response,
                            }),
                            Err(e) => {
                                eprintln!("Codex invocation failed: {}", e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
            };

            let gemini_future = {
                let ctx = context.clone();
                task::spawn(async move {
                    if gemini_clone.is_available() {
                        match gemini_clone.invoke(&ctx, timeout).await {
                            Ok(response) => Some(RoundResponse {
                                cli: "Gemini".to_string(),
                                response,
                            }),
                            Err(e) => {
                                eprintln!("Gemini invocation failed: {}", e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
            };

            // Wait for all to complete
            let (claude_result, codex_result, gemini_result) =
                tokio::join!(claude_future, codex_future, gemini_future);

            let mut round_responses = Vec::new();

            if let Ok(Some(response)) = claude_result {
                round_responses.push(response);
            }
            if let Ok(Some(response)) = codex_result {
                round_responses.push(response);
            }
            if let Ok(Some(response)) = gemini_result {
                round_responses.push(response);
            }

            // Ensure at least one responder per round
            if round_responses.is_empty() {
                return Err(anyhow::anyhow!(
                    "No CLIs available for debate round {}. Please ensure at least one of the following is installed and available: claude, codex, or gemini",
                    round
                ));
            }

            rounds.push(round_responses);
        }

        Ok(DebateResult {
            topic: topic.to_string(),
            rounds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_response_creation() {
        let response = RoundResponse {
            cli: "Claude".to_string(),
            response: "This is Claude's perspective".to_string(),
        };

        assert_eq!(response.cli, "Claude");
        assert_eq!(response.response, "This is Claude's perspective");
    }

    #[test]
    fn test_round_response_serialization() {
        let response = RoundResponse {
            cli: "Codex".to_string(),
            response: "This is Codex's perspective".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RoundResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.cli, "Codex");
        assert_eq!(deserialized.response, "This is Codex's perspective");
    }

    #[test]
    fn test_debate_result_creation() {
        let result = DebateResult {
            topic: "Should we use Rust?".to_string(),
            rounds: vec![vec![
                RoundResponse {
                    cli: "Claude".to_string(),
                    response: "Yes, Rust is great".to_string(),
                },
                RoundResponse {
                    cli: "Gemini".to_string(),
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
                    response: "Round 1: Claude's view".to_string(),
                },
                RoundResponse {
                    cli: "Codex".to_string(),
                    response: "Round 1: Codex's view".to_string(),
                },
            ],
            // Round 2
            vec![
                RoundResponse {
                    cli: "Claude".to_string(),
                    response: "Round 2: Claude's refined view".to_string(),
                },
                RoundResponse {
                    cli: "Codex".to_string(),
                    response: "Round 2: Codex's refined view".to_string(),
                },
            ],
        ];

        let result = DebateResult {
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
            topic: "Tabs vs Spaces".to_string(),
            rounds: vec![vec![
                RoundResponse {
                    cli: "Claude".to_string(),
                    response: "Tabs are consistent".to_string(),
                },
                RoundResponse {
                    cli: "Gemini".to_string(),
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
            response: "Test response".to_string(),
        };

        let response2 = response1.clone();

        assert_eq!(response1.cli, response2.cli);
        assert_eq!(response1.response, response2.response);
    }

    #[test]
    fn test_round_response_with_long_content() {
        let long_response = "a".repeat(10000);
        let response = RoundResponse {
            cli: "Claude".to_string(),
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
            topic: "Test with 特殊 characters & symbols! 🚀".to_string(),
            rounds: vec![vec![RoundResponse {
                cli: "Claude".to_string(),
                response: "Response with unicode: émojis: 🎉".to_string(),
            }]],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: DebateResult = serde_json::from_str(&json).unwrap();

        assert!(deserialized.topic.contains("特殊"));
        assert!(deserialized.rounds[0][0].response.contains("🎉"));
    }
}
