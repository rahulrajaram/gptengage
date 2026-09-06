//! Codex CLI invoker

use super::base::{command_exists, execute_command_report};
use super::{AccessMode, InvocationOutcome, InvocationReport, Invoker};
use async_trait::async_trait;

#[derive(Clone)]
pub struct CodexInvoker;

#[async_trait]
impl Invoker for CodexInvoker {
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
        report.command = Some("codex".into());
        report.capabilities.model_selection = Some(true);
        report.capabilities.provenance = "builtin_adapter".into();
        report.access.provenance = "builtin_adapter".into();
        if let Err(error) = self.validate_model(model) {
            return InvocationReport {
                outcome: InvocationOutcome::Rejected,
                diagnostic: Some(error.to_string()),
                ..report
            };
        }

        let mut args: Vec<&str> = vec!["exec"];

        // Add model if specified
        // Example models: gpt-4o, gpt-4.1, o3
        if let Some(m) = model {
            args.push("--model");
            args.push(m);
        }

        let access_start = args.len();
        // Add access mode flags
        match access_mode {
            AccessMode::ReadOnly => {
                args.extend_from_slice(&["--sandbox", "read-only", "--cd", "."]);
            }
            AccessMode::WorkspaceWrite => {
                args.extend_from_slice(&["--sandbox", "workspace-write", "--cd", "."]);
            }
        };

        report.access.submitted_args = args[access_start..]
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect();
        execute_command_report("codex", &args, prompt, timeout, report).await
    }

    fn name(&self) -> &str {
        "codex"
    }

    fn is_available(&self) -> bool {
        command_exists("codex")
    }
}

impl Default for CodexInvoker {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexInvoker {
    pub fn new() -> Self {
        CodexInvoker
    }
}
