# Contributing to GPT Engage

Thank you for your interest in contributing to GPT Engage! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

Be respectful, inclusive, and constructive in all interactions with other contributors.

## Getting Started

### Prerequisites

- Rust 1.86 or later
- Cargo (comes with Rust)
- At least one LLM CLI installed (Claude Code, Codex, or Gemini)
- Git

### Setting Up Your Development Environment

```bash
# Clone the repository
git clone https://github.com/rahulrajaram/gptengage.git
cd gptengage

# Build in debug mode
cargo build

# Run tests
cargo test

# Build release binary for testing
cargo build --release

# Install locally for testing
./install.sh
```

### Project Structure

```
gptengage/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library exports
│   ├── cli.rs               # CLI parsing and dispatching
│   ├── commands/            # Command implementations
│   │   ├── mod.rs
│   │   ├── debate.rs        # Debate orchestration
│   │   ├── generate_agents.rs # Agent definition generation
│   │   ├── invoke.rs        # Single CLI invocation
│   │   ├── session.rs       # Session management
│   │   └── status.rs        # Status display
│   ├── config/              # Configuration management
│   │   └── mod.rs
│   ├── invokers/            # CLI-specific invokers
│   │   ├── mod.rs
│   │   ├── base.rs          # Shared subprocess logic
│   │   ├── claude.rs        # Claude Code invoker
│   │   ├── codex.rs         # Codex invoker
│   │   └── gemini.rs        # Gemini invoker
│   ├── orchestrator/        # Multi-AI orchestration
│   │   ├── mod.rs
│   │   ├── debate.rs        # Debate logic
│   │   └── agent.rs         # Agent definitions (AgentFile, AgentDefinition)
│   ├── session/             # Session persistence
│   │   └── mod.rs
│   └── utils/               # Utilities
│       └── mod.rs
├── Cargo.toml               # Dependencies
├── Cargo.lock               # Lock file
├── install.sh               # Installation script
├── README.md                # Project readme
├── CONTRIBUTING.md          # This file
├── LICENSE                  # MIT License
├── docs/
│   ├── COMMANDS.md          # Command reference
│   └── EXAMPLES.md          # Usage examples
└── *.json                   # Agent definition files (generated, gitignored)
```

**Agent Definition Files**: JSON files (e.g., `agents.json`) contain structured participant definitions with schema version, personas, instructions, expertise, and communication styles. These are generated via `generate-agents` and consumed by `debate --agent-file`.

## Types of Contributions

### Bug Reports

Found a bug? Help us fix it!

**Before reporting:**
- Check [existing issues](https://github.com/rahulrajaram/gptengage/issues)
- Verify you're using the latest version
- Try reproducing with a minimal example

**When reporting:**
```bash
# Include version info
$ gptengage --version

# Include your environment
$ rustc --version
$ which claude  # or codex, gemini

# Provide steps to reproduce
# Provide expected vs actual behavior
# Include error messages and logs
```

**Example issue:**
```
Title: Codex fails when invoked outside git repository

Environment:
- GPT Engage v1.0.0
- Rust 1.86
- Codex CLI v2.1.0
- Ubuntu 22.04

Steps to reproduce:
1. cd /tmp
2. gptengage invoke codex "hello"

Expected: Response from Codex
Actual: Error about git repository trust

Error message:
"Not inside a trusted directory and --skip-git-repo-check was not specified"
```

### Feature Requests

Have an idea for a new feature?

**Before requesting:**
- Check if it aligns with project goals (CLI-only, no APIs)
- Search existing issues for similar requests
- Consider the scope and complexity

**When requesting:**
```
Title: Clear description of the feature

Problem: What problem does this solve?

Proposed solution: How should it work?

Example usage:
  gptengage new-command --option value

Alternative approaches: Other ways to solve this?
```

### Code Contributions

Want to contribute code? Follow these steps:

#### 1. Fork and Branch

```bash
# Fork the repository on GitHub

# Clone your fork
git clone https://github.com/yourname/gptengage.git
cd gptengage

# Create a feature branch
git checkout -b feature/your-feature-name
# or for fixes:
git checkout -b fix/issue-description
```

#### 2. Make Your Changes

```bash
# Make your code changes
# Follow the style guidelines (see below)

# Build and test
cargo build
cargo test

# Check formatting
cargo fmt --check

# Check lints
cargo clippy

# Fix any issues
cargo fmt  # Auto-format
cargo clippy --fix  # Auto-fix some issues
```

#### 3. Commit Your Changes

```bash
# Commit with a clear message
git commit -m "Brief description of change

More detailed explanation if needed. Explain the 'why' not just the 'what'.

Fixes #123 (if applicable)
"
```

**Commit message guidelines:**
- Start with a verb: "Add", "Fix", "Improve", "Refactor", "Remove"
- Be specific: "Add context file support to invoke" not "Update code"
- Reference issues: "Fixes #123" or "Closes #456"
- Keep first line under 50 characters
- Wrap body at 72 characters

#### 4. Test Your Changes

```bash
# Run full test suite
cargo test

# Test specific functionality
cargo test session::  # Test session module
cargo test invoke_command  # Test a specific test

# Test with real CLIs
gptengage invoke claude "test"
gptengage session list
gptengage debate "test topic" --rounds 1
```

#### 5. Push and Create Pull Request

```bash
# Push your branch
git push origin feature/your-feature-name

# Create a pull request on GitHub
# Include:
# - Description of changes
# - Why this change is needed
# - How to test it
# - Any breaking changes
```

## Style Guidelines

### Rust Code Style

```rust
// Follow Rust conventions and idiomatic patterns

// Use meaningful variable names
let session_manager = SessionManager::new();  // ✓
let sm = SessionManager::new();               // ✗

// Keep functions focused and small
pub async fn invoke_claude(prompt: &str) -> Result<String> {
    // Do one thing well
}

// Document public APIs
/// Run a debate between multiple AI systems.
///
/// # Arguments
/// * `topic` - The debate topic
/// * `num_rounds` - Number of rounds (1-10)
///
/// # Returns
/// A DebateResult containing all round responses
pub async fn run_debate(topic: &str, num_rounds: usize) -> Result<DebateResult> {
    // ...
}

// Use Result and ? operator for error handling
let content = tokio::fs::read_to_string(&file).await?;
let output = invoker.invoke(&content, timeout).await?;
```

### CLI Help Text Formatting

Use `verbatim_doc_comment` for multi-line help text with examples:

```rust
/// Run a multi-AI debate
///
/// Examples:
///   gptengage debate "Topic" --rounds 5
///   gptengage debate "Topic" --agent claude --instances 3
///
/// Cannot be used with --participants
#[command(verbatim_doc_comment)]  // Preserves formatting in --help
Debate {
    /// The topic to debate
    topic: String,

    /// Number of debate rounds (default: 3)
    #[arg(long, short = 'r', default_value = "3")]
    rounds: usize,
}
```

### Short Flag Conventions

Follow these conventions for short flags to maintain consistency:

| Flag | Short | Usage |
|------|-------|-------|
| `--rounds` | `-r` | Number of rounds/iterations |
| `--output` | `-o` | Output file or format |
| `--timeout` | `-t` | Timeout in seconds |
| `--session` | `-s` | Session name |
| `--context-file` | `-c` | Context/input file |
| `--participants` | `-p` | Participant specification |

```rust
// ✓ Good: Short flags for frequently used options
#[arg(long, short = 'r', default_value = "3")]
rounds: usize,

#[arg(long, short = 'o')]
output: String,

// ✗ Bad: No short flag for common options, or inconsistent letters
#[arg(long)]  // Missing short flag for frequently used option
timeout: u64,

#[arg(long, short = 'x')]  // 'x' doesn't relate to 'timeout'
timeout: u64,
```

### Error Message Formatting

Provide clear, actionable error messages:

```rust
// ✓ Good: Clear problem statement with suggestion
return Err(anyhow::anyhow!(
    "Unknown CLI '{}'. Use 'claude', 'codex', or 'gemini'",
    cli_name
));

// ✓ Good: Include context about what failed
return Err(anyhow::anyhow!(
    "Failed to parse agent definitions from response. Error: {}\nResponse: {}",
    parse_error,
    response
));

// ✓ Good: Explain validation failures
return Err(anyhow::anyhow!(
    "Generated agent {} ({}) failed validation: {}",
    idx + 1,
    agent.persona,
    validation_error
));

// ✗ Bad: Vague error message
return Err(anyhow::anyhow!("Invalid input"));

// ✗ Bad: Technical jargon without explanation
return Err(anyhow::anyhow!("serde deserialization failed"));
```

### Error Handling

```rust
// ✓ Use anyhow::Result and provide context
pub async fn invoke(&self, prompt: &str) -> Result<String> {
    let output = command.execute(timeout).await
        .context("Failed to execute CLI")?;
    Ok(output)
}

// ✗ Don't swallow errors or use unwrap
let output = command.execute(timeout).await.unwrap();
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        // Arrange
        let manager = SessionManager::new().unwrap();

        // Act
        let session = manager.create_session(
            "test".to_string(),
            "claude".to_string(),
            "Test topic".to_string(),
        );

        // Assert
        assert!(session.is_ok());
        let s = session.unwrap();
        assert_eq!(s.name, "test");
    }

    #[tokio::test]
    async fn test_invoke_claude() {
        // Use tokio::test for async tests
    }
}
```

#### Testing Agent File Validation

```rust
#[test]
fn test_agent_definition_validation() {
    // Test valid agent definition
    let agent = AgentDefinition {
        cli: "claude".to_string(),
        persona: "CEO".to_string(),
        instructions: "Focus on business impact...".to_string(),
        expertise: vec!["strategy".to_string(), "finance".to_string()],
        communication_style: "Executive - concise".to_string(),
    };
    assert!(agent.validate().is_ok());

    // Test invalid CLI
    let bad_agent = AgentDefinition {
        cli: "invalid".to_string(),
        ..agent.clone()
    };
    assert!(bad_agent.validate().is_err());

    // Test empty fields
    let empty_persona = AgentDefinition {
        persona: "".to_string(),
        ..agent.clone()
    };
    assert!(empty_persona.validate().is_err());
}

#[test]
fn test_agent_file_parsing() {
    let json = r#"{
        "schema_version": "1.0",
        "participants": [{
            "cli": "claude",
            "persona": "Engineer",
            "instructions": "Technical focus",
            "expertise": ["rust", "systems"],
            "communication_style": "Technical"
        }]
    }"#;

    let agent_file: AgentFile = serde_json::from_str(json).unwrap();
    assert_eq!(agent_file.participants.len(), 1);
}
```

#### Testing Multi-Instance Debates

```rust
#[tokio::test]
async fn test_multi_instance_debate() {
    // Test that --agent with --instances creates correct participant count
    let options = DebateOptions {
        topic: "Test".to_string(),
        agent: Some("claude".to_string()),
        instances: Some(3),
        participants: None,
        agent_file: None,
        rounds: 1,
        output: "text".to_string(),
        timeout: 60,
    };

    // Verify 3 participants are created
    let participants = build_participants(&options).unwrap();
    assert_eq!(participants.len(), 3);
    assert!(participants.iter().all(|p| p.cli == "claude"));
}
```

#### Testing JSON Output Parsing

```rust
#[test]
fn test_json_output_format() {
    let result = DebateResult {
        topic: "Test topic".to_string(),
        rounds: vec![/* ... */],
        consensus: Some("Agreement reached".to_string()),
    };

    let json = serde_json::to_string_pretty(&result).unwrap();
    assert!(json.contains("\"topic\""));
    assert!(json.contains("\"rounds\""));

    // Verify round-trip parsing
    let parsed: DebateResult = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.topic, result.topic);
}

#[test]
fn test_extract_json_array() {
    // Test extraction from LLM response with surrounding text
    let response = "Here are the agents:\n[{\"cli\": \"claude\"}]\nDone!";
    let extracted = extract_json_array(response).unwrap();
    assert_eq!(extracted, "[{\"cli\": \"claude\"}]");

    // Test missing array
    let bad_response = "No JSON here";
    assert!(extract_json_array(bad_response).is_err());
}
```

### File Organization

- One module per file or small related modules in one file
- Group related functionality together
- Use `mod.rs` to organize module exports

## Before Submitting a PR

- [ ] Code compiles without warnings: `cargo build`
- [ ] All tests pass: `cargo test`
- [ ] Code is formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Documentation is updated
- [ ] Commit messages are clear
- [ ] PR description explains the changes

### Checklist for PRs

```markdown
## Description
[Brief description of the change]

## Type of Change
- [ ] Bug fix (fixes #_)
- [ ] New feature (closes #_)
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Added new tests
- [ ] All tests pass: `cargo test`
- [ ] Tested with Claude CLI
- [ ] Tested with Codex CLI
- [ ] Tested with Gemini CLI

## Documentation
- [ ] Updated README.md if needed
- [ ] Updated docs/COMMANDS.md if needed
- [ ] Updated docs/EXAMPLES.md if needed
- [ ] Added doc comments for public APIs

## CLI Options (if adding/modifying commands)
- [ ] Updated cli.rs help text if adding options
- [ ] Added short flags for common options (see Short Flag Conventions)
- [ ] Exit codes documented (0=success, 1=error, 2=validation error)

## Code Quality
- [ ] Formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy`
- [ ] No compiler warnings: `cargo build`
```

## Architecture Guidelines

### Principles

1. **CLI-Only Design**: Never use APIs directly. Always invoke CLI tools.
2. **Non-Invasive**: Don't modify ~/.claude/, ~/.codex/, or ~/.gemini/
3. **Session via Context Injection**: Maintain history by injecting `[CONVERSATION_HISTORY]` blocks
4. **Async/Await**: Use Tokio for all I/O operations
5. **Error Context**: Use `anyhow::Context` to provide meaningful error messages
6. **Isolation**: Each CLI invocation should be independent

### When Adding Features

- Ask: "Can this be done with CLI invocations?" If not, reconsider.
- Keep invokers simple: they should just spawn processes
- Commands should coordinate invokers, not duplicate logic
- Sessions should be transparent to users (automatic context injection)

### When Modifying Commands

- Ensure backward compatibility where possible
- Update documentation alongside code
- Test with all three CLIs (Claude, Codex, Gemini)
- Handle CLI failures gracefully

## Common Development Tasks

### Adding a New CLI

To support a new LLM CLI (e.g., Llama):

```rust
// 1. Create invoker in src/invokers/llama.rs
pub struct LlamaInvoker;

#[async_trait]
impl Invoker for LlamaInvoker {
    async fn invoke(&self, prompt: &str, timeout: u64) -> Result<String> {
        execute_command("llama", &["--mode", "chat"], prompt, timeout).await
    }

    fn name(&self) -> &str {
        "llama"
    }

    fn is_available(&self) -> bool {
        command_exists("llama")
    }
}

// 2. Export in src/invokers/mod.rs
pub mod llama;
pub use llama::LlamaInvoker;

// 3. Update src/cli.rs to support the new CLI
#[derive(Debug, ValueEnum, Clone)]
pub enum Cli {
    Claude,
    Codex,
    Gemini,
    #[value(name = "llama")]
    Llama,
}

// 4. Update src/commands/invoke.rs
"llama" => {
    let invoker = LlamaInvoker::new();
    invoker.invoke(&full_prompt, timeout).await?
}

// 5. Update src/commands/debate.rs if needed

// 6. Update documentation
```

### Adding a New Command

To add a new GPT Engage command:

```rust
// 1. Create src/commands/newcmd.rs
pub async fn run_newcommand(args: String) -> Result<()> {
    // Implementation
    Ok(())
}

// 2. Add to src/commands/mod.rs
pub mod newcmd;

// 3. Add to src/cli.rs Subcommand enum
#[derive(Debug, Subcommand)]
pub enum Commands {
    // ...
    #[command(about = "Description of the command")]
    NewCmd {
        #[arg(help = "What the argument does")]
        arg: String,
    },
}

// 4. Add to cli.rs execute() match
Commands::NewCmd { arg } => {
    commands::newcmd::run_newcommand(arg).await?
}

// 5. Test thoroughly
// 6. Update README.md and docs/COMMANDS.md
```

#### Example: Adding a Command with Generation/Subagent Capabilities

For commands that generate content or use AI to create structured output (like `generate-agents`),
follow this pattern from `src/commands/generate_agents.rs`:

```rust
// src/commands/generate_cmd.rs
use crate::invokers::{ClaudeInvoker, CodexInvoker, GeminiInvoker, Invoker};

pub async fn run_generate_command(
    input: String,
    output_path: String,
    use_cli: String,
    timeout: u64,
) -> anyhow::Result<()> {
    // 1. Select invoker based on --use-cli flag
    let invoker: Box<dyn Invoker> = match use_cli.to_lowercase().as_str() {
        "claude" => Box::new(ClaudeInvoker::new()),
        "codex" => Box::new(CodexInvoker::new()),
        "gemini" => Box::new(GeminiInvoker::new()),
        _ => return Err(anyhow::anyhow!(
            "Unknown CLI '{}'. Use 'claude', 'codex', or 'gemini'",
            use_cli
        )),
    };

    // 2. Check CLI availability
    if !invoker.is_available() {
        return Err(anyhow::anyhow!(
            "CLI '{}' is not available. Please ensure it's installed and in your PATH",
            use_cli
        ));
    }

    // 3. Build a structured prompt that requests JSON output
    let prompt = format!(
        r#"Generate output for: {}

IMPORTANT: Return ONLY valid JSON, no markdown or explanations.
Format: {{ "field": "value" }}"#,
        input
    );

    // 4. Invoke and parse response
    let response = invoker.invoke(&prompt, timeout).await?;
    let parsed = parse_json_response(&response)?;

    // 5. Validate the parsed structure
    parsed.validate()?;

    // 6. Write to output file
    let json = serde_json::to_string_pretty(&parsed)?;
    std::fs::write(&output_path, json)?;

    Ok(())
}

// Helper to extract JSON from LLM response (handles surrounding text)
fn extract_json(text: &str) -> anyhow::Result<String> {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if start < end {
                return Ok(text[start..=end].to_string());
            }
        }
    }
    Err(anyhow::anyhow!("Could not find valid JSON in response"))
}
```

Key patterns:
- Use `Box<dyn Invoker>` for runtime CLI selection
- Request structured JSON output in prompts
- Use `extract_json_array()` or `extract_json()` helpers to handle LLM response variations
- Validate parsed output before saving
- Provide actionable error messages with context

### Adding Agent Definition Fields

To extend the `AgentDefinition` schema in `src/orchestrator/agent.rs`:

```rust
// 1. Add the new field to AgentDefinition struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub cli: String,
    pub persona: String,
    pub instructions: String,
    pub expertise: Vec<String>,
    pub communication_style: String,
    // Add new field with serde defaults for backward compatibility
    #[serde(default)]
    pub temperature: Option<f32>,
}

// 2. Update the validate() method to validate the new field
impl AgentDefinition {
    pub fn validate(&self) -> Result<(), String> {
        // ... existing validations ...

        // Add validation for new field
        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err("Temperature must be between 0.0 and 2.0".to_string());
            }
        }

        Ok(())
    }
}

// 3. Update generate_agents.rs prompt to include new field
let prompt = format!(
    r#"Generate agent definitions with these fields:
- "cli": ...
- "temperature": Optional float 0.0-2.0 for response variability
..."#
);

// 4. Update documentation
// - docs/COMMANDS.md: Document new field in agent file format
// - Update schema_version if breaking change
```

### Updating Validation Rules

To modify validation logic for agent definitions or other structures:

```rust
// In src/orchestrator/agent.rs or relevant module

impl AgentDefinition {
    pub fn validate(&self) -> Result<(), String> {
        // CLI validation - maintain list of supported CLIs
        let valid_clis = ["claude", "codex", "gemini"];
        if !valid_clis.contains(&self.cli.to_lowercase().as_str()) {
            return Err(format!(
                "Invalid CLI '{}'. Must be one of: {}",
                self.cli,
                valid_clis.join(", ")
            ));
        }

        // Required field validation
        if self.persona.trim().is_empty() {
            return Err("Persona cannot be empty".to_string());
        }

        if self.instructions.len() < 10 {
            return Err("Instructions must be at least 10 characters".to_string());
        }

        // Array validation
        if self.expertise.is_empty() {
            return Err("At least one expertise area is required".to_string());
        }

        if self.expertise.len() > 10 {
            return Err("Maximum 10 expertise areas allowed".to_string());
        }

        // String length limits
        if self.communication_style.len() > 100 {
            return Err("Communication style must be under 100 characters".to_string());
        }

        Ok(())
    }
}

// Testing validation changes
#[cfg(test)]
mod tests {
    #[test]
    fn test_validation_rules() {
        // Test boundary conditions
        let agent = AgentDefinition {
            expertise: vec!["a".to_string(); 11],  // Over limit
            ..default_agent()
        };
        assert!(agent.validate().is_err());
        assert!(agent.validate().unwrap_err().contains("Maximum 10"));
    }
}
```

Validation best practices:
- Return descriptive error messages explaining the constraint
- Test boundary conditions (empty, min, max values)
- Consider backward compatibility when tightening rules
- Document validation rules in help text

## Documentation

### Code Comments

```rust
// ✓ Good: Explains WHY, not WHAT
// We retry because the CLI may be temporarily busy
for attempt in 0..3 {
    match invoke_cli() {
        Ok(result) => return Ok(result),
        Err(_) if attempt < 2 => continue,
        Err(e) => return Err(e),
    }
}

// ✗ Bad: Just repeats the code
// Increment i
i += 1;
```

### Updating Documentation

When making changes:

1. **README.md**: Update quick-start, features, or installation sections
2. **docs/COMMANDS.md**: Update/add command reference
3. **docs/EXAMPLES.md**: Add new usage examples if relevant

## Performance Considerations

- Parallel execution: Use `tokio::task::spawn` for concurrent work
- Timeouts: Always set reasonable timeouts for CLI invocations
- Sessions: Lazy-load session history; don't load entire session directory
- I/O: Use async APIs (tokio::fs), never block

## Security Considerations

- **Input validation**: Validate session names, file paths
- **Prompt injection**: Be careful when building prompts from user input
- **File access**: Only read files explicitly provided by user
- **Subprocess safety**: Never pass unsanitized user input to shell

## Review Process

1. **Automated checks**: CI will run tests, formatting, and clippy
2. **Code review**: Maintainers will review for:
   - Correctness and robustness
   - Adherence to guidelines
   - Documentation completeness
   - Testing coverage
3. **Feedback**: Address review feedback in follow-up commits
4. **Merge**: Once approved, maintainers will merge your PR

## Getting Help

- **Questions**: Open a discussion issue
- **Design advice**: Open an issue before major refactors
- **Documentation**: Help improve docs by opening PRs
- **Testing**: Help test new features before release

## Release Process

Maintainers will:
1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Create a git tag
4. Build and publish to crates.io
5. Create release notes on GitHub

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to GPT Engage! 🎉
