---
name: Runtime Engineering
description: Skill for orchestrating the complete VirtualCharacter interaction lifecycle.
---

# Runtime Engineering

## Purpose
This skill guides the implementation of `vc-runtime`, which acts as the orchestrator. It ensures the runtime coordinates the flow of data without accidentally becoming a second domain layer.

## When to Apply
Apply this skill when modifying the interaction pipeline, session management, or top-level orchestration code.

## Required Reading
- [docs/design/interaction.md](file:///d:/Project-17/VirtualCharactor/docs/design/interaction.md)
- [docs/design/session.md](file:///d:/Project-17/VirtualCharactor/docs/design/session.md)

## Core Principles
1. **Orchestration Only**: The runtime connects Inputs -> Context -> Decision -> LLM -> Outputs.
2. **No Business Semantics**: The runtime does not decide *what* the character feels, it just calls `vc-core` to find out.
3. **Concurrency Control**: The runtime handles serializing requests per-character/user to prevent state corruption.

## Workflow
The orchestration lifecycle:
1. Input -> 2. Situation -> 3. Memory Retrieval -> 4. State check -> 5. Context -> 6. Decision -> 7. LLM -> 8. Outcome -> 9. State/Memory update.

## Rules
- **Avoid Business Logic**: Do not write `if user_input.contains("angry") { emotion = angry }` in the runtime. Pass the input to the domain's Decision Engine.
- **Isolate Sessions**: Ensure `vc-runtime` correctly isolates multi-user sessions when coordinating memory and state.

## Anti-Patterns
- Hardcoding specific emotional responses or personality traits in the orchestration pipeline.
- Dropping errors from `vc-core` instead of returning them through the runtime boundary.

## Validation Checklist
- [ ] Does the runtime strictly delegate decision-making to `vc-core`?
- [ ] Is the interaction pipeline fully implemented according to `interaction.md`?
- [ ] Are session boundaries maintained?

## Completion Criteria
The runtime successfully glues infrastructure and domain together, efficiently passing data between them without harboring any hidden business logic.
