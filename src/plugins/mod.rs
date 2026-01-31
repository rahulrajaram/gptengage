//! Plugin system for custom CLI integrations
//!
//! Plugins allow users to add custom LLM CLIs without modifying GPT Engage source code.
//! Each plugin defines how to invoke a CLI, including command, arguments, and access modes.
//!
//! Plugin files are stored as TOML in `~/.gptengage/plugins/`.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Plugin configuration loaded from a TOML file
#[derive(Debug, Clone, Deserialize)]
pub struct PluginConfig {
    pub plugin: PluginMeta,
    pub invoke: InvokeConfig,
    pub access: AccessConfig,
    pub detection: DetectionConfig,
}

/// Plugin metadata
#[derive(Debug, Clone, Deserialize)]
pub struct PluginMeta {
    /// Unique plugin name (used as CLI identifier)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Command to execute
    pub command: String,
}

/// Invocation configuration
#[derive(Debug, Clone, Deserialize)]
pub struct InvokeConfig {
    /// Base arguments passed to the command
    pub base_args: Vec<String>,
    /// How to pass the prompt to the CLI
    pub prompt_mode: PromptMode,
    /// Argument flag for prompt (used with Arg mode)
    pub prompt_arg: Option<String>,
}

/// How the prompt is passed to the CLI
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PromptMode {
    /// Pass prompt via stdin
    Stdin,
    /// Pass prompt as a named argument (uses prompt_arg)
    Arg,
    /// Pass prompt as the last positional argument
    ArgLast,
}

/// Access mode configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AccessConfig {
    /// Additional arguments for read-only mode
    #[serde(default)]
    pub readonly_args: Vec<String>,
    /// Additional arguments for write mode
    #[serde(default)]
    pub write_args: Vec<String>,
}

/// CLI detection configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DetectionConfig {
    /// Command to check for availability
    pub check_command: String,
    /// Arguments for the check command
    #[serde(default)]
    pub check_args: Vec<String>,
}

/// Manages loading and accessing plugins
pub struct PluginManager {
    plugins_dir: PathBuf,
    plugins: HashMap<String, PluginConfig>,
}

impl PluginManager {
    /// Create a new PluginManager and load plugins from the default directory
    pub fn new() -> Result<Self> {
        let plugins_dir = Self::get_plugins_dir()?;
        let mut manager = Self {
            plugins_dir,
            plugins: HashMap::new(),
        };
        manager.load_plugins()?;
        Ok(manager)
    }

    /// Get the plugins directory path
    fn get_plugins_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".gptengage").join("plugins"))
    }

    /// Load all plugins from the plugins directory
    pub fn load_plugins(&mut self) -> Result<()> {
        self.plugins.clear();

        if !self.plugins_dir.exists() {
            // No plugins directory, nothing to load
            return Ok(());
        }

        let entries =
            std::fs::read_dir(&self.plugins_dir).context("Failed to read plugins directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                match self.load_plugin_file(&path) {
                    Ok(config) => {
                        self.plugins.insert(config.plugin.name.clone(), config);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load plugin {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single plugin from a TOML file
    fn load_plugin_file(&self, path: &PathBuf) -> Result<PluginConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read plugin file: {}", path.display()))?;

        let config: PluginConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse plugin file: {}", path.display()))?;

        self.validate_plugin(&config)?;

        Ok(config)
    }

    /// Validate a plugin configuration
    fn validate_plugin(&self, config: &PluginConfig) -> Result<()> {
        if config.plugin.name.is_empty() {
            anyhow::bail!("Plugin name cannot be empty");
        }

        if config.plugin.command.is_empty() {
            anyhow::bail!("Plugin command cannot be empty");
        }

        // Ensure name doesn't conflict with built-in CLIs
        let reserved = ["claude", "codex", "gemini"];
        if reserved.contains(&config.plugin.name.to_lowercase().as_str()) {
            anyhow::bail!(
                "Plugin name '{}' conflicts with built-in CLI",
                config.plugin.name
            );
        }

        Ok(())
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&PluginConfig> {
        self.plugins.get(name)
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<&PluginConfig> {
        self.plugins.values().collect()
    }

    /// Check if a plugin exists
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Validate a plugin file without loading it into the manager
    pub fn validate_plugin_file(path: &str) -> Result<PluginConfig> {
        let path = PathBuf::from(path);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read plugin file: {}", path.display()))?;

        let config: PluginConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse plugin file: {}", path.display()))?;

        // Basic validation
        if config.plugin.name.is_empty() {
            anyhow::bail!("Plugin name cannot be empty");
        }

        if config.plugin.command.is_empty() {
            anyhow::bail!("Plugin command cannot be empty");
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plugin_config() {
        let toml_content = r#"
[plugin]
name = "test-cli"
description = "A test CLI"
command = "test-cmd"

[invoke]
base_args = ["run"]
prompt_mode = "stdin"

[access]
readonly_args = ["--readonly"]
write_args = ["--write"]

[detection]
check_command = "test-cmd"
check_args = ["--version"]
"#;

        let config: PluginConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.plugin.name, "test-cli");
        assert_eq!(config.plugin.command, "test-cmd");
        assert_eq!(config.invoke.prompt_mode, PromptMode::Stdin);
        assert_eq!(config.access.readonly_args, vec!["--readonly"]);
    }

    #[test]
    fn test_parse_arg_last_mode() {
        let toml_content = r#"
[plugin]
name = "test"
description = "Test"
command = "cmd"

[invoke]
base_args = ["--message"]
prompt_mode = "arg_last"

[access]

[detection]
check_command = "cmd"
"#;

        let config: PluginConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.invoke.prompt_mode, PromptMode::ArgLast);
    }

    #[test]
    fn test_parse_arg_mode_with_flag() {
        let toml_content = r#"
[plugin]
name = "test"
description = "Test"
command = "cmd"

[invoke]
base_args = []
prompt_mode = "arg"
prompt_arg = "-p"

[access]

[detection]
check_command = "cmd"
"#;

        let config: PluginConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.invoke.prompt_mode, PromptMode::Arg);
        assert_eq!(config.invoke.prompt_arg, Some("-p".to_string()));
    }
}
