# GPT Engage Commands Reference

Complete reference documentation for all GPT Engage commands.

## Table of Contents

- [gptengage invoke](#gptengage-invoke) - Invoke a specific LLM CLI
- [gptengage debate](#gptengage-debate) - Run multi-AI debates
- [Structured Output Specification](#structured-output-specification) - JSON and Markdown output schemas
- [gptengage session](#gptengage-session) - Manage conversation sessions
- [gptengage status](#gptengage-status) - Show available CLIs and sessions
- [gptengage config](#gptengage-config) - Manage configuration

---

## gptengage invoke

Invoke a specific LLM CLI (Claude, Codex, or Gemini) with optional session support for multi-turn conversations.

### Syntax

```bash
gptengage invoke <CLI> <PROMPT> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<CLI>` | The CLI to invoke: `claude`, `codex`, or `gemini` |
| `<PROMPT>` | The prompt/request to send to the CLI (quoted if contains spaces) |

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--session <NAME>` | Create or continue a named session | (none - one-shot) |
| `--topic <DESC>` | Set the session topic (auto-generated if omitted) | Auto-generated from prompt |
| `--context-file <PATH>` | Include file contents in the prompt | (none) |
| `--timeout <SECONDS>` | Command timeout in seconds | 120 |
| `--help` | Show help for this command | - |

### Examples

#### Basic Invocation

```bash
# Simple one-shot request
$ gptengage invoke claude "What is machine learning?"
Machine learning is...

# Invoke a different CLI
$ gptengage invoke codex "Optimize this algorithm"
$ gptengage invoke gemini "Explain this error"
```

#### With File Context

Include a file's contents in your prompt:

```bash
# Review a file
$ gptengage invoke claude "Review this code for security issues" --context-file src/auth.rs
[Claude reviews the file...]

# Include multiple concepts
$ gptengage invoke claude "Help me understand this file" --context-file README.md
```

#### With Sessions (Multi-turn Conversations)

Create and continue a persistent session:

```bash
# Start a new session
$ gptengage invoke claude "Explain closures in JavaScript" --session js-learning
Here's how closures work...
(Session 'js-learning' saved)

# Continue the same session later
$ gptengage invoke claude "Give me a practical example" --session js-learning
[Claude remembers the previous conversation and provides an example]
(Session 'js-learning' saved)

# Continue with a follow-up
$ gptengage invoke claude "How about with async/await?" --session js-learning
[Full context maintained across all three messages]
(Session 'js-learning' saved)
```

#### With Custom Topics

Set a specific topic for the session:

```bash
$ gptengage invoke claude "What's the difference between let and const?" \
  --session variables --topic "JavaScript variable declarations"
[Response saved with custom topic]

$ gptengage session show variables
# Shows: "JavaScript variable declarations"
```

#### Custom Timeouts

Increase timeout for slower CLIs (Gemini often needs 60+ seconds):

```bash
# Standard timeout
$ gptengage invoke claude "Quick question" --timeout 30

# Extended timeout for complex tasks
$ gptengage invoke gemini "Complex analysis" --timeout 120

# Very short timeout for testing
$ gptengage invoke codex "test" --timeout 5
```

#### Combined Options

```bash
# File context + session + custom timeout
$ gptengage invoke claude "Review this for security" \
  --context-file src/auth.rs \
  --session security-review \
  --timeout 60
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | CLI not found or invocation failed |
| 2 | Session error |
| 3 | File not found (for --context-file) |
| 4 | Timeout exceeded |

### Notes

- **Sessions are persistent**: Once created, a session file lives in `~/.gptengage/sessions/` until deleted
- **Context injection**: Session history is automatically prepended to your prompt as `[CONVERSATION HISTORY]...[/CONVERSATION HISTORY]`
- **File context**: Content is prepended to the prompt as `File: <path>\n\n<content>\n\n<prompt>`
- **Prompt passing**: Prompts are sent via stdin, not as command-line arguments, for security and length

> **Warning**: Large context files may cause slower responses or timeouts. Consider splitting files over 100KB.

### Timeout Behavior

When the timeout is exceeded during an invoke:

1. **Process termination**: The CLI process receives SIGTERM immediately, followed by SIGKILL after 5 seconds
2. **Partial output**: Any output received before timeout is discarded; the command fails with exit code 4
3. **Error output**: Timeout error message is written to stderr, not stdout
4. **Session state**: If using `--session`, the session is NOT updated on timeout (preserves last successful state)

> **Important**: Unlike debates, a single invoke timeout results in command failure. There is no fallback.

> **Note**: Default timeout is 120 seconds. Gemini often requires 60+ seconds for complex prompts.

### See Also

- [gptengage debate](#gptengage-debate) - For multi-AI discussions on a topic
- [gptengage session](#gptengage-session) - To manage and view session history
- [gptengage config](#gptengage-config) - To set `defaultTimeout`

---

## gptengage debate

Run a structured debate between multiple AI systems on any topic.

### Syntax

```bash
gptengage debate <TOPIC> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<TOPIC>` | The debate topic (quoted if contains spaces) |

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--rounds <N>` | Number of debate rounds | 3 |
| `--output <FORMAT>` | Output format: `text`, `json`, or `markdown` | text |
| `--timeout <SECONDS>` | Timeout per CLI per round | 120 |
| `--help` | Show help for this command | - |

### Examples

#### Basic Debate

```bash
# 3-round debate with text output
$ gptengage debate "Should we use Rust instead of C?"
GPT ENGAGE DEBATE
Topic: Should we use Rust instead of C?

Running round 1 of 3...

ROUND 1
────────────────────────────────────────
CLAUDE:
Rust offers strong type safety and memory safety...

CODEX:
C provides low-level control and performance...

GEMINI:
It depends on your use case...

[Rounds 2 and 3 follow with each AI responding to prior arguments]

DEBATE COMPLETE
```

#### Multiple Rounds

```bash
# 5-round debate for deeper discussion
$ gptengage debate "Microservices vs Monolith" --rounds 5

# Single round for quick perspectives
$ gptengage debate "Tabs vs Spaces" --rounds 1
```

#### JSON Output

```bash
# Get structured JSON for programmatic processing
$ gptengage debate "AI Safety" --output json

# Save to file for further analysis
$ gptengage debate "AI Safety" --output json > debate.json

# Output can be parsed with jq
$ gptengage debate "AI Safety" --output json | jq '.rounds[0]'
```

JSON format:
```json
{
  "topic": "AI Safety",
  "rounds": [
    [
      {
        "cli": "Claude",
        "response": "AI safety is..."
      },
      {
        "cli": "Codex",
        "response": "From a development perspective..."
      }
    ]
  ]
}
```

#### Markdown Output

```bash
# Generate markdown for documentation
$ gptengage debate "REST vs GraphQL" --output markdown > debate.md

# Create a formatted document
$ gptengage debate "Team Retrospective" --output markdown > retro.md
# Then view with your favorite markdown viewer
```

Markdown output format:
```markdown
# REST vs GraphQL

## Round 1

### Claude

REST provides clear semantics...

### Codex

GraphQL offers flexible querying...

### Gemini

Both have merit depending on...
```

#### Extended Timeouts

```bash
# Standard timeout (one CLI might be slow)
$ gptengage debate "Complex topic" --timeout 120

# Very long timeout for deep analysis
$ gptengage debate "Philosophical question" --timeout 180

# Quick rounds with short timeout
$ gptengage debate "Simple topic" --timeout 30
```

#### Real-World Examples

```bash
# Team decision-making
$ gptengage debate "Should we migrate to microservices?" \
  --rounds 4 \
  --output markdown > migration-analysis.md

# Educational debate
$ gptengage debate "What is the future of AI?" --rounds 5

# Documentation generation
$ gptengage debate "Design patterns: Factory vs Builder" \
  --output markdown >> design-guide.md

# Quick perspective gathering
$ gptengage debate "New tool or existing tool?" --rounds 1
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (at least one CLI responded) |
| 1 | No CLIs available or all failed |
| 2 | Invalid output format |
| 4 | Timeout exceeded |

### Notes

- **Parallel execution**: All available CLIs run simultaneously each round
- **Graceful degradation**: If a CLI is unavailable or times out, the debate continues with others
- **Round context**: Each round includes previous responses, so CLIs can respond to each other's points
- **Output format**: Default `text` is human-readable; use `json` for programmatic access or `markdown` for documentation

> **Warning**: If all CLIs timeout or fail in a round, the debate continues to the next round. Check the `error` field in JSON output to detect partial failures.

> **Note**: Timeout applies per-CLI per-round. A 3-round debate with 120s timeout could take up to 360 seconds total if CLIs respond slowly.

### Timeout Behavior

When a CLI exceeds its timeout:

1. **Process termination**: The CLI process is sent SIGTERM, followed by SIGKILL after 5 seconds if unresponsive
2. **Partial output handling**: Any output received before timeout is discarded; the CLI entry shows an error
3. **Error reporting**: Timeout errors are written to stderr and included in the JSON `error` field
4. **Continuation**: The debate continues with remaining CLIs and proceeds to the next round

> **Important**: Timeouts do not cause immediate command failure. The debate completes with available responses.

### See Also

- [gptengage invoke](#gptengage-invoke) - For single-CLI interactions
- [gptengage config](#gptengage-config) - To set `defaultDebateRounds` and `defaultTimeout`
- [gptengage status](#gptengage-status) - To verify which CLIs are available

---

## Structured Output Specification

This section documents the stable output formats for `--output json` and `--output markdown`. LLM agents and automation tools can rely on these schemas as stable contracts.

### JSON Output Schema (`--output json`)

The JSON output conforms to the following schema. All fields are guaranteed present unless marked optional.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GPT Engage Debate Output",
  "type": "object",
  "required": ["topic", "rounds"],
  "properties": {
    "topic": {
      "type": "string",
      "description": "The debate topic as provided by the user"
    },
    "rounds": {
      "type": "array",
      "description": "Array of rounds, each containing CLI responses",
      "items": {
        "type": "array",
        "description": "Array of CLI responses for this round",
        "items": {
          "type": "object",
          "required": ["cli", "response"],
          "properties": {
            "cli": {
              "type": "string",
              "enum": ["Claude", "Codex", "Gemini"],
              "description": "Display name of the responding CLI"
            },
            "response": {
              "type": "string",
              "description": "The CLI's response text. Empty string if error occurred."
            },
            "persona": {
              "type": "string",
              "description": "Optional. The persona/role assigned to this CLI for the debate."
            },
            "error": {
              "type": "string",
              "description": "Optional. Error message if CLI failed or timed out. Absent on success."
            }
          }
        }
      }
    }
  }
}
```

#### Example: Successful Debate

```json
{
  "topic": "Should we use Rust instead of C?",
  "rounds": [
    [
      {
        "cli": "Claude",
        "response": "Rust offers memory safety guarantees through its ownership system..."
      },
      {
        "cli": "Codex",
        "response": "C provides decades of optimization and universal platform support..."
      },
      {
        "cli": "Gemini",
        "response": "The choice depends heavily on your project constraints..."
      }
    ],
    [
      {
        "cli": "Claude",
        "response": "Responding to Codex's point about platform support..."
      },
      {
        "cli": "Codex",
        "response": "While memory safety is valuable, C developers can use static analyzers..."
      },
      {
        "cli": "Gemini",
        "response": "Both previous responses highlight valid tradeoffs..."
      }
    ]
  ]
}
```

#### Example: Debate with Errors

```json
{
  "topic": "AI Safety",
  "rounds": [
    [
      {
        "cli": "Claude",
        "response": "AI safety encompasses several key concerns..."
      },
      {
        "cli": "Codex",
        "response": "",
        "error": "Timeout: CLI did not respond within 120 seconds"
      },
      {
        "cli": "Gemini",
        "response": "From a technical perspective, AI alignment..."
      }
    ]
  ]
}
```

#### Example: Debate with Personas

```json
{
  "topic": "Microservices vs Monolith",
  "rounds": [
    [
      {
        "cli": "Claude",
        "persona": "Senior Architect",
        "response": "As a senior architect, I recommend evaluating team size first..."
      },
      {
        "cli": "Codex",
        "persona": "DevOps Engineer",
        "response": "From an operations standpoint, microservices require robust CI/CD..."
      }
    ]
  ]
}
```

### Markdown Output Schema (`--output markdown`)

The markdown output follows a consistent structure suitable for documentation and rendering.

```markdown
# {topic}

## Round {n}

### {CLI Name}

{response text}

### {CLI Name}

{response text}

## Round {n+1}

...
```

#### Structure Specification

| Element | Format | Description |
|---------|--------|-------------|
| Document title | `# {topic}` | H1 heading with the debate topic |
| Round headers | `## Round {n}` | H2 heading, 1-indexed round number |
| CLI headers | `### {CLI Name}` | H3 heading with CLI display name (Claude, Codex, Gemini) |
| Response body | Plain text | CLI response with original formatting preserved |
| Error indication | `*Error: {message}*` | Italicized error message in place of response |
| Persona indicator | `**Persona: {name}**` | Bold persona name before response, if assigned |

#### Complete Markdown Example

```markdown
# Should we use Rust instead of C?

## Round 1

### Claude

Rust offers memory safety guarantees through its ownership system, preventing common bugs like null pointer dereferences and buffer overflows at compile time.

### Codex

C provides decades of optimization and universal platform support. Its simplicity makes it ideal for embedded systems and OS kernels.

### Gemini

The choice depends heavily on your project constraints. New projects may benefit from Rust's safety, while existing C codebases have maintenance considerations.

## Round 2

### Claude

Responding to Codex's point about platform support: Rust now targets most platforms C does, including embedded systems via `no_std`.

### Codex

While memory safety is valuable, C developers can use static analyzers and sanitizers to catch many of the same issues Rust prevents.

### Gemini

Both previous responses highlight valid tradeoffs. Consider team expertise and long-term maintenance costs in your decision.
```

### Schema Stability Guarantee

> **Important**: These output formats are considered stable public APIs. Breaking changes will only occur in major version releases (e.g., v2.0.0) and will be documented in the changelog with migration guidance.

**Agents can rely on:**
- Field names and types remaining consistent
- Required fields always being present
- The structure of nested arrays (rounds containing CLI responses)
- Error information being present in the `error` field when failures occur

**May change without notice:**
- Addition of new optional fields
- Formatting of error messages
- Whitespace in markdown output

---

## gptengage session

Manage persistent conversation sessions.

### Syntax

```bash
gptengage session <SUBCOMMAND> [ARGS] [OPTIONS]
```

### Subcommands

#### session list

List all active sessions.

```bash
$ gptengage session list

┌──────────────┬────────┬─────────────────────────┬──────────────┐
│ Session      │ CLI    │ Topic                   │ Last Used    │
├──────────────┼────────┼─────────────────────────┼──────────────┤
│ auth-review  │ claude │ Review my authenticatio │ 5 mins ago   │
│ perf-check   │ codex  │ Optimize database query │ 2 hours ago  │
└──────────────┴────────┴─────────────────────────┴──────────────┘
```

**Output columns:**
- **Session**: Session name (use with `--session <name>` in invoke)
- **CLI**: Which LLM is being used (claude, codex, or gemini)
- **Topic**: Session topic (first 23 characters)
- **Last Used**: Human-readable time ago (5s ago, 2h ago, etc.)

#### session show

Display full session history with all turns.

```bash
$ gptengage session show auth-review

Session: auth-review
CLI: claude
Topic: Review my authentication code
Created: 2024-01-04 08:30:00 UTC
Last interaction: 2024-01-04 10:45:00 UTC
Turns: 4

┌─────────────────────────────────────────────────────────┐
│ 1: [You]                                                │
│                                                         │
│ Review my authentication code                           │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ 2: [Claude]                                             │
│                                                         │
│ I found 3 potential security issues...                  │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ 3: [You]                                                │
│                                                         │
│ Fix the JWT vulnerability                              │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ 4: [Claude]                                             │
│                                                         │
│ For JWT, use RS256 instead of HS256...                 │
│                                                         │
└─────────────────────────────────────────────────────────┘

To continue this session, run:
  gptengage invoke claude "<your message>" --session auth-review
```

#### session end

End and delete a session.

```bash
# Delete a specific session
$ gptengage session end auth-review
✓ Session 'auth-review' deleted.

# Delete all sessions
$ gptengage session end --all
✓ Deleted session: auth-review
✓ Deleted session: perf-check
✓ Deleted session: learning

All sessions deleted.
```

**Options:**
- `<NAME>` - Session name to delete (required unless using --all)
- `--all` - Delete all sessions at once

### Example Workflow

```bash
# Create a session for learning
$ gptengage invoke claude "Explain callbacks in JavaScript" --session callbacks

# Continue later
$ gptengage invoke claude "What's the difference between callbacks and promises?" --session callbacks

# Check session list
$ gptengage session list

# View full conversation
$ gptengage session show callbacks

# When done, clean up
$ gptengage session end callbacks
```

### Notes

- **Session files**: Stored in `~/.gptengage/sessions/<name>.json`
- **Persistence**: Sessions survive across terminal sessions and even reboots
- **Privacy**: Sessions are stored locally—nothing is sent to external servers
- **No automatic cleanup**: Sessions persist until you explicitly delete them with `session end`

> **Note**: Session names must be alphanumeric with hyphens or underscores. Spaces and special characters are not allowed.

> **Warning**: Deleting a session with `session end` is permanent and cannot be undone.

### See Also

- [gptengage invoke](#gptengage-invoke) - To create and continue sessions
- [gptengage status](#gptengage-status) - Quick overview of active sessions

---

## gptengage status

Show available CLIs and active sessions.

### Syntax

```bash
gptengage status
```

### Example Output

```bash
$ gptengage status

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

### Interpretation

- **✓ CLI Name**: CLI is available and can be invoked
- **✗ CLI Name**: CLI is not installed or not in PATH
- **Active Sessions**: Count and quick preview of sessions
- **Paths**: Where configuration and sessions are stored

### Notes

- **No arguments**: `status` doesn't take any options
- **Quick check**: Use this to verify your setup before using debates or invoke
- **Troubleshooting**: If a CLI shows ✗, check installation and PATH

> **Note**: The `status` command does not invoke any CLIs. It only checks if executables are found in PATH.

### See Also

- [gptengage config](#gptengage-config) - View and modify configuration
- [gptengage session list](#session-list) - Detailed session listing
- [gptengage invoke](#gptengage-invoke) - Start using a detected CLI

---

## gptengage config

Manage GPT Engage configuration settings from the command line.

### Syntax

```bash
gptengage config <SUBCOMMAND> [ARGS]
```

### Subcommands

#### config list

Display all configuration settings and their current values.

```bash
$ gptengage config list

Configuration (~/.gptengage/config.json):

  defaultTimeout      = 120
  defaultDebateRounds = 3

CLI Configuration:

  claude:
    command    = claude
    invokeArgs = ["-p"]
    detected   = true

  codex:
    command    = codex
    invokeArgs = ["exec", "--full-auto"]
    detected   = true

  gemini:
    command    = gemini
    invokeArgs = ["--yolo"]
    detected   = false
```

#### config get

Get the value of a specific configuration key.

```bash
# Get a simple value
$ gptengage config get defaultTimeout
120

# Get a nested value using dot notation
$ gptengage config get clis.claude.command
claude

$ gptengage config get clis.codex.invokeArgs
["exec", "--full-auto"]
```

**Syntax:**
```bash
gptengage config get <KEY>
```

**Supported keys:**
| Key | Type | Description |
|-----|------|-------------|
| `defaultTimeout` | integer | Default timeout in seconds for CLI invocations |
| `defaultDebateRounds` | integer | Default number of rounds for debates |
| `clis.<name>.command` | string | Executable name for the CLI |
| `clis.<name>.invokeArgs` | array | Arguments passed before the prompt |
| `clis.<name>.detected` | boolean | Whether CLI was found in PATH (read-only) |

#### config set

Set a configuration value.

```bash
# Set default timeout to 180 seconds
$ gptengage config set defaultTimeout 180
✓ defaultTimeout = 180

# Set default debate rounds
$ gptengage config set defaultDebateRounds 5
✓ defaultDebateRounds = 5

# Change a CLI's command
$ gptengage config set clis.claude.command /usr/local/bin/claude
✓ clis.claude.command = /usr/local/bin/claude

# Set invoke arguments (use JSON array syntax)
$ gptengage config set clis.codex.invokeArgs '["exec", "--quiet"]'
✓ clis.codex.invokeArgs = ["exec", "--quiet"]
```

**Syntax:**
```bash
gptengage config set <KEY> <VALUE>
```

> **Warning**: Invalid configuration values may cause commands to fail. Use `gptengage status` to verify CLI detection after changing CLI settings.

> **Note**: The `detected` field is read-only and computed automatically based on PATH availability.

### Available Configuration Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `defaultTimeout` | integer | 120 | Timeout in seconds for CLI invocations. Applied when `--timeout` is not specified. |
| `defaultDebateRounds` | integer | 3 | Number of debate rounds. Applied when `--rounds` is not specified. |
| `clis.claude.command` | string | `"claude"` | Command to invoke Claude CLI |
| `clis.claude.invokeArgs` | array | `["-p"]` | Arguments for Claude (the `-p` flag enables print mode) |
| `clis.codex.command` | string | `"codex"` | Command to invoke Codex CLI |
| `clis.codex.invokeArgs` | array | `["exec", "--full-auto"]` | Arguments for Codex |
| `clis.gemini.command` | string | `"gemini"` | Command to invoke Gemini CLI |
| `clis.gemini.invokeArgs` | array | `["--yolo"]` | Arguments for Gemini |

### Examples

#### View Current Configuration

```bash
# List everything
$ gptengage config list

# Check specific values
$ gptengage config get defaultTimeout
$ gptengage config get clis.claude.invokeArgs
```

#### Adjust Timeouts

```bash
# Increase default timeout for slow network
$ gptengage config set defaultTimeout 180

# Verify the change
$ gptengage config get defaultTimeout
180
```

#### Configure a Custom CLI Path

```bash
# Use a specific Claude binary
$ gptengage config set clis.claude.command /opt/anthropic/claude

# Verify
$ gptengage status
# Should show: ✓ claude (Claude Code)
```

#### Modify CLI Arguments

```bash
# Add verbose flag to Gemini
$ gptengage config set clis.gemini.invokeArgs '["--yolo", "--verbose"]'

# Check the result
$ gptengage config get clis.gemini.invokeArgs
["--yolo", "--verbose"]
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid key or configuration error |
| 2 | Invalid value format (e.g., non-integer for timeout) |

### Notes

- **Config file location**: `~/.gptengage/config.json`
- **Auto-creation**: Config file is created with defaults on first run if it doesn't exist
- **JSON format**: The config file is standard JSON; you can also edit it directly
- **Validation**: Values are validated on set; invalid values are rejected with an error message

> **Important**: Changes take effect immediately for subsequent commands. No restart is required.

### See Also

- [gptengage status](#gptengage-status) - Verify CLI detection after config changes
- [Configuration Reference](#configuration-reference) - Full config file documentation

---

## Global Options

These options work with any command:

| Option | Description |
|--------|-------------|
| `--help` | Show help for the command |
| `--version` | Show GPT Engage version |

---

## Configuration Reference

Modify behavior by editing `~/.gptengage/config.json`:

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

**Configuration keys:**
- `defaultTimeout`: Default timeout in seconds (used if --timeout not specified)
- `defaultDebateRounds`: Default number of debate rounds (used if --rounds not specified)
- `clis[<name>].command`: The command to execute
- `clis[<name>].invokeArgs`: Arguments to pass before the prompt
- `clis[<name>].detected`: Whether this CLI was detected on your system

---

## Tips & Tricks

### Redirect Debate Output

```bash
# Save debate to file
$ gptengage debate "Topic" > debate.txt

# Append to existing document
$ gptengage debate "Topic 2" >> document.md

# Pipe to other tools
$ gptengage debate "Topic" | less
$ gptengage debate "Topic" --output json | jq '.rounds | length'
```

### Invoke with Large Files

```bash
# Include a file that's too large to review manually
$ gptengage invoke claude "Find bugs in this" --context-file huge_file.rs

# Multiple files (use cat to combine)
$ gptengage invoke claude "Compare these" --context-file <(cat file1.rs file2.rs)
```

### Session Management

```bash
# Quickly check session count
$ gptengage session list | wc -l

# Find sessions by CLI
$ gptengage session list | grep claude

# Delete all claude sessions
$ gptengage session list | grep claude | awk '{print $1}' | xargs -I {} gptengage session end {}
```

### Batch Debates

```bash
# Run multiple debates and save results
for topic in "Rust vs Go" "Monolith vs Microservices" "REST vs GraphQL"
do
  gptengage debate "$topic" --output markdown >> debates.md
  echo "---" >> debates.md
done
```
