//! Configuration management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigManager {
    pub default_timeout: u64,
    pub default_debate_rounds: usize,
    pub clis: std::collections::HashMap<String, CliConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub command: String,
    pub invoke_args: Vec<String>,
    pub detected: bool,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("config.json");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: ConfigManager = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = ConfigManager {
                default_timeout: 120,
                default_debate_rounds: 3,
                clis: std::collections::HashMap::new(),
            };

            // Create config dir if needed
            if !config_dir.exists() {
                std::fs::create_dir_all(&config_dir)?;
            }

            // Write default config
            let content = serde_json::to_string_pretty(&config)?;
            std::fs::write(&config_path, content)?;

            Ok(config)
        }
    }

    pub fn get_config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .ok()
            .and_then(|h| {
                if h.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(h))
                }
            })
            .or_else(|| {
                // Fallback for Windows
                std::env::var("USERPROFILE").ok().and_then(|h| {
                    if h.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(h))
                    }
                })
            })
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".gptengage"))
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "default_timeout" => Some(self.default_timeout.to_string()),
            "default_debate_rounds" => Some(self.default_debate_rounds.to_string()),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "default_timeout" => {
                self.default_timeout = value.parse()?;
            }
            "default_debate_rounds" => {
                self.default_debate_rounds = value.parse()?;
            }
            _ => return Err(anyhow::anyhow!("Unknown config key: {}", key)),
        }
        // Save config to disk after updating
        self.save()?;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("config.json");

        // Create config dir if needed
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        // Write config
        let content = serde_json::to_string_pretty(&self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}
