//! Debate templates for pre-configured debate scenarios
//!
//! Templates provide ready-to-use debate configurations with pre-defined
//! participants, personas, and context prompts for common use cases.

mod builtin;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub use builtin::get_builtin_templates;

/// A debate template with pre-configured participants and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateTemplate {
    /// Unique template name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Default number of rounds
    pub default_rounds: usize,
    /// Pre-configured participants
    pub participants: Vec<TemplateParticipant>,
    /// Optional context configuration
    #[serde(default)]
    pub context: Option<TemplateContext>,
}

/// A participant defined in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParticipant {
    /// CLI to use (claude, codex, gemini, or plugin name)
    pub cli: String,
    /// Persona name
    pub persona: String,
    /// Instructions for this participant
    pub instructions: String,
    /// Areas of expertise
    #[serde(default)]
    pub expertise: Vec<String>,
}

/// Context configuration for a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    /// Text to prepend to the topic
    pub prefix: Option<String>,
    /// Text to append to the topic
    pub suffix: Option<String>,
}

/// Summary information about a template
#[derive(Debug, Clone)]
pub struct TemplateSummary {
    pub name: String,
    pub description: String,
    pub participant_count: usize,
    pub default_rounds: usize,
    pub is_builtin: bool,
}

/// Manages loading and accessing templates
pub struct TemplateManager {
    builtin_templates: HashMap<String, DebateTemplate>,
    user_templates: HashMap<String, DebateTemplate>,
    user_templates_dir: PathBuf,
}

impl TemplateManager {
    /// Create a new TemplateManager with built-in templates and user templates
    pub fn new() -> Result<Self> {
        let builtin_templates = get_builtin_templates();
        let user_templates_dir = Self::get_templates_dir()?;

        let mut manager = Self {
            builtin_templates,
            user_templates: HashMap::new(),
            user_templates_dir,
        };

        // Load user templates (non-fatal if directory doesn't exist)
        let _ = manager.load_user_templates();

        Ok(manager)
    }

    /// Get the user templates directory path
    fn get_templates_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".gptengage").join("templates"))
    }

    /// Load user templates from the templates directory
    pub fn load_user_templates(&mut self) -> Result<()> {
        self.user_templates.clear();

        if !self.user_templates_dir.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.user_templates_dir)
            .context("Failed to read templates directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                match self.load_template_file(&path) {
                    Ok(template) => {
                        self.user_templates.insert(template.name.clone(), template);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load template {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single template from a TOML file
    fn load_template_file(&self, path: &PathBuf) -> Result<DebateTemplate> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read template file: {}", path.display()))?;

        // Parse the TOML wrapper
        #[derive(Deserialize)]
        struct TemplateWrapper {
            template: TemplateInner,
            #[serde(default)]
            participants: Vec<TemplateParticipant>,
            #[serde(default)]
            context: Option<TemplateContext>,
        }

        #[derive(Deserialize)]
        struct TemplateInner {
            name: String,
            description: String,
            default_rounds: usize,
        }

        let wrapper: TemplateWrapper = toml::from_str(&content)
            .with_context(|| format!("Failed to parse template file: {}", path.display()))?;

        let template = DebateTemplate {
            name: wrapper.template.name,
            description: wrapper.template.description,
            default_rounds: wrapper.template.default_rounds,
            participants: wrapper.participants,
            context: wrapper.context,
        };

        self.validate_template(&template)?;

        Ok(template)
    }

    /// Validate a template
    fn validate_template(&self, template: &DebateTemplate) -> Result<()> {
        if template.name.is_empty() {
            anyhow::bail!("Template name cannot be empty");
        }

        if template.participants.is_empty() {
            anyhow::bail!("Template must have at least one participant");
        }

        for (i, p) in template.participants.iter().enumerate() {
            if p.cli.is_empty() {
                anyhow::bail!("Participant {} has empty CLI", i + 1);
            }
            if p.persona.is_empty() {
                anyhow::bail!("Participant {} has empty persona", i + 1);
            }
            if p.instructions.len() < 10 {
                anyhow::bail!(
                    "Participant {} instructions must be at least 10 characters",
                    i + 1
                );
            }
        }

        Ok(())
    }

    /// Get a template by name (checks user templates first, then built-in)
    pub fn get_template(&self, name: &str) -> Option<&DebateTemplate> {
        self.user_templates
            .get(name)
            .or_else(|| self.builtin_templates.get(name))
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<TemplateSummary> {
        let mut summaries = Vec::new();

        // Add built-in templates
        for template in self.builtin_templates.values() {
            summaries.push(TemplateSummary {
                name: template.name.clone(),
                description: template.description.clone(),
                participant_count: template.participants.len(),
                default_rounds: template.default_rounds,
                is_builtin: true,
            });
        }

        // Add user templates (may override built-in)
        for template in self.user_templates.values() {
            // Remove built-in if user has override
            summaries.retain(|s| s.name != template.name);
            summaries.push(TemplateSummary {
                name: template.name.clone(),
                description: template.description.clone(),
                participant_count: template.participants.len(),
                default_rounds: template.default_rounds,
                is_builtin: false,
            });
        }

        // Sort by name
        summaries.sort_by(|a, b| a.name.cmp(&b.name));

        summaries
    }
}

impl DebateTemplate {
    /// Convert template participants to orchestrator participants
    pub fn to_participants(&self) -> Vec<crate::orchestrator::Participant> {
        self.participants
            .iter()
            .map(|p| {
                let agent_def = crate::orchestrator::AgentDefinition {
                    cli: p.cli.clone(),
                    persona: p.persona.clone(),
                    instructions: p.instructions.clone(),
                    expertise: p.expertise.clone(),
                    communication_style: None,
                };
                agent_def.to_participant()
            })
            .collect()
    }

    /// Apply context prefix/suffix to a topic
    pub fn apply_context(&self, topic: &str) -> String {
        let mut result = String::new();

        if let Some(ref ctx) = self.context {
            if let Some(ref prefix) = ctx.prefix {
                result.push_str(prefix);
                result.push_str("\n\n");
            }
        }

        result.push_str(topic);

        if let Some(ref ctx) = self.context {
            if let Some(ref suffix) = ctx.suffix {
                result.push_str("\n\n");
                result.push_str(suffix);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager_has_builtin_templates() {
        let manager = TemplateManager::new().unwrap();
        let templates = manager.list_templates();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_get_builtin_template() {
        let manager = TemplateManager::new().unwrap();
        let template = manager.get_template("code-review");
        assert!(template.is_some());
    }

    #[test]
    fn test_apply_context() {
        let template = DebateTemplate {
            name: "test".to_string(),
            description: "Test".to_string(),
            default_rounds: 2,
            participants: vec![],
            context: Some(TemplateContext {
                prefix: Some("PREFIX:".to_string()),
                suffix: Some("SUFFIX.".to_string()),
            }),
        };

        let result = template.apply_context("TOPIC");
        assert!(result.contains("PREFIX:"));
        assert!(result.contains("TOPIC"));
        assert!(result.contains("SUFFIX."));
    }
}
