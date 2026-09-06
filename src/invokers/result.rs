//! Observable invocation facts. Unreported provider facts remain explicitly unknown.
use super::AccessMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvocationOutcome {
    Succeeded,
    Failed,
    TimedOut,
    Rejected,
    SpawnFailed,
    IoFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationCapabilities {
    pub model_selection: Option<bool>,
    pub provenance: String,
    pub usage_reporting: Option<bool>,
    pub effective_model_reporting: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessObservation {
    pub requested: AccessMode,
    pub submitted_args: Vec<String>,
    pub provenance: String,
    /// None means enforcement has not been verified.
    pub enforced: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationReport {
    pub schema_version: u32,
    pub backend: String,
    pub command: Option<String>,
    pub requested_model: Option<String>,
    pub effective_model: Option<String>,
    pub provider: Option<String>,
    pub identity_provenance: String,
    pub completion: Option<String>,
    pub usage: Option<Usage>,
    pub capabilities: InvocationCapabilities,
    pub access: AccessObservation,
    pub outcome: InvocationOutcome,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub elapsed_ms: u64,
    pub diagnostic: Option<String>,
}

impl InvocationReport {
    pub fn unknown(backend: &str, model: Option<&str>, access: AccessMode) -> Self {
        Self {
            schema_version: 1,
            backend: backend.into(),
            command: None,
            requested_model: model.map(str::to_owned),
            effective_model: None,
            provider: None,
            identity_provenance: "unknown".into(),
            completion: None,
            usage: None,
            capabilities: InvocationCapabilities {
                model_selection: None,
                provenance: "unknown".into(),
                usage_reporting: None,
                effective_model_reporting: None,
            },
            access: AccessObservation {
                requested: access,
                submitted_args: vec![],
                provenance: "unknown".into(),
                enforced: None,
            },
            outcome: InvocationOutcome::Rejected,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            elapsed_ms: 0,
            diagnostic: None,
        }
    }
    pub fn is_success(&self) -> bool {
        self.outcome == InvocationOutcome::Succeeded
    }
    pub fn into_text(self) -> anyhow::Result<String> {
        if self.is_success() {
            Ok(self.stdout)
        } else {
            Err(anyhow::anyhow!(format!(
                "{}{}",
                self.diagnostic
                    .unwrap_or_else(|| format!("{} invocation {:?}", self.backend, self.outcome)),
                if self.stderr.is_empty() {
                    String::new()
                } else {
                    format!(": {}", self.stderr)
                }
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_metadata_serializes_as_null_not_zero_or_inference() {
        let report = InvocationReport::unknown("fixture", Some("requested"), AccessMode::ReadOnly);
        let value = serde_json::to_value(&report).unwrap();
        for field in ["effective_model", "provider", "usage", "completion"] {
            assert!(value[field].is_null());
        }
        assert_eq!(value["requested_model"], "requested");
        assert!(value["access"]["enforced"].is_null());
    }
}
