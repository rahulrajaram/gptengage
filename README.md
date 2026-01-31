# GPT Engage

GPT Engage is a command-line orchestrator for multi-AI debates. It runs structured debates between Claude, Codex, Gemini, and any CLI-based LLM. GPT Engage executes all participants in parallel, injects debate context between rounds, and produces synthesis reports.

## Overview

GPT Engage solves a specific problem: getting multiple AI perspectives on a single question without manually copying responses between chat interfaces. You provide a topic, GPT Engage invokes each participant simultaneously, shares each round's responses with all participants, and optionally generates a synthesis that identifies consensus and disagreements.

**Key capabilities:**

- **Multi-AI debates** - Run Claude, Codex, and Gemini against the same prompt in parallel
- **Multi-instance debates** - Spawn multiple instances of the same LLM to leverage nondeterminism
- **Debate templates** - Use pre-configured scenarios for code review, architecture decisions, and security audits
- **Synthesis generation** - Automatically generate conclusions that summarize consensus and disagreements
- **Plugin system** - Add custom CLIs through TOML configuration files
- **Unix composability** - Pipe content via stdin as topic or context
- **Persistent sessions** - Maintain conversation history across invocations

## Installation

### Prerequisites

GPT Engage requires Rust 1.86 or later and at least one LLM CLI:

| CLI | Installation |
|-----|--------------|
| Claude Code | https://claude.com/claude-code |
| Codex CLI | https://github.com/openai/codex-cli |
| Gemini CLI | https://ai.google.dev/docs/gemini_cli |

### Build and Install

Clone the repository and run the installer:

```bash
git clone https://github.com/rahulrajaram/gptengage
cd gptengage
./install.sh
```

The installer builds GPT Engage in release mode, copies the binary to `~/.local/bin/gptengage`, and verifies your PATH configuration.

For manual installation:

```bash
cargo build --release
cp target/release/gptengage ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
```

Verify the installation:

```bash
gptengage status
```

## Quick Start

### Check Available CLIs

Run the status command to see which CLIs GPT Engage detects:

```bash
gptengage status
```

Output:

```
GPT Engage v1.0.0

Detected CLIs:
  ✓ claude (Claude Code)
  ✓ codex (Codex CLI)
  ✗ gemini (Gemini CLI)

Plugins:
  ✓ ollama (Local LLM via Ollama)

Active Sessions: 0
```

### Run a Debate

Start a debate on any topic:

```bash
gptengage debate "Should we use TypeScript or JavaScript for our new project?"
```

GPT Engage runs all detected CLIs in parallel, collects responses, and displays each round:

```
GPT ENGAGE DEBATE
Topic: Should we use TypeScript or JavaScript for our new project?

ROUND 1
────────────────────────────────────────
Claude:
TypeScript provides compile-time type checking that catches errors before runtime...

Codex:
JavaScript offers faster iteration and lower barrier to entry...

ROUND 2
────────────────────────────────────────
[Each participant responds with context from Round 1]

DEBATE COMPLETE
Summary: 3 round(s), 2 participant(s)
```

### Generate Synthesis

Add `--synthesize` to generate a structured conclusion:

```bash
gptengage debate "Microservices vs Monolith" --synthesize
```

The synthesis includes:
- Summary of the debate
- Points of consensus
- Points of disagreement
- Key insights
- Recommendation (when applicable)

### Use Templates

List available templates:

```bash
gptengage template list
```

Run a debate with a template:

```bash
gptengage debate "Review PR #123" --template code-review
```

### Pipe Content

Pipe file content as context:

```bash
cat src/auth.rs | gptengage debate "Review this authentication code" --stdin-as context
```

Pipe content as the topic:

```bash
echo "Is Rust better than Go for CLI tools?" | gptengage debate
```

## Commands

### debate

Run a structured debate between multiple AI participants.

```bash
gptengage debate <TOPIC> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `TOPIC` | The debate topic. Provide as argument or pipe via stdin. |

**Options:**

| Option | Description |
|--------|-------------|
| `--agent <CLI>` | Run multiple instances of a single CLI instead of cross-AI debate. |
| `--instances <N>` | Number of instances when using `--agent`. Default: 3. |
| `-p, --participants <LIST>` | Specify participants with optional personas. Format: `cli:persona,cli:persona`. |
| `--agent-file <FILE>` | Load participant definitions from a JSON file. |
| `--template <NAME>` | Use a predefined debate template. |
| `--rounds <N>` | Number of debate rounds. Default: 3. |
| `--synthesize` | Generate a synthesis after the debate completes. |
| `--synthesizer <CLI>` | CLI to use for synthesis generation. Default: `claude`. |
| `--output <FORMAT>` | Output format: `text`, `json`, or `markdown`. Default: `text`. |
| `--stdin-as <MODE>` | How to interpret stdin: `auto`, `context`, or `ignore`. Default: `auto`. |
| `--timeout <SECONDS>` | Timeout per CLI per round. Default: 120. |
| `--write` | Allow write access within the current directory. Default: read-only. |

**Examples:**

Cross-AI debate with default participants:

```bash
gptengage debate "Should we adopt Kubernetes?"
```

Multi-instance debate with the same CLI:

```bash
gptengage debate "Code review best practices" --agent claude --instances 5
```

Debate with personas:

```bash
gptengage debate "API design strategy" -p "claude:Backend Lead,codex:Frontend Lead,gemini:Product Manager"
```

Debate with template and synthesis:

```bash
gptengage debate "Review this PR" --template code-review --synthesize --output markdown
```

JSON output for programmatic consumption:

```bash
gptengage debate "topic" --output json > result.json
```

### invoke

Invoke a single CLI with optional session support.

```bash
gptengage invoke <CLI> <PROMPT> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `CLI` | The CLI to invoke: `claude`, `codex`, `gemini`, or a plugin name. |
| `PROMPT` | The prompt to send. Provide as argument or pipe via stdin. |

**Options:**

| Option | Description |
|--------|-------------|
| `--session <NAME>` | Use or create a persistent session. |
| `--topic <DESC>` | Set the session topic. Auto-generated if omitted. |
| `--context-file <PATH>` | Include file contents in the prompt. |
| `--stdin-as <MODE>` | How to interpret stdin: `auto`, `context`, or `ignore`. Default: `auto`. |
| `--timeout <SECONDS>` | Command timeout. Default: 120. |
| `--write` | Allow write access within the current directory. |

**Examples:**

Simple invocation:

```bash
gptengage invoke claude "Explain async/await in Rust"
```

With session for multi-turn conversation:

```bash
gptengage invoke claude "Review my authentication code" --session auth-review
gptengage invoke claude "Fix the JWT vulnerability you found" --session auth-review
```

With context file:

```bash
gptengage invoke codex "Optimize this function" --context-file src/parser.rs
```

Pipe content as prompt:

```bash
echo "What is 2 + 2?" | gptengage invoke claude
```

### template

Manage debate templates.

```bash
gptengage template <COMMAND>
```

**Subcommands:**

| Command | Description |
|---------|-------------|
| `list` | List all available templates (built-in and user-defined). |
| `show <NAME>` | Display template details including participants and context. |

**Examples:**

List templates:

```bash
gptengage template list
```

Output:

```
Available Templates:

  code-review (built-in)
      Multi-perspective code review with security, performance, and maintainability focus
      Participants: 3, Rounds: 2

  architecture-decision (built-in)
      Evaluate architectural choices from multiple stakeholder perspectives
      Participants: 3, Rounds: 3

  security-audit (built-in)
      Security-focused analysis from CISO, engineer, and compliance perspectives
      Participants: 3, Rounds: 2

Use a template: gptengage debate "topic" --template <name>
```

Show template details:

```bash
gptengage template show code-review
```

### plugin

Manage custom CLI plugins.

```bash
gptengage plugin <COMMAND>
```

**Subcommands:**

| Command | Description |
|---------|-------------|
| `list` | List all installed plugins. |
| `validate <FILE>` | Validate a plugin configuration file. |

**Examples:**

List plugins:

```bash
gptengage plugin list
```

Validate a plugin file:

```bash
gptengage plugin validate ~/.gptengage/plugins/ollama.toml
```

### session

Manage persistent conversation sessions.

```bash
gptengage session <COMMAND>
```

**Subcommands:**

| Command | Description |
|---------|-------------|
| `list` | List all active sessions. |
| `show <NAME>` | Display session history. |
| `end <NAME>` | Delete a session. |
| `end --all` | Delete all sessions. |

### generate-agents

Generate AI-powered agent definitions for structured debates.

```bash
gptengage generate-agents --topic <TOPIC> --roles <ROLES> --output <FILE> [OPTIONS]
```

**Required Options:**

| Option | Description |
|--------|-------------|
| `--topic <TOPIC>` | The debate topic for context. |
| `--roles <ROLES>` | Comma-separated list of roles. |
| `--output <FILE>` | Output file path for generated JSON. |

**Optional Options:**

| Option | Description |
|--------|-------------|
| `--use-cli <CLI>` | CLI to use for generation. Default: `claude`. |
| `--timeout <SECONDS>` | Timeout. Default: 120. |

**Example:**

```bash
gptengage generate-agents \
  --topic "Should we migrate to microservices?" \
  --roles "CEO,Principal Architect,Product Manager" \
  --output agents.json

gptengage debate "Should we migrate to microservices?" --agent-file agents.json
```

### status

Display GPT Engage status, detected CLIs, plugins, and active sessions.

```bash
gptengage status
```

## Debate Templates

GPT Engage includes five built-in templates. Each template defines participants with specific personas, instructions, and expertise areas.

### code-review

Multi-perspective code review focusing on security, performance, and maintainability.

**Participants:**
- Security Reviewer (Claude) - Identifies vulnerabilities, injection risks, and authentication issues
- Performance Reviewer (Claude) - Finds bottlenecks, inefficient algorithms, and resource leaks
- Maintainability Reviewer (Claude) - Evaluates readability, test coverage, and coding standards

**Usage:**

```bash
cat pull_request.diff | gptengage debate "Review these changes" --template code-review --stdin-as context
```

### architecture-decision

Evaluate architectural choices from multiple stakeholder perspectives.

**Participants:**
- Solution Architect (Claude) - Focuses on scalability, reliability, and technical trade-offs
- Senior Developer (Codex) - Considers implementation complexity and developer experience
- Operations Engineer (Gemini) - Evaluates deployment, monitoring, and operational concerns

### security-audit

Security-focused analysis from multiple security perspectives.

**Participants:**
- CISO (Claude) - Strategic security risks and business impact
- Security Engineer (Claude) - Technical vulnerabilities and attack vectors
- Compliance Officer (Claude) - Regulatory requirements and audit concerns

### api-design

API design review from consumer and provider perspectives.

**Participants:**
- API Architect (Claude) - RESTful design, versioning, and documentation
- Frontend Developer (Codex) - Developer experience and client integration
- Backend Developer (Gemini) - Implementation concerns and performance

### incident-postmortem

Structured incident analysis for post-incident reviews.

**Participants:**
- SRE Lead (Claude) - Root cause analysis and reliability improvements
- Development Lead (Codex) - Code-level issues and fix strategies
- Product Manager (Gemini) - Customer impact and communication

### Custom Templates

Create custom templates by adding TOML files to `~/.gptengage/templates/`:

```toml
# ~/.gptengage/templates/my-template.toml
[template]
name = "my-template"
description = "Custom debate template"
default_rounds = 3

[[participants]]
cli = "claude"
persona = "Expert A"
instructions = "Focus on aspect X. Provide specific examples and cite sources."
expertise = ["topic1", "topic2", "topic3"]

[[participants]]
cli = "codex"
persona = "Expert B"
instructions = "Focus on aspect Y. Challenge assumptions and propose alternatives."
expertise = ["topic4", "topic5"]

[context]
prefix = "Consider the following context:"
suffix = "Provide actionable recommendations."
```

## Plugin System

GPT Engage supports custom CLIs through TOML-based plugin configuration. Plugins enable integration with any command-line LLM tool.

### Plugin Location

Place plugin files in `~/.gptengage/plugins/` with `.toml` extension.

### Plugin Format

```toml
# ~/.gptengage/plugins/ollama.toml
[plugin]
name = "ollama"
description = "Local LLM via Ollama"
command = "ollama"

[invoke]
base_args = ["run", "llama3"]
prompt_mode = "stdin"

[access]
readonly_args = []
write_args = []

[detection]
check_command = "ollama"
check_args = ["--version"]
```

### Configuration Fields

**[plugin] section:**

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Plugin identifier used in commands. |
| `description` | Yes | Human-readable description. |
| `command` | Yes | Executable command name. |

**[invoke] section:**

| Field | Required | Description |
|-------|----------|-------------|
| `base_args` | Yes | Base arguments passed to the command. |
| `prompt_mode` | Yes | How to pass the prompt: `stdin`, `arg`, or `arg_last`. |
| `prompt_arg` | No | Argument flag for prompt when using `arg` mode. |

**[access] section:**

| Field | Required | Description |
|-------|----------|-------------|
| `readonly_args` | Yes | Additional arguments for read-only mode. |
| `write_args` | Yes | Additional arguments for write mode. |

**[detection] section:**

| Field | Required | Description |
|-------|----------|-------------|
| `check_command` | Yes | Command to verify CLI availability. |
| `check_args` | Yes | Arguments for availability check. |

### Example: Aider Plugin

```toml
# ~/.gptengage/plugins/aider.toml
[plugin]
name = "aider"
description = "AI pair programming with Aider"
command = "aider"

[invoke]
base_args = ["--message"]
prompt_mode = "arg_last"

[access]
readonly_args = ["--no-auto-commits"]
write_args = ["--auto-commits"]

[detection]
check_command = "aider"
check_args = ["--version"]
```

### Using Plugins

After creating a plugin file, verify it appears in status:

```bash
gptengage status
```

Use the plugin in debates:

```bash
gptengage debate "topic" -p "claude:Analyst,ollama:Reviewer"
```

Or invoke directly:

```bash
gptengage invoke ollama "Explain this concept"
```

## Stdin Piping

GPT Engage accepts input via Unix pipes for composability with other tools.

### Piping Modes

| Mode | Behavior |
|------|----------|
| `auto` | Use stdin as topic if no topic argument provided; otherwise use as context. |
| `context` | Always prepend stdin as context with `[PIPED CONTEXT]` markers. |
| `ignore` | Discard stdin input. |

### Examples

Pipe as topic:

```bash
echo "Is functional programming better than OOP?" | gptengage debate
```

Pipe file as context with explicit topic:

```bash
cat error.log | gptengage debate "What caused this error?" --stdin-as context
```

Pipe git diff for code review:

```bash
git diff HEAD~1 | gptengage debate "Review these changes" --template code-review --stdin-as context
```

Pipe to invoke:

```bash
cat README.md | gptengage invoke claude "Summarize this document"
```

### Context Injection Format

When stdin is used as context, GPT Engage injects it with markers:

```
[PIPED CONTEXT]
<stdin content>
[/PIPED CONTEXT]

<topic or prompt>
```

## Synthesis Generation

GPT Engage generates structured synthesis reports after debates complete.

### Enabling Synthesis

Add `--synthesize` to any debate command:

```bash
gptengage debate "topic" --synthesize
```

### Specifying Synthesizer

By default, Claude generates the synthesis. Specify a different CLI with `--synthesizer`:

```bash
gptengage debate "topic" --synthesize --synthesizer codex
```

### Synthesis Output

The synthesis includes:

| Field | Description |
|-------|-------------|
| `summary` | 2-3 sentence overview of the debate. |
| `consensus_points` | Points where participants agreed. |
| `disagreement_points` | Points where participants disagreed. |
| `key_insights` | Notable insights that emerged. |
| `recommendation` | Actionable recommendation when applicable. |

**Text output:**

```
SYNTHESIS
────────────────────────────────────────

Summary:
  The debate focused on microservices vs monolith architecture...

Consensus:
  • All participants agreed that team size affects the decision
  • Performance requirements favor monolithic architecture initially

Disagreements:
  • Timeline for migration varied significantly between perspectives

Key Insights:
  • Start with a modular monolith and extract services when needed

Recommendation:
  Begin with a well-structured monolith and identify extraction candidates...
```

**JSON output:**

```json
{
  "topic": "Microservices vs Monolith",
  "rounds": [...],
  "synthesis": {
    "summary": "The debate focused on...",
    "consensus_points": ["Point 1", "Point 2"],
    "disagreement_points": ["Point 1"],
    "key_insights": ["Insight 1"],
    "recommendation": "Begin with..."
  }
}
```

## Agent Definition Files

Agent definition files provide structured participant configurations for programmatic use.

### File Format

```json
{
  "schema_version": "1.0",
  "generated_by": "gptengage-claude",
  "participants": [
    {
      "cli": "claude",
      "persona": "CEO",
      "instructions": "Focus on business impact, ROI, and strategic alignment. Be decisive but ask about risks.",
      "expertise": ["business strategy", "finance", "leadership"],
      "communication_style": "Executive - concise and action-oriented"
    }
  ]
}
```

### Field Validation

| Field | Required | Validation |
|-------|----------|------------|
| `schema_version` | Yes | Must be `"1.0"`. |
| `generated_by` | No | Non-empty if present. |
| `participants` | Yes | At least one participant. |
| `participants[].cli` | Yes | Valid CLI name or plugin. |
| `participants[].persona` | Yes | Non-empty string. |
| `participants[].instructions` | Yes | Minimum 10 characters. |
| `participants[].expertise` | No | Array of strings. |
| `participants[].communication_style` | No | Non-empty if present. |

### Generating Agent Files

Use `generate-agents` to create agent files with AI assistance:

```bash
gptengage generate-agents \
  --topic "Cloud migration strategy" \
  --roles "CTO,Cloud Architect,Security Lead" \
  --output agents.json
```

## Session Management

Sessions maintain conversation history across invocations without modifying underlying CLIs.

### How Sessions Work

When you continue a session, GPT Engage injects the full history into the prompt:

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

### Session Storage

Sessions are stored in `~/.gptengage/sessions/` as JSON files:

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

### Security Considerations

Session files are stored unencrypted. Do not include sensitive information (passwords, API keys, PII) in session prompts. Use `gptengage session end --all` after working with sensitive topics.

## Output Formats

GPT Engage supports three output formats for debate results.

### Text (Default)

Human-readable format with visual separators:

```bash
gptengage debate "topic"
```

### JSON

Machine-readable format for programmatic consumption:

```bash
gptengage debate "topic" --output json
```

Output structure:

```json
{
  "gptengage_version": "1.0.0",
  "topic": "Should we use Rust?",
  "rounds": [
    [
      {"cli": "claude", "persona": null, "response": "..."},
      {"cli": "codex", "persona": null, "response": "..."}
    ]
  ],
  "synthesis": null
}
```

### Markdown

Formatted for documentation or reports:

```bash
gptengage debate "topic" --output markdown > debate.md
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | CLI not found or invocation failed |
| 2 | Session error or invalid format |
| 3 | File not found |
| 4 | Timeout exceeded |

Use exit codes for scripting:

```bash
gptengage invoke claude "test" --timeout 60
if [ $? -eq 4 ]; then
  echo "Timeout - increase --timeout value"
fi
```

## CLI Requirements

### Claude Code

GPT Engage invokes Claude with:

```bash
claude -p --tools Read --allowed-tools Read
```

With `--write` flag:

```bash
claude -p
```

### Codex CLI

GPT Engage invokes Codex with:

```bash
codex exec --sandbox read-only --cd .
```

With `--write` flag:

```bash
codex exec --cd .
```

### Gemini CLI

GPT Engage invokes Gemini with:

```bash
gemini --sandbox --include-directories .
```

Gemini typically requires longer timeouts. Use `--timeout 180` for complex prompts.

## File Locations

| Path | Purpose |
|------|---------|
| `~/.gptengage/config.json` | Configuration file |
| `~/.gptengage/sessions/` | Session storage |
| `~/.gptengage/plugins/` | Custom CLI plugins |
| `~/.gptengage/templates/` | User-defined templates |

## Troubleshooting

### CLI not found

Verify the CLI is installed and in your PATH:

```bash
which claude
which codex
which gemini
```

### Codex requires git repository trust

Run Codex once in a trusted directory to add it to the trust list, or run GPT Engage from within a git repository.

### Gemini timeout errors

Increase the timeout:

```bash
gptengage invoke gemini "prompt" --timeout 180
```

### Plugin not detected

Verify the plugin file syntax:

```bash
gptengage plugin validate ~/.gptengage/plugins/myplugin.toml
```

Check that the plugin command exists:

```bash
which <command-name>
```

## Development

### Building from Source

```bash
git clone https://github.com/rahulrajaram/gptengage
cd gptengage
cargo build --release
cargo test
```

### Project Structure

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library root
├── cli.rs            # Argument parsing
├── commands/         # Command implementations
├── invokers/         # CLI invokers
├── orchestrator/     # Debate orchestration
├── plugins/          # Plugin system
├── templates/        # Template system
├── session/          # Session management
└── utils/            # Utilities including stdin handling
```

### Running Tests

```bash
cargo test
```

## License

MIT License. See LICENSE file.

## Support

Report issues at https://github.com/rahulrajaram/gptengage/issues
