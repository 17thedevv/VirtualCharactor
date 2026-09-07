---
name: Security Engineering
description: Skill for enforcing data isolation, least privilege, and privacy boundaries.
---

# Security Engineering

## Purpose
This skill ensures agents respect the strict security and privacy boundaries of VirtualCharacter. It prevents cross-user data leakage and protects the integrity of the system from prompt injection or unauthorized access.

## When to Apply
Apply this skill universally, but especially when dealing with Relationships, Memory retrieval, and external APIs.

## Required Reading
- [docs/design/security.md](file:///d:/Project-17/VirtualCharactor/docs/design/security.md)

## Core Principles
1. **Isolation**: `Character A ↔ User X` must NEVER leak data to `User Y`.
2. **Least Privilege**: Components only get the data they need. (Context Engine minimizes data sent to LLM).
3. **Authentication ≠ Authorization**: Just because a user has an ID doesn't mean they can access a memory.

## Workflow
1. Validate ownership before retrieving any persistent data (Memory, Relationship).
2. Ensure secrets (API keys) are loaded via the Configuration subsystem, never hardcoded.
3. When building context for the LLM, verify no private data from other users is included.

## Rules
- **Fail Closed**: Security failures should result in an error or safe fallback, not unauthorized access.
- **Data Minimization**: Do not load the entire database into memory. Filter strictly by `ActorId`.
- **Sanitize LLM inputs**: Treat all raw user input as potentially malicious (prompt injection) before it hits the Decision Engine.

## Anti-Patterns
- Hardcoding API keys for tests or quick fixes.
- Retrieving memories by just `CharacterId` without filtering by the specific `ActorId` involved in the session.

## Validation Checklist
- [ ] Are all database queries for user-specific data strictly scoped by `ActorId`?
- [ ] Are API keys injected securely via the environment or config?
- [ ] Does the code fail closed on authorization/lookup errors?

## Completion Criteria
The implementation robustly prevents cross-user leakage, protects secrets, and limits the blast radius of any individual subsystem.
