//! Bounded grilling command. Private artifacts are explicit and never resumed.
use crate::invokers::get_invoker;
use crate::orchestrator::grilling::{run_grilling, GrillConfig, Phase, RoleParticipant};
use anyhow::{bail, Context};
use std::path::PathBuf;

pub struct GrillOptions {
    pub topic: String,
    pub griller: String,
    pub respondent: String,
    pub griller_instructions: PathBuf,
    pub respondent_instructions: PathBuf,
    pub griller_model: Option<String>,
    pub respondent_model: Option<String>,
    pub run_dir: PathBuf,
    pub exchanges: u32,
    pub timeout: u64,
    pub show_dialogue: bool,
}

pub async fn run_grill(options: GrillOptions) -> anyhow::Result<()> {
    let config = GrillConfig {
        topic: options.topic,
        griller: RoleParticipant {
            backend: options.griller,
            model: options.griller_model,
            instructions: std::fs::read_to_string(&options.griller_instructions)
                .context("Cannot read griller instructions")?,
        },
        respondent: RoleParticipant {
            backend: options.respondent,
            model: options.respondent_model,
            instructions: std::fs::read_to_string(&options.respondent_instructions)
                .context("Cannot read respondent instructions")?,
        },
        exchanges: options.exchanges,
        timeout: options.timeout,
    };
    config.validate()?;
    let griller = get_invoker(&config.griller.backend).context("Unknown griller backend")?;
    let respondent =
        get_invoker(&config.respondent.backend).context("Unknown respondent backend")?;
    let result = run_grilling(
        config,
        griller.as_ref(),
        respondent.as_ref(),
        &options.run_dir,
    )
    .await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "run_directory": options.run_dir,
            "last_checkpoint": result.last_checkpoint,
            "result": result.state.public_projection(options.show_dialogue),
        }))?
    );
    match result.state.phase {
        Phase::Completed => Ok(()),
        Phase::Failed { .. } => {
            bail!("Grilling stopped after an invocation failure; inspect the private checkpoint")
        }
        Phase::Ready { .. } | Phase::Invoking { .. } => bail!("Grilling did not complete"),
    }
}
