# Testing Protocol — VirtualCharacter

## Testing Pyramid

```
           ▲
          / \     Level 5: Full Workspace Regression (`cargo test --workspace`)
         /   \    Level 4: Realistic Smoke Tests & E2E Verification
        /     \   Level 3: Cross-Crate Integration Tests (`tests/integration.rs`)
       /       \  Level 2: Subsystem & Provider Component Tests
      /_________\ Level 1: Pure Logic Unit Tests (Fast, deterministic)
```

## Level Guidelines
- **Level 1 (Unit)**: Parsing, math, scoring, enum conversions, state mutations, clamping.
- **Level 2 (Component)**:
  - LLM: Mock provider, response mapping, error handling, rate limiting.
  - Storage: In-memory store, SQLite persistence, Vector cosine similarity.
  - Vision: Sensing diff detection, Attention threshold, VisionRouter fallback.
  - Audio: VAD threshold, viseme generation, emotion modulation.
- **Level 3 (Integration)**: Full 9-stage runtime lifecycle (`vc-core` ↔ `vc-runtime` ↔ `vc-storage` ↔ `vc-llm`).
- **Level 4 (Smoke)**: Real interactions with actual local services (Ollama, SQLite, Audio, Web UI).
- **Level 5 (Regression)**: Every change must run `cargo test --workspace` and verify test count does not decrease.
