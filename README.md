# VirtualCharacter (VC) 🌟
> **Local-First Embodied AI Character & AI VTuber Runtime**
> *Xây dựng trên nền tảng Rust hiệu năng cao, tối ưu hóa cho phần cứng cá nhân (RTX 3050 Laptop 4GB VRAM).*

[![Build & Test](https://img.shields.io/badge/tests-199%20passed-brightgreen.svg)]()
[![Rust Edition](https://img.shields.io/badge/rust-2021%20edition-orange.svg)]()
[![Local First](https://img.shields.io/badge/mode-Local--First%20%2B%20Cloud%20Fallback-blue.svg)]()
[![Hardware Target](https://img.shields.io/badge/GPU-RTX%203050%204GB%20VRAM-red.svg)]()

---

## 📖 Giới thiệu (Overview)

**VirtualCharacter** không đơn thuần là một chatbot bọc giao diện, mà là một **Embodied AI Character Runtime** (Hệ thống điều hành nhân vật ảo có cơ thể & nhận thức) độc lập. Dự án hướng tới trải nghiệm tương tự các AI VTuber / AI Companion thế hệ mới như **Neuro-sama** hay **AIRI**, nhưng sở hữu kiến trúc hoàn chỉnh, chạy **100% Local-First** trên PC cá nhân, không bị phụ thuộc cứng vào bất kỳ dịch vụ Cloud nào.

### Triết lý cốt lõi: *"Character ≠ LLM"*
Hệ thống tuân thủ nguyên tắc phân tách nhận thức nghiêm ngặt:
- **LLM chỉ là bộ phận ngôn ngữ / suy luận (Cognitive Component)**, không phải toàn bộ nhân vật.
- **Bản sắc (Personality), Cảm xúc (Emotion 8 trục), Quan hệ (Relationship đa chiều), Bộ nhớ (4-Tier Memory), và Động cơ (Goals & Decision)** được lưu trữ, tính toán và bảo vệ độc lập bên trong `vc-core` bằng Rust.
- Khi hoán đổi model LLM (từ Ollama Qwen 3B sang Llama 3B, Gemma, hoặc Cloud Gemini), nhân vật vẫn giữ nguyên 100% ký ức, cảm xúc và tính cách riêng biệt.

---

## 🏛️ Kiến trúc tổng thể (Embodied Architecture)

Hệ thống được tổ chức thành 6 trụ cột tương ứng với một thực thể sống:

```
                      ┌─────────────────────────────────────────┐
                      │             WORLD & SENSES              │
                      │  Screen Sensing • Audio In • Livestream │
                      └────────────────────┬────────────────────┘
                                           │
                                           ▼
                      ┌─────────────────────────────────────────┐
                      │            ATTENTION ENGINE             │
                      │   Salience • Interest • Urgency Budget  │
                      └────────────────────┬────────────────────┘
                                           │
                                           ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                   CHARACTER BRAIN                                      │
│                                                                                        │
│   ┌─────────────────────┐      ┌─────────────────────┐      ┌──────────────────────┐   │
│   │     PERSONALITY     │      │   EMOTION (8-Axes)  │      │  RELATIONSHIP ENGINE │   │
│   │  Values, Tendencies │      │ Valence, Arousal... │      │  Trust, Affection... │   │
│   └──────────┬──────────┘      └──────────┬──────────┘      └──────────┬───────────┘   │
│              │                            │                            │               │
│              └────────────────────────────┼────────────────────────────┘               │
│                                           │                                            │
│                                           ▼                                            │
│   ┌────────────────────────────────────────────────────────────────────────────────┐   │
│   │                                4-TIER MEMORY                                   │   │
│   │    Working Memory  •  Episodic Memory  •  Semantic Memory  •  Core Identity    │   │
│   └───────────────────────────────────────┬────────────────────────────────────────┘   │
│                                           │                                            │
│                                           ▼                                            │
│   ┌────────────────────────────────────────────────────────────────────────────────┐   │
│   │                             DECISION ENGINE & TRACE                            │   │
│   │               Candidate Generation • Scoring • Inner Monologue                 │   │
│   └───────────────────────────────────────┬────────────────────────────────────────┘   │
└───────────────────────────────────────────┼────────────────────────────────────────────┘
                                            │
                                            ▼
                      ┌─────────────────────────────────────────┐
                      │              ACTION ROUTER              │
                      │       Speech  •  Tools  •  Avatar       │
                      └───────┬─────────────┬─────────────┬─────┘
                              │             │             │
                              ▼             ▼             ▼
                        ┌───────────┐ ┌───────────┐ ┌───────────┐
                        │   VOICE   │ │ COMPUTER  │ │  AVATAR   │
                        │ Piper/TTS │ │ OS Tools  │ │ Live2D/VRM│
                        └───────────┘ └───────────┘ └───────────┘
```

1. **Brain (Bộ não)**: `vc-core` (Personality, Emotion 8 trục, Relationship, 4-tier Memory, Decision Engine).
2. **Senses (Giác quan)**: Voice Input (Whisper STT), Adaptive Screen Sensing (Desktop Duplication + OCR + VLM), Chat Ingestion (Discord, YouTube, Twitch).
3. **Attention (Sự chú ý)**: Lọc sự kiện, tính toán độ khẩn cấp (Urgency/Salience), ngăn chặn spam LLM và suy luận liên tục gây tràn tài nguyên.
4. **Body (Cơ thể)**: Avatar Controller (Live2D Cubism / 3D VRM) phản hồi theo trạng thái cảm xúc, biểu cảm khuôn mặt, tự động chớp mắt và lip-sync theo âm thanh thời gian thực.
5. **Hands (Bàn tay)**: Sandboxed Computer Interaction (chụp màn hình vùng chọn, mở app, click, gõ phím với hệ thống Permission Policy an toàn).
6. **Social (Xã hội)**: Đọc chat đa nền tảng, nhận diện người quen, Stream Mode dành cho buổi livestream tương tác với khán giả.

---

## ⚡ Chiến lược phần cứng RTX 3050 Laptop (4GB VRAM)

Hệ thống được thiết kế đặc thù cho máy tính cá nhân cấu hình chuẩn:
- **CPU**: Intel Core i5-12500H (12 cores / 16 threads)
- **RAM**: 32 GB DDR4/DDR5
- **GPU**: NVIDIA GeForce RTX 3050 Laptop GPU (4 GB VRAM)
- **OS**: Windows 11

### Bảng phân bổ tài nguyên tối ưu:
| Thành phần | Công nghệ | Thiết bị | Mức chiếm dụng ước tính |
|---|---|---|---|
| **Core Brain, State, Event Bus** | Rust (`vc-core`, `vc-runtime`) | CPU | ~80 MB RAM, <1% CPU |
| **Storage & Semantic Search** | SQLite + `fastembed-rs` (`bge-small`) | CPU | ~300 MB RAM, 0 VRAM |
| **STT (Nghe)** | `whisper.cpp` (Base/Small int8) | CPU (4-6 threads) | ~400 MB RAM, 0 VRAM |
| **TTS (Nói)** | Piper TTS / Kokoro 82M | CPU | ~250 MB RAM, 0 VRAM |
| **Avatar Body** | Live2D WebGL / Three.js VRM | GPU (D3D/OpenGL) | ~300 MB VRAM |
| **Chat LLM (Suy nghĩ/Nói)** | Ollama `qwen2.5:3b-instruct-q4_K_M` | GPU (CUDA) | **~2.2 GB VRAM** |
| **Vision (Nhìn màn hình)** | 3-Level Adaptive (Level 1/2 trên CPU, Level 3 VLM on-demand) | CPU + VLM On-demand | **~0.8 GB VRAM** (hoặc Cloud Fallback) |
| **Tổng cộng VRAM** | | **GPU** | **~3.3 GB / 4.0 GB (An toàn, không tràn VRAM)** |

---

## 📁 Cấu trúc Workspace Crate

```
VirtualCharacter/
├── apps/
│   ├── vc-cli/                  # Giao diện dòng lệnh tương tác trực tiếp
│   ├── vc-server/               # Backend API Server (Axum, REST, WebSocket Streaming)
│   └── vc-web/                  # Frontend Web HUD & Avatar Viewer
├── crates/
│   ├── vc-core/                 # Domain logic độc lập: Personality, State, Memory, Decision
│   ├── vc-runtime/              # Orchestration, SessionManager, Lifecycle 9 bước, Resource Manager
│   ├── vc-llm/                  # Provider abstraction: Ollama, Gemini, Mock
│   └── vc-storage/              # Repository: InMemoryStorage, SqliteStorage (bền vững)
├── docs/                        # Toàn bộ tài liệu kiến trúc, đặc tả & lộ trình phát triển
│   ├── vision.md                # Bản thiết kế kiến trúc toàn diện 32 phần
│   ├── checklist.md             # Master Checklist tiến độ 10 Phase
│   ├── plan.md                  # Master Plan chi tiết
│   ├── tasks.md                 # Phân chia nhiệm vụ & Sprint backlog
│   ├── architecture.md          # Đặc tả kiến trúc tầng & ranh giới crate
│   ├── TechStack.md             # Toàn cảnh công nghệ sử dụng
│   ├── development.md           # Hướng dẫn thiết lập môi trường lập trình
│   └── contracts.md             # Hợp đồng giao tiếp giữa các module
└── data/                        # Thư mục chứa SQLite database, models và bộ nhớ cục bộ
```

---

## 🚀 Trạng thái hiện tại & Bắt đầu nhanh (Getting Started)

### Trạng thái dự án:
- ✅ **Phase 0 (Foundation)**: Hoàn thành 100%. Workspace chuẩn Rust, ranh giới crate nghiêm ngặt.
- ✅ **Phase 1 (Cognitive Core & Storage)**: Hoàn thành 100%. **114 automated tests pass**. Cảm xúc 8 trục, Memory 4 tầng, Quan hệ đa người dùng, SQLite bền vững, WebSocket streaming.
- 🔄 **Phase 2 (Local AI & Memory 2.0)**: Đang khởi động (Ollama integration, ModelRegistry, Semantic Search cục bộ).

### Yêu cầu tiên quyết:
- [Rust 1.75+](https://rustup.rs/)
- [Ollama](https://ollama.com/) (Dành cho Local LLM)
  ```bash
  ollama pull qwen2.5:3b-instruct-q4_K_M
  ```

### Chạy kiểm thử tự động (Workspace Tests):
```bash
cargo test --workspace
```

### Chạy thử nghiệm qua CLI:
```bash
cargo run -p vc-cli
```

### Khởi động Server & Web HUD:
```bash
# Khởi động Backend API & WebSocket Server
cargo run -p vc-server

# Mở trình duyệt truy cập Web UI tại:
# http://localhost:3000
```

---

## 📚 Tài liệu tham khảo quan trọng
- 📑 [docs/vision.md](file:///d:/VirtualCharactor/docs/vision.md): Bản thiết kế đại cương dài hạn (AI VTuber, Adaptive Vision, Resource Manager).
- 📑 [docs/checklist.md](file:///d:/VirtualCharactor/docs/checklist.md): Danh sách kiểm tra tiến độ 10 Phase và tiêu chuẩn nghiệm thu.
- 📑 [docs/plan.md](file:///d:/VirtualCharactor/docs/plan.md): Kế hoạch phát triển chi tiết từng giai đoạn.
- 📑 [docs/architecture.md](file:///d:/VirtualCharactor/docs/architecture.md): Ranh giới kiến trúc và luồng dữ liệu.

---

## 📜 Giấy phép (License)
Dự án được phát hành theo giấy phép [MIT](LICENSE.md).