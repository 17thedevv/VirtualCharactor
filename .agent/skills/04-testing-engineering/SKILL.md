---
name: Testing Engineering
description: Skill for writing deterministic, fast, and boundary-respecting tests.
---

# Testing Engineering

## Purpose
This skill teaches agents how to validate the VirtualCharacter system using the correct testing strategies, ensuring domain invariants are preserved without flaky or slow tests.

## When to Apply
Apply this skill whenever a new feature is added, a bug is fixed, or a contract is defined.

## Required Reading
- [docs/development.md](file:///d:/Project-17/VirtualCharactor/docs/development.md) (Testing sections if available)

## Core Principles
1. **Deterministic by Default**: Tests must yield the exact same result every time.
2. **Fast Domain Tests**: `vc-core` tests must run in milliseconds and require zero I/O.
3. **Mock Infrastructure**: External dependencies (LLM, Database) must be mocked for domain and runtime tests.

## Workflow
1. **Unit Tests**: Place in the same file `#[cfg(test)] mod tests { ... }`. Use for testing specific domain state transitions or logic.
2. **Integration Tests**: Place in `tests/` directory of the crate. Use for testing the interaction between multiple modules.
3. **Contract Tests**: Ensure that different implementations of a trait (e.g., `MemoryRepository`) behave identically.

## Rules
- **Require tests for**: Domain invariants (e.g., emotion bounds), state transitions, memory ranking, decision logic, and error paths.
- **Do NOT require LLM calls for ordinary tests**: Integration tests of the runtime must use the `MockDecisionEngine` or a Mock LLM provider. Live Gemini API calls are strictly reserved for manual E2E validation or specific isolated adapter tests.
- **Serialization**: Test serialization/deserialization ONLY where contractually relevant (e.g., persistence layers).

## Anti-Patterns
- Writing an integration test that makes an actual HTTP call to Google's Gemini API and fails when the network drops or quota is exceeded.
- Testing private helper functions excessively instead of testing the public contract of the module.
- Skipping tests for error conditions (e.g., what happens if storage fails?).

## Validation Checklist
- [ ] Are domain tests purely synchronous and free of I/O?
- [ ] Is the LLM mocked out for runtime integration tests?
- [ ] Do tests cover both the "happy path" and expected failure modes?

## Completion Criteria
All tests pass deterministically via `cargo test`, and coverage adequately proves that the implemented domain contracts and state transitions are correct.
