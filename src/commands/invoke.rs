//! Invoke command - Invoke a specific CLI with optional session support

use std::path::{Path, PathBuf};

use crate::cli::StdinMode;
use crate::invokers::{get_invoker, AccessMode};
use crate::session::SessionManager;
use crate::utils::stdin::{format_piped_context, read_stdin_if_piped};

/// File extensions recognized as images for vision-capable CLIs.
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp"];

/// CLIs that currently support image input. Extend as other invokers gain
/// native image-passthrough.
const IMAGE_CAPABLE_CLIS: &[&str] = &["claude"];

/// Check whether a path has an image extension we know how to pass through.
fn is_image_path(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => IMAGE_EXTENSIONS
            .iter()
            .any(|known| known.eq_ignore_ascii_case(ext)),
        None => false,
    }
}

/// Append `@<absolute-path>` references to a prompt so Claude Code treats
/// them as vision inputs. The Claude CLI natively resolves these in -p mode.
fn append_image_refs(mut prompt: String, image_paths: &[PathBuf]) -> String {
    for p in image_paths {
        prompt.push_str("\n\n@");
        prompt.push_str(&p.to_string_lossy());
    }
    prompt
}

/// Invoke a specific CLI with a prompt
#[allow(clippy::too_many_arguments)]
pub async fn run_invoke(
    cli: String,
    model: Option<String>,
    mut prompt: String,
    session_name: Option<String>,
    topic: Option<String>,
    context_file: Option<String>,
    images: Vec<String>,
    timeout: u64,
    access_mode: AccessMode,
    stdin_as: StdinMode,
) -> anyhow::Result<()> {
    // Handle stdin input based on mode
    if let Some(stdin_content) = read_stdin_if_piped() {
        match stdin_as {
            StdinMode::Auto => {
                if prompt.is_empty() {
                    // No prompt provided, use stdin as prompt
                    prompt = stdin_content;
                } else {
                    // Prompt provided, prepend stdin as context
                    prompt = format!("{}\n\n{}", format_piped_context(&stdin_content), prompt);
                }
            }
            StdinMode::Context => {
                // Always prepend stdin as context
                prompt = format!("{}\n\n{}", format_piped_context(&stdin_content), prompt);
            }
            StdinMode::Ignore => {
                // Do nothing with stdin
            }
        }
    }

    // Validate that prompt is not empty
    if prompt.is_empty() {
        return Err(anyhow::anyhow!(
            "Prompt is required. Provide as argument or pipe via stdin."
        ));
    }

    // Load context from file if provided
    if let Some(file) = context_file {
        let file_content = tokio::fs::read_to_string(&file).await?;
        prompt = format!("File: {}\n\n{}\n\n{}", file, file_content, prompt);
    }

    // Validate and attach images if provided. Images are currently only
    // supported for Claude; other CLIs will gain native passthrough later.
    if !images.is_empty() {
        let cli_lower = cli.to_lowercase();
        if !IMAGE_CAPABLE_CLIS.contains(&cli_lower.as_str()) {
            return Err(anyhow::anyhow!(
                "Image input is not yet supported for '{}'. Currently supported: {}",
                cli,
                IMAGE_CAPABLE_CLIS.join(", ")
            ));
        }

        let mut resolved: Vec<PathBuf> = Vec::with_capacity(images.len());
        for image in &images {
            let path = PathBuf::from(image);
            if !is_image_path(&path) {
                return Err(anyhow::anyhow!(
                    "Not a recognized image file (expected one of {}): {}",
                    IMAGE_EXTENSIONS.join(", "),
                    image
                ));
            }
            let abs = tokio::fs::canonicalize(&path)
                .await
                .map_err(|e| anyhow::anyhow!("Image not found or unreadable: {} ({})", image, e))?;
            resolved.push(abs);
        }

        prompt = append_image_refs(prompt, &resolved);
    }

    // Handle session if provided
    let mut session_manager = None;
    if session_name.is_some() || topic.is_some() {
        session_manager = Some(SessionManager::new()?);
    }

    // Load existing session if it exists
    let session = if let Some(ref name) = session_name {
        session_manager
            .as_ref()
            .unwrap()
            .load_session(name)
            .await
            .ok()
    } else {
        None
    };

    // Build full prompt with history if session exists
    let full_prompt = if let Some(ref s) = session {
        session_manager
            .as_ref()
            .unwrap()
            .build_prompt_with_history(s, &prompt)
    } else {
        prompt.clone()
    };

    // Get the appropriate invoker (built-in or plugin)
    let invoker = get_invoker(&cli).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown CLI: '{}'. Use a built-in CLI (claude, codex, gemini) or an installed plugin.",
            cli
        )
    })?;

    // Check if the CLI is available
    if !invoker.is_available() {
        return Err(anyhow::anyhow!(
            "CLI '{}' not found in PATH. Ensure it is installed and accessible.",
            cli
        ));
    }

    // Invoke the CLI
    let cli_display = match &model {
        Some(m) => format!("{}:{}", cli, m),
        None => cli.clone(),
    };
    eprintln!("Invoking {}...", cli_display);
    let response = invoker
        .invoke(&full_prompt, timeout, access_mode, model.as_deref())
        .await?;

    // Print response
    println!("{}", response);

    // Update session if applicable
    if let (Some(name), Some(manager)) = (&session_name, &session_manager) {
        let mut s = if let Some(existing) = session {
            existing
        } else {
            let topic_str =
                topic.unwrap_or_else(|| prompt.split('\n').next().unwrap_or("Chat").to_string());
            manager.create_session(name.clone(), cli.clone(), topic_str)?
        };

        // Add user message and response to session
        manager.add_turn(&mut s, "user".to_string(), prompt);
        manager.add_turn(&mut s, "assistant".to_string(), response);

        // Save session
        manager.save_session(&s).await?;
        println!("\n(Session '{}' saved)", name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_image_path_accepts_common_extensions() {
        for ext in ["png", "PNG", "jpg", "Jpeg", "gif", "webp", "bmp"] {
            let p = PathBuf::from(format!("/tmp/x.{ext}"));
            assert!(is_image_path(&p), "should accept .{ext}");
        }
    }

    #[test]
    fn is_image_path_rejects_non_images() {
        for name in ["doc.pdf", "notes.txt", "code.rs", "no-extension", "dir/"] {
            let p = PathBuf::from(name);
            assert!(!is_image_path(&p), "should reject {name}");
        }
    }

    #[test]
    fn append_image_refs_noop_when_empty() {
        let out = append_image_refs("hello".into(), &[]);
        assert_eq!(out, "hello");
    }

    #[test]
    fn append_image_refs_appends_absolute_refs() {
        let paths = vec![PathBuf::from("/tmp/a.png"), PathBuf::from("/tmp/b.jpg")];
        let out = append_image_refs("review:".into(), &paths);
        assert_eq!(out, "review:\n\n@/tmp/a.png\n\n@/tmp/b.jpg");
    }
}
