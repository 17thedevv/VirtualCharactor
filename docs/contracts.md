# VirtualCharacter Public Domain Contracts

This document is the internal API agreement between two developers working in parallel.

Changes to any item marked **stable** require explicit approval from both developers.

---

## Stability Matrix

| Contract | Owner | Stability |
|---|---|---|
| Personality | Dev A | Stable |
| CharacterState | Dev A | Stable |
| EmotionState | Dev A | Stable |
| Relationship | Dev A | Stable |
| Memory | Dev A | Stable |
| DecisionEngine | Dev A | Stable |
| Context | Dev B | Stable |
| LlmProvider | Dev B | Stable |
| Storage repositories | Dev B | Stable |
| Runtime orchestration | Dev B | Evolving |
| Gemini adapter | Dev B | Provider-specific |
| Learning components | Future | Evolving |

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
