//! Plugin-based CLI invoker
//!
//! Allows invoking custom CLIs defined via TOML plugin files.

use super::base::{command_exists, execute_command_report};
use super::{AccessMode, InvocationOutcome, InvocationReport, Invoker};
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
        model: Option<&str>,
    ) -> anyhow::Result<String> {
        self.invoke_report(prompt, timeout, access_mode, model)
            .await
            .into_text()
    }

    async fn invoke_report(
        &self,
        prompt: &str,
        timeout: u64,
        access_mode: AccessMode,
        model: Option<&str>,
    ) -> InvocationReport {
        let mut report = InvocationReport::unknown(self.name(), model, access_mode);
        report.command = Some(self.config.plugin.command.clone());
        report.capabilities.model_selection = Some(
            self.config
                .invoke
                .model_arg
                .as_ref()
                .is_some_and(|arg| !arg.trim().is_empty()),
        );
        report.capabilities.provenance = "plugin_configuration".into();
        report.access.provenance = "plugin_configuration".into();
        if let Err(error) = self.validate_model(model) {
            return InvocationReport {
                outcome: InvocationOutcome::Rejected,
                diagnostic: Some(error.to_string()),
                ..report
            };
        }

        // Build argument list
        let mut args: Vec<String> = self.config.invoke.base_args.clone();

        // Add model if specified and plugin supports it
        if let Some(m) = model {
            if let Some(ref model_arg) = self.config.invoke.model_arg {
                args.push(model_arg.clone());
                args.push(m.to_string());
            }
        }

        report.access.submitted_args = match access_mode {
            AccessMode::ReadOnly => self.config.access.readonly_args.clone(),
            AccessMode::WorkspaceWrite => self.config.access.write_args.clone(),
        };
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

        execute_command_report(
            &self.config.plugin.command,
            &args_ref,
            &input,
            timeout,
            report,
        )
        .await
    }

    fn validate_model(&self, model: Option<&str>) -> anyhow::Result<()> {
        if model.is_some_and(|model| model.trim().is_empty()) {
            anyhow::bail!("Explicit model must not be empty");
        }
        if model.is_some()
            && !self
                .config
                .invoke
                .model_arg
                .as_ref()
                .is_some_and(|arg| !arg.trim().is_empty())
        {
            anyhow::bail!(
                "Plugin '{}' does not support explicit model selection: configure invoke.model_arg",
                self.name()
            );
        }
        Ok(())
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
                model_arg: None,
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
            .invoke("hello world", 30, AccessMode::ReadOnly, None)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello world"));
    }

    #[tokio::test]
    async fn test_stdin_mode_round_trips_prompt_larger_than_argv_string_limit() {
        let config = PluginConfig {
            plugin: PluginMeta {
                name: "stdin-test".to_string(),
                description: "large stdin transport check".to_string(),
                command: "cat".to_string(),
            },
            invoke: InvokeConfig {
                base_args: vec![],
                prompt_mode: PromptMode::Stdin,
                prompt_arg: None,
                model_arg: None,
            },
            access: AccessConfig {
                readonly_args: vec![],
                write_args: vec![],
            },
            detection: DetectionConfig {
                check_command: "cat".to_string(),
                check_args: vec![],
            },
        };
        let invoker = PluginInvoker::new(config);
        // Linux commonly limits a single argv string to 128 KiB even when the
        // aggregate ARG_MAX is larger. Stdin must carry this prompt losslessly.
        let prompt = format!("BEGIN\n{}\nEND", "x".repeat(256 * 1024));

        let result = invoker
            .invoke(&prompt, 30, AccessMode::ReadOnly, None)
            .await
            .expect("large stdin prompt failed");

        assert_eq!(result, prompt);
    }

    #[tokio::test]
    async fn test_invoker_access_args_after_model_and_prompt_last() {
        // Verify argument ordering: base_args -> model pair -> access args ->
        // prompt (last, in arg_last mode). The single --tools comes from the
        // access-mode args only.
        let config = PluginConfig {
            plugin: PluginMeta {
                name: "echo".to_string(),
                description: "arg order check".to_string(),
                command: "echo".to_string(),
            },
            invoke: InvokeConfig {
                base_args: vec!["BASE".to_string()],
                prompt_mode: PromptMode::ArgLast,
                prompt_arg: None,
                model_arg: Some("--model".to_string()),
            },
            access: AccessConfig {
                readonly_args: vec!["--tools".to_string(), "ro-set".to_string()],
                write_args: vec!["--tools".to_string(), "w-set".to_string()],
            },
            detection: DetectionConfig {
                check_command: "echo".to_string(),
                check_args: vec![],
            },
        };
        let invoker = PluginInvoker::new(config);

        let ro = invoker
            .invoke("PROMPT", 30, AccessMode::ReadOnly, Some("m"))
            .await
            .expect("read-only invoke failed");
        assert_eq!(ro.trim(), "BASE --model m --tools ro-set PROMPT");

        let w = invoker
            .invoke("PROMPT", 30, AccessMode::WorkspaceWrite, Some("m"))
            .await
            .expect("write invoke failed");
        assert_eq!(w.trim(), "BASE --model m --tools w-set PROMPT");
    }
    #[tokio::test]
    async fn rejects_unsupported_model_before_launch() {
        let mut config = create_test_config();
        config.plugin.command = "/nonexistent/would-fail-if-launched".into();
        let report = PluginInvoker::new(config)
            .invoke_report("prompt", 5, AccessMode::ReadOnly, Some("chosen"))
            .await;
        assert_eq!(report.outcome, InvocationOutcome::Rejected);
        assert_eq!(report.requested_model.as_deref(), Some("chosen"));
        assert!(report.diagnostic.unwrap().contains("model_arg"));
    }

    #[tokio::test]
    async fn forwarding_is_not_observed_identity_or_enforcement() {
        let mut config = create_test_config();
        config.invoke.model_arg = Some("--model".into());
        let report = PluginInvoker::new(config)
            .invoke_report("prompt", 5, AccessMode::ReadOnly, Some("chosen"))
            .await;
        assert!(report.is_success());
        assert!(report.stdout.contains("--model chosen"));
        assert_eq!(report.capabilities.model_selection, Some(true));
        assert_eq!(report.capabilities.provenance, "plugin_configuration");
        assert!(report.effective_model.is_none());
        assert!(report.provider.is_none());
        assert!(report.usage.is_none());
        assert!(report.access.enforced.is_none());
    }

    #[tokio::test]
    async fn rejects_blank_model_and_blank_model_flag() {
        let mut config = create_test_config();
        config.invoke.model_arg = Some("--model".into());
        let report = PluginInvoker::new(config.clone())
            .invoke_report("prompt", 5, AccessMode::ReadOnly, Some(" "))
            .await;
        assert_eq!(report.outcome, InvocationOutcome::Rejected);
        config.invoke.model_arg = Some(" ".into());
        let report = PluginInvoker::new(config)
            .invoke_report("prompt", 5, AccessMode::ReadOnly, Some("chosen"))
            .await;
        assert_eq!(report.outcome, InvocationOutcome::Rejected);
    }

    #[test]
    fn preflight_rejects_unsupported_model_without_process() {
        let mut config = create_test_config();
        config.plugin.command = "/nonexistent/no-launch".into();
        let invoker = PluginInvoker::new(config);
        assert!(invoker.validate_model(None).is_ok());
        assert!(invoker.validate_model(Some("chosen")).is_err());
        assert!(invoker.validate_model(Some(" ")).is_err());
    }
}
