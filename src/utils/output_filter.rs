//! Output filtering - Clean CLI output

pub struct OutputFilter;

impl OutputFilter {
    /// Filter Claude Code output (remove startup messages)
    pub fn filter_claude(output: &str) -> String {
        output
            .lines()
            .filter(|line| {
                !line.contains("[STARTUP]")
                    && !line.contains("Claude Code initialized")
                    && !line.is_empty()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Filter Codex output
    pub fn filter_codex(output: &str) -> String {
        output
            .lines()
            .filter(|line| {
                !line.contains("Codex executed")
                    && !line.contains("Full auto mode enabled")
                    && !line.is_empty()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Filter Gemini output
    pub fn filter_gemini(output: &str) -> String {
        output
            .lines()
            .filter(|line| {
                !line.contains("[STARTUP]")
                    && !line.contains("YOLO mode is enabled")
                    && !line.is_empty()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Limit output to max lines
    pub fn limit_lines(output: &str, max_lines: usize) -> String {
        output
            .lines()
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n")
    }
}
