# VirtualCharacter Development Workflow

## Two-Developer Ownership Model

The repository is designed to be split between two developers working in parallel.

### Developer A (Character Intelligence)
Owns `vc-core` modules:
- `personality/`
- `memory/`
- `state/`
- `relationship/`
- `decision/`

Focuses on domain semantics, state transitions, memory lifecycle, and decision algorithms.

### Developer B (Runtime Boundary)
Owns:
- `vc-runtime/`
- `vc-llm/`
- `vc-storage/`
- `vc-core/context/`

Focuses on orchestration, LLM provider integration, and context construction.

## Branch Workflow
- Work in a feature branch (e.g. `feature/personality`).
- Keep changes scoped to the owned subsystem.
- Do not rewrite another developer's subsystem without coordination.

## PR Expectations
All Pull Requests must pass the following checks:
- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features`

## Coordination
Changes to core domain types, trait signatures, or interaction IO types require explicit agreement from both developers.
