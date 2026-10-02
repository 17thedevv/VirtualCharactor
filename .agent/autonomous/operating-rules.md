# Autonomous Operating Rules — VirtualCharacter

## Core Principles
1. **Loop Discipline**: Every task executes:
   `ANALYZE` → `PLAN` → `IMPLEMENT` → `BUILD` → `UNIT TEST` → `COMPONENT TEST` → `INTEGRATION TEST` → `SMOKE TEST` → `REGRESSION TEST` → `DOCUMENT` → `CHECKLIST UPDATE` → `NEXT TASK`.
2. **Never Break Working Features**:
   - Baseline test before editing.
   - Full regression (`cargo test --workspace`) after editing.
   - If test count decreases or tests fail, STOP and fix regression immediately.
   - Never delete or weaken tests to make them pass.
3. **Architecture Invariant**:
   - `vc-core` contains pure domain entities and business rules. It MUST NOT depend on Ollama, Gemini, Vision models, TTS/STT engines, Avatar renderers, Discord, YouTube, or Win32 APIs.
   - Runtime orchestrates domain models and providers.
   - Providers adapt external APIs/transports to domain traits.
4. **Hardware Policy (i5-12500H / 32GB RAM / RTX 3050 Laptop 4GB VRAM)**:
   - Prefer CPU for non-generative tasks: SQLite, Memory vector cosine, Attention scoring, Window sensing, Audio VAD.
   - GPU for Generative AI: Chat LLM (`qwen2.5:3b`) and Vision VLM (`qwen2.5vl:3b` on-demand).
   - Enforce GPU multiplexing: Never run multiple heavy GPU models concurrently.
5. **No Premature Resource Downloading**:
   - Check if current task actually requires an asset/model before downloading.
   - Verify license and file size.
