//! Plugin-based CLI invoker
//!
//! Allows invoking custom CLIs defined via TOML plugin files.

use super::base::{command_exists, execute_command};
use super::{AccessMode, Invoker};
use crate::plugins::{PluginConfig, PromptMode};
use async_trait::async_trait;

/// Invoker for plugin-defined CLIs
#[derive(Clone)]
pub struct PluginInvoker {
    config: PluginConfig,
}

impl PluginInvoker {
    /// Create a new PluginInvoker from a plugin configuration
    pub fn new(config: PluginConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Invoker for PluginInvoker {
    async fn invoke(
        &self,
        prompt: &str,
        timeout: u64,
        access_mode: AccessMode,
    ) -> anyhow::Result<String> {
        // Build argument list
        let mut args: Vec<String> = self.config.invoke.base_args.clone();

        // Add access mode arguments
        match access_mode {
            AccessMode::ReadOnly => {
                args.extend(self.config.access.readonly_args.clone());
            }
            AccessMode::WorkspaceWrite => {
                args.extend(self.config.access.write_args.clone());
            }
        }

        // Handle prompt based on mode
        let input = match self.config.invoke.prompt_mode {
            PromptMode::Stdin => {
                // Prompt passed via stdin
                prompt.to_string()
            }
            PromptMode::Arg => {
                // Prompt passed as named argument
                if let Some(ref arg) = self.config.invoke.prompt_arg {
                    args.push(arg.clone());
                }
                args.push(prompt.to_string());
                String::new()
            }
            PromptMode::ArgLast => {
                // Prompt passed as last positional argument
                args.push(prompt.to_string());
                String::new()
            }
        };

        // Convert Vec<String> to Vec<&str> for execute_command
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        execute_command(&self.config.plugin.command, &args_ref, &input, timeout).await
    }

    fn name(&self) -> &str {
        &self.config.plugin.name
    }

    fn is_available(&self) -> bool {
        command_exists(&self.config.detection.check_command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{AccessConfig, DetectionConfig, InvokeConfig, PluginMeta};

    fn create_test_config() -> PluginConfig {
        PluginConfig {
            plugin: PluginMeta {
                name: "test-plugin".to_string(),
                description: "A test plugin".to_string(),
                command: "echo".to_string(),
            },
            invoke: InvokeConfig {
                base_args: vec![],
                prompt_mode: PromptMode::ArgLast,
                prompt_arg: None,
            },
            access: AccessConfig {
                readonly_args: vec![],
                write_args: vec![],
            },
            detection: DetectionConfig {
                check_command: "echo".to_string(),
                check_args: vec![],
            },
        }
    }

    #[test]
    fn test_plugin_invoker_name() {
        let config = create_test_config();
        let invoker = PluginInvoker::new(config);
        assert_eq!(invoker.name(), "test-plugin");
    }

    #[test]
    fn test_plugin_invoker_is_available() {
        let config = create_test_config();
        let invoker = PluginInvoker::new(config);
        // echo should always be available
        assert!(invoker.is_available());
    }

    #[tokio::test]
    async fn test_plugin_invoker_invoke() {
        let config = create_test_config();
        let invoker = PluginInvoker::new(config);

        let result = invoker
            .invoke("hello world", 30, AccessMode::ReadOnly)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello world"));
    }
}
