# GPT Engage

A unified CLI orchestrator for multiple LLM tools. Run debates between AI systems, maintain persistent conversation sessions, and invoke any CLI-based LLM (Claude Code, Codex, Gemini, and more).

## Features

- **Multi-AI Debates**: Run structured debates between Claude, Codex, and Gemini with configurable rounds
- **Persistent Sessions**: Maintain conversation history across multiple invocations with automatic context injection
- **CLI-Only Design**: No API keys required—works with existing CLI installations
- **Parallel Execution**: All AI systems run simultaneously during debates
- **Flexible Output Formats**: Text, JSON, or Markdown output for debates
- **CLI Agnostic**: Extensible architecture supports any CLI-based LLM tool

## Installation

### Quick Install (Recommended)

```bash
git clone https://github.com/yourusername/gptengage
cd gptengage
./install.sh
```

The installer will:
1. Build GPT Engage in release mode
2. Copy the binary to `~/.local/bin/gptengage`
3. Detect your installed CLIs
4. Show PATH configuration if needed

### Manual Install

```bash
cargo build --release
cp target/release/gptengage ~/.local/bin/
chmod +x ~/.local/bin/gptengage

# Add to PATH if needed
export PATH="$HOME/.local/bin:$PATH"
```

### Prerequisites

- **Rust 1.86+** (for building from source)
- At least one of the following LLM CLIs:
  - [Claude Code](https://claude.com/claude-code) (`claude` command)
  - [Codex CLI](https://github.com/openai/codex-cli) (`codex` command)
  - [Google Gemini CLI](https://ai.google.dev/docs/gemini_cli) (`gemini` command)

### Verify Installation

```bash
gptengage --help
gptengage status
```

## Quick Start

### Check Available CLIs

```bash
$ gptengage status

GPT Engage v1.0.0

Detected CLIs:
  ✓ claude (Claude Code)
  ✓ codex (Codex CLI)
  ✓ gemini (Gemini CLI)

Active Sessions: 0
```

### Invoke a Single CLI

```bash
$ gptengage invoke claude "Explain quantum computing"
```

### Create a Persistent Session

Start a conversation that you can continue later:

```bash
$ gptengage invoke claude "Review my authentication code" --session auth-review
[Claude's response...]
(Session 'auth-review' saved)

$ gptengage invoke claude "Fix the JWT vulnerability" --session auth-review
[Claude now has context from the previous message...]
(Session 'auth-review' saved)
```

### View Session History

```bash
$ gptengage session list
┌──────────────┬────────┬─────────────────────────┬──────────────┐
│ Session      │ CLI    │ Topic                   │ Last Used    │
├──────────────┼────────┼─────────────────────────┼──────────────┤
│ auth-review  │ claude │ Review my authenticatio │ 5 mins ago   │
└──────────────┴────────┴─────────────────────────┴──────────────┘

$ gptengage session show auth-review
```

### Run a Multi-AI Debate

Start a debate on any topic:

```bash
$ gptengage debate "Should we use TypeScript or JavaScript?"

GPT ENGAGE DEBATE
Topic: Should we use TypeScript or JavaScript?

Running round 1 of 3...

ROUND 1
────────────────────────────────────────
CLAUDE:
TypeScript offers type safety and better tooling...

CODEX:
JavaScript provides speed and simplicity...

GEMINI:
Consider the team's experience level...

ROUND 2
────────────────────────────────────────
[Each AI responds, now with context from previous round...]

DEBATE COMPLETE
```

## Commands

### Invoke Command

Invoke a specific LLM CLI with optional session support.

```bash
gptengage invoke <cli> <prompt> [OPTIONS]

Arguments:
  <CLI>      The CLI to invoke: claude, codex, or gemini
  <PROMPT>   The prompt/request to send

Options:
  --session <NAME>       Use or create a persistent session
  --topic <DESC>         Set the session topic (auto-generated if omitted)
  --context-file <PATH>  Include file contents in the prompt
  --timeout <SECONDS>    Command timeout (default: 120)
```

**Examples:**

```bash
# Simple invocation
gptengage invoke claude "What is machine learning?"

# With context file
gptengage invoke claude "Review this code" --context-file src/auth.rs

# With session
gptengage invoke claude "Explain async/await" --session learning

# Continue session
gptengage invoke claude "Give an example" --session learning

# Custom timeout (for slower CLIs like Gemini)
gptengage invoke gemini "Complex task" --timeout 120
```

### Session Commands

Manage persistent conversation sessions.

```bash
gptengage session list         # List all active sessions
gptengage session show <NAME>  # Show session history
gptengage session end <NAME>   # End and delete a session
gptengage session end --all    # End all sessions
```

**Examples:**

```bash
$ gptengage session list
┌──────────────┬────────┬─────────────────────────┬──────────────┐
│ Session      │ CLI    │ Topic                   │ Last Used    │
├──────────────┼────────┼─────────────────────────┼──────────────┤
│ auth-review  │ claude │ Review my authenticatio │ 5 mins ago   │
│ perf-check   │ codex  │ Optimize database query │ 2 hours ago  │
└──────────────┴────────┴─────────────────────────┴──────────────┘

$ gptengage session show auth-review
[Shows full conversation history with timestamps]

$ gptengage session end auth-review
✓ Session 'auth-review' deleted.
```

### Debate Command

Run a structured debate between multiple AI systems.

```bash
gptengage debate <topic> [OPTIONS]

Arguments:
  <TOPIC>  The debate topic

Options:
  --rounds <N>         Number of rounds (default: 3)
  --output <FORMAT>    Output format: text, json, markdown (default: text)
  --timeout <SECONDS>  Timeout per CLI per round (default: 120)
```

**Examples:**

```bash
# Simple 3-round debate
gptengage debate "Tabs or spaces?"

# 5-round debate with JSON output
gptengage debate "Should we migrate to Rust?" --rounds 5 --output json

# Markdown output for documentation
gptengage debate "REST vs GraphQL" --output markdown > debate.md
```

### Status Command

Show available CLIs and active sessions.

```bash
gptengage status
```

**Output:**

```
GPT Engage v1.0.0

Detected CLIs:
  ✓ claude (Claude Code)
  ✓ codex (Codex CLI)
  ✓ gemini (Gemini CLI)

Active Sessions: 2
  • auth-review (claude) - 5 mins ago
  • perf-check (codex) - 2 hours ago

Config: ~/.gptengage/config.json
Sessions: ~/.gptengage/sessions/
```

## How Sessions Work

GPT Engage maintains conversation history without modifying the underlying CLIs. When you continue a session, the full conversation history is injected into the prompt:

```
[CONVERSATION HISTORY]
User: Review my authentication code
Assistant: I found 3 vulnerabilities...

User: Fix the JWT vulnerability
[/CONVERSATION HISTORY]

[CURRENT REQUEST]
Explain how to implement secure token rotation
[/CURRENT REQUEST]
```

This allows Claude, Codex, or Gemini to see the full context and respond appropriately.

## File Storage

GPT Engage stores all data in `~/.gptengage/`:

```
~/.gptengage/
├── config.json                  # Configuration
├── sessions/                    # Conversation history
│   ├── auth-review.json
│   ├── perf-check.json
│   └── ...
└── logs/                        # Optional debug logs
```

Session files are simple JSON:

```json
{
  "name": "auth-review",
  "cli": "claude",
  "topic": "Review my authentication code",
  "createdAt": "2024-01-04T08:30:00Z",
  "lastInteraction": "2024-01-04T10:45:00Z",
  "turns": [
    {
      "role": "user",
      "content": "Review my authentication code",
      "timestamp": "2024-01-04T08:30:00Z"
    },
    {
      "role": "assistant",
      "content": "I found 3 vulnerabilities...",
      "timestamp": "2024-01-04T08:31:15Z"
    }
  ]
}
```

## CLI Requirements & Flags

### Claude Code

**Command:** `claude -p`

- Requires Claude Code CLI to be installed and authenticated
- `-p` flag enables print mode (non-interactive, single-shot)
- Works with any model available via Claude Code

### Codex

**Command:** `codex exec --full-auto`

- Requires Codex CLI to be installed
- `exec` = execute mode
- `--full-auto` = auto-approve operations
- **Note:** May require `--skip-git-repo-check` when outside a trusted git directory

### Gemini

**Command:** `gemini --yolo`

- Requires Google Gemini CLI to be installed and authenticated
- `--yolo` = auto-approve all operations
- **Note:** Typically requires longer timeouts (60+ seconds)

## Troubleshooting

### "Claude Code CLI not found in PATH"

Install Claude Code or add it to your PATH:

```bash
# Check if claude is available
which claude

# Add Claude Code to PATH (adjust path as needed)
export PATH="/path/to/claude:$PATH"
```

### Codex requires git repository trust

If you see: `Not inside a trusted directory and --skip-git-repo-check was not specified`

Run codex once in a trusted directory to add it to the trust list, or use Codex from within a git repository.

### Gemini timeout errors

Gemini can be slow. Increase the timeout:

```bash
gptengage invoke gemini "complex prompt" --timeout 120
```

Or set a global default in `~/.gptengage/config.json`:

```json
{
  "defaultTimeout": 120
}
```

### Sessions not persisting

Check that `~/.gptengage/sessions/` exists and is writable:

```bash
ls -la ~/.gptengage/
# Should show: sessions/ (directory)
```

### Debate incomplete (some CLIs missing)

GPT Engage skips unavailable CLIs and continues with others. Ensure at least one CLI is installed:

```bash
gptengage status
```

## Configuration

GPT Engage configuration is stored in `~/.gptengage/config.json`:

```json
{
  "defaultTimeout": 120,
  "defaultDebateRounds": 3,
  "clis": {
    "claude": {
      "command": "claude",
      "invokeArgs": ["-p"],
      "detected": true
    },
    "codex": {
      "command": "codex",
      "invokeArgs": ["exec", "--full-auto"],
      "detected": true
    },
    "gemini": {
      "command": "gemini",
      "invokeArgs": ["--yolo"],
      "detected": false
    }
  }
}
```

You can modify timeouts or add custom CLIs by editing this file.

## Security Considerations

- **No API Keys Stored**: GPT Engage doesn't store or manage API keys—use the underlying CLI tools' authentication
- **Local Storage Only**: All sessions are stored locally in `~/.gptengage/`—nothing is sent to external servers
- **Subprocess Isolation**: Each CLI invocation is a fresh process with no state leakage
- **Prompt Visibility**: Session prompts include full conversation history—be mindful of sensitive information

## Performance Tips

- **Parallel Debates**: All CLIs run simultaneously during debates, so debate time is determined by the slowest CLI
- **Session Reuse**: Use sessions for related queries to maintain context and reduce redundant explanations
- **Custom Timeouts**: Slow CLIs (like Gemini) benefit from higher timeouts to avoid false timeouts

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/gptengage
cd gptengage

# Build in release mode
cargo build --release

# Binary is at: target/release/gptengage

# Run tests (if available)
cargo test

# Install to ~/.local/bin
./install.sh
```

### Project Structure

```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library root
├── cli.rs                  # CLI argument parsing
├── commands/
│   ├── mod.rs
│   ├── debate.rs
│   ├── invoke.rs
│   ├── session.rs
│   └── status.rs
├── config/
│   └── mod.rs
├── invokers/
│   ├── mod.rs
│   ├── base.rs
│   ├── claude.rs
│   ├── codex.rs
│   └── gemini.rs
├── orchestrator/
│   ├── mod.rs
│   └── debate.rs
├── session/
│   └── mod.rs
└── utils/
    └── mod.rs
```

### Running Tests

```bash
cargo test
```

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see LICENSE file for details

## Support

- **Issues**: Report bugs at [GitHub Issues](https://github.com/yourusername/gptengage/issues)
- **Documentation**: Full docs in [docs/](docs/) directory
- **Examples**: See [examples.md](examples.md) for more use cases

## Roadmap

- [ ] Support for additional LLM CLIs (Llama, Mistral, etc.)
- [ ] Response caching for identical prompts
- [ ] Debate synthesis and conclusion generation
- [ ] Web-based session viewer
- [ ] Debate result export (PDF, HTML)
- [ ] Plugin system for custom CLIs

## Related Projects

- [Claude Code](https://claude.com/claude-code) - Claude's official CLI
- [Codex CLI](https://github.com/openai/codex-cli) - OpenAI's Codex command-line interface
- [Gemini CLI](https://ai.google.dev/docs/gemini_cli) - Google's Gemini command-line tool

---

Made with ❤️ for the open source community
