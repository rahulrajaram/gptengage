//! Session management - Store and manage conversation history

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    pub cli: String,
    pub topic: String,
    pub created_at: DateTime<Utc>,
    pub last_interaction: DateTime<Utc>,
    pub turns: Vec<Turn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct SessionManager {
    sessions_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let config_dir = crate::config::ConfigManager::get_config_dir()?;
        let sessions_dir = config_dir.join("sessions");

        // Create sessions directory if it doesn't exist
        std::fs::create_dir_all(&sessions_dir)?;

        Ok(SessionManager { sessions_dir })
    }

    /// Validate session name to prevent directory traversal
    fn validate_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(anyhow::anyhow!("Session name cannot be empty"));
        }
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return Err(anyhow::anyhow!(
                "Invalid session name: must not contain path separators"
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(anyhow::anyhow!(
                "Invalid session name: use only letters, numbers, dashes, and underscores"
            ));
        }
        Ok(())
    }

    /// Create a new session
    pub fn create_session(&self, name: String, cli: String, topic: String) -> Result<Session> {
        Self::validate_name(&name)?;
        let now = Utc::now();
        Ok(Session {
            name,
            cli,
            topic,
            created_at: now,
            last_interaction: now,
            turns: Vec::new(),
        })
    }

    /// Load a session from disk
    pub async fn load_session(&self, name: &str) -> Result<Session> {
        Self::validate_name(name)?;
        let path = self.sessions_dir.join(format!("{}.json", name));
        if !path.exists() {
            return Err(anyhow::anyhow!("Session '{}' not found", name));
        }
        let content = tokio::fs::read_to_string(&path).await?;
        let session = serde_json::from_str(&content)?;
        Ok(session)
    }

    /// Save a session to disk
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        Self::validate_name(&session.name)?;
        let path = self.sessions_dir.join(format!("{}.json", session.name));
        let content = serde_json::to_string_pretty(&session)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        let mut entries = tokio::fs::read_dir(&self.sessions_dir).await?;
        let mut summaries = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        summaries.push(SessionSummary {
                            name: session.name,
                            cli: session.cli,
                            topic: session.topic,
                            last_interaction: session.last_interaction,
                        });
                    }
                }
            }
        }

        // Sort by last interaction (most recent first)
        summaries.sort_by(|a, b| b.last_interaction.cmp(&a.last_interaction));
        Ok(summaries)
    }

    /// Delete a session
    pub async fn delete_session(&self, name: &str) -> Result<()> {
        Self::validate_name(name)?;
        let path = self.sessions_dir.join(format!("{}.json", name));
        if !path.exists() {
            return Err(anyhow::anyhow!("Session '{}' not found", name));
        }
        tokio::fs::remove_file(&path).await?;
        Ok(())
    }

    /// Add a turn to a session
    pub fn add_turn(&self, session: &mut Session, role: String, content: String) {
        let turn = Turn {
            role,
            content,
            timestamp: Utc::now(),
        };
        session.turns.push(turn);
        session.last_interaction = Utc::now();
    }

    /// Build prompt with session history injected
    pub fn build_prompt_with_history(&self, session: &Session, current_prompt: &str) -> String {
        if session.turns.is_empty() {
            return current_prompt.to_string();
        }

        let mut prompt = String::new();
        prompt.push_str("[CONVERSATION HISTORY]\n");

        for turn in &session.turns {
            let role = if turn.role == "user" {
                "User"
            } else {
                "Assistant"
            };
            prompt.push_str(&format!("{}: {}\n\n", role, turn.content));
        }

        prompt.push_str("[/CONVERSATION HISTORY]\n\n");
        prompt.push_str("[CURRENT REQUEST]\n");
        prompt.push_str(current_prompt);
        prompt.push_str("\n[/CURRENT REQUEST]");

        prompt
    }
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub name: String,
    pub cli: String,
    pub topic: String,
    pub last_interaction: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(SessionManager::validate_name("valid-name").is_ok());
        assert!(SessionManager::validate_name("valid_name").is_ok());
        assert!(SessionManager::validate_name("validname123").is_ok());
        assert!(SessionManager::validate_name("a").is_ok());
        assert!(SessionManager::validate_name("a-b_c123").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(SessionManager::validate_name("").is_err());
        assert!(SessionManager::validate_name("   ").is_err());
    }

    #[test]
    fn test_validate_name_directory_traversal() {
        assert!(SessionManager::validate_name("..").is_err());
        assert!(SessionManager::validate_name("../evil").is_err());
        assert!(SessionManager::validate_name("../..//evil").is_err());
        assert!(SessionManager::validate_name("name/path").is_err());
        assert!(SessionManager::validate_name("name\\path").is_err());
    }

    #[test]
    fn test_validate_name_special_chars() {
        assert!(SessionManager::validate_name("name@invalid").is_err());
        assert!(SessionManager::validate_name("name!invalid").is_err());
        assert!(SessionManager::validate_name("name invalid").is_err());
        assert!(SessionManager::validate_name("name.json").is_err());
    }

    #[test]
    fn test_create_session() {
        let manager = SessionManager::new().unwrap();
        let session = manager
            .create_session(
                "test-session".to_string(),
                "claude".to_string(),
                "Test Topic".to_string(),
            )
            .unwrap();

        assert_eq!(session.name, "test-session");
        assert_eq!(session.cli, "claude");
        assert_eq!(session.topic, "Test Topic");
        assert_eq!(session.turns.len(), 0);
    }

    #[test]
    fn test_add_turn() {
        let manager = SessionManager::new().unwrap();
        let mut session = manager
            .create_session(
                "test-session".to_string(),
                "claude".to_string(),
                "Test Topic".to_string(),
            )
            .unwrap();

        manager.add_turn(&mut session, "user".to_string(), "Hello".to_string());
        assert_eq!(session.turns.len(), 1);
        assert_eq!(session.turns[0].role, "user");
        assert_eq!(session.turns[0].content, "Hello");

        manager.add_turn(
            &mut session,
            "assistant".to_string(),
            "Hi there!".to_string(),
        );
        assert_eq!(session.turns.len(), 2);
        assert_eq!(session.turns[1].role, "assistant");
        assert_eq!(session.turns[1].content, "Hi there!");
    }

    #[test]
    fn test_build_prompt_with_empty_history() {
        let manager = SessionManager::new().unwrap();
        let session = manager
            .create_session(
                "test-session".to_string(),
                "claude".to_string(),
                "Test Topic".to_string(),
            )
            .unwrap();

        let prompt = manager.build_prompt_with_history(&session, "My question");
        assert_eq!(prompt, "My question");
    }

    #[test]
    fn test_build_prompt_with_history() {
        let manager = SessionManager::new().unwrap();
        let mut session = manager
            .create_session(
                "test-session".to_string(),
                "claude".to_string(),
                "Test Topic".to_string(),
            )
            .unwrap();

        manager.add_turn(
            &mut session,
            "user".to_string(),
            "Explain closures".to_string(),
        );
        manager.add_turn(
            &mut session,
            "assistant".to_string(),
            "Closures are functions that capture variables from their enclosing scope.".to_string(),
        );

        let prompt = manager.build_prompt_with_history(&session, "Give an example");

        // Verify structure
        assert!(prompt.contains("[CONVERSATION HISTORY]"));
        assert!(prompt.contains("[/CONVERSATION HISTORY]"));
        assert!(prompt.contains("[CURRENT REQUEST]"));
        assert!(prompt.contains("[/CURRENT REQUEST]"));

        // Verify content
        assert!(prompt.contains("User: Explain closures"));
        assert!(prompt.contains("Assistant: Closures are functions that capture variables"));
        assert!(prompt.contains("Give an example"));

        // Verify order
        let hist_start = prompt.find("[CONVERSATION HISTORY]").unwrap();
        let hist_end = prompt.find("[/CONVERSATION HISTORY]").unwrap();
        let req_start = prompt.find("[CURRENT REQUEST]").unwrap();
        assert!(hist_start < hist_end);
        assert!(hist_end < req_start);
    }

    #[test]
    fn test_prompt_injection_multiple_turns() {
        let manager = SessionManager::new().unwrap();
        let mut session = manager
            .create_session(
                "test".to_string(),
                "claude".to_string(),
                "Topic".to_string(),
            )
            .unwrap();

        // Add 3 turns
        for i in 0..3 {
            manager.add_turn(&mut session, "user".to_string(), format!("Question {}", i));
            manager.add_turn(
                &mut session,
                "assistant".to_string(),
                format!("Answer {}", i),
            );
        }

        let prompt = manager.build_prompt_with_history(&session, "New question");

        // All previous turns should be in history
        for i in 0..3 {
            assert!(
                prompt.contains(&format!("Question {}", i)),
                "Missing question {}",
                i
            );
            assert!(
                prompt.contains(&format!("Answer {}", i)),
                "Missing answer {}",
                i
            );
        }

        // New question should be in current request
        assert!(prompt.contains("New question"));
    }

    #[test]
    fn test_session_serialization() {
        let now = Utc::now();
        let session = Session {
            name: "test".to_string(),
            cli: "claude".to_string(),
            topic: "Test".to_string(),
            created_at: now,
            last_interaction: now,
            turns: vec![
                Turn {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                    timestamp: now,
                },
                Turn {
                    role: "assistant".to_string(),
                    content: "Hi".to_string(),
                    timestamp: now,
                },
            ],
        };

        // Serialize
        let json = serde_json::to_string(&session).unwrap();

        // Deserialize
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, session.name);
        assert_eq!(deserialized.cli, session.cli);
        assert_eq!(deserialized.topic, session.topic);
        assert_eq!(deserialized.turns.len(), 2);
    }
}
