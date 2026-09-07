---
name: Contract-First Development
description: Skill for establishing and respecting interface contracts between parallel developers.
---

# Contract-First Development

## Purpose
This skill ensures agents design and respect the interfaces (contracts) between subsystems before implementing the underlying logic. It prevents the two-developer (Dev A / Dev B) parallel workflow from breaking due to unannounced API changes.

## When to Apply
Apply this skill whenever creating new traits, modifying public functions in a crate, or implementing an interface owned by another developer.

## Required Reading
- [docs/contracts.md](file:///d:/Project-17/VirtualCharactor/docs/contracts.md)

## Core Principles
1. **Contract Before Code**: Define the inputs, outputs, and side-effects of an interface before writing the implementation details.
2. **Respect Stability**: Code stability guarantees allow parallel development. A "Stable" contract is a promise.

## Workflow
1. Check `docs/contracts.md` to see if the interface you are modifying is listed.
2. Note its Stability level:
   - **Stable**: You CANNOT change the signature without explicit user approval.
   - **Evolving**: You can modify it, but you must fix any internal consumers you break.
   - **Internal**: Free to change, but keep it private.
   - **Provider-specific**: Encapsulated within an infrastructure adapter.
3. Write or update the tests for the contract first (TDD approach).
4. Implement the logic to satisfy the contract.

## Rules
- **No Silent Rewrites**: Never rewrite an interface just because it is more convenient for the current implementation.
- **Additive Changes First**: Prefer adding new fields (wrapped in `Option` if necessary) or new enum variants over deleting or renaming existing ones.

## Anti-Patterns
- Writing the implementation first, realizing the current contract doesn't fit, and silently changing a Stable trait signature without notifying the user.
- Exporting internal helper functions as public APIs, accidentally creating new unintentional contracts.

## Validation Checklist
- [ ] Did I check `contracts.md` before changing a public struct or trait?
- [ ] If changing a Stable contract, did I explicitly stop and ask the user for approval?
- [ ] Are my changes strictly additive if possible?

## Completion Criteria
The interface changes (if any) are fully documented, respect the stability matrix, and have explicit tests verifying the contract's behavior.
