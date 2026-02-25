//! Ideate command - Generate divergent idea trees from a seed

use crate::invokers::AccessMode;
use crate::orchestrator::ideation::{
    expand_leaves, generate_l1, generate_l2, validate_sigma, IdeationResult,
};
use crate::utils::colors::ColorMode;
use crate::utils::pager::run_pager;
use crate::utils::tree_renderer::render_idea_tree;
use std::io::{self, BufRead, Write};

/// Options for the ideate command
pub struct IdeateOptions {
    pub seed: String,
    pub sigma: f32,
    pub select: bool,
    pub depth: u8,
    pub cli: String,
    pub output: String,
    pub timeout: u64,
    pub access_mode: AccessMode,
    pub color: String,
    pub pager: bool,
    pub force: bool,
}

/// Run the ideate command
pub async fn run_ideate(opts: IdeateOptions) -> anyhow::Result<()> {
    // Validate inputs
    validate_sigma(opts.sigma, opts.force)?;

    if opts.depth == 0 {
        return Err(anyhow::anyhow!("Invalid depth: 0. Must be at least 1."));
    }
    if opts.depth > 5 && !opts.force {
        return Err(anyhow::anyhow!(
            "Depth {} exceeds the safety limit of 5 (3^{} = {} LLM invocations). \
             Pass --force to proceed anyway.",
            opts.depth,
            opts.depth,
            3u64.pow(opts.depth as u32),
        ));
    }

    if opts.seed.trim().is_empty() {
        return Err(anyhow::anyhow!("Seed idea cannot be empty."));
    }

    // Use stderr for user-facing output so JSON stays clean on stdout.
    // When --pager is active, suppress progress lines since the user will
    // immediately enter the alternate screen and never see them.
    let quiet = opts.pager;

    if !quiet {
        eprintln!("Generating ideas from seed...");
        eprintln!("  Seed: \"{}\"", opts.seed);
        eprintln!(
            "  Sigma: {} | CLI: {} | Depth: {}",
            opts.sigma, opts.cli, opts.depth
        );
        eprintln!();
        eprintln!("Level 1: Generating 3 divergent ideas...");
    }
    let mut l1_ideas = generate_l1(
        &opts.seed,
        opts.sigma,
        &opts.cli,
        opts.timeout,
        opts.access_mode,
    )
    .await?;

    if !quiet {
        eprintln!("  Generated {} L1 ideas.", l1_ideas.len());
    }

    // Recursive expansion for depth >= 2
    if opts.depth >= 2 {
        // Level 2: expand L1 ideas (with optional interactive selection)
        let selected = if opts.select {
            prompt_selection(l1_ideas.len())?
        } else {
            (0..l1_ideas.len()).collect()
        };

        if selected.is_empty() {
            if !quiet {
                eprintln!("No ideas selected for expansion.");
            }
        } else {
            if !quiet {
                let labels: Vec<String> =
                    selected.iter().map(|&i| l1_ideas[i].id.clone()).collect();
                eprintln!("Level 2: Expanding ideas {} ...", labels.join(", "));
            }

            generate_l2(
                &mut l1_ideas,
                &selected,
                opts.sigma,
                &opts.cli,
                opts.timeout,
                opts.access_mode,
            )
            .await?;
        }

        // Levels 3..depth: recursively expand leaf nodes at each level
        for level in 3..=opts.depth {
            if !quiet {
                eprintln!("Level {}: Expanding leaf nodes ...", level);
            }
            expand_leaves(
                &mut l1_ideas,
                opts.sigma,
                &opts.cli,
                opts.timeout,
                opts.access_mode,
            )
            .await?;
        }
    }

    // Build result
    let result = IdeationResult {
        seed: opts.seed,
        sigma: opts.sigma,
        ideas: l1_ideas,
    };

    // Output
    match opts.output.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&result)?;
            println!("{}", json);
        }
        _ => {
            // Render the tree with color support
            let color_mode = ColorMode::from_flag(&opts.color);
            if opts.pager {
                let mut buf = Vec::new();
                render_idea_tree(&result, color_mode, &mut buf)?;
                let content = String::from_utf8_lossy(&buf).into_owned();
                run_pager(&content)?;
            } else {
                let mut stderr = io::stderr().lock();
                eprintln!();
                render_idea_tree(&result, color_mode, &mut stderr)?;
            }
        }
    }

    Ok(())
}

/// Prompt the user to select which L1 ideas to expand
fn prompt_selection(count: usize) -> anyhow::Result<Vec<usize>> {
    eprint!(
        "Which ideas should we expand to level 2? [1-{} or \"all\"]: ",
        count
    );
    io::stderr().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let input = line.trim();

    if input.eq_ignore_ascii_case("all") {
        return Ok((0..count).collect());
    }

    let mut selected = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.parse::<usize>() {
            Ok(n) if n >= 1 && n <= count => {
                let idx = n - 1;
                if !selected.contains(&idx) {
                    selected.push(idx);
                }
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid selection: '{}'. Use numbers 1-{} separated by commas, or \"all\".",
                    part,
                    count
                ));
            }
        }
    }

    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::ideation::IdeaNode;

    #[test]
    fn test_ideation_result_renders_in_none_mode() {
        let result = IdeationResult {
            seed: "Test seed".to_string(),
            sigma: 1.0,
            ideas: vec![
                IdeaNode {
                    id: "A".to_string(),
                    title: "Idea A".to_string(),
                    description: "Description A".to_string(),
                    axis: "audience".to_string(),
                    deviation_rationale: "Rationale A".to_string(),
                    children: vec![IdeaNode {
                        id: "A.1".to_string(),
                        title: "Idea A.1".to_string(),
                        description: "Description A.1".to_string(),
                        axis: "mechanism".to_string(),
                        deviation_rationale: "Rationale A.1".to_string(),
                        children: vec![],
                    }],
                },
                IdeaNode {
                    id: "B".to_string(),
                    title: "Idea B".to_string(),
                    description: "Description B".to_string(),
                    axis: "domain".to_string(),
                    deviation_rationale: "Rationale B".to_string(),
                    children: vec![],
                },
            ],
        };
        let mut buf = Vec::new();
        render_idea_tree(&result, ColorMode::None, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("seed:"));
        assert!(output.contains("Test seed"));
        assert!(output.contains("A  Idea A"));
        assert!(output.contains("B  Idea B"));
        assert!(output.contains("A.1  Idea A.1"));
    }
}
