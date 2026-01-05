# GPT Engage: Architecture Overview

## Executive Summary

**GPT Engage** is a standalone Rust CLI tool that orchestrates multiple LLM CLIs without modifying them. It enables seamless invocation and coordination between Claude Code, OpenAI Codex, and Google Gemini with built-in session management, multi-AI debates, and parallel execution. The system is designed as a lightweight, non-invasive orchestration layer that respects the independence of each CLI tool.

---

## 1. System Architecture

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    GPT Engage CLI (Rust)                        │
│              (Standalone Orchestrator Process)                  │
└──────────────┬──────────────────────────────────────────────────┘
               │
       ┌───────┼───────┬─────────┐
       │       │       │         │
       ▼       ▼       ▼         ▼
  ┌────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐
  │Invoke  │ │Debate    │ │Session   │ │Status/Config │
  │Command │ │Command   │ │Command   │ │Commands      │
  └────┬───┘ └────┬─────┘ └────┬─────┘ └──────┬───────┘
       │          │            │              │
       └──────────┼────────────┬┴──────────────┘
                  │            │
         ┌────────▼────────────▼────────┐
         │   CLI Invokers (Async)       │
         │                              │
         │ • Base executor              │
         │ • Claude Code invoker        │
         │ • Codex invoker              │
         │ • Gemini invoker             │
         └────────┬─────────┬───────┬───┘
                  │         │       │
       ┌──────────▼──┐ ┌────▼────┐ ┌┴──────────┐
       │ claude -p   │ │codex    │ │gemini    │
       │<prompt>     │ │exec ... │ │--yolo    │
       └──────────┬──┘ └────┬────┘ └┬─────────┘
                  │         │      │
              [Subprocess Execution with stdin/stdout piping]
                  │         │      │
       ┌──────────▼──┐ ┌────▼────┐ ┌┴──────────┐
       │Claude       │ │Codex    │ │Gemini    │
       │(Process)    │ │(Process)│ │(Process) │
       └─────────────┘ └─────────┘ └──────────┘
```

### 1.2 Key Design Principles

1. **CLI-Only**: No direct API calls. All invocation via CLI subprocesses.
2. **Non-Invasive**: Doesn't modify `~/.claude/`, `~/.codex/`, or `~/.gemini/` directories.
3. **Independent Processes**: Each CLI execution is a fresh subprocess with no shared state.
4. **Async/Await**: Uses Tokio for non-blocking parallel execution.
5. **Session Persistence**: Maintains conversation history via JSON files with context injection.
6. **Error Resilience**: Gracefully handles missing CLIs, timeouts, and failures.

### 1.3 Invocation Flows

#### Flow 1: Single CLI Invocation
```
User: gptengage invoke claude "Explain closures"
                ↓
GPT Engage parses arguments
                ↓
Load/create session (optional)
                ↓
Invoke: claude -p "<context><prompt>"
                ↓
Capture stdout, save to session
                ↓
Display response to user
```

#### Flow 2: Multi-Turn Session
```
User: gptengage invoke claude "Explain closures" --session learning
                ↓
Check ~/.gptengage/sessions/learning.json
                ↓
Build prompt: [CONVERSATION HISTORY]...[CURRENT REQUEST]
                ↓
Invoke: claude -p "<built-prompt>"
                ↓
Append user message + response to session
                ↓
Save updated session
```

#### Flow 3: Multi-AI Debate
```
User: gptengage debate "Tabs or spaces?" --rounds 2
                ↓
For each round 1..2:
  ├─ Build context (topic + previous responses)
  ├─ Spawn 3 async tasks (Claude, Codex, Gemini)
  ├─ Wait for all to complete (or timeout)
  └─ Collect responses
                ↓
Format output (text/json/markdown)
                ↓
Display debate results
```

---

## 2. Core Components

### 2.1 Commands Module (`src/commands/`)

Each command is a standalone async function that orchestrates CLI invocations.

#### invoke.rs
- **Purpose**: Invoke a specific CLI with optional session support
- **Inputs**: CLI name, prompt, session name, context file, timeout
- **Outputs**: Response text (displayed to user)
- **Key Logic**:
  - Load existing session or create new
  - Build prompt with history injection
  - Select appropriate invoker
  - Persist session to disk

#### debate.rs
- **Purpose**: Run multi-AI debate on a topic
- **Inputs**: Topic, number of rounds, output format, timeout
- **Outputs**: Debate result (text/JSON/markdown)
- **Key Logic**:
  - Orchestrate parallel CLI execution per round
  - Build context with previous responses
  - Format output based on user preference

#### session.rs
- **Purpose**: Manage persistent conversation sessions
- **Subcommands**:
  - `list`: Show all active sessions with metadata
  - `show`: Display full conversation history
  - `end`: Delete a session (or all sessions)
- **Key Logic**:
  - CRUD operations on session JSON files
  - Formatting with box-drawing characters
  - Human-readable timestamp formatting

#### status.rs
- **Purpose**: Show system status and available CLIs
- **Outputs**: Detected CLIs, active sessions, config location
- **Key Logic**:
  - Run `which` to detect CLI availability
  - Format output with checkmarks/X marks

### 2.2 Invokers Module (`src/invokers/`)

Abstracts CLI invocation behind a common trait.

#### base.rs
```rust
pub async fn execute_command(
    cmd: &str,
    args: &[&str],
    input: &str,
    timeout: u64,
) -> Result<String>
```
- **Purpose**: Core subprocess execution logic
- **Implementation**:
  - Spawn tokio::process::Command
  - Write input to stdin
  - Capture stdout with timeout
  - Check exit status

#### claude.rs, codex.rs, gemini.rs
Each implements the `Invoker` trait:
```rust
#[async_trait]
pub trait Invoker {
    async fn invoke(&self, prompt: &str, timeout: u64) -> Result<String>;
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
}
```

**Claude Code Invoker**:
- Command: `claude -p`
- Flag: `-p` enables print mode (non-interactive)
- Passes prompt via stdin

**Codex Invoker**:
- Command: `codex exec --full-auto`
- Flags: `exec` (execute mode), `--full-auto` (auto-approve)
- Passes prompt via stdin
- Note: Requires git repo trust or `--skip-git-repo-check`

**Gemini Invoker**:
- Command: `gemini --yolo`
- Flag: `--yolo` (auto-approve all operations)
- Passes prompt via stdin
- Note: Typically slower, recommend 60+ second timeout

### 2.3 Session Module (`src/session/`)

Persistent conversation management via JSON files.

#### Data Structures
```rust
pub struct Session {
    pub name: String,
    pub cli: String,
    pub topic: String,
    pub created_at: DateTime<Utc>,
    pub last_interaction: DateTime<Utc>,
    pub turns: Vec<Turn>,
}

pub struct Turn {
    pub role: String,  // "user" or "assistant"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}
```

#### Context Injection Pattern
```
[CONVERSATION HISTORY]
User: <first message>
Assistant: <first response>

User: <second message>
Assistant: <second response>
[/CONVERSATION HISTORY]

[CURRENT REQUEST]
<new message>
[/CURRENT REQUEST]
```

This allows stateless CLIs to maintain context without native multi-turn support.

#### Validation
Session names are validated to prevent directory traversal:
- Alphanumeric characters, dashes, underscores only
- Rejects "..", "/", "\" separators
- Maximum length checked

### 2.4 Orchestrator Module (`src/orchestrator/`)

Multi-AI debate orchestration.

#### debate.rs
```rust
pub async fn run_debate(
    topic: &str,
    num_rounds: usize,
    timeout: u64,
) -> Result<DebateResult>
```

**Algorithm**:
1. For each round 1..num_rounds:
   - Build context string (topic + previous responses)
   - Clone invokers for this iteration
   - Spawn 3 async tasks with tokio::task::spawn
   - Use tokio::join! to wait for all to complete
   - Handle failures gracefully (skip missing CLI, collect responses)
2. Return DebateResult with all rounds

**Key Features**:
- Parallel execution: All 3 CLIs run simultaneously
- Graceful degradation: Works with 1-3 CLIs available
- Round context: Each round sees previous responses
- Timeout management: Each CLI has independent timeout

### 2.5 Configuration Module (`src/config/`)

User configuration management.

#### File: `~/.gptengage/config.json`
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

#### Features
- Auto-loads config on startup
- Auto-generates default config if missing
- Allows user customization of timeouts and CLI commands
- Persists detected CLI status

---

## 3. Data Flow Examples

### Example 1: Session Creation and Continuation

```
Round 1:
┌─ User: gptengage invoke claude "Explain closures" --session learning
│
├─ Check: ~/.gptengage/sessions/learning.json (doesn't exist)
│
├─ Create new Session:
│  ├─ name: "learning"
│  ├─ cli: "claude"
│  ├─ topic: "Explain closures"
│  └─ turns: []
│
├─ Invoke: claude -p "Explain closures"
│
├─ Append Turn(user, "Explain closures")
├─ Append Turn(assistant, "[Claude's response...]")
│
└─ Save to: ~/.gptengage/sessions/learning.json

---

Round 2:
┌─ User: gptengage invoke claude "Give an example" --session learning
│
├─ Load: ~/.gptengage/sessions/learning.json
│
├─ Build prompt:
│  [CONVERSATION HISTORY]
│  User: Explain closures
│  Assistant: [Claude's response from Round 1]
│  [/CONVERSATION HISTORY]
│
│  [CURRENT REQUEST]
│  Give an example
│  [/CURRENT REQUEST]
│
├─ Invoke: claude -p "[built-prompt]"
│
├─ Append Turn(user, "Give an example")
├─ Append Turn(assistant, "[Claude's new response]")
│
└─ Save updated session
```

### Example 2: Debate Execution

```
User: gptengage debate "Rust vs Go" --rounds 1

┌─ Topic: "Rust vs Go"
│  Round: 1
│  Context: "Topic: Rust vs Go\n\nRound 1\n\nPlease provide your perspective on this topic."
│
├─ Spawn 3 parallel tasks:
│  │
│  ├─ Task 1 (Claude):     claude -p "[context]" → "Rust offers memory safety..."
│  │
│  ├─ Task 2 (Codex):      codex exec --full-auto "[context]" → "Go is simpler and..."
│  │
│  └─ Task 3 (Gemini):     gemini --yolo "[context]" → "Both have trade-offs..."
│
├─ Wait for all to complete (or timeout)
│
├─ Collect responses:
│  [
│    RoundResponse { cli: "Claude", response: "..." },
│    RoundResponse { cli: "Codex", response: "..." },
│    RoundResponse { cli: "Gemini", response: "..." }
│  ]
│
└─ Format and display results
```

---

## 4. Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Language** | Rust 1.86+ | Type safety, performance, single binary distribution |
| **Runtime** | Tokio | Async/await for parallel CLI execution |
| **CLI Parsing** | Clap 4.4 | Ergonomic, comprehensive argument parsing |
| **Serialization** | Serde + serde_json | Fast, idiomatic JSON handling |
| **Date/Time** | Chrono | Standard Rust datetime library |
| **Error Handling** | Anyhow | Context-rich error messages |
| **Async Traits** | async-trait | Trait support for async functions |

---

## 5. File Organization

```
~/.gptengage/
├── config.json              # User configuration (auto-created)
└── sessions/                # Conversation sessions
    ├── auth-review.json
    ├── learning.json
    └── ...
```

Each session file is a standalone JSON document containing full conversation history.

---

## 6. Security Considerations

### Subprocess Isolation
- Each CLI runs in a fresh subprocess with independent environment
- No shared state between invocations
- Timeouts prevent hanging processes

### Input Validation
- Session names validated against directory traversal
- File paths checked before access
- Prompts passed via stdin (not shell command-line arguments)

### Local-Only Operation
- No external API calls
- No data transmission to remote servers
- All configuration and sessions stored locally

---

## 7. Design Decisions & Tradeoffs

### Decision: CLI-Only, No APIs
**Why**:
- Users can run offline
- No API key management
- Works with CLI tools they already have
**Tradeoff**:
- Limited to CLI capabilities (no streaming, token counting, etc.)
- Slower than direct API (subprocess overhead)

### Decision: Prompt Injection for Sessions
**Why**:
- Stateless CLIs become stateful without modification
- Simple to implement and debug
- Works with any CLI
**Tradeoff**:
- Prompts grow with history (scalability limit)
- Less sophisticated than native multi-turn protocols

### Decision: Standalone Binary in Rust
**Why**:
- Single distribution file
- No runtime dependencies
- High performance
**Tradeoff**:
- Longer build times than interpreted languages
- Rust learning curve for contributors

### Decision: Session Files, Not Database
**Why**:
- Portable (can backup/share with git)
- Human-readable for debugging
- No additional dependencies
**Tradeoff**:
- Limited query capabilities
- Scalability limits for very large sessions

---

## 8. Extension Points

Future versions could add:

1. **Plugin System**: Custom CLI integrations via YAML
2. **Response Caching**: Cache identical prompts
3. **Debate Synthesis**: Use Claude Opus to synthesize debates
4. **Web Viewer**: Lightweight UI for session review
5. **Response Filtering**: Normalize output across CLIs

---

## 9. Performance Characteristics

| Operation | Latency | Notes |
|-----------|---------|-------|
| invoke (single CLI) | 20-120s | Depends on CLI and task |
| debate (parallel) | 60-180s | Bottlenecked by slowest CLI |
| session list | <100ms | Reading JSON files |
| session show | <500ms | Loading and formatting |
| status | <100ms | Running `which` commands |

**Optimization Strategy**:
- Parallel execution reduces debate time significantly
- Async I/O prevents blocking on slow CLIs
- Session context injection adds <10ms overhead

---

## 10. For Contributors

This architecture is designed for extensibility:

### Adding a New CLI
1. Create `src/invokers/newcli.rs`
2. Implement `Invoker` trait
3. Add to CLI enum in `src/cli.rs`
4. Update documentation

### Adding a New Command
1. Create `src/commands/newcmd.rs`
2. Add to `Commands` enum in `src/cli.rs`
3. Wire up in `cli.rs execute()` method
4. Add tests and documentation

### Modifying Session Format
- Update `Session` struct in `src/session/mod.rs`
- Handle migration from old format
- Update context injection pattern

---

## Conclusion

GPT Engage is a lightweight, focused tool for CLI-based multi-AI orchestration. Its architecture prioritizes:

1. **Simplicity**: Easy to understand and contribute to
2. **Reliability**: No external dependencies, local-only operation
3. **Extensibility**: Clean trait-based design for invokers
4. **Performance**: Async/await for parallel execution
5. **User-Friendliness**: Session persistence, debate features, clear error messages

The design reflects the principle that good orchestration shouldn't require understanding the internals of orchestrated systems—it should just coordinate their inputs and outputs effectively.
