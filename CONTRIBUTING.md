# Contributing to VirtualCharacter

Thank you for your interest in contributing to VirtualCharacter!

## Basic Rules
1. **No Web/API dependencies in Phase 0/1**: Ensure the core logic remains independent of any delivery mechanism.
2. **Respect the Boundaries**: `vc-core` must remain independent of infrastructure crates like `vc-llm` and `vc-storage`.
3. **Tests**: New features should include unit tests and integrate with the CLI test harness.
4. **Code Quality**: Follow standard Rust idioms. Run `cargo clippy` and `cargo fmt` before submitting PRs.
5. **Coordination**: Check `docs/development.md` for our two-developer ownership model before modifying stable interfaces.
