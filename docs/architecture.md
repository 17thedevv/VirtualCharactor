# VirtualCharacter Architecture

## 1. Layer Responsibilities

- **vc-core**: The domain layer. Contains pure business logic, domain entities (Character, Memory, Personality, etc.), and interfaces for external dependencies. It is completely infrastructure-agnostic.
- **vc-runtime**: The orchestration layer. Contains use cases, workflows, and the runtime loop. It orchestrates the domain entities and the infrastructure adapters.
- **vc-llm**: The LLM infrastructure layer. Contains the implementations of the `LlmProvider` trait defined in `vc-core`. E.g., a Gemini integration or a mock provider.
- **vc-storage**: The persistence layer. Contains the implementations of repository traits. Handles database connections, ORM logic, and file storage if necessary.
- **vc-cli**: The development/testing application. It provides a local demonstration of the runtime without requiring an API or web framework.

## 2. Dependency Direction

```
vc-cli -> (vc-runtime, vc-core, vc-llm, vc-storage)
vc-runtime -> (vc-core, vc-llm, vc-storage)
vc-llm -> (vc-core)
vc-storage -> (vc-core)
```

The domain core (`vc-core`) depends on nothing except standard libraries or utility crates like `uuid` and `serde`.

## 3. Domain/Infrastructure Boundary

Infrastructure details (like SQL databases, LLM APIs) are hidden behind traits.

## 4. LLM Abstraction

The `LlmProvider` trait abstracts away the specific provider (e.g., Gemini). The domain only knows about `LlmRequest` and `LlmResponse`.

## 5. Storage Boundary

Storage and retrieval mechanisms are hidden behind the `CharacterRepository` and `MemoryRepository` traits.

## 6. CLI Role

The `vc-cli` application is the first integration point to prove that the architecture is composable. It avoids the overhead of web frameworks in Phase 0.

`Context` is treated as a dynamically generated payload passed to the LLM, decoupling conversation history from what the LLM actually "sees" at any given moment.

## 7. Decision Boundary

The `DecisionEngine` takes `Context`, `Personality`, and `CharacterState` and produces a discrete `Decision`, completely isolating the "what to do" logic from the "how to do it" side effects.

## 8. Extension Points

- **LLMs**: Implement a new `LlmProvider`.
- **Storage**: Implement a new `CharacterRepository` or `MemoryRepository`.
- **API**: Add new routes or WebSocket events in `vc-api` without touching the domain logic.
