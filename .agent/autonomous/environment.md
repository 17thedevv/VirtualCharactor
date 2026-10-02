# Hardware & Runtime Environment — VirtualCharacter

## Target Hardware Specifications
- **CPU**: Intel Core i5-12500H (12 cores, 16 threads: 4 P-cores + 8 E-cores)
- **RAM**: 32 GB DDR4/DDR5
- **GPU**: NVIDIA GeForce RTX 3050 Laptop GPU
- **VRAM**: 4.0 GB (4096 MiB)
- **Operating System**: Windows 10/11 (win32)

## Workload Allocation Matrix

| Subsystem | Target Processor | Resource Footprint | Strategy |
|---|---|---|---|
| **Character Core** | CPU | ~50 MB RAM | Pure logic, fast, deterministic |
| **SQLite DB** | CPU (Disk) | ~10 MB RAM | Embedded, WAL mode, ACID |
| **Vector Embedding** | CPU | ~200-300 MB RAM | `fastembed-rs` ONNX Runtime, 0 VRAM |
| **Chat LLM** | GPU | ~2.0 - 2.2 GB VRAM | Ollama `qwen2.5:3b` Q4_K_M |
| **Vision Model** | GPU (On-Demand) | ~2.5 - 3.2 GB VRAM | `qwen2.5vl:3b` or `qwen3-vl:2b` multiplexed |
| **Screen Sensing** | CPU | ~50-80 MB RAM | Win32 API, perceptual hash, 0 VRAM |
| **Audio VAD & STT** | CPU | ~200-400 MB RAM | Silero VAD / whisper.cpp, 0 VRAM |
| **TTS Engine** | CPU | ~150-250 MB RAM | Piper / Edge-TTS / Kokoro, 0 VRAM |
| **Avatar Viewport** | GPU (WebGL) | ~150-250 MB VRAM | Three.js + VRM transparent canvas |

## Multiplexing Safety Policy
`Chat` and `Vision` generative inferences must not run concurrently on GPU to prevent RTX 3050 VRAM overflow. The `ResourceManager` manages the active `ExecutionMode`:
- `ExecutionMode::Chat`: LLM response active
- `ExecutionMode::Vision`: VLM observation active
- `ExecutionMode::Idle`: Baseline state
