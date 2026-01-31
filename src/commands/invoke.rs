//! Invoke command - Invoke a specific CLI with optional session support

use crate::cli::StdinMode;
use crate::invokers::{get_invoker, AccessMode};
use crate::session::SessionManager;
use crate::utils::stdin::{format_piped_context, read_stdin_if_piped};

/// Invoke a specific CLI with a prompt
#[allow(clippy::too_many_arguments)]
pub async fn run_invoke(
    cli: String,
    mut prompt: String,
    session_name: Option<String>,
    topic: Option<String>,
    context_file: Option<String>,
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
    eprintln!("Invoking {}...", cli);
    let response = invoker.invoke(&full_prompt, timeout, access_mode).await?;

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
