//! Ideation orchestrator - Generates divergent idea trees from a seed

use crate::invokers::{get_invoker, AccessMode};
use serde::{Deserialize, Serialize};
use tokio::task;

/// A single idea node in the ideation tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeaNode {
    /// Node identifier: "A", "B", "C" for L1; "A.1", "A.2", "A.3" for L2
    pub id: String,
    /// Short idea title
    pub title: String,
    /// 2-3 sentence description
    pub description: String,
    /// The variation dimension used
    pub axis: String,
    /// How this idea deviates from the original
    pub deviation_rationale: String,
    /// Child ideas (empty for L1-only or L2 leaf nodes)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<IdeaNode>,
}

/// Result of an ideation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeationResult {
    pub seed: String,
    pub sigma: f32,
    pub ideas: Vec<IdeaNode>,
}

/// Raw idea parsed from LLM JSON response
#[derive(Debug, Deserialize)]
struct RawIdea {
    axis: String,
    title: String,
    description: String,
    deviation_rationale: String,
}

/// Raw LLM response structure
#[derive(Debug, Deserialize)]
struct RawIdeationResponse {
    #[allow(dead_code)]
    variation_axes: Vec<String>,
    ideas: Vec<RawIdea>,
}

/// Sigma creativity configuration
struct SigmaConfig {
    label: &'static str,
    guidance: &'static str,
}

fn get_sigma_config(sigma: f32) -> SigmaConfig {
    // Exact presets
    if (sigma - 0.5).abs() < f32::EPSILON {
        return SigmaConfig {
            label: "Conservative twist",
            guidance: "Keep the core intact. Change one significant aspect but the idea should remain recognizably the same concept.",
        };
    }
    if (sigma - 1.0).abs() < f32::EPSILON {
        return SigmaConfig {
            label: "Notable departure",
            guidance: "Preserve the spirit or problem domain but change the approach fundamentally. Someone should see the connection but say \"that's a different idea.\"",
        };
    }
    if (sigma - 1.5).abs() < f32::EPSILON {
        return SigmaConfig {
            label: "Significant reimagining",
            guidance: "Only a thematic thread connects this to the original. The solution, domain, or audience may all change.",
        };
    }
    if (sigma - 2.0).abs() < f32::EPSILON {
        return SigmaConfig {
            label: "Radical departure",
            guidance: "The connection is abstract or metaphorical. Use the original as a springboard into an entirely different space.",
        };
    }

    // Continuous interpolation for arbitrary sigma values
    if sigma < 0.5 {
        SigmaConfig {
            label: "Minimal variation",
            guidance: "Make only the slightest change. The result should be almost indistinguishable from the original, with one subtle twist.",
        }
    } else if sigma < 1.0 {
        SigmaConfig {
            label: "Mild departure",
            guidance: "Keep most of the core intact but introduce a meaningful change in one dimension. The connection to the original should be immediately obvious.",
        }
    } else if sigma < 1.5 {
        SigmaConfig {
            label: "Moderate departure",
            guidance: "Preserve the problem domain but change the approach significantly. The original idea should be recognizable but the solution feels different.",
        }
    } else if sigma < 2.0 {
        SigmaConfig {
            label: "Major reimagining",
            guidance: "Only a loose thematic thread connects this to the original. Change the solution, domain, or audience substantially.",
        }
    } else if sigma <= 3.0 {
        SigmaConfig {
            label: "Extreme departure",
            guidance: "The connection is abstract, metaphorical, or purely inspirational. Use the original as a distant springboard into an entirely different conceptual space.",
        }
    } else {
        SigmaConfig {
            label: "Unconstrained divergence",
            guidance: "No constraints on how far you diverge. The original idea is merely a catalyst — go wherever creativity takes you, even if the connection is invisible.",
        }
    }
}

/// Validate that sigma is within allowed bounds
pub fn validate_sigma(sigma: f32, force: bool) -> anyhow::Result<()> {
    if sigma <= 0.0 {
        return Err(anyhow::anyhow!(
            "Invalid sigma value: {}. Must be greater than 0.",
            sigma
        ));
    }
    if sigma > 3.0 && !force {
        return Err(anyhow::anyhow!(
            "Sigma {} exceeds the safety limit of 3.0. Pass --force to proceed anyway.",
            sigma
        ));
    }
    Ok(())
}

/// Build the Level 1 (seed divergence) prompt
pub fn build_l1_prompt(seed: &str, sigma: f32) -> String {
    let config = get_sigma_config(sigma);
    format!(
        r#"[IDEATION REQUEST]
You are a creative ideation engine.

IDEA:
"{seed}"

CREATIVITY LEVEL: {label}
{guidance}

STEP 1: Identify exactly 3 orthogonal dimensions along which this idea
could be varied (e.g., target audience, core mechanism, domain, scale,
business model, medium, philosophy).

STEP 2: For each dimension, generate ONE idea that deviates from the
original along that dimension. The 3 ideas should feel like they belong
to completely different conversations.

Return ONLY valid JSON:
{{
  "variation_axes": ["axis1", "axis2", "axis3"],
  "ideas": [
    {{
      "axis": "axis1",
      "title": "Short title",
      "description": "2-3 sentence description",
      "deviation_rationale": "How this differs from the original"
    }},
    {{
      "axis": "axis2",
      "title": "Short title",
      "description": "2-3 sentence description",
      "deviation_rationale": "How this differs from the original"
    }},
    {{
      "axis": "axis3",
      "title": "Short title",
      "description": "2-3 sentence description",
      "deviation_rationale": "How this differs from the original"
    }}
  ]
}}
[/IDEATION REQUEST]"#,
        seed = seed,
        label = config.label,
        guidance = config.guidance,
    )
}

/// Build the Level 2 (branch divergence) prompt
pub fn build_l2_prompt(parent_description: &str, sigma: f32, sibling_summaries: &str) -> String {
    let config = get_sigma_config(sigma);
    format!(
        r#"[IDEATION REQUEST]
You are a creative ideation engine.

IDEA:
"{parent}"

CREATIVITY LEVEL: {label}
{guidance}

OTHER IDEAS ALREADY IN THIS SPACE (do not overlap with these):
{siblings}

Generate 3 variations of the above idea that are orthogonal to each
other and distinct from the existing ideas listed above.

Return ONLY valid JSON:
{{
  "variation_axes": ["axis1", "axis2", "axis3"],
  "ideas": [
    {{
      "axis": "axis1",
      "title": "Short title",
      "description": "2-3 sentence description",
      "deviation_rationale": "How this differs from the parent idea"
    }},
    {{
      "axis": "axis2",
      "title": "Short title",
      "description": "2-3 sentence description",
      "deviation_rationale": "How this differs from the parent idea"
    }},
    {{
      "axis": "axis3",
      "title": "Short title",
      "description": "2-3 sentence description",
      "deviation_rationale": "How this differs from the parent idea"
    }}
  ]
}}
[/IDEATION REQUEST]"#,
        parent = parent_description,
        label = config.label,
        guidance = config.guidance,
        siblings = sibling_summaries,
    )
}

/// Extract a JSON object from LLM response text
pub fn extract_json_object(text: &str) -> anyhow::Result<String> {
    // Find the first '{' and last '}' to extract the JSON object
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if start < end {
                return Ok(text[start..=end].to_string());
            }
        }
    }

    Err(anyhow::anyhow!(
        "Could not find valid JSON object in response. Expected format: {{...}}"
    ))
}

/// Parse raw ideas from an LLM response into IdeaNodes with proper IDs
fn parse_ideas_response(response: &str, id_prefix: &str) -> anyhow::Result<Vec<IdeaNode>> {
    let json_str = extract_json_object(response)?;
    let raw: RawIdeationResponse = serde_json::from_str(&json_str).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse ideation response. Error: {}\nJSON: {}",
            e,
            json_str
        )
    })?;

    if raw.ideas.len() != 3 {
        return Err(anyhow::anyhow!(
            "Expected exactly 3 ideas, got {}",
            raw.ideas.len()
        ));
    }

    let id_labels = if id_prefix.is_empty() {
        // L1: "A", "B", "C"
        vec!["A".to_string(), "B".to_string(), "C".to_string()]
    } else {
        // L2: "A.1", "A.2", "A.3"
        vec![
            format!("{}.1", id_prefix),
            format!("{}.2", id_prefix),
            format!("{}.3", id_prefix),
        ]
    };

    let nodes = raw
        .ideas
        .into_iter()
        .zip(id_labels)
        .map(|(idea, id)| IdeaNode {
            id,
            title: idea.title,
            description: idea.description,
            axis: idea.axis,
            deviation_rationale: idea.deviation_rationale,
            children: vec![],
        })
        .collect();

    Ok(nodes)
}

/// Generate Level 1 ideas from a seed
pub async fn generate_l1(
    seed: &str,
    sigma: f32,
    cli: &str,
    timeout: u64,
    access_mode: AccessMode,
) -> anyhow::Result<Vec<IdeaNode>> {
    let invoker = get_invoker(cli).ok_or_else(|| {
        anyhow::anyhow!(
            "CLI '{}' not found. Use claude, codex, gemini, or an installed plugin.",
            cli
        )
    })?;

    if !invoker.is_available() {
        return Err(anyhow::anyhow!("CLI '{}' is not available in PATH.", cli));
    }

    let prompt = build_l1_prompt(seed, sigma);
    let response = invoker.invoke(&prompt, timeout, access_mode, None).await?;
    parse_ideas_response(&response, "")
}

/// Collect all leaf nodes (nodes with no children) with their IDs and descriptions
fn collect_leaves(ideas: &[IdeaNode]) -> Vec<(String, String)> {
    let mut leaves = Vec::new();
    for idea in ideas {
        if idea.children.is_empty() {
            leaves.push((idea.id.clone(), idea.description.clone()));
        } else {
            leaves.extend(collect_leaves(&idea.children));
        }
    }
    leaves
}

/// Find a mutable reference to a node by its ID
fn find_node_mut<'a>(ideas: &'a mut [IdeaNode], id: &str) -> Option<&'a mut IdeaNode> {
    for idea in ideas.iter_mut() {
        if idea.id == id {
            return Some(idea);
        }
        if let Some(found) = find_node_mut(&mut idea.children, id) {
            return Some(found);
        }
    }
    None
}

/// Expand all leaf nodes one level deeper (in parallel)
pub async fn expand_leaves(
    ideas: &mut [IdeaNode],
    sigma: f32,
    cli: &str,
    timeout: u64,
    access_mode: AccessMode,
) -> anyhow::Result<()> {
    let leaves = collect_leaves(ideas);
    if leaves.is_empty() {
        return Ok(());
    }

    // Build sibling summaries from all leaves
    let sibling_summaries: String = leaves
        .iter()
        .map(|(id, desc)| format!("- [{}] {}", id, desc))
        .collect::<Vec<_>>()
        .join("\n");

    let mut tasks = Vec::new();
    for (id, description) in &leaves {
        let parent_desc = description.clone();
        let id_prefix = id.clone();
        let siblings = sibling_summaries.clone();
        let cli_name = cli.to_string();

        let task_handle = task::spawn(async move {
            let invoker = get_invoker(&cli_name)
                .ok_or_else(|| anyhow::anyhow!("CLI '{}' not found.", cli_name))?;

            let prompt = build_l2_prompt(&parent_desc, sigma, &siblings);
            let response = invoker.invoke(&prompt, timeout, access_mode, None).await?;
            let children = parse_ideas_response(&response, &id_prefix)?;
            Ok::<(String, Vec<IdeaNode>), anyhow::Error>((id_prefix, children))
        });

        tasks.push(task_handle);
    }

    let results = futures::future::join_all(tasks).await;

    for result in results {
        match result {
            Ok(Ok((id, children))) => {
                if let Some(node) = find_node_mut(ideas, &id) {
                    node.children = children;
                }
            }
            Ok(Err(e)) => {
                eprintln!("Warning: expansion failed: {}", e);
            }
            Err(e) => {
                eprintln!("Warning: expansion task panicked: {}", e);
            }
        }
    }

    Ok(())
}

/// Generate Level 2 ideas for selected L1 nodes (in parallel)
pub async fn generate_l2(
    l1_ideas: &mut [IdeaNode],
    selected_indices: &[usize],
    sigma: f32,
    cli: &str,
    timeout: u64,
    access_mode: AccessMode,
) -> anyhow::Result<()> {
    // Build sibling summaries (all L1 idea titles)
    let sibling_summaries: String = l1_ideas
        .iter()
        .map(|idea| format!("- {} ({})", idea.title, idea.axis))
        .collect::<Vec<_>>()
        .join("\n");

    // Spawn parallel tasks for each selected L1 idea
    let mut tasks = Vec::new();

    for &idx in selected_indices {
        let idea = &l1_ideas[idx];
        let parent_desc = idea.description.clone();
        let id_prefix = idea.id.clone();
        let siblings = sibling_summaries.clone();
        let cli_name = cli.to_string();

        let task_handle = task::spawn(async move {
            let invoker = get_invoker(&cli_name)
                .ok_or_else(|| anyhow::anyhow!("CLI '{}' not found.", cli_name))?;

            let prompt = build_l2_prompt(&parent_desc, sigma, &siblings);
            let response = invoker.invoke(&prompt, timeout, access_mode, None).await?;
            let children = parse_ideas_response(&response, &id_prefix)?;
            Ok::<(usize, Vec<IdeaNode>), anyhow::Error>((idx, children))
        });

        tasks.push(task_handle);
    }

    // Collect results
    let results = futures::future::join_all(tasks).await;

    for result in results {
        match result {
            Ok(Ok((idx, children))) => {
                l1_ideas[idx].children = children;
            }
            Ok(Err(e)) => {
                eprintln!("Warning: L2 generation failed: {}", e);
            }
            Err(e) => {
                eprintln!("Warning: L2 task panicked: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sigma_valid() {
        // Presets
        assert!(validate_sigma(0.5, false).is_ok());
        assert!(validate_sigma(1.0, false).is_ok());
        assert!(validate_sigma(1.5, false).is_ok());
        assert!(validate_sigma(2.0, false).is_ok());
        // Arbitrary values within range
        assert!(validate_sigma(0.1, false).is_ok());
        assert!(validate_sigma(0.7, false).is_ok());
        assert!(validate_sigma(2.5, false).is_ok());
        assert!(validate_sigma(3.0, false).is_ok());
    }

    #[test]
    fn test_validate_sigma_invalid() {
        assert!(validate_sigma(0.0, false).is_err());
        assert!(validate_sigma(-1.0, false).is_err());
        // Above 3.0 requires force
        assert!(validate_sigma(3.5, false).is_err());
        assert!(validate_sigma(5.0, false).is_err());
    }

    #[test]
    fn test_validate_sigma_force() {
        // --force bypasses the 3.0 limit
        assert!(validate_sigma(5.0, true).is_ok());
        assert!(validate_sigma(10.0, true).is_ok());
        // --force does NOT bypass the <=0 check
        assert!(validate_sigma(0.0, true).is_err());
    }

    #[test]
    fn test_build_l1_prompt_contains_seed() {
        let prompt = build_l1_prompt("Build an app", 1.0);
        assert!(prompt.contains("Build an app"));
        assert!(prompt.contains("Notable departure"));
        assert!(prompt.contains("[IDEATION REQUEST]"));
    }

    #[test]
    fn test_build_l1_prompt_sigma_labels() {
        assert!(build_l1_prompt("test", 0.5).contains("Conservative twist"));
        assert!(build_l1_prompt("test", 1.0).contains("Notable departure"));
        assert!(build_l1_prompt("test", 1.5).contains("Significant reimagining"));
        assert!(build_l1_prompt("test", 2.0).contains("Radical departure"));
    }

    #[test]
    fn test_build_l2_prompt_contains_parent() {
        let prompt = build_l2_prompt("Parent idea description", 1.0, "- Sibling A\n- Sibling B");
        assert!(prompt.contains("Parent idea description"));
        assert!(prompt.contains("Sibling A"));
        assert!(prompt.contains("Notable departure"));
    }

    #[test]
    fn test_extract_json_object() {
        let text = r#"Here is the JSON: {"key": "value"} done"#;
        let result = extract_json_object(text).unwrap();
        assert_eq!(result, r#"{"key": "value"}"#);
    }

    #[test]
    fn test_extract_json_object_no_json() {
        let text = "No JSON here";
        assert!(extract_json_object(text).is_err());
    }

    #[test]
    fn test_parse_ideas_response_l1() {
        let response = r#"{"variation_axes":["audience","mechanism","domain"],"ideas":[{"axis":"audience","title":"Title A","description":"Desc A","deviation_rationale":"Rationale A"},{"axis":"mechanism","title":"Title B","description":"Desc B","deviation_rationale":"Rationale B"},{"axis":"domain","title":"Title C","description":"Desc C","deviation_rationale":"Rationale C"}]}"#;
        let nodes = parse_ideas_response(response, "").unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].id, "A");
        assert_eq!(nodes[1].id, "B");
        assert_eq!(nodes[2].id, "C");
        assert_eq!(nodes[0].title, "Title A");
    }

    #[test]
    fn test_parse_ideas_response_l2() {
        let response = r#"{"variation_axes":["x","y","z"],"ideas":[{"axis":"x","title":"T1","description":"D1","deviation_rationale":"R1"},{"axis":"y","title":"T2","description":"D2","deviation_rationale":"R2"},{"axis":"z","title":"T3","description":"D3","deviation_rationale":"R3"}]}"#;
        let nodes = parse_ideas_response(response, "A").unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].id, "A.1");
        assert_eq!(nodes[1].id, "A.2");
        assert_eq!(nodes[2].id, "A.3");
    }

    #[test]
    fn test_parse_ideas_response_wrong_count() {
        let response = r#"{"variation_axes":["x"],"ideas":[{"axis":"x","title":"T","description":"D","deviation_rationale":"R"}]}"#;
        assert!(parse_ideas_response(response, "").is_err());
    }

    #[test]
    fn test_ideation_result_serialization() {
        let result = IdeationResult {
            seed: "Test seed".to_string(),
            sigma: 1.0,
            ideas: vec![IdeaNode {
                id: "A".to_string(),
                title: "Test".to_string(),
                description: "Description".to_string(),
                axis: "audience".to_string(),
                deviation_rationale: "Rationale".to_string(),
                children: vec![],
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: IdeationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.seed, "Test seed");
        assert_eq!(deserialized.ideas.len(), 1);
        assert_eq!(deserialized.ideas[0].id, "A");
    }
}
