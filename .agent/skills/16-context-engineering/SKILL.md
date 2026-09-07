---
name: Context Engineering
description: Skill for assembling, compressing, and prioritizing information for the LLM.
---

# Context Engineering

## Purpose
This skill teaches agents how to build the `Context` object that represents the LLM's "working memory" for a single interaction, emphasizing budget constraints and relevance.

## When to Apply
Apply this skill when modifying what information is passed to the LLM, adjusting token budgets, or implementing the `ContextBuilder`.

## Required Reading
- [docs/design/context.md](file:///d:/Project-17/VirtualCharactor/docs/design/context.md)

## Core Principles
1. **Context ≠ Memory**: Context is the specific subset of memories, state, and history relevant *right now*.
2. **Context ≠ Personality**: Context contains a compressed representation of personality, not the entire database record.
3. **Budgeting is Mandatory**: The LLM has a finite context window.

## Workflow
1. Collect raw inputs from Memory, State, Personality, and Conversation History.
2. Assign priority scores to `ContextItem`s.
3. Compress or trim items starting from the lowest priority until the token budget is met.
4. Assemble into a structured `Context` output.

## Rules
- **Context Minimization**: Give the LLM the absolute minimum information required for the current task. Do not dump the entire character database into the prompt.
- **Source Tracking**: Every `ContextItem` must trace back to its source (e.g., MemoryId, SessionId) for observability.

## Anti-Patterns
- Concatenating 100 past messages into a string without checking the token limit.
- Injecting raw SQLite rows into the context prompt.

## Validation Checklist
- [ ] Does the `ContextBuilder` enforce a strict budget?
- [ ] Are context items prioritized intelligently?
- [ ] Is the final output abstracted away from the raw storage formats?

## Completion Criteria
The context engine deterministically assembles a highly relevant, budget-constrained snapshot of the character's reality for the LLM.
