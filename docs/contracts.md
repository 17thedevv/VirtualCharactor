# VirtualCharacter Public Domain Contracts

This document is the internal API agreement between two developers working in parallel.

Changes to any item marked **stable** require explicit approval from both developers.

---

## Identity Types

| Type | Location | Owner | Stability | Notes |
|------|----------|-------|-----------|-------|
| `CharacterId` | `vc-core::character` | Shared | **Stable** | Newtype over `Uuid` |
| `PersonalityId` | `vc-core::personality` | Dev A | **Stable** | Newtype over `Uuid` |
| `MemoryId` | `vc-core::memory` | Dev A | **Stable** | Newtype over `Uuid` |
| `RelationshipId` | `vc-core::relationship` | Dev A | **Stable** | Newtype over `Uuid` |
| `DecisionId` | `vc-core::decision` | Dev A | **Stable** | Newtype over `Uuid` |

---

## Domain Entities

| Type | Location | Owner | Stability | Notes |
|------|----------|-------|-----------|-------|
| `Character` | `vc-core::character` | Shared | **Stable** | Aggregate root |
| `Personality` | `vc-core::personality` | Dev A | **Stable** (structure) | Fields may grow, but existing fields must not be removed without review |
| `Memory` | `vc-core::memory` | Dev A | Evolving | Internal representation may change as memory engine develops |
| `MemoryType` | `vc-core::memory` | Dev A | **Stable** | Enum variants are additive only |
| `MemoryQuery` | `vc-core::memory` | Dev A | Evolving | Query interface will expand |
| `CharacterState` | `vc-core::state` | Dev A | **Stable** (structure) | Submodules: emotion, cognitive, behavior, goals, session |
| `EmotionState` | `vc-core::state::emotion` | Dev A | Evolving | Will grow when emotion engine is built |
| `Relationship` | `vc-core::relationship` | Dev A | **Stable** (structure) | Contains `RelationshipState` |
| `RelationshipState` | `vc-core::relationship` | Dev A | Evolving | Per-relationship dynamic state |
| `Context` | `vc-core::context` | Dev B | **Stable** | List of `ContextItem`s |
| `ContextItem` | `vc-core::context` | Dev B | **Stable** | Source + content + priority |
| `Decision` | `vc-core::decision` | Dev A | **Stable** | Contains `DecisionResult` |
| `Action` | `vc-core::decision` | Dev A | **Stable** | action_type + payload |
| `CharacterEvent` | `vc-core::event` | Shared | Evolving | Event variants will grow |

---

## Trait Boundaries

| Trait | Location | Owner | Stability | Approval to change |
|-------|----------|-------|-----------|-------------------|
| `DecisionEngine` | `vc-core::decision` | Dev A (trait), Dev B (mock in `vc-runtime`) | **Stable** | Both developers |
| `LlmProvider` | `vc-llm::provider` | Dev B | **Stable** | Both developers |
| `CharacterRepository` | `vc-storage::repository` | Dev B | **Stable** | Both developers |
| `MemoryRepository` | `vc-storage::repository` | Dev B | **Stable** | Both developers |

---

## Ownership Map

### Developer A — Character Intelligence

Owns `vc-core` modules:
- `personality/`
- `memory/`
- `state/` (emotion, cognitive, behavior, goals, session)
- `relationship/`
- `decision/` (trait definition only, not mock implementation)

### Developer B — Runtime & Boundary

Owns:
- `vc-runtime/` (including `MockDecisionEngine`)
- `vc-llm/` (provider abstraction, Gemini adapter, mock)
- `vc-storage/` (repository traits, SQLite adapter)
- `vc-core::context/`

---

## Visibility Rules

| Item | Visible outside its crate? |
|------|---------------------------|
| `DecisionEngine` trait | Yes — public contract |
| `MockDecisionEngine` | Yes — lives in `vc-runtime`, not `vc-core` |
| `LlmProvider` trait | Yes — public contract |
| `LlmRequest` / `LlmResponse` | Yes — provider-neutral types |
| Gemini request/response format | **No** — private to `vc-llm::gemini` |
| SQLite schema | **No** — private to `vc-storage::sqlite` |
| Memory internals | **No** — implementation detail of Dev A |

---

## Change Policy

- **Stable** items: signature changes require PR review from both developers.
- **Evolving** items: the owner may change internals freely but must not break existing consumers.
- **Additive changes** (new fields, new enum variants): generally safe, notify the other developer.
- **Breaking changes** (removed fields, changed signatures): require explicit agreement and ADR.
