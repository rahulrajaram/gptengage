//! Claude Code CLI invoker

use super::base::{command_exists, execute_command_report};
use super::{AccessMode, InvocationOutcome, InvocationReport, Invoker};
use async_trait::async_trait;

#[derive(Clone)]
pub struct ClaudeInvoker;

#[async_trait]
impl Invoker for ClaudeInvoker {
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
        report.command = Some("claude".into());
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

        let mut args: Vec<&str> = vec!["-p"];

        // Add model if specified
        // Example models: claude-sonnet-4-20250514, claude-opus-4-20250514
        if let Some(m) = model {
            args.push("--model");
            args.push(m);
        }

        let access_start = args.len();
        // Add access mode flags
        match access_mode {
            AccessMode::ReadOnly => {
                args.extend_from_slice(&["--tools", "Read", "--allowed-tools", "Read"]);
            }
            AccessMode::WorkspaceWrite => {
                args.extend_from_slice(&["--tools", "Read,Edit", "--allowed-tools", "Read,Edit"]);
            }
        };

        report.access.submitted_args = args[access_start..]
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect();
        execute_command_report("claude", &args, prompt, timeout, report).await
    }

    fn name(&self) -> &str {
        "claude"
    }

    fn is_available(&self) -> bool {
        command_exists("claude")
    }
}

impl Default for ClaudeInvoker {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeInvoker {
    pub fn new() -> Self {
        ClaudeInvoker
    }
}
