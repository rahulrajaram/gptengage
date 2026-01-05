# GPT Engage Examples

Real-world usage scenarios and practical examples.

## Table of Contents

- [Code Review Workflow](#code-review-workflow)
- [Learning & Teaching](#learning--teaching)
- [Decision Making](#decision-making)
- [Content Generation](#content-generation)
- [Problem Solving](#problem-solving)
- [Team Collaboration](#team-collaboration)

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
  --rounds 3 \
  --output markdown > arch-decision.md

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
  --rounds 5 \
  --output markdown > tech-decision.md

# Follow up with specific question
$ gptengage invoke claude \
  "Given the team has Python experience, does this change your recommendation?" \
  --session tech-choice

# Document the final decision
$ gptengage session show tech-choice >> tech-decision.md
```

### Scenario: Architecture Decision Record (ADR)

Create a formal architecture decision:

```bash
# Get the debate
$ gptengage debate "Monolithic vs Microservices for a startup" \
  --rounds 3 \
  --output markdown > adr.md

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
  --rounds 3 \
  --output json > features.json

# Discuss with your team's decision-maker
$ gptengage invoke claude \
  "You've analyzed the trade-offs. Which is more valuable for our user base?" \
  --session feature-priority
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
  --rounds 3 \
  --output markdown > coding-standards.md

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
  --rounds 2 \
  --output markdown > retro-notes.md

# Discuss improvements
$ gptengage invoke claude \
  "Based on these perspectives, what are the top 3 improvements we should try?" \
  --session retro-improvements

# Create action items
$ gptengage invoke claude \
  "Format these as specific, measurable action items" \
  --session retro-improvements
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

## Tips for Effective Usage

### Multi-AI Strategy

```bash
# Different AIs have different strengths:
# - Claude: Deep analysis, explanations, writing
# - Codex: Code generation, specific implementations
# - Gemini: Quick answers, creative thinking

# Use a debate to explore different perspectives
$ gptengage debate "What's the best approach to this problem?" --rounds 3

# Follow up with specific AIs for their strengths
$ gptengage invoke claude "Deep analysis" --session followup
$ gptengage invoke codex "Show me the code" --session followup
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
$ gptengage debate "Option A vs Option B" --output markdown > decision.md
$ gptengage invoke claude "Why is this the right choice?" --session decision
$ gptengage session show decision >> decision.md
```

---

## Performance Tips

```bash
# For faster debates, reduce timeout
$ gptengage debate "Quick topic" --rounds 1 --timeout 30

# For complex analyses, increase timeout
$ gptengage invoke claude "Complex analysis" --timeout 180

# Check available CLIs before starting long tasks
$ gptengage status

# Reuse sessions for related questions to save thinking time
$ gptengage invoke claude "First question" --session my-analysis
$ gptengage invoke claude "Related follow-up" --session my-analysis
```
