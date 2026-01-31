//! Stdin handling utilities for Unix-style piping

use std::io::{self, IsTerminal, Read};

/// Read from stdin if input is piped (not an interactive terminal).
///
/// Returns `Some(content)` if stdin contains piped data, `None` if stdin
/// is a terminal or if the piped content is empty.
///
/// # Example
///
/// ```bash
/// echo "topic" | gptengage debate
/// cat code.rs | gptengage invoke claude "Review this"
/// ```
pub fn read_stdin_if_piped() -> Option<String> {
    let stdin = io::stdin();

    // Return None if stdin is an interactive terminal (not piped)
    if stdin.is_terminal() {
        return None;
    }

    let mut buffer = String::new();
    if stdin.lock().read_to_string(&mut buffer).is_err() {
        return None;
    }

    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Format piped content with context markers for injection into prompts.
///
/// Wraps the content in `[PIPED CONTEXT]` markers to clearly delineate
/// piped input from the main topic or prompt.
pub fn format_piped_context(content: &str) -> String {
    format!("[PIPED CONTEXT]\n{}\n[/PIPED CONTEXT]", content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_piped_context() {
        let content = "some code here";
        let formatted = format_piped_context(content);
        assert!(formatted.contains("[PIPED CONTEXT]"));
        assert!(formatted.contains("some code here"));
        assert!(formatted.contains("[/PIPED CONTEXT]"));
    }
}
