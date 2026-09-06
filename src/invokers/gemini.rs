//! Gemini CLI invoker

use super::base::{command_exists, execute_command_report};
use super::{AccessMode, InvocationOutcome, InvocationReport, Invoker};
use async_trait::async_trait;

#[derive(Clone)]
pub struct GeminiInvoker;

#[async_trait]
impl Invoker for GeminiInvoker {
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
        report.command = Some("gemini".into());
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

        let mut args: Vec<&str> = Vec::new();

        // Add model if specified
        // Example models: gemini-2.5-pro, gemini-2.0-flash
        if let Some(m) = model {
            args.push("--model");
            args.push(m);
        }

        let access_start = args.len();
        // Add access mode flags
        match access_mode {
            AccessMode::ReadOnly => {
                args.extend_from_slice(&["--sandbox", "--include-directories", "."]);
            }
            AccessMode::WorkspaceWrite => {
                args.extend_from_slice(&[
                    "--sandbox",
                    "--include-directories",
                    ".",
                    "--approval-mode",
                    "auto_edit",
                ]);
            }
        };

        report.access.submitted_args = args[access_start..]
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect();
        execute_command_report("gemini", &args, prompt, timeout, report).await
    }

    fn name(&self) -> &str {
        "gemini"
    }

    fn is_available(&self) -> bool {
        command_exists("gemini")
    }
}

impl Default for GeminiInvoker {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiInvoker {
    pub fn new() -> Self {
        GeminiInvoker
    }
}
