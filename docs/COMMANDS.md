# GPT Engage Commands Reference

Complete reference documentation for all GPT Engage commands.

## Table of Contents

- [gptengage invoke](#gptengage-invoke) - Invoke a specific LLM CLI
- [gptengage debate](#gptengage-debate) - Run multi-AI debates
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
