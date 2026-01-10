//! Invoke command - Invoke a specific CLI with optional session support

use crate::invokers::{ClaudeInvoker, CodexInvoker, GeminiInvoker, Invoker};
use crate::session::SessionManager;

/// Invoke a specific CLI with a prompt
pub async fn run_invoke(
    cli: String,
    mut prompt: String,
    session_name: Option<String>,
    topic: Option<String>,
    context_file: Option<String>,
    timeout: u64,
) -> anyhow::Result<()> {
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

    // Invoke the appropriate CLI
    eprintln!("Invoking {}...", cli);
    let response = match cli.as_str() {
        "claude" => {
            let invoker = ClaudeInvoker::new();
            if !invoker.is_available() {
                return Err(anyhow::anyhow!(
                    "Claude Code CLI not found in PATH. Install from: https://claude.ai/code"
                ));
            }
            invoker.invoke(&full_prompt, timeout).await?
        }
        "codex" => {
            let invoker = CodexInvoker::new();
            if !invoker.is_available() {
                return Err(anyhow::anyhow!(
                    "Codex CLI not found in PATH. Install from: https://github.com/openai/codex"
                ));
            }
            invoker.invoke(&full_prompt, timeout).await?
        }
        "gemini" => {
            let invoker = GeminiInvoker::new();
            if !invoker.is_available() {
                return Err(anyhow::anyhow!("Gemini CLI not found in PATH. Install from: https://ai.google.dev/gemini-api/docs/cli"));
            }
            invoker.invoke(&full_prompt, timeout).await?
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown CLI: {}. Use 'claude', 'codex', or 'gemini'.",
                cli
            ))
        }
    };

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
