---
name: Relationship Engineering
description: Skill for enforcing the isolation and evolution of user-character relationships.
---

# Relationship Engineering

## Purpose
This skill teaches agents to model interactions as a bipartite graph (`Character ↔ Target`), ensuring that relationship dynamics like trust, affection, and familiarity are strictly isolated per user.

## When to Apply
Apply this skill when implementing features involving user trust, personalized character responses based on familiarity, or multi-user session isolation.

## Required Reading
- [docs/design/relationship.md](file:///d:/Project-17/VirtualCharactor/docs/design/relationship.md)
- [docs/design/security.md](file:///d:/Project-17/VirtualCharactor/docs/design/security.md)

## Core Principles
1. **Bipartite Relationship**: The relationship exists between the Character and an external Actor (User).
2. **Strict Isolation**: `Character A ↔ User X` is completely independent of `Character A ↔ User Y`.
3. **Relationship-Specific Knowledge**: Things learned about the user belong here or in scoped Memory, not in global Character configuration.

## Workflow
1. Always require an `ActorId` when fetching or modifying Relationship data.
2. Define the relationship dimensions (Trust, Closeness, Tension).
3. Write tests to prove that modifying User X's relationship does not affect User Y.

## Rules
- **Do not allow session state to silently become relationship state**: A bad mood in one session does not instantly destroy long-term relationship trust.
- **Scope Isolation**: Security boundaries mandate that relationships cannot cross-contaminate.

## Anti-Patterns
- Adding `pub user_trust: f32` directly to `CharacterState`.
- Allowing an Interaction from User B to read Relationship data from User A.

## Validation Checklist
- [ ] Are all relationship structures keyed by `ActorId`/`ParticipantId`?
- [ ] Have I prevented cross-user leakage in the codebase?

## Completion Criteria
Relationship dynamics are strictly isolated, securely scoped, and correctly modeled independently from the global character state.
