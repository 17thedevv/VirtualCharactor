# Acceptance Criteria Framework — VirtualCharacter

A milestone or feature is ONLY marked `[x] DONE` when ALL of the following criteria are fulfilled:

1. **Compilation**:
   - `cargo check --workspace` finishes with 0 errors and 0 unhandled warnings.
   - Frontend `npm run build` completes with exit code 0.
2. **Test Coverage**:
   - Unit tests written for all new logic, state transitions, validation, and calculations.
   - Component / Mock tests verifying provider adapters offline and deterministically.
   - Integration tests verifying cross-crate orchestrations.
3. **Regression Proof**:
   - `cargo test --workspace` passes 100% of existing tests with no regressions.
4. **Contract Fidelity**:
   - Contracts in `docs/contracts.md` are respected.
   - Core domain types remain isolated.
5. **Hardware Safety**:
   - Memory usage, VRAM usage, and compute load stay within the RTX 3050 (4GB) profile.
6. **Documentation & Traceability**:
   - `docs/tasks.md` and `docs/checklist.md` updated with accurate progress.
   - Model and dependency changes documented with version, license, and size in `docs/TechStack.md`.
