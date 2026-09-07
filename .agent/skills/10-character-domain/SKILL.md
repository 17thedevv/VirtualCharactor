---
name: Character Domain
description: Skill for reasoning about the core character entities and their distinct boundaries.
---

# Character Domain

## Purpose
This skill teaches agents the fundamental relationship between the primary entities in the VirtualCharacter core domain. It reinforces the rigid boundaries between what a character *is*, what it *feels*, what it *remembers*, and how it relates to *others*.

## When to Apply
Apply this skill whenever you are modifying structs inside `vc-core` or writing logic that relies on the character's internal state.

## Required Reading
- [docs/character.md](file:///d:/Project-17/VirtualCharactor/docs/character.md)
- [docs/contracts.md](file:///d:/Project-17/VirtualCharactor/docs/contracts.md)

## Core Principles
1. **Identity ≠ Personality**: Identity is the ID and immutable baseline. Personality is the structured traits.
2. **Personality ≠ State**: Personality is stable. State is temporary (emotion).
3. **State ≠ Memory**: State is right now. Memory is the past.
4. **Relationship ≠ CharacterState**: Relationships are strictly per-user, not global.

## Workflow
1. Identify which entity you need to modify or read.
2. Verify its scope (Global vs Per-User vs Ephemeral).
3. If writing data, ensure it goes to the correct repository (e.g., don't save a user's name in `CharacterState`, save it in `RelationshipState` or `Memory`).

## Rules
- **Respect Scope and Ownership**: Dev A owns these structures. Do not add fields to these structs just to make orchestration (Dev B) easier.
- **No Persistence Logic**: Do not add SQLite annotations to domain structs.

## Anti-Patterns
- Storing "User A likes cats" in `CharacterState`. (It belongs in Memory or Relationship).
- Creating a single monolithic `Character` struct that holds *all* memories and relationships in a `Vec`. (Repositories handle this).

## Validation Checklist
- [ ] Is the data being added to the semantically correct struct?
- [ ] Does this change preserve the separation of Identity, Personality, State, and Memory?

## Completion Criteria
Domain models accurately reflect the business semantics without blurring the boundaries between distinct psychological concepts.
