---
name: Architecture Governance
description: Skill for respecting strict architectural boundaries and preventing dependency drift.
---

# Architecture Governance

## Purpose
This skill teaches agents how to preserve the structural integrity of VirtualCharacter by enforcing strict dependency directions and preventing architectural drift between the core domain, orchestration layer, and infrastructure adapters.

## When to Apply
Apply this skill whenever you are adding a new module, introducing a new external crate/dependency, moving code between crates, or defining a new trait interface.

## Required Reading
- [docs/architecture.md](file:///d:/Project-17/VirtualCharactor/docs/architecture.md)

## Core Principles
1. **Dependency Inversion**: `vc-core` contains the business logic and defines the interfaces (Traits). It must NEVER depend on infrastructure.
2. **Orchestration Only**: `vc-runtime` glues everything together but does not own domain semantics.
3. **Infrastructure Isolation**: `vc-storage` and `vc-llm` are mere implementations of core traits. They do not dictate how the core behaves.

## Workflow
1. Check the crate of the file you are modifying.
2. Determine its architectural layer (Core, Runtime, Infrastructure).
3. If importing a module or adding a `Cargo.toml` dependency, verify it follows the allowed dependency graph.
4. If an operation requires breaking the graph, STOP and request an Architectural Decision Record (ADR) from the user.

## Rules
- **Explicit Boundaries**:
  - `vc-core` MUST NOT depend on `vc-llm`, `vc-storage`, `web`, `tokio` (unless strictly necessary for standard abstractions), or provider SDKs.
  - `vc-runtime` MUST depend on `vc-core` and infrastructure adapters.
  - `vc-storage` MUST depend on `vc-core` (to implement its traits).
  - `vc-llm` MUST depend on `vc-core` (to implement its traits).
- **Minimal Changes**: Prefer the smallest possible change to achieve the goal. Do not redesign the architecture for a minor feature.

## Anti-Patterns
- Leaking SQL/DB tags (like `sqlx` macros) into `vc-core` structs.
- Leaking LLM-specific parameters (like Gemini `temperature`) into core Decision structs.
- `vc-core` calling `reqwest` or performing file I/O directly.

## Validation Checklist
- [ ] Does the new code respect the `vc-core` isolation boundary?
- [ ] Are external dependencies strictly confined to `vc-storage` or `vc-llm`?
- [ ] Has the `Cargo.toml` dependency graph been verified against the rules?

## Completion Criteria
The architectural change is implemented without violating any dependency rules, and domain code remains completely agnostic of its runtime execution environment.
