# GPT Engage Examples

Real-world usage scenarios and practical examples.

## Table of Contents

- [Code Review Workflow](#code-review-workflow)
- [Learning & Teaching](#learning--teaching)
- [Decision Making](#decision-making)
- [Content Generation](#content-generation)
- [Problem Solving](#problem-solving)
- [Team Collaboration](#team-collaboration)
- [Advanced Workflows](#advanced-workflows)
- [Agent-Driven Workflows](#agent-driven-workflows)
- [Tips for Effective Usage](#tips-for-effective-usage)
- [Common Patterns](#common-patterns)
- [For AI Agents](#for-ai-agents)
- [Performance Tips](#performance-tips)

---

## Code Review Workflow

### Scenario: Multi-AI Code Review

Review your code with multiple AI perspectives:

```bash
# Get Claude's perspective on security
$ gptengage invoke claude "Security review of this code" \
  --context-file src/auth.rs \
  --session code-review

# Get Codex's performance perspective
$ gptengage invoke codex "Optimize this code for performance" \
  --context-file src/auth.rs \
  --session code-review

# Get Gemini's readability perspective
$ gptengage invoke gemini "Suggest clarity improvements" \
  --context-file src/auth.rs \
  --session code-review

# View all feedback in one place
$ gptengage session show code-review
```

### Scenario: Iterative Bug Fix

Work through a bug fix with Claude, maintaining full context:

```bash
# Initial issue analysis
$ gptengage invoke claude "This function returns wrong results when passed an empty array" \
  --context-file src/parser.rs \
  --session bug-fix

# Follow-up with proposed fix
$ gptengage invoke claude "I tried your first suggestion. Now it crashes with null pointer. What's wrong?" \
  --session bug-fix

# Refine the solution
$ gptengage invoke claude "That worked! But can we make it more efficient?" \
  --session bug-fix

# See the full conversation
$ gptengage session show bug-fix
```

### Scenario: Architecture Review

Get multi-perspective architecture feedback:

```bash
# Run a debate on your architecture choice
$ gptengage debate "Should our microservice use sync or async database calls?" \
  -r 3 \
  -o markdown > arch-decision.md

# Discuss with a single AI
$ gptengage invoke claude \
  "Based on the debate about sync vs async, what would you recommend for a payment service?" \
  --session payment-arch
```

---

## Learning & Teaching

### Scenario: Concept Deep Dive

Learn a concept from multiple AI perspectives:

```bash
# Start with Claude's explanation
$ gptengage invoke claude "Explain the concept of closures in JavaScript" \
  --session closures

# Get Codex's implementation-focused perspective
$ gptengage invoke codex "Show me practical examples of closures" \
  --session closures

# Get Gemini's teaching perspective
$ gptengage invoke gemini "What are common mistakes when using closures?" \
  --session closures

# View the complete learning material
$ gptengage session show closures
```

### Scenario: Tutorial Creation

Generate comprehensive tutorial content:

```bash
# Create outline
$ gptengage invoke claude "Create an outline for teaching REST API design" \
  --session rest-tutorial --topic "REST API Design Tutorial"

# Expand with examples
$ gptengage invoke claude "Add practical code examples to each section" \
  --session rest-tutorial

# Polish with best practices
$ gptengage invoke claude "Add a 'Common Mistakes' section based on your previous examples" \
  --session rest-tutorial

# Export to markdown
$ gptengage session show rest-tutorial > tutorial.md
```

### Scenario: Student Homework Help

Help a student understand a concept:

```bash
# Initial explanation
$ gptengage invoke claude \
  "I'm learning about recursion. Can you explain how it works?" \
  --session recursion-learning

# Build on previous explanation
$ gptengage invoke claude \
  "Can you show an example with a linked list?" \
  --session recursion-learning

# Advanced follow-up
$ gptengage invoke claude \
  "How does tail recursion optimization work?" \
  --session recursion-learning
```

---

## Decision Making

### Scenario: Technology Choice Debate

Facilitate a structured technology decision:

```bash
# Run a comprehensive debate
$ gptengage debate "TypeScript vs JavaScript for our new project" \
  -r 5 \
  -o markdown > tech-decision.md

# Follow up with specific question
$ gptengage invoke claude \
  "Given the team has Python experience, does this change your recommendation?" \
  -s tech-choice

# Document the final decision
$ gptengage session show tech-choice >> tech-decision.md
```

### Scenario: Architecture Decision Record (ADR)

Create a formal architecture decision:

```bash
# Get the debate
$ gptengage debate "Monolithic vs Microservices for a startup" \
  -r 3 \
  -o markdown > adr.md

# Add context-specific analysis
$ gptengage invoke claude \
  "Given our team size of 3 and budget constraints, which architecture makes sense?" \
  --session adr-context

# Document the decision rationale
$ gptengage invoke claude \
  "Write a formal ADR (Architecture Decision Record) based on this analysis" \
  --session adr-context

# Export everything
$ gptengage session show adr-context >> adr.md
```

### Scenario: Feature Prioritization

Get AI perspectives on feature prioritization:

```bash
# Run a debate on which features to build first
$ gptengage debate "Should we build offline support or real-time sync first?" \
  -r 3 \
  -o json > features.json

# Discuss with your team's decision-maker
$ gptengage invoke claude \
  "You've analyzed the trade-offs. Which is more valuable for our user base?" \
  -s feature-priority
```

### Scenario: Persona-Based Debates

Use personas to get specialized perspectives on decisions:

```bash
# Security-focused debate on a new feature
$ gptengage debate "Should we implement OAuth or custom JWT auth?" \
  -p security-expert \
  -r 4 \
  -o markdown > auth-decision.md

# Performance-focused architecture review
$ gptengage debate "Which caching strategy is best for our API?" \
  -p performance-engineer \
  -r 3 \
  -o json > caching-analysis.json

# UX-focused product decision
$ gptengage debate "Should we use infinite scroll or pagination?" \
  -p ux-advocate \
  --rounds 3 \
  --output markdown > ux-decision.md

# Combine persona with context file
$ gptengage debate "Is this database schema secure?" \
  -p security-expert \
  --context-file schema.sql \
  -r 3 \
  -o markdown > schema-review.md
```

### Scenario: Multi-Perspective Technical Review

Get different expert perspectives on the same decision:

```bash
# First, security perspective
$ gptengage invoke claude \
  "Review this API design for security concerns" \
  -p security-expert \
  --context-file api-spec.yaml \
  -s api-review

# Then, performance perspective
$ gptengage invoke claude \
  "Review this API design for performance bottlenecks" \
  -p performance-engineer \
  --context-file api-spec.yaml \
  -s api-review

# Finally, usability perspective
$ gptengage invoke claude \
  "Review this API design for developer experience" \
  -p ux-advocate \
  --context-file api-spec.yaml \
  -s api-review

# Export the combined review
$ gptengage session show api-review > api-review-complete.md
```

---

## Content Generation

### Scenario: Blog Post Creation

Write a blog post with AI help:

```bash
# Outline
$ gptengage invoke claude \
  "Create a blog post outline about async/await in JavaScript" \
  --session blog-post --topic "Async/Await Blog Post"

# First draft
$ gptengage invoke claude \
  "Write the introduction section" \
  --session blog-post

# Examples
$ gptengage invoke claude \
  "Add code examples for the main concepts" \
  --session blog-post

# Polish
$ gptengage invoke claude \
  "Improve the writing style and add a conclusion" \
  --session blog-post

# Export
$ gptengage session show blog-post > blog-post.md
```

### Scenario: Documentation Generation

Generate API documentation:

```bash
# Start with interface
$ gptengage invoke claude \
  "Generate API documentation for this interface" \
  --context-file src/api.ts \
  --session api-docs

# Add examples
$ gptengage invoke claude \
  "Add usage examples for each endpoint" \
  --session api-docs

# Add error handling
$ gptengage invoke claude \
  "Document all possible error codes and their meanings" \
  --session api-docs

# Export to markdown
$ gptengage session show api-docs > API.md
```

### Scenario: Email Campaign

Write a marketing email with AI help:

```bash
# Get Claude's copywriting help
$ gptengage invoke claude \
  "Write a compelling email about our new product launch" \
  --session email-campaign

# Refine tone
$ gptengage invoke claude \
  "Make this more personable and less corporate" \
  --session email-campaign

# Add CTA
$ gptengage invoke claude \
  "Add a compelling call-to-action at the end" \
  --session email-campaign
```

---

## Problem Solving

### Scenario: Debugging with Multiple Perspectives

Debug an issue with help from multiple AIs:

```bash
# Initial analysis
$ gptengage invoke claude \
  "I'm getting a memory leak in this C++ code. Where would you start investigating?" \
  --context-file src/memory_leak.cpp \
  --session debugging

# Get implementation details
$ gptengage invoke codex \
  "Show me specific valgrind commands to find this memory leak" \
  --session debugging

# Alternative perspective
$ gptengage invoke gemini \
  "Are there common patterns in C++ that cause this type of leak?" \
  --session debugging

# Document the solution
$ gptengage session show debugging > memory-leak-solution.md
```

### Scenario: Error Analysis

Understand complex error messages:

```bash
# Initial exploration
$ gptengage invoke claude \
  "I'm getting this error: SEGFAULT at 0x1234567890. What does it mean?" \
  --session error-analysis

# Deep dive
$ gptengage invoke claude \
  "I'm using valgrind. Can you help me interpret these valgrind outputs?" \
  --context-file valgrind.log \
  --session error-analysis

# Solution search
$ gptengage invoke claude \
  "Based on this analysis, what's the most likely cause?" \
  --session error-analysis
```

### Scenario: Algorithm Optimization

Improve algorithm performance:

```bash
# Profile analysis
$ gptengage invoke claude \
  "I profiled my code. It spends 80% of time in this sort function. Can I optimize it?" \
  --context-file profile.txt \
  --session optimization

# Get specific improvements
$ gptengage invoke codex \
  "Show me optimized code using these techniques" \
  --session optimization

# Verify understanding
$ gptengage invoke claude \
  "Explain why these optimizations work" \
  --session optimization
```

---

## Team Collaboration

### Scenario: Design Review Meeting Prep

Prepare for a design review with AI input:

```bash
# Get feedback on design
$ gptengage invoke claude \
  "Here's my UI design. What potential UX issues do you see?" \
  --context-file design.png \
  --session design-review

# Get accessibility perspective
$ gptengage invoke claude \
  "How accessible is this design? Any improvements?" \
  --session design-review

# Performance perspective
$ gptengage invoke codex \
  "How can I implement this design efficiently?" \
  --session design-review

# Export feedback
$ gptengage session show design-review > design-feedback.md
```

### Scenario: Code Standards Definition

Create team coding standards:

```bash
# Start with best practices debate
$ gptengage debate "Should we enforce strict type checking in TypeScript?" \
  -r 3 \
  -o markdown > coding-standards.md

# Get specific implementations
$ gptengage invoke claude \
  "Create a TSConfig that enforces these standards" \
  --session coding-standards

# Add ESLint config
$ gptengage invoke claude \
  "Create an ESLint configuration that matches these standards" \
  --session coding-standards

# Document for team
$ gptengage session show coding-standards >> coding-standards.md
```

### Scenario: Retrospective Facilitation

Conduct an AI-assisted retrospective:

```bash
# Get perspective on team process
$ gptengage debate "Was our sprint structure effective?" \
  -r 2 \
  -o markdown > retro-notes.md

# Discuss improvements
$ gptengage invoke claude \
  "Based on these perspectives, what are the top 3 improvements we should try?" \
  -s retro-improvements

# Create action items
$ gptengage invoke claude \
  "Format these as specific, measurable action items" \
  -s retro-improvements
```

---

## Advanced Workflows

### Scenario: Research Paper Summarization

Summarize research with AI help:

```bash
# Extract key points
$ gptengage invoke claude \
  "Summarize the key findings of this paper" \
  --context-file paper.pdf \
  --session paper-summary

# Explain implications
$ gptengage invoke claude \
  "What are the practical implications of these findings?" \
  --session paper-summary

# Compare with similar work
$ gptengage invoke claude \
  "How does this compare to the previous approaches?" \
  --session paper-summary
```

### Scenario: Code Golf Challenge

Tackle a programming challenge:

```bash
# Initial approach
$ gptengage invoke claude \
  "Here's a code golf challenge: write the shortest solution" \
  --session code-golf --topic "Code Golf: Find All Duplicates"

# Optimize further
$ gptengage invoke codex \
  "Can we make this even shorter?" \
  --session code-golf

# Explain the solution
$ gptengage invoke claude \
  "Explain how this solution works" \
  --session code-golf
```

### Scenario: Multi-language Translation

Translate code between languages:

```bash
# Start with requirements
$ gptengage invoke claude \
  "I have this Python function. How would you write it in Rust?" \
  --context-file function.py \
  --session lang-translation

# Get idiomatic version
$ gptengage invoke claude \
  "Make it more idiomatic Rust" \
  --session lang-translation

# Verify correctness
$ gptengage invoke claude \
  "Is this equivalent to the Python version? Any subtle differences?" \
  --session lang-translation
```

---

## Agent-Driven Workflows

### Scenario: Multi-Instance Debates

Run debates with multiple instances of the same AI for diverse perspectives:

```bash
# Run a debate with 3 Claude instances
$ gptengage debate "What's the best testing strategy for microservices?" \
  --agent claude \
  --instances 3 \
  -r 4 \
  -o markdown > testing-strategy.md

# Use 5 instances for more diverse perspectives
$ gptengage debate "Should we use GraphQL or REST for our API?" \
  --agent claude \
  --instances 5 \
  --rounds 3 \
  -o json > api-decision.json

# Quick debate with minimal instances
$ gptengage debate "Is this refactoring worth the effort?" \
  --agent claude \
  --instances 2 \
  -r 2 \
  -t 60
```

### Scenario: Generate Agent Definitions

Create reusable agent definition files for complex personas:

```bash
# Generate agent definition files
$ gptengage generate-agents

# This creates YAML files in ~/.config/gptengage/agents/
# - security-expert.yaml
# - performance-engineer.yaml
# - ux-advocate.yaml
# - etc.

# List generated agents
$ ls ~/.config/gptengage/agents/
security-expert.yaml  performance-engineer.yaml  ux-advocate.yaml  ...
```

Example agent definition file (`security-expert.yaml`):
```yaml
name: Security Expert
role: security
expertise:
  - vulnerability analysis
  - secure coding practices
  - threat modeling
  - penetration testing
personality: thorough, cautious, detail-oriented
focus: identifying security risks and recommending mitigations
```

### Scenario: Using Generated Agents in Debates

Use your custom agent definitions for specialized debates:

```bash
# Run a security-focused debate using agent files
$ gptengage debate "Is our authentication implementation secure?" \
  -p security-expert \
  --agent claude \
  --instances 3 \
  -o markdown > security-review.md

# Combine multiple personas for comprehensive review
$ gptengage debate "Should we migrate to a new database?" \
  --context-file schema.sql \
  -r 4 \
  -o json > migration-analysis.json
```

### Scenario: JSON Output for Programmatic Use

Parse debate output programmatically for automation:

```bash
# Generate JSON output
$ gptengage debate "Optimize database queries" \
  --agent claude \
  --instances 3 \
  -o json > debate.json

# Extract the final consensus
$ jq '.consensus' debate.json

# Get all arguments from a specific round
$ jq '.rounds[0].arguments' debate.json

# Count total arguments
$ jq '[.rounds[].arguments[]] | length' debate.json

# Extract participant names
$ jq '.participants | keys' debate.json

# Get the winning position
$ jq '.result.winner' debate.json
```

---

## Tips for Effective Usage

### Multi-AI Strategy

```bash
# Different AIs have different strengths:
# - Claude: Deep analysis, explanations, writing
# - Codex: Code generation, specific implementations
# - Gemini: Quick answers, creative thinking

# Use a debate to explore different perspectives
$ gptengage debate "What's the best approach to this problem?" -r 3

# Follow up with specific AIs for their strengths
$ gptengage invoke claude "Deep analysis" -s followup
$ gptengage invoke codex "Show me the code" -s followup
```

### Multi-Instance vs Cross-AI Debates

Choose the right debate mode for your use case:

```bash
# Cross-AI debates (default): Use when you want different AI perspectives
# Each AI (Claude, Codex, Gemini) brings unique strengths and biases
$ gptengage debate "Which framework should we use?" -r 3

# Multi-instance debates: Use when you want consistent reasoning style
# Multiple instances of the same AI explore different angles
$ gptengage debate "Which framework should we use?" \
  --agent claude \
  --instances 3 \
  -r 3

# When to use multi-instance:
# - You trust one AI's judgment more than others
# - You want stylistically consistent output
# - You're building automation that expects consistent response format
# - You're testing the robustness of an AI's reasoning

# When to use cross-AI:
# - You want diverse perspectives from different models
# - You're exploring a topic where different AIs have different expertise
# - You want to identify blind spots in any single AI's reasoning
```

### Using Personas Effectively

```bash
# Generate persona definition files for reuse
$ gptengage generate-agents

# Use built-in personas for common roles
$ gptengage debate "Is this secure?" -p security-expert -r 3

# Create custom personas for your domain
# Edit ~/.config/gptengage/agents/your-persona.yaml

# Combine personas with multi-instance for deep expertise
$ gptengage debate "Review this authentication flow" \
  -p security-expert \
  --agent claude \
  --instances 3 \
  -r 4
```

### Session Organization

```bash
# Use descriptive session names
✓ gptengage invoke claude "..." --session auth-module-refactor
✓ gptengage invoke claude "..." --session k8s-migration-planning
✗ gptengage invoke claude "..." --session s1

# Use topics for clarity
$ gptengage invoke claude "..." \
  --session auth-refactor \
  --topic "Authentication Module Refactoring"
```

### Exporting Results

```bash
# Export to markdown
$ gptengage session show my-session > results.md

# Append to document
$ gptengage session show my-session >> team-decisions.md

# Format as JSON
$ gptengage debate "topic" --output json > debate.json
$ jq . debate.json  # Pretty-print

# Create reports
$ gptengage debate "Architecture" --rounds 5 --output markdown > ARCHITECTURE.md
```

---

## Common Patterns

### Pattern: Follow-up Questions

Keep building on previous responses:

```bash
$ gptengage invoke claude "What is XYZ?" --session learning
$ gptengage invoke claude "Can you give an example?" --session learning
$ gptengage invoke claude "How would I use this in practice?" --session learning
$ gptengage invoke claude "What are common mistakes?" --session learning
```

### Pattern: Multi-Perspective Analysis

Get different viewpoints on the same topic:

```bash
$ gptengage invoke claude "Analyze this proposal" --session analysis
$ gptengage invoke codex "What are the implementation challenges?" --session analysis
$ gptengage invoke gemini "What's the user impact?" --session analysis
```

### Pattern: Iterative Refinement

Build something better with each iteration:

```bash
$ gptengage invoke claude "Draft a proposal" --session proposal
$ gptengage invoke claude "Improve the language and clarity" --session proposal
$ gptengage invoke claude "Add metrics and examples" --session proposal
$ gptengage invoke claude "Format for executive review" --session proposal
```

### Pattern: Decision Documentation

Capture the reasoning behind a decision:

```bash
$ gptengage debate "Option A vs Option B" -o markdown > decision.md
$ gptengage invoke claude "Why is this the right choice?" --session decision
$ gptengage session show decision >> decision.md
```

---

## For AI Agents

This section provides patterns for AI agents and automated systems that integrate with gptengage.

### Parsing JSON Output

Use jq to extract specific data from debate results:

```bash
# Run debate and capture JSON output
$ gptengage debate "Best approach for error handling" \
  --agent claude \
  --instances 3 \
  -o json > result.json

# Extract the consensus summary
$ jq -r '.consensus.summary' result.json

# Get all arguments from round 1
$ jq '.rounds[0].arguments[]' result.json

# Extract just the conclusions
$ jq -r '.rounds[].conclusions[]' result.json

# Get participant count
$ jq '.participants | length' result.json

# Filter for arguments containing a specific keyword
$ jq -r '.rounds[].arguments[] | select(. | contains("performance"))' result.json

# Create a summary object
$ jq '{
  topic: .topic,
  rounds: (.rounds | length),
  consensus: .consensus.summary,
  participants: [.participants[].name]
}' result.json
```

### Error Handling with Exit Codes

Handle gptengage exit codes in scripts:

```bash
#!/bin/bash
# error-handling.sh - Robust error handling for gptengage

set -e

run_debate() {
    local topic="$1"
    local output_file="$2"

    if gptengage debate "$topic" -o json > "$output_file" 2>&1; then
        echo "Debate completed successfully"
        return 0
    else
        local exit_code=$?
        case $exit_code in
            1)
                echo "Error: Invalid arguments or configuration"
                ;;
            2)
                echo "Error: AI CLI not available"
                ;;
            3)
                echo "Error: Timeout during debate"
                ;;
            *)
                echo "Error: Unknown error (code: $exit_code)"
                ;;
        esac
        return $exit_code
    fi
}

# Usage
run_debate "Should we use containers?" "debate-result.json"
```

### Automated Debate Workflow

Complete bash script for automated debate workflows:

```bash
#!/bin/bash
# automated-debate.sh - Run debates and process results automatically

set -euo pipefail

# Configuration
DEBATE_TOPIC="${1:-"Best practices for API design"}"
OUTPUT_DIR="${2:-./debate-results}"
ROUNDS="${3:-3}"
INSTANCES="${4:-3}"

# Create output directory
mkdir -p "$OUTPUT_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="$OUTPUT_DIR/debate_$TIMESTAMP.json"
SUMMARY_FILE="$OUTPUT_DIR/summary_$TIMESTAMP.md"

echo "Starting automated debate workflow..."
echo "Topic: $DEBATE_TOPIC"
echo "Output: $OUTPUT_FILE"

# Check that required tools are available
if ! command -v gptengage &> /dev/null; then
    echo "Error: gptengage not found in PATH"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "Error: jq not found (required for JSON parsing)"
    exit 1
fi

# Check AI CLI availability
if ! gptengage status &> /dev/null; then
    echo "Warning: Some AI CLIs may not be available"
fi

# Run the debate
echo "Running debate with $INSTANCES instances for $ROUNDS rounds..."
if ! gptengage debate "$DEBATE_TOPIC" \
    --agent claude \
    --instances "$INSTANCES" \
    -r "$ROUNDS" \
    -t 120 \
    -o json > "$OUTPUT_FILE" 2>&1; then
    echo "Debate failed. Check $OUTPUT_FILE for details."
    exit 1
fi

# Validate JSON output
if ! jq empty "$OUTPUT_FILE" 2>/dev/null; then
    echo "Error: Invalid JSON output"
    exit 1
fi

# Generate summary
echo "Generating summary..."
{
    echo "# Debate Summary"
    echo ""
    echo "**Topic:** $DEBATE_TOPIC"
    echo "**Date:** $(date)"
    echo "**Rounds:** $ROUNDS"
    echo "**Instances:** $INSTANCES"
    echo ""
    echo "## Consensus"
    echo ""
    jq -r '.consensus.summary // "No consensus reached"' "$OUTPUT_FILE"
    echo ""
    echo "## Key Points"
    echo ""
    jq -r '.rounds[-1].arguments[] | "- " + .' "$OUTPUT_FILE" 2>/dev/null || echo "- No key points extracted"
} > "$SUMMARY_FILE"

echo "Workflow complete!"
echo "Full results: $OUTPUT_FILE"
echo "Summary: $SUMMARY_FILE"

# Return success
exit 0
```

### Chaining Debates

Chain multiple debates for complex decision-making:

```bash
#!/bin/bash
# chain-debates.sh - Chain multiple related debates

# Phase 1: High-level decision
gptengage debate "Monolith or Microservices?" \
  --agent claude --instances 3 \
  -r 2 -o json > phase1.json

PHASE1_RESULT=$(jq -r '.consensus.summary' phase1.json)

# Phase 2: Follow-up based on Phase 1 result
gptengage debate "Given: $PHASE1_RESULT - What communication pattern?" \
  --agent claude --instances 3 \
  -r 2 -o json > phase2.json

# Phase 3: Implementation details
PHASE2_RESULT=$(jq -r '.consensus.summary' phase2.json)
gptengage debate "Implement $PHASE2_RESULT with which framework?" \
  --agent claude --instances 3 \
  -r 2 -o json > phase3.json

# Combine results
jq -s '{
  phase1: .[0].consensus,
  phase2: .[1].consensus,
  phase3: .[2].consensus
}' phase1.json phase2.json phase3.json > final-decision.json

echo "Decision chain complete. See final-decision.json"
```

### Integrating with CI/CD

Use gptengage in CI/CD pipelines:

```yaml
# .github/workflows/architecture-review.yml
name: Architecture Review

on:
  pull_request:
    paths:
      - 'src/architecture/**'

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install gptengage
        run: cargo install gptengage

      - name: Run architecture debate
        run: |
          gptengage debate "Review these architecture changes" \
            --context-file src/architecture/design.md \
            --agent claude \
            --instances 2 \
            -r 2 \
            -o json > review.json

      - name: Extract recommendations
        run: |
          echo "## Architecture Review" >> $GITHUB_STEP_SUMMARY
          jq -r '.consensus.summary' review.json >> $GITHUB_STEP_SUMMARY
```

---

## Performance Tips

```bash
# For faster debates, reduce timeout
$ gptengage debate "Quick topic" -r 1 -t 30

# For complex analyses, increase timeout
$ gptengage invoke claude "Complex analysis" -t 180

# Check available CLIs before starting long tasks
$ gptengage status

# Reuse sessions for related questions to save thinking time
$ gptengage invoke claude "First question" -s my-analysis
$ gptengage invoke claude "Related follow-up" -s my-analysis
```
