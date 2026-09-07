VirtualCharacter Development Plan

Project type: Rust-first VirtualCharacter runtime/library
Initial direction: library/runtime-first, not a web application
Development model: 2 developers working in parallel
Primary goal: create a persistent AI character runtime with independent personality, memory, state, context, and decision systems.

1. Project Vision

VirtualCharacter is a runtime for persistent AI characters.

The character is not defined solely by the underlying LLM. Instead, the system maintains an independent character model consisting of:

Personality
    +
Memory
    +
State
    +
Relationship
    +
Context
    +
Decision
    ↓
LLM
    ↓
Response / Action

The LLM is treated as a cognitive/language engine. It must not become the source of truth for the character's identity, persistent memory, or long-lived state.

The architecture must allow the LLM provider to be replaced without rewriting the character core.

2. Initial Product Scope

In scope

Character identity and personality representation.

Persistent and temporary memory.

Character emotional/cognitive/behavioral state.

Per-user relationship state.

Context construction and prioritization.

Decision/action abstraction.

LLM provider abstraction.

Gemini adapter as the first provider.

Persistence abstraction and initial local storage.

Runtime orchestration.

Deterministic mocks and integration tests.

Future support for learned models in decision, retrieval, and state-transition tasks.

Out of scope for the first architecture phase

Web UI.

Mobile UI.

Full SaaS/server product.

Local LLM training.

Large-scale ML infrastructure.

Production vector database.

Complete RAG implementation.

Full production API.

Automatic online fine-tuning after every interaction.

3. Architectural Principles

3.1 Core independence

The domain core must not depend on:

Gemini SDKs.

HTTP clients.

Axum.

SQLite implementation details.

UI.

Provider-specific request/response structures.

3.2 Provider independence

The system should support:

Gemini
Claude
OpenAI-compatible providers
Local models
Mock models

without changing the domain model.

3.3 Memory is not context

Memory answers:

What does the character know or remember?

Context answers:

What information does the LLM need right now?

3.4 Personality is not state

Personality defines relatively stable tendencies.

State defines current conditions.

3.5 Decision is not generation

Decision answers:

What should the character do?

LLM generation answers:

How should that decision be expressed or executed?

3.6 Learned models are optional components

A small ML model may later be used for:

memory formation;

memory retrieval/ranking;

emotion/state transition;

decision candidate ranking;

personalization.

These models must plug into stable interfaces and must not become hard dependencies of the core architecture.

4. High-Level Architecture

                         VirtualCharacter
                                │
          ┌─────────────────────┼─────────────────────┐
          ↓                     ↓                     ↓
     Personality             Memory                State
          │                     │                     │
          └──────────────┬──────┴──────┬──────────────┘
                         ↓             ↓
                    Relationship    Context
                         │             │
                         └──────┬──────┘
                                ↓
                           Decision
                                ↓
                         LLM Provider
                                ↓
                        Response / Action
                                ↓
                         Outcome Analysis
                                ↓
                      State / Memory Update

5. Rust Workspace Architecture

Initial workspace:

virtual-character/
├── Cargo.toml
├── crates/
│   ├── vc-core/
│   ├── vc-runtime/
│   ├── vc-llm/
│   └── vc-storage/
│
├── apps/
│   └── vc-cli/
│
├── docs/
├── tests/
└── README.md

5.1 vc-core

Domain layer.

Expected modules:

vc-core/
└── src/
    ├── character/
    ├── personality/
    ├── memory/
    ├── state/
    ├── relationship/
    ├── context/
    ├── decision/
    ├── event/
    ├── error.rs
    └── lib.rs

Responsibilities:

Domain entities.

Value objects.

Domain rules.

Domain traits/interfaces.

Domain events.

Domain-level validation.

Must not depend on infrastructure.

5.2 vc-runtime

Application/orchestration layer.

Responsibilities:

Interaction lifecycle.

Character session lifecycle.

Orchestration between domain services.

Context building.

Decision execution.

LLM invocation through provider abstraction.

State and memory update flow.

5.3 vc-llm

LLM boundary.

Expected structure:

vc-llm/
└── src/
    ├── provider.rs
    ├── types.rs
    ├── gemini/
    ├── mock/
    └── lib.rs

Responsibilities:

Provider abstraction.

Provider-neutral request/response types.

Gemini implementation.

Mock implementation.

Error mapping.

Streaming support later.

Provider-specific types must not leak into vc-core.

5.4 vc-storage

Persistence boundary.

Expected structure:

vc-storage/
└── src/
    ├── repository/
    ├── sqlite/
    └── lib.rs

Responsibilities:

Repository abstractions.

Persistence adapters.

SQLite implementation.

Mapping between domain objects and persistence models.

The first storage implementation does not need to be production-grade.

5.5 vc-cli

Development/testing application.

The CLI exists to exercise the runtime without introducing a web UI.

Initial flow:

CLI input
   ↓
vc-runtime
   ↓
vc-core
   ↓
Mock LLM
   ↓
CLI output

Gemini can be plugged in after the mock path works.

6. Dependency Direction

The intended dependency direction is:

vc-cli
   ↓
vc-runtime
   ├────────→ vc-llm
   ├────────→ vc-storage
   └────────→ vc-core

Rules:

vc-core     ✗ vc-llm
vc-core     ✗ vc-storage
vc-core     ✗ vc-cli
vc-core     ✗ web framework
vc-runtime  ✓ vc-core
vc-runtime  ✓ vc-llm
vc-runtime  ✓ vc-storage

The exact dependency graph may be adjusted if the implementation reveals a cleaner inversion boundary, but the core/infrastructure separation must remain intact.

7. Domain Contracts

Before feature implementation, freeze initial contracts for:

Character
CharacterId

Personality
Traits
Values
Preferences
BehaviorTendencies

Memory
MemoryId
MemoryType
MemoryQuery
MemoryMetadata

CharacterState
EmotionState
CognitiveState
BehaviorState

Relationship
RelationshipId

Context
ContextItem
ContextPriority
ContextSource

Decision
Action
DecisionCandidate
DecisionResult

CharacterEvent

Also define initial interfaces for:

DecisionEngine
MemoryStore / MemoryRepository
LlmProvider
StateRepository
CharacterRepository

These contracts are the main coordination surface between the two developers.

8. Parallel Development Strategy

Two-person ownership:

Developer A — Character Intelligence

Owns:

vc-core/
├── personality/
├── memory/
├── state/
├── relationship/
└── decision/

Primary responsibilities:

Domain semantics.

State transitions.

Memory lifecycle.

Relationship model.

Decision model.

Domain unit tests.

Developer B — Runtime Boundary

Owns:

vc-runtime/
vc-llm/
vc-storage/
vc-core/context/

Primary responsibilities:

Context construction.

Runtime orchestration.

LLM provider abstraction.

Gemini integration.

Persistence adapter.

Runtime/integration tests.

Shared responsibility

Both developers must agree before changing:

Public domain contracts.

Core IDs.

Interaction input/output types.

Event names.

Trait signatures.

Serialization formats used across crates.

9. Git Workflow

main must always remain buildable.

Branches:

feature/personality
feature/memory
feature/state
feature/relationship
feature/decision
feature/context
feature/llm
feature/storage
feature/runtime

Rules:

Work in a feature branch.

Keep changes scoped to the owned subsystem.

Do not rewrite another developer's subsystem without coordination.

Open a PR for integration.

PR must pass formatting, build, tests and clippy.

Avoid merging unrelated refactors together with feature work.

Required checks:

cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features

10. Development Phases

Phase 0 — Architecture Bootstrap

Goal:

Workspace
+
Module boundaries
+
Initial domain contracts
+
Infrastructure interfaces
+
Mocks
+
Documentation
+
Build/test baseline

No real AI behavior yet.

Phase 1 — Domain Foundation

Implement:

Personality domain.

State domain.

Relationship domain.

Memory domain.

Decision domain.

Domain events.

Validation/invariants.

Phase 2 — Memory System

Implement:

Memory formation.

Memory storage.

Retrieval.

Ranking.

Importance.

Recency.

Emotional relevance.

Memory decay.

Consolidation.

Conflict handling.

Phase 3 — State System

Implement:

Emotional state.

Cognitive state.

Behavioral state.

Relationship state.

State transitions.

Decay.

Persistence policy.

Phase 4 — Decision System

Implement:

Candidate generation.

Candidate evaluation.

Action selection.

Confidence.

Constraints.

Decision traces.

Deterministic baseline implementation.

Phase 5 — Context System

Implement:

Context assembly.

Retrieval integration.

Prioritization.

Token budget.

Compression.

Context validation.

Phase 6 — LLM Integration

Implement:

Gemini adapter.

Streaming.

Retry/error handling.

Provider configuration.

End-to-end response generation.

Phase 7 — Runtime Integration

Complete:

Input
 ↓
Situation analysis
 ↓
State update
 ↓
Memory retrieval
 ↓
Context building
 ↓
Decision
 ↓
LLM
 ↓
Response
 ↓
Outcome
 ↓
Memory/State update

Phase 8 — Learning / Adaptation

Only after deterministic behavior is stable.

Potential learned components:

Memory relevance model
Memory formation model
State transition model
Decision ranking model
User preference model

Start with offline training and evaluation.

Do not make online self-training a prerequisite for the runtime.

11. Integration Milestones

Milestone A

Character
→ Mock Decision
→ Mock LLM
→ Response

Milestone B

Character
→ Memory
→ State
→ Decision
→ Mock LLM

Milestone C

Character
→ Memory Retrieval
→ Context
→ Decision
→ Gemini

Milestone D

Conversation
→ State Update
→ Memory Formation
→ Retrieval
→ Decision
→ Gemini
→ Outcome

Milestone D is the first complete VirtualCharacter loop.

12. Definition of Done

A subsystem is considered complete only when:

Domain/API contract is defined.

Implementation is covered by tests.

Error cases are considered.

Serialization behavior is intentional.

Dependencies respect architecture.

Documentation is updated.

cargo test --workspace passes.

A feature is not considered complete merely because it compiles.

13. Architecture Change Policy

Architecture is expected to evolve.

However, changes to these items require explicit review by both developers:

Core domain types
Public traits
Dependency direction
Interaction pipeline
Memory semantics
State semantics
Decision semantics
LLM abstraction
Persistence abstraction

Use ADRs for significant decisions.

Recommended structure:

docs/
└── adr/
    ├── ADR-001-core-boundaries.md
    ├── ADR-002-memory-model.md
    ├── ADR-003-state-model.md
    └── ...

14. Initial Project Success Criteria

The initial architecture is successful when:

Two developers can work independently without constant merge conflicts.

vc-core can be tested without network access.

Gemini can be replaced by a mock provider.

Storage can be replaced without changing domain logic.

Character identity is independent from the LLM.

Memory and context are separate concepts.

Personality and state are separate concepts.

Decision is separate from text generation.

The full interaction pipeline can run end-to-end with mocks.

