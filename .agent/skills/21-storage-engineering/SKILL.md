---
name: Storage Engineering
description: Skill for designing persistence boundaries and repository implementations.
---

# Storage Engineering

## Purpose
This skill teaches agents how to implement the repository pattern securely and effectively, distinguishing between persistent character data and ephemeral runtime state.

## When to Apply
Apply this skill when modifying `vc-storage`, adding new database tables, or changing how domain objects are saved/loaded.

## Required Reading
- [docs/design/storage.md](file:///d:/Project-17/VirtualCharactor/docs/design/storage.md)

## Core Principles
1. **Repository Abstraction**: The domain defines the `Trait` (e.g., `CharacterRepository`). `vc-storage` provides the `SqliteCharacterRepository`.
2. **Data Separation**: Storage must not contain domain logic (personality, emotion, decision).
3. **Consistency & Idempotency**: Storage operations should be safe to retry.

## Workflow
1. Identify if the data is Persistent or Ephemeral.
2. If Persistent, define the repository interface in `vc-core`.
3. Implement the SQLite/Storage adapter in `vc-storage`.
4. Ensure schema migrations are isolated and versioned.

## Rules
- **No Domain Logic in Storage**: Do not use SQL triggers to implement memory decay or emotion normalization.
- **Do not store Ephemeral State**: `SessionState`, `Context`, and `Decision Intent` are ephemeral and do not belong in SQLite.

## Anti-Patterns
- Returning SQLx row types directly to `vc-core` instead of mapping them to domain structs.
- Implementing business logic (like deciding if a user is trusted) inside a SQL query.

## Validation Checklist
- [ ] Are all database operations hidden behind a repository trait?
- [ ] Are domain rules kept out of the SQL schema/queries?
- [ ] Is ephemeral data correctly excluded from persistence?

## Completion Criteria
The storage layer acts strictly as a dumb persistence mechanism, fulfilling the repository contracts defined by the domain without leaking database concepts upward.
