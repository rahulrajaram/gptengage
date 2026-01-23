# GPT Engage

A unified CLI orchestrator for multiple LLM tools. Run debates between AI systems, maintain persistent conversation sessions, and invoke any CLI-based LLM (Claude Code, Codex, Gemini, and more).

## Features

- **Multi-AI Debates**: Run structured debates between Claude, Codex, and Gemini with configurable rounds
- **Multi-Instance Debates**: Spawn multiple instances of the same LLM to leverage nondeterminism and debate dynamics
- **Role-Based Personas**: Assign perspectives (CEO, Architect, PM) for context-specific debates
- **Agent Definition Files**: Structured JSON with validation for programmatic use by AI agents
- **AI-Powered Generation**: Auto-generate rich agent definitions using `generate-agents`
- **Persistent Sessions**: Maintain conversation history across multiple invocations with automatic context injection
- **CLI-Only Design**: No API keys required—works with existing CLI installations
- **Parallel Execution**: All AI systems run simultaneously during debates
- **Flexible Output Formats**: Text, JSON, or Markdown output for debates
- **CLI Agnostic**: Extensible architecture supports any CLI-based LLM tool

## Quick Reference

| Task | Command |
|------|---------|
| Check status | `gptengage status` |
| Single invocation | `gptengage invoke <cli> "prompt"` |
| With session | `gptengage invoke <cli> "prompt" --session name` |
| Cross-AI debate | `gptengage debate "topic"` |
| Multi-instance debate | `gptengage debate "topic" --agent claude` |
| Persona debate | `gptengage debate "topic" -p "claude:CEO,codex:CTO"` |
| Agent-file debate | `gptengage debate "topic" --agent-file agents.json` |
| Generate agents | `gptengage generate-agents --topic "..." --roles "..." -o file.json` |

## For AI Agents (Programmatic Use)

If you are an AI agent using gptengage, you have multiple debate options:

### Option 1: Multi-Instance Debate (Quick & Simple)

For multiple independent evaluations from the same LLM:

```bash
# 3 Claude instances (leverages nondeterminism)
gptengage debate "Your topic" --agent claude

# 5 Gemini instances (custom count)
gptengage debate "Your topic" --agent gemini --instances 5
```

### Option 2: Agent Definition Files (Full Control)

For structured debates with personas, instructions, and expertise:

**Step 1: Generate Agent Definitions**

```bash
gptengage generate-agents \
  --topic "Your debate topic" \
  --roles "Role1,Role2,Role3" \
  --output agents.json
```

This creates a validated JSON file with schema version 1.0.

**Step 2: Run Debate with Agent File**

```bash
gptengage debate "Your debate topic" --agent-file agents.json
```

**Required fields** in agent files (validated):
- `cli`: string (non-empty)
- `persona`: string (non-empty, required)
- `instructions`: string (min 10 chars, required)
- `expertise`: array of strings (optional)
- `communication_style`: string (optional)

> **Important:** The `instructions` field must be at least 10 characters. This ensures meaningful behavioral guidance for each participant. Instructions like "Be brief" (8 chars) will fail validation. Use descriptive instructions like "Focus on technical accuracy and provide concrete examples." (55 chars).

**Example agent file:**

```json
{
  "schema_version": "1.0",
  "generated_by": "your-agent-name",
  "participants": [
    {
      "cli": "claude",
      "persona": "Technical Architect",
      "instructions": "Focus on scalability, performance, and technical trade-offs. Challenge assumptions and ask clarifying questions.",
      "expertise": ["system design", "distributed systems", "performance optimization"],
      "communication_style": "Technical and detailed"
    }
  ]
}
```

### Option 3: Parse Output

Use `--output json` for machine-readable output:

```bash
gptengage debate "topic" --agent-file agents.json --output json > result.json
```

### Agent Integration Contract

This section defines the stable interface contract for programmatic integration.

#### Input Contract (Agent Definition File)

| Field | Type | Required | Validation | Description |
|-------|------|----------|------------|-------------|
| `schema_version` | string | Yes | Must be `"1.0"` | Schema version for forward compatibility |
| `generated_by` | string | No | Non-empty if present | Identifier of generating agent |
| `participants` | array | Yes | Min 1 element | Array of participant definitions |
| `participants[].cli` | string | Yes | `claude`, `codex`, or `gemini` | Target CLI for this participant |
| `participants[].persona` | string | Yes | Non-empty | Role name (e.g., "CEO", "Architect") |
| `participants[].instructions` | string | Yes | Min 10 characters | Behavioral instructions for the participant |
| `participants[].expertise` | array | No | Array of strings | Areas of expertise (3-5 recommended) |
| `participants[].communication_style` | string | No | Non-empty if present | Communication style descriptor |

#### Output Contract (`--output json`)

| Field | Type | Always Present | Description |
|-------|------|----------------|-------------|
| `topic` | string | Yes | The debate topic |
| `rounds` | number | Yes | Number of rounds completed |
| `participants` | array | Yes | List of participant identifiers |
| `responses` | array | Yes | Array of round response objects |
| `responses[].round` | number | Yes | Round number (1-indexed) |
| `responses[].participant` | string | Yes | Participant identifier |
| `responses[].content` | string | Yes | Response content |
| `responses[].timestamp` | string | Yes | ISO 8601 timestamp |
| `metadata.duration_ms` | number | Yes | Total execution time |
| `metadata.schema_version` | string | Yes | Output schema version |

> **Note:** The schema version follows semantic versioning. Breaking changes will increment the major version. Minor additions (new optional fields) increment the minor version. The current stable version is `1.0`.

## Installation

### Quick Install (Recommended)

```bash
git clone https://github.com/rahulrajaram/gptengage
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

Run a structured debate between multiple AI systems with optional personas and agent definitions.

```bash
gptengage debate <topic> [OPTIONS]

Arguments:
  <TOPIC>  The debate topic

Options:
      --agent <CLI>                  Single CLI to use (claude, codex, or gemini)
      --instances <N>                Number of instances (default: 3 when --agent specified)
  -p, --participants <PARTICIPANTS>  Participants with personas (format: "cli:persona,cli:persona")
      --agent-file <FILE>            Path to agent definition JSON file
      --rounds <N>                   Number of rounds (default: 3)
      --output <FORMAT>              Output format: text, json, markdown (default: text)
      --timeout <SECONDS>            Timeout per CLI per round (default: 120)
```

#### Simple Debate (Cross-AI, No Personas)

```bash
# Default 3-round debate with Claude, Codex, and Gemini
gptengage debate "Should we migrate to microservices?"
```

This mode provides **model diversity** - different LLMs (Claude vs Codex vs Gemini) bring different perspectives.

#### Multi-Instance Debate (Same LLM, Leverages Nondeterminism)

```bash
# 3 Claude instances (default)
gptengage debate "Code review best practices" --agent claude

# 5 Gemini instances
gptengage debate "API design patterns" --agent gemini --instances 5

# Custom rounds
gptengage debate "Security audit checklist" --agent claude --instances 3 --rounds 4
```

This mode leverages **LLM nondeterminism** - the same LLM will produce different outputs each time, and participants respond to each other's inputs during the debate. Useful when you want multiple perspectives from the same model.

**When to use:**
- You want multiple independent evaluations from the same LLM
- You need diverse viewpoints but prefer a single model's style
- You're testing for consistency across nondeterministic outputs

#### Debate with Personas (Perspective Diversity)

Assign roles/personas to participants for perspective-based debates:

```bash
# Three Claude instances with different roles
gptengage debate "Should we adopt Kubernetes?" \
  -p "claude:CTO,claude:Principal Architect,claude:DevOps Lead"

# Mixed CLIs with personas
gptengage debate "API design strategy" \
  -p "claude:Backend Lead,codex:Frontend Lead,gemini:Product Manager"

# 5 rounds with personas and JSON output
gptengage debate "Microservices vs Monolith" \
  -p "claude:CEO,claude:Architect,codex:Engineer" \
  --rounds 5 --output json
```

#### Debate with Agent Definition Files (For Agents/Programmatic Use)

Agent files provide full structured definitions with instructions, expertise, and communication styles:

```bash
# Generate agent definitions using AI
gptengage generate-agents \
  --topic "Should we migrate to microservices?" \
  --roles "CEO,Principal Architect,Product Manager" \
  --output agents.json

# Use the generated agents in a debate
gptengage debate "Should we migrate to microservices?" \
  --agent-file agents.json
```

**Agent file format** (JSON schema version 1.0):

```json
{
  "schema_version": "1.0",
  "generated_by": "gptengage-claude",
  "participants": [
    {
      "cli": "claude",
      "persona": "CEO",
      "instructions": "Focus on business impact, ROI, and strategic alignment. Be decisive but ask about risks. Keep responses under 3 paragraphs.",
      "expertise": ["business strategy", "finance", "leadership", "market analysis"],
      "communication_style": "Executive - concise and action-oriented"
    },
    {
      "cli": "claude",
      "persona": "Principal Architect",
      "instructions": "Evaluate technical feasibility, scalability, and maintainability. Raise concerns about technical debt and long-term consequences.",
      "expertise": ["system design", "distributed systems", "security", "scalability"],
      "communication_style": "Technical and thorough"
    }
  ]
}
```

**Validation:** Agent files enforce strict validation:
- ✅ `persona` field is required and non-empty
- ✅ `instructions` field is required (minimum 10 characters)
- ✅ Schema version must be "1.0"
- ❌ Fails with clear error message if validation fails

This allows **agents** (programmatic use) to ensure structure while **humans** keep the simple `-p` format.

### Generate Agents Command

**FOR AGENTS/PROGRAMMATIC USE:** Generate AI-powered agent definitions with full structured metadata.

```bash
gptengage generate-agents --topic <TOPIC> --roles <ROLES> --output <FILE> [OPTIONS]

Required:
  --topic <TOPIC>   The debate topic (provides context for agent generation)
  --roles <ROLES>   Comma-separated list of roles (e.g., "CEO,Architect,PM")
  --output <FILE>   Output file path for the generated JSON

Options:
  --use-cli <CLI>   CLI to use for generation: claude, codex, gemini (default: claude)
  --timeout <SECS>  Timeout in seconds (default: 120)
```

**Examples:**

```bash
# Generate 3 agents for a microservices debate
gptengage generate-agents \
  --topic "Should we migrate to microservices?" \
  --roles "CEO,Principal Architect,Product Manager" \
  --output agents.json

# Use Codex instead of Claude for generation
gptengage generate-agents \
  --topic "API design strategy" \
  --roles "Backend Lead,Frontend Lead,DBA" \
  --output api-agents.json \
  --use-cli codex

# Generate security-focused agents
gptengage generate-agents \
  --topic "Cloud security audit findings" \
  --roles "CISO,Security Architect,Compliance Officer" \
  --output security-agents.json
```

**What it does:**

1. Uses AI (Claude/Codex/Gemini) to generate detailed agent definitions
2. Creates structured JSON with:
   - **persona**: Role name (e.g., "CEO", "Principal Architect")
   - **instructions**: Detailed behavioral instructions (2-4 sentences)
   - **expertise**: Array of 3-5 expertise areas
   - **communication_style**: How this role communicates
3. Validates all fields before saving
4. Outputs ready-to-use JSON file with schema version 1.0

**Output:**

```
✅ Generated 3 agent definition(s)
📄 Saved to: agents.json

Agents:
  - claude (CEO)
  - claude (Principal Architect)
  - codex (Product Manager)

Use with: gptengage debate "Should we migrate to microservices?" --agent-file agents.json
```

**Why use this?**

- **For agents**: Ensures strict validation and structured output
- **For complex debates**: Rich context injection with instructions and expertise
- **For repeatability**: Save and reuse agent definitions across multiple debates

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

## Exit Codes

GPT Engage uses standardized exit codes for scripting and automation:

| Exit Code | Meaning | Example Scenario |
|-----------|---------|------------------|
| 0 | Success | Command completed successfully |
| 1 | CLI not found or invocation failed | Specified CLI not installed or returned error |
| 2 | Session error or invalid format | Invalid session name, corrupted session file, or malformed JSON |
| 3 | File not found | Agent file, context file, or config file does not exist |
| 4 | Timeout exceeded | CLI did not respond within the specified timeout |

**Example usage in scripts:**

```bash
gptengage invoke claude "test prompt" --timeout 60
exit_code=$?

case $exit_code in
  0) echo "Success" ;;
  1) echo "CLI error - check installation" ;;
  2) echo "Session/format error" ;;
  3) echo "File not found" ;;
  4) echo "Timeout - increase --timeout value" ;;
  *) echo "Unknown error: $exit_code" ;;
esac
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

> **Warning:** Session files are stored unencrypted on disk. Do not include sensitive information (passwords, API keys, PII) in session prompts or responses. Consider using `gptengage session end --all` after working with sensitive topics.

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

**Command:** `claude -p --tools Read --allowed-tools Read`

- Requires Claude Code CLI to be installed and authenticated
- `-p` flag enables print mode (non-interactive, single-shot)
- `--tools Read` restricts available tools to read-only access
- `--allowed-tools Read` allows read-only access without prompting
- Works with any model available via Claude Code

### Codex

**Command:** `codex exec --sandbox read-only --cd .`

- Requires Codex CLI to be installed
- `exec` = execute mode
- `--sandbox read-only` = restricts shell execution to read-only operations
- `--cd .` = restricts the workspace root to the current directory
- **Note:** May require `--skip-git-repo-check` when outside a trusted git directory

### Gemini

**Command:** `gemini --sandbox --include-directories .`

- Requires Google Gemini CLI to be installed and authenticated
- `--sandbox` = run in sandboxed mode with approval prompts (no YOLO)
- `--include-directories .` = limit workspace to the current directory

> **Note:** Gemini CLI typically requires longer timeouts (60-120 seconds) compared to Claude or Codex. For debates, consider using `--timeout 120` or higher. The default 120-second timeout is usually sufficient, but complex prompts may need more time.

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
      "invokeArgs": ["-p", "--tools", "Read", "--allowed-tools", "Read"],
      "detected": true
    },
    "codex": {
      "command": "codex",
      "invokeArgs": ["exec", "--sandbox", "read-only", "--cd", "."],
      "detected": true
    },
    "gemini": {
      "command": "gemini",
      "invokeArgs": ["--sandbox", "--include-directories", "."],
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
git clone https://github.com/rahulrajaram/gptengage
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

- **Issues**: Report bugs at [GitHub Issues](https://github.com/rahulrajaram/gptengage/issues)
- **Documentation**: Full docs in [docs/](docs/) directory
- **Examples**: See [docs/EXAMPLES.md](docs/EXAMPLES.md) for more use cases

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
