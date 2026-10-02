# AUTONOMOUS LONG-RUN DEVELOPMENT & CONTINUOUS VALIDATION PROTOCOL
# Project: VirtualCharacter
# Target: Local-First AI VTuber / Virtual Character Runtime
# Environment: Windows
# Hardware: i5-12500H / 32GB RAM / RTX 3050 Laptop 4GB VRAM

## Protocol Purpose
Continuous development, verification, regression testing, and architecture governance loop.
No feature is considered "DONE" solely because code compiles.

```
    IMPLEMENT
        ↓
    BUILD
        ↓
    UNIT TEST
        ↓
    INTEGRATION TEST
        ↓
    REALISTIC SMOKE TEST
        ↓
    REGRESSION TEST
        ↓
    VERIFY ARCHITECTURE
        ↓
    UPDATE DOCUMENTATION
        ↓
    UPDATE CHECKLIST
        ↓
    NEXT TASK
        ↓
    REPEAT
```

## Hardware & Resource Policy
- CPU: Intel i5-12500H
- RAM: 32GB
- GPU: RTX 3050 Laptop (4GB VRAM)
- OS: Windows
- CPU Tasks: SQLite, Memory, Attention, Event processing, Whisper/Piper/Kokoro (lightweight)
- GPU Tasks: Chat LLM (e.g. qwen2.5:3b), Vision model (e.g. qwen3-vl:2b or lightweight VLM)
- Multiplexing: Do NOT run multiple heavy GPU models concurrently. Offload or multiplex where appropriate.
- Model downloads: Lazy evaluation. Only download models required for the *current* phase/task after checking disk & VRAM feasibility.

## Testing Pyramid
1. Level 1 - Unit Test: Logic, parsing, validation, serialization
2. Level 2 - Component Test: OllamaProvider, Memory, AttentionEngine, VisionRouter, BodyController
3. Level 3 - Integration Test: vc-core ↔ vc-runtime ↔ vc-llm ↔ providers
4. Level 4 - Realistic Smoke Test: End-to-end user message → LLM → Emotion → Voice/Avatar
5. Level 5 - Regression Test: `cargo test --workspace` across all crates

## Architecture Invariants
1. Character Core does NOT depend on Ollama, Gemini, Vision models, or Avatar renderers.
2. Runtime orchestrates all subsystems.
3. Provider layer contains adapters/backends.
4. Vision model only provides observations; does not decide actions.
5. Computer actions require: Decision Engine → ToolPermissionPolicy → Action → Screen Verification.
6. Public contracts in `docs/contracts.md` are sacred.
