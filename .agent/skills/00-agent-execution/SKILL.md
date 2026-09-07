---
name: Agent Execution Workflow
description: The meta-skill defining the universal workflow for all AI agents working on VirtualCharacter.
---

# Agent Execution Workflow

## Purpose
This is the meta-skill for all AI agents working on the VirtualCharacter project. It defines the universal workflow required to safely and effectively modify the system, ensuring that no agent acts on assumptions, breaks contracts, or introduces architectural drift.

## When to Apply
Apply this skill at the very beginning of **every single task** before taking any action or writing any code.

## Required Reading
- [docs/development.md](file:///d:/Project-17/VirtualCharactor/docs/development.md)
- [docs/contracts.md](file:///d:/Project-17/VirtualCharactor/docs/contracts.md)

## Core Principles
1. **Understand Before Acting**: Never generate implementation code without fully understanding the domain rules and the current state of the repository.
2. **Respect Boundaries**: The project relies on strict isolation (e.g., `vc-core` vs `vc-runtime`).
3. **Contract Adherence**: Public interfaces marked as "Stable" in `contracts.md` are sacred.

## Workflow
Execute every task using the following strict sequence:
1. **READ**: Read the user's request and identify the targeted subsystem.
2. **UNDERSTAND**: Read the relevant `docs/design/*.md` specifications for that subsystem.
3. **CLASSIFY TASK**: Determine if this is a Domain (Dev A) or Infrastructure (Dev B) task.
4. **CHECK OWNERSHIP**: Ensure you are not modifying code owned by the other developer unless explicitly instructed.
5. **CHECK CONTRACTS**: Look up the affected components in `docs/contracts.md`. Are they Stable or Evolving?
6. **PLAN**: Draft an implementation plan artifact and wait for user approval.
7. **IMPLEMENT**: Write the code.
8. **TEST**: Add deterministic tests (unit/integration) to prove correctness.
9. **REVIEW**: Self-audit the changes against the anti-patterns.
10. **REPORT**: Summarize the completed work.

## Rules
- **Stop and Ask**: You MUST pause execution and explicitly ask the user for a decision if your task requires:
  - Breaking a contract marked as **Stable**.
  - Changing the dependency direction (e.g., `vc-core` depending on `vc-llm`).
  - Cross-owner modifications (e.g., modifying `vc-core::memory` while working as Dev B).
  - Ambiguous semantics not covered by the current specifications.

## Anti-Patterns
- Jumping straight to code generation (`write_to_file`) without reading the relevant architecture document first.
- Implementing features based on generic software knowledge rather than reading the specific VirtualCharacter documentation.
- Silently bypassing a contract or creating a "temporary" workaround.

## Validation Checklist
- [ ] Have I identified the specific subsystem this task affects?
- [ ] Have I read the corresponding design document?
- [ ] Is my proposed change within my assigned ownership area (Dev A vs Dev B)?
- [ ] Does this change respect the stability matrix in `contracts.md`?

## Completion Criteria
The workflow is complete when the task has been planned, implemented, tested, and a final report artifact has been delivered to the user without violating any architecture rules.
