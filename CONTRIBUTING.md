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
git clone https://github.com/yourusername/gptengage.git
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
│   │   └── debate.rs        # Debate logic
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
└── docs/
    ├── COMMANDS.md          # Command reference
    └── EXAMPLES.md          # Usage examples
```

## Types of Contributions

### Bug Reports

Found a bug? Help us fix it!

**Before reporting:**
- Check [existing issues](https://github.com/yourusername/gptengage/issues)
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
