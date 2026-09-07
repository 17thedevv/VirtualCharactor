---
name: Decision Engineering
description: Skill for modeling the character's internal decision-making process separately from LLM generation.
---

# Decision Engineering

## Purpose
This skill teaches agents the critical distinction between determining *what* to do (Decision) and determining *how to say it* (Generation). It defines the pipeline for the `DecisionEngine`.

## When to Apply
Apply this skill when modifying the character's capability to choose actions, evaluate rules, or rank possible responses.

## Required Reading
- [docs/design/decision.md](file:///d:/Project-17/VirtualCharactor/docs/design/decision.md)

## Core Principles
1. **Decision ≠ Generation**: `Decision` = what should the character do? `LLM generation` = how should it be expressed?
2. **Structured Intent**: Decisions must be strongly typed (e.g., `ActionType::Comfort`, `ActionType::AskQuestion`).
3. **Validation Authority**: The domain holds ultimate authority over whether a decision is valid.

## Workflow
Implement the decision pipeline:
1. **Context** -> Input to the engine.
2. **Candidate generation** -> Propose multiple possible actions.
3. **Candidate evaluation & Ranking** -> Score them based on Personality and State.
4. **Decision** -> Select the highest ranked valid action.
5. **Execution** -> Pass to runtime/LLM.

## Rules
- **Use Structured Decisions**: Return enums/structs from the Decision Engine, not raw strings.
- **Expose Confidence**: Include confidence scores where appropriate to allow fallbacks.
- **Preserve Domain Constraints**: Never let a learned model bypass domain validation (e.g., a learned model cannot force the character to violate core values).

## Anti-Patterns
- Generating final natural-language responses directly from the `vc-core` decision layer.
- Hardcoding LLM prompts inside the `DecisionEngine`.

## Validation Checklist
- [ ] Is the decision output a structured domain type?
- [ ] Can the decision engine be replaced by a mock in tests?
- [ ] Are learned/hybrid components treated as advisory rather than authoritative?

## Completion Criteria
The decision pipeline is strongly typed, testable without an LLM, and explicitly separates intent from natural language generation.
