---
name: Memory Engineering
description: Skill for designing and interacting with the memory subsystem.
---

# Memory Engineering

## Purpose
This is a critical skill for working with the VirtualCharacter memory subsystem. It teaches agents to distinguish the semantics of memory (what is remembered, why, and how to retrieve it) from the persistence of memory (databases).

## When to Apply
Apply this skill when modifying memory formation, retrieval, ranking, decay, or conflict resolution logic in `vc-core::memory`.

## Required Reading
- [docs/design/memory.md](file:///d:/Project-17/VirtualCharactor/docs/design/memory.md)

## Core Principles
1. **Semantic Ownership**: Memory semantics (decay rules, relevance ranking) live in `vc-core`.
2. **Persistence Agnosticism**: How memories are saved to disk lives in `vc-storage`.
3. **Relevance over Recency**: Retrieving memory relies on situational relevance, not just retrieving the last 10 things said.

## Workflow
Whenever you touch memory logic, explicitly ask:
1. Should this be remembered?
2. Why? (What is the significance?)
3. For how long? (Does it decay?)
4. Who is allowed to retrieve it? (Privacy scope by ActorId).
5. Is it relevant now?

## Rules
- **Do NOT store everything**: Raw interaction logs are not domain memories. Memory is *retained information*.
- **Do NOT retrieve everything**: The retrieval engine must aggressively filter based on the current context budget.
- **Do NOT put persistence logic inside memory semantics**: No SQL queries in `vc-core::memory`.
- **Do NOT leak private memory**: Memories must be scoped. User A's memory cannot be retrieved during User B's interaction.
- **Keep learned retrieval optional**: ML models for memory ranking are optional components.

## Anti-Patterns
- Fetching all memories for a character and doing an O(N) string search in memory inside the core domain.
- Assuming `Memory = Database Row`.

## Validation Checklist
- [ ] Is memory decay semantic (core) rather than procedural (database cronjob)?
- [ ] Are private memories strictly isolated by Actor/User ID?
- [ ] Does retrieval respect the context token budget?

## Completion Criteria
Memory operations are semantically rich, strictly isolated by user, and decoupled from underlying storage mechanisms.
