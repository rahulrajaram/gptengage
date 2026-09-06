//! Sequential grilling with private, immutable checkpoints. No resume or replay.
use std::fs::{self, File, OpenOptions};
#[cfg(unix)]
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context};
use serde::{Deserialize, Serialize};

use crate::invokers::{AccessMode, InvocationOutcome, InvocationReport, Invoker};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Griller,
    Respondent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleParticipant {
    pub backend: String,
    pub model: Option<String>,
    pub instructions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrillConfig {
    pub topic: String,
    pub griller: RoleParticipant,
    pub respondent: RoleParticipant,
    pub exchanges: u32,
    pub timeout: u64,
}

impl GrillConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(!self.topic.trim().is_empty(), "Topic must not be empty");
        ensure!(
            self.exchanges > 0 && self.exchanges.checked_mul(2).is_some(),
            "Exchange count must be positive and fit the direct-call counter"
        );
        ensure!(self.timeout > 0, "Timeout must be positive");
        for participant in [&self.griller, &self.respondent] {
            ensure!(
                !participant.backend.trim().is_empty(),
                "Backend must not be empty"
            );
            ensure!(
                !participant.instructions.trim().is_empty(),
                "Role instructions must not be empty"
            );
            ensure!(
                participant
                    .model
                    .as_ref()
                    .is_none_or(|model| !model.trim().is_empty()),
                "Requested model must not be empty"
            );
        }
        Ok(())
    }

    fn participant(&self, role: Role) -> &RoleParticipant {
        match role {
            Role::Griller => &self.griller,
            Role::Respondent => &self.respondent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Phase {
    Ready { exchange: u32, role: Role },
    Invoking { exchange: u32, role: Role },
    Completed,
    Failed { exchange: u32, role: Role },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateTurn {
    pub exchange: u32,
    pub role: Role,
    pub prompt: String,
    pub report: Option<InvocationReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrillState {
    pub schema_version: u32,
    pub config: GrillConfig,
    pub phase: Phase,
    pub turns: Vec<PrivateTurn>,
}

pub enum Event {
    Begin,
    Finish(Box<InvocationReport>),
}

impl Event {
    fn finish(report: InvocationReport) -> Self {
        Self::Finish(Box::new(report))
    }
}

impl GrillState {
    pub fn new(config: GrillConfig) -> anyhow::Result<Self> {
        config.validate()?;
        Ok(Self {
            schema_version: 1,
            config,
            phase: Phase::Ready {
                exchange: 1,
                role: Role::Griller,
            },
            turns: Vec::new(),
        })
    }

    fn prompt(&self, exchange: u32, role: Role) -> String {
        // Role policy comes from the caller. Only dialogue, never another
        // participant's instructions or process diagnostics, enters history.
        let dialogue: Vec<_> = self
            .turns
            .iter()
            .filter_map(|turn| {
                turn.report
                    .as_ref()
                    .filter(|report| report.is_success())
                    .map(|report| {
                        serde_json::json!({"exchange": turn.exchange, "role": turn.role,
                    "text": report.stdout})
                    })
            })
            .collect();
        serde_json::json!({
            "role": role,
            "instructions": self.config.participant(role).instructions,
            "topic": self.config.topic,
            "exchange": exchange,
            "dialogue": dialogue,
        })
        .to_string()
    }

    /// Structural projection only: model text can repeat private input.
    pub fn public_projection(&self, show_dialogue: bool) -> PublicRun {
        PublicRun {
            schema_version: 1,
            phase: self.phase,
            requested_exchanges: self.config.exchanges,
            direct_calls_started: self.turns.len(),
            turns: self
                .turns
                .iter()
                .map(|turn| PublicTurn {
                    exchange: turn.exchange,
                    role: turn.role,
                    outcome: turn.report.as_ref().map(|report| report.outcome),
                    text: turn
                        .report
                        .as_ref()
                        .filter(|report| show_dialogue && report.is_success())
                        .map(|report| report.stdout.clone()),
                })
                .collect(),
        }
    }
}

/// Pure state change; callers persist its result before executing another call.
pub fn transition(state: &GrillState, event: Event) -> anyhow::Result<GrillState> {
    match (state.phase, event) {
        (Phase::Ready { exchange, role }, Event::Begin) => {
            let turn = PrivateTurn {
                exchange,
                role,
                prompt: state.prompt(exchange, role),
                report: None,
            };
            Ok(GrillState {
                phase: Phase::Invoking { exchange, role },
                turns: state
                    .turns
                    .iter()
                    .cloned()
                    .chain(std::iter::once(turn))
                    .collect(),
                ..state.clone()
            })
        }
        (Phase::Invoking { exchange, role }, Event::Finish(report)) => {
            let (last, previous) = state
                .turns
                .split_last()
                .context("Missing invocation slot")?;
            ensure!(
                last.exchange == exchange && last.role == role && last.report.is_none(),
                "Invocation slot does not match state"
            );
            let phase = if !report.is_success() {
                Phase::Failed { exchange, role }
            } else {
                match role {
                    Role::Griller => Phase::Ready {
                        exchange,
                        role: Role::Respondent,
                    },
                    Role::Respondent if exchange == state.config.exchanges => Phase::Completed,
                    Role::Respondent => Phase::Ready {
                        exchange: exchange + 1,
                        role: Role::Griller,
                    },
                }
            };
            let turn = PrivateTurn {
                report: Some(*report),
                ..last.clone()
            };
            Ok(GrillState {
                phase,
                turns: previous
                    .iter()
                    .cloned()
                    .chain(std::iter::once(turn))
                    .collect(),
                ..state.clone()
            })
        }
        _ => bail!("Event is not valid in the current grilling phase"),
    }
}

#[derive(Debug, Serialize)]
pub struct PublicTurn {
    pub exchange: u32,
    pub role: Role,
    pub outcome: Option<InvocationOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicRun {
    pub schema_version: u32,
    pub phase: Phase,
    pub requested_exchanges: u32,
    pub direct_calls_started: usize,
    pub turns: Vec<PublicTurn>,
}

/// Exclusive directory creation prevents overwriting a prior run, including
/// symlink destinations. Files are private from creation, independent of umask.
#[cfg(unix)]
struct Checkpoints {
    directory: PathBuf,
    sequence: u64,
}

#[cfg(not(unix))]
struct Checkpoints;

impl Checkpoints {
    #[cfg(unix)]
    fn create(directory: &Path) -> anyhow::Result<Self> {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new()
            .mode(0o700)
            .create(directory)
            .context("Cannot create a new private run directory; its parent must exist")?;
        Ok(Self {
            directory: directory.to_path_buf(),
            sequence: 0,
        })
    }

    #[cfg(not(unix))]
    fn create(_directory: &Path) -> anyhow::Result<Self> {
        bail!("Private checkpoint permissions are supported only on Unix")
    }

    #[cfg(unix)]
    fn save(&mut self, state: &GrillState) -> anyhow::Result<PathBuf> {
        use std::os::unix::fs::OpenOptionsExt;
        let staging = self
            .directory
            .join(format!(".checkpoint-{:06}.pending", self.sequence));
        let destination = self
            .directory
            .join(format!("checkpoint-{:06}.json", self.sequence));
        let bytes = serde_json::to_vec_pretty(state)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&staging)
            .context("Cannot create private checkpoint staging file")?;
        file.write_all(&bytes)
            .context("Cannot write private checkpoint")?;
        file.sync_all().context("Cannot flush private checkpoint")?;
        // Same-directory hard-link publication is atomic and refuses an existing
        // destination, unlike rename. Incomplete staging files are never loaded.
        fs::hard_link(&staging, &destination).context("Cannot publish private checkpoint")?;
        fs::remove_file(&staging).context("Cannot finish private checkpoint publication")?;
        File::open(&self.directory)?
            .sync_all()
            .context("Cannot flush checkpoint directory")?;
        self.sequence += 1;
        Ok(destination)
    }

    #[cfg(not(unix))]
    fn save(&mut self, _state: &GrillState) -> anyhow::Result<PathBuf> {
        bail!("Private checkpoint permissions are supported only on Unix")
    }
}

pub struct GrillRun {
    pub state: GrillState,
    pub last_checkpoint: PathBuf,
}

/// At most 2 * exchanges direct calls, each with the selected invoker timeout.
/// Provider-internal calls/tokens and sandbox enforcement are not guaranteed.
pub async fn run_grilling(
    config: GrillConfig,
    griller: &dyn Invoker,
    respondent: &dyn Invoker,
    directory: &Path,
) -> anyhow::Result<GrillRun> {
    let mut state = GrillState::new(config)?;
    griller
        .validate_model(state.config.griller.model.as_deref())
        .context("Griller model selection is unsupported")?;
    respondent
        .validate_model(state.config.respondent.model.as_deref())
        .context("Respondent model selection is unsupported")?;
    ensure!(griller.is_available(), "Griller backend is unavailable");
    ensure!(
        respondent.is_available(),
        "Respondent backend is unavailable"
    );
    let mut checkpoints = Checkpoints::create(directory)?;
    let mut last_checkpoint = checkpoints.save(&state)?;
    loop {
        let role = match state.phase {
            Phase::Ready { role, .. } => role,
            Phase::Completed | Phase::Failed { .. } => {
                return Ok(GrillRun {
                    state,
                    last_checkpoint,
                })
            }
            Phase::Invoking { .. } => {
                bail!("Unexpected in-flight state; automatic resume is unsupported")
            }
        };
        state = transition(&state, Event::Begin)?;
        // A crash here leaves an explicit in-flight slot; never automatically retry it.
        checkpoints.save(&state)?;
        let participant = state.config.participant(role);
        let invoker = match role {
            Role::Griller => griller,
            Role::Respondent => respondent,
        };
        let prompt = &state
            .turns
            .last()
            .context("Missing invocation prompt")?
            .prompt;
        let report = invoker
            .invoke_report(
                prompt,
                state.config.timeout,
                AccessMode::ReadOnly,
                participant.model.as_deref(),
            )
            .await;
        state = transition(&state, Event::finish(report))?;
        last_checkpoint = checkpoints.save(&state)?;
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone)]
    struct Call {
        role: String,
        prompt: String,
        timeout: u64,
        access: AccessMode,
        model: Option<String>,
    }

    struct FakeInvoker {
        name: &'static str,
        calls: Arc<Mutex<Vec<Call>>>,
        outcome: InvocationOutcome,
        sabotage: Option<PathBuf>,
        checkpoint_directory: PathBuf,
    }

    #[async_trait]
    impl Invoker for FakeInvoker {
        async fn invoke(
            &self,
            _prompt: &str,
            _timeout: u64,
            _access: AccessMode,
            _model: Option<&str>,
        ) -> anyhow::Result<String> {
            bail!("The orchestrator must consume typed reports")
        }

        async fn invoke_report(
            &self,
            prompt: &str,
            timeout: u64,
            access: AccessMode,
            model: Option<&str>,
        ) -> InvocationReport {
            let calls_before = self.calls.lock().unwrap().len();
            let path = self
                .checkpoint_directory
                .join(format!("checkpoint-{:06}.json", 1 + 2 * calls_before));
            let saved: GrillState = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            assert!(matches!(saved.phase, Phase::Invoking { .. }));
            assert_eq!(saved.turns.last().unwrap().prompt, prompt);
            assert!(saved.turns.last().unwrap().report.is_none());
            self.calls.lock().unwrap().push(Call {
                role: self.name.into(),
                prompt: prompt.into(),
                timeout,
                access,
                model: model.map(str::to_owned),
            });
            if let Some(path) = &self.sabotage {
                fs::write(path, "EXISTING CHECKPOINT").unwrap();
            }
            InvocationReport {
                outcome: self.outcome,
                stdout: format!("DIALOGUE from {}", self.name),
                stderr: "PRIVATE STDERR".into(),
                diagnostic: Some("PRIVATE DIAGNOSTIC".into()),
                ..InvocationReport::unknown(self.name, model, access)
            }
        }

        fn name(&self) -> &str {
            self.name
        }
        fn is_available(&self) -> bool {
            true
        }
        fn validate_model(&self, model: Option<&str>) -> anyhow::Result<()> {
            ensure!(model != Some("unsupported"), "Unsupported fake model");
            Ok(())
        }
    }

    fn config() -> GrillConfig {
        GrillConfig {
            topic: "PRIVATE TOPIC".into(),
            griller: RoleParticipant {
                backend: "griller".into(),
                model: Some("chosen-g".into()),
                instructions: "GRILLER SECRET INSTRUCTIONS".into(),
            },
            respondent: RoleParticipant {
                backend: "respondent".into(),
                model: Some("chosen-r".into()),
                instructions: "RESPONDENT SECRET INSTRUCTIONS".into(),
            },
            exchanges: 2,
            timeout: 7,
        }
    }

    fn fakes(directory: &Path) -> (FakeInvoker, FakeInvoker, Arc<Mutex<Vec<Call>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let make = |name| FakeInvoker {
            name,
            calls: calls.clone(),
            outcome: InvocationOutcome::Succeeded,
            sabotage: None,
            checkpoint_directory: directory.to_path_buf(),
        };
        (make("griller"), make("respondent"), calls)
    }

    #[tokio::test]
    async fn sequential_calls_have_limits_models_and_private_checkpoints() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("run");
        let (griller, respondent, calls) = fakes(&directory);
        let run = run_grilling(config(), &griller, &respondent, &directory)
            .await
            .unwrap();
        assert_eq!(run.state.phase, Phase::Completed);
        let calls = calls.lock().unwrap();
        assert_eq!(
            calls
                .iter()
                .map(|call| call.role.as_str())
                .collect::<Vec<_>>(),
            ["griller", "respondent", "griller", "respondent"]
        );
        assert!(calls
            .iter()
            .all(|call| call.timeout == 7 && call.access == AccessMode::ReadOnly));
        assert_eq!(calls[0].model.as_deref(), Some("chosen-g"));
        assert_eq!(calls[1].model.as_deref(), Some("chosen-r"));
        assert!(!calls[1].prompt.contains("GRILLER SECRET INSTRUCTIONS"));
        assert!(!calls[2].prompt.contains("RESPONDENT SECRET INSTRUCTIONS"));
        assert!(!calls[1].prompt.contains("PRIVATE STDERR"));
        assert!(calls[1].prompt.contains("DIALOGUE from griller"));
        assert_eq!(
            fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 9);
        for entry in fs::read_dir(&directory).unwrap() {
            assert_eq!(
                entry.unwrap().metadata().unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let saved: GrillState =
            serde_json::from_slice(&fs::read(run.last_checkpoint).unwrap()).unwrap();
        assert_eq!(saved.phase, Phase::Completed);
        assert_eq!(saved.turns.len(), 4);
    }

    #[tokio::test]
    async fn invalid_respondent_model_is_rejected_before_griller_spend() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("run");
        let (griller, respondent, calls) = fakes(&directory);
        let config = GrillConfig {
            respondent: RoleParticipant {
                model: Some("unsupported".into()),
                ..config().respondent
            },
            ..config()
        };
        assert!(run_grilling(config, &griller, &respondent, &directory)
            .await
            .is_err());
        assert!(calls.lock().unwrap().is_empty());
        assert!(!directory.exists());
    }

    #[tokio::test]
    async fn failures_stop_and_preserve_failed_slots() {
        for (role, outcome, count) in [
            (Role::Griller, InvocationOutcome::TimedOut, 1),
            (Role::Griller, InvocationOutcome::Rejected, 1),
            (Role::Respondent, InvocationOutcome::Failed, 2),
        ] {
            let temp = tempfile::tempdir().unwrap();
            let directory = temp.path().join("run");
            let (mut griller, mut respondent, calls) = fakes(&directory);
            match role {
                Role::Griller => griller.outcome = outcome,
                Role::Respondent => respondent.outcome = outcome,
            }
            let run = run_grilling(config(), &griller, &respondent, &directory)
                .await
                .unwrap();
            assert_eq!(run.state.phase, Phase::Failed { exchange: 1, role });
            assert_eq!(calls.lock().unwrap().len(), count);
            let saved: GrillState =
                serde_json::from_slice(&fs::read(run.last_checkpoint).unwrap()).unwrap();
            assert_eq!(
                saved.turns.last().unwrap().report.as_ref().unwrap().outcome,
                outcome
            );
            assert_eq!(
                saved.turns.last().unwrap().report.as_ref().unwrap().stderr,
                "PRIVATE STDERR"
            );
        }
    }

    #[tokio::test]
    async fn persistence_failure_stops_and_never_clobbers() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("run");
        let (mut griller, respondent, calls) = fakes(&directory);
        let collision = directory.join("checkpoint-000002.json");
        griller.sabotage = Some(collision.clone());
        assert!(run_grilling(config(), &griller, &respondent, &directory)
            .await
            .is_err());
        assert_eq!(calls.lock().unwrap().len(), 1);
        assert_eq!(
            fs::read_to_string(collision).unwrap(),
            "EXISTING CHECKPOINT"
        );
        let saved: GrillState =
            serde_json::from_slice(&fs::read(directory.join("checkpoint-000001.json")).unwrap())
                .unwrap();
        assert!(matches!(
            saved.phase,
            Phase::Invoking {
                role: Role::Griller,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn existing_run_symlink_and_missing_parent_make_no_calls() {
        let temp = tempfile::tempdir().unwrap();
        let existing = temp.path().join("existing");
        fs::create_dir(&existing).unwrap();
        fs::write(existing.join("sentinel"), "KEEP").unwrap();
        let link = temp.path().join("link");
        symlink(&existing, &link).unwrap();
        for directory in [existing.clone(), link, temp.path().join("absent/run")] {
            let (griller, respondent, calls) = fakes(&directory);
            assert!(run_grilling(config(), &griller, &respondent, &directory)
                .await
                .is_err());
            assert!(calls.lock().unwrap().is_empty());
        }
        assert_eq!(
            fs::read_to_string(existing.join("sentinel")).unwrap(),
            "KEEP"
        );
    }

    #[test]
    fn bounds_and_transition_order_are_checked_without_mutating_input() {
        for config in [
            GrillConfig {
                exchanges: 0,
                ..config()
            },
            GrillConfig {
                exchanges: u32::MAX,
                ..config()
            },
            GrillConfig {
                timeout: 0,
                ..config()
            },
        ] {
            assert!(GrillState::new(config).is_err());
        }
        let state = GrillState::new(config()).unwrap();
        assert!(transition(
            &state,
            Event::finish(InvocationReport::unknown(
                "griller",
                None,
                AccessMode::ReadOnly
            ))
        )
        .is_err());
        let running = transition(&state, Event::Begin).unwrap();
        assert!(state.turns.is_empty());
        assert!(transition(&running, Event::Begin).is_err());
        let failed = transition(
            &running,
            Event::finish(InvocationReport::unknown(
                "griller",
                None,
                AccessMode::ReadOnly,
            )),
        )
        .unwrap();
        assert!(transition(&failed, Event::Begin).is_err());
    }

    #[tokio::test]
    async fn public_projection_separates_dialogue_from_private_control_and_diagnostics() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("run");
        let (griller, respondent, _) = fakes(&directory);
        let run = run_grilling(config(), &griller, &respondent, &directory)
            .await
            .unwrap();
        let status = serde_json::to_string(&run.state.public_projection(false)).unwrap();
        assert!(!status.contains("DIALOGUE"));
        let dialogue = serde_json::to_string(&run.state.public_projection(true)).unwrap();
        assert!(dialogue.contains("DIALOGUE from griller"));
        assert!(dialogue.contains("DIALOGUE from respondent"));
        for private in [
            "PRIVATE TOPIC",
            "SECRET INSTRUCTIONS",
            "PRIVATE STDERR",
            "PRIVATE DIAGNOSTIC",
            "chosen-g",
        ] {
            assert!(!status.contains(private));
            assert!(!dialogue.contains(private));
        }
    }
}
