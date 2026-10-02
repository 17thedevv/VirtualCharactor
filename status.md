# BÁO CÁO TRẠNG THÁI DỰ ÁN (PROJECT STATUS)

> **Cập nhật gần nhất:** Tháng 10/2026  
> **Trạng thái tổng thể:** **BƯỚC 1.8 (KIẾN TRÚC RAG DATABASE & RETRIEVAL) HOÀN TẤT (100% HEALTHY)**  
> **Tổng số bài kiểm thử:** **255 / 255 PASSED** (0 failed, 0 ignored)  
> **Kiến trúc:** Local-First, Zero-Cloud dependency, Clean Architecture (Rust Core + Hybrid RAG + Axum + React 18 VRM).

---

## 1. Tổng quan Dự án (Project Overview)

**VirtualCharacter** là hệ thống nhân vật ảo 3D AI (Virtual Companion / VTuber) vận hành độc lập trên máy cục bộ với độ trễ thấp, mô phỏng đầy đủ nhận thức, cảm xúc, quan hệ, trí nhớ và khả năng tương tác bằng giọng nói & thị giác thời gian thực.

---

## 2. Kết quả Kiểm thử & Đảm bảo Chất lượng (Quality Assurance)

Tất cả các crate trong workspace Rust đều được biên dịch và kiểm thử tự động nghiêm ngặt:

| Crate / Module | Số lượng Test | Kết quả | Mô tả |
| :--- | :---: | :---: | :--- |
| **`crates/vc-core`** | 77 tests | ✅ PASSED | Domain Core: Personality, Emotion, State, Memory, Context, Decision & RAG Domain Types |
| **`crates/vc-llm`** | 27 tests | ✅ PASSED | Ollama Client, Embedding Provider, Real Streaming E2E, Gemini API, Mock Provider |
| **`crates/vc-runtime`** | 140 tests | ✅ PASSED | Conversation FSM, Audio Chunker, SAPI TTS, Whisper STT, Vision, Attention & RAG Indexer/Archiver |
| **`crates/vc-storage`** | 11 tests | ✅ PASSED | SQLite Persistence, Semantic Vector Search & Hybrid RAG (Dense Cosine + FTS5 Sparse Keywords) |
| **`apps/vc-server`** | 0 tests | ✅ OK | Axum WebSocket Gateway, RAG Knowledge Injection & Realtime Streaming Pipeline |
| **`apps/vc-cli`** | 0 tests | ✅ OK | Native Windows CLI tool |
| **TỔNG CỘNG** | **255 tests** | **100% PASS** | **Không có lỗi hồi quy (No regression)** |

- `cargo check --workspace`: **0 error, 0 warning**.
- `cargo fmt --all -- --check`: **100% tuân thủ chuẩn formatting**.

---

## 3. Các Thành phần Đã Triển khai (Implemented Modules)

### 3.1. 🧠 Bộ não AI (LLM & Context Engineering)
- **Local Ollama Integration:**
  - Model mặc định: `qwen2.5:3b` (hỗ trợ thêm `qwen3:8b`, `qwen2.5vl:3b`, `dolphin-llama3`).
  - **Real Token Streaming:** Nhận luồng HTTP chunked JSON từ Ollama `/api/chat` và phát trực tiếp đến WebSocket client, **không dùng fake delay/sleep**.
  - Tự động giải phóng VRAM (`unload_model` qua `keep_alive: 0`) khi chuyển đổi giữa các model nặng.
- **Cloud Fallback:** Tích hợp Google Gemini (`gemini-1.5-flash`) khi có `GEMINI_API_KEY`.
- **Context Engineering:** Xây dựng prompt có ngân sách token nghiêm ngặt, ưu tiên thông tin thiết yếu (System directive, Persona, Ký ức liên quan, Trạng thái cảm xúc).

### 3.2. 🗣️ Giọng nói & Âm thanh (Audio Pipeline)
- **Text-to-Speech (TTS):**
  - **Windows SAPI / OneCore TTS:** Chạy 100% offline nội bộ Windows, độ trễ ~20ms, 0 tốn VRAM.
  - **Yae Miko RVC Voice Daemon:** Server Python (Port 5005) chuyển đổi giọng Anime chất lượng cao kèm bộ nhớ đệm âm thanh tức thì.
  - **TextStreamChunker:** Cắt câu tự động theo dấu câu ngay khi Ollama đang sinh token đầu tiên.
  - **OrderedTtsQueue:** Tổng hợp âm thanh song song và xếp hàng phát tuần tự.
  - **RMS 20ms Volume Slices:** Tính toán độ mở miệng theo âm lượng để gửi lên Avatar làm Lip-sync chuẩn xác.
- **Speech-to-Text (STT):**
  - **WASAPI / CPAL Input:** Thu âm trực tiếp từ microphone máy tính, tự động kích hoạt khi có tiếng nói (VAD).
  - **Whisper CPU:** Nhận diện và chuyển giọng nói thành văn bản offline qua Whisper C++.

### 3.3. 🎭 Tâm lý, Trạng thái & Quyết định (Character Core)
- **Rust Conversation State Machine:**
  - Quản lý nghiêm ngặt các trạng thái: `IDLE` ➔ `LISTENING` ➔ `THINKING` ➔ `SPEAKING` ➔ `IDLE`.
  - **Barge-In Interruption:** Khi nhân vật đang nói mà người dùng cất giọng, hệ thống ngắt lời ngay lập tức, tính toán chính xác phần văn bản người dùng đã nghe (`heard_text`) và loại bỏ phần chưa nói khỏi lịch sử hội thoại để tránh làm bẩn ngữ cảnh.
- **Emotion Engine:** Mô hình 8 chiều cảm xúc (Joy, Sadness, Anger, Fear, Surprise, Affection, Embarrassment, Curiosity) với cơ chế tự phân rã (decay) tự nhiên theo thời gian.
- **Relationship System:** Theo dõi quan hệ động (Closeness, Trust, Familiarity, Affection) và tự động ghi nhớ các sự kiện do người dùng chia sẻ.
- **Decision Engine:** Tách rời khâu ra quyết định hành vi khỏi LLM generation (theo clean architecture).

### 3.4. 👁️ Thị giác & Giám sát Màn hình (Vision & Perception)
- **Win32 Screen Capture:** Chụp màn hình Desktop native với `BitBlt` cực nhanh.
- **Frame Sensing:** So sánh tỉ lệ thay đổi khung hình (`diff %`).
- **Temporal Attention Gate:** Chỉ kích hoạt VLM (`qwen2.5vl:3b`) khi màn hình có biến đổi đáng kể hoặc người dùng trực tiếp yêu cầu xem màn hình, tránh lãng phí GPU.

### 3.5. 💃 Giao diện Người dùng (Frontend Web)
- **React 18 + TypeScript + Vite (`apps/vc-web`):**
  - **3D VRM Avatar (`three.js` + `@pixiv/three-vrm`):** Hiển thị mô hình 3D VRM, đồng bộ chớp mắt tự động, nhịp thở vô thức và mấp máy môi theo luồng audio WebSocket.
  - **Dialogue Stream:** Khung chat hiển thị token stream mượt mà thời gian thực.
  - **Mind Inspector Drawer:** Bảng điều khiển trực quan theo dõi tâm trí nhân vật (Cảm xúc hiện tại, Quan hệ, Trí nhớ, Cây quyết định).

### 3.6. 📚 Hệ thống RAG & Cơ sở Dữ liệu Tri thức (Hybrid RAG Knowledge Engine - Bước 1.8)
- **Kiến trúc Domain RAG thuần túy (`crates/vc-core/src/rag`):**
  - Các cấu trúc dữ liệu cốt lõi: `DocumentChunk`, `EmbeddingVector`, `RagQuery`, `RagQueryResult`.
  - Thuật toán chấm điểm tổng hợp đa yếu tố: `calculate_composite_score` (kết hợp Dense Similarity, Recency Decay, Importance Weight).
  - Thuật toán trộn xếp hạng Reciprocal Rank Fusion (RRF) dung hòa kết quả tìm kiếm ngữ nghĩa và từ khóa.
- **Local Embedding Provider (`crates/vc-llm`):**
  - Tích hợp Ollama Embedding API (`nomic-embed-text` / `bge-m3`).
  - Hỗ trợ cơ chế Deterministic Fallback Vector đảm bảo hệ thống không bao giờ crash nếu Ollama chưa bật cờ embedding.
- **SQLite Hybrid Storage & Bảo mật Cách ly Danh tính (`crates/vc-storage`):**
  - Bảng `rag_chunks` lưu trữ vector nhị phân IEEE-754 Little-Endian `BLOB` tốc độ cao.
  - Bảng ảo `rag_chunks_fts` (SQLite FTS5) hỗ trợ BM25 full-text keyword search.
  - Cơ chế **Actor Privacy Isolation**: dữ liệu người dùng A tuyệt đối không rò rỉ sang phiên của người dùng B (chỉ truy xuất dữ liệu riêng của Actor + Tri thức chung).
- **Tự động Chunking & Đóng băng Hội thoại (`crates/vc-runtime/src/rag`):**
  - `SemanticTextChunker`: Phân đoạn tài liệu Markdown, văn bản theo đoạn và câu kèm overlap.
  - `LoreIndexer`: Tự động quét và lập chỉ mục thư mục `data/knowledge/` (ví dụ `aria_lore.md`) ngay khi server khởi động.
  - `ConversationArchiver`: Tự động nhóm các lượt thoại vừa diễn ra thành các episode chunk và nhúng vector vào kho lưu trữ RAG dài hạn.


---

## 4. Danh mục Cổng & Dịch vụ Mạng (Ports & Network)

| Dịch vụ | Cổng | Giao thức | Mô tả |
| :--- | :---: | :---: | :--- |
| **Ollama Local AI** | `11434` | HTTP / SSE | Cung cấp model LLM và VLM cục bộ |
| **Backend Gateway** | `3000` | HTTP / WebSocket | `vc-server` (Rust Axum gateway xử lý logic) |
| **Voice Daemon** | `5005` | HTTP REST | Yae Miko RVC Voice Engine (Python FastAPI/Flask) |
| **Frontend Web** | `5173` | HTTP | Giao diện Web Client (Vite Dev Server) |

---

## 5. Hướng dẫn Khởi động Nhanh (Quick Start)

Dự án cung cấp các file batch script tự động:

- **Khởi động toàn bộ hệ thống:**
  ```cmd
  run.bat
  ```
  *(Script sẽ tự động dọn dẹp tiến trình cũ, kiểm tra Ollama, bật Voice Engine 5005, bật Backend 3000, bật Frontend 5173 và tự động mở trình duyệt).*

- **Dừng toàn bộ hệ thống:**
  ```cmd
  stop.bat
  ```

---

## 6. Kế hoạch Tiếp theo (Next Steps / Roadmap)

1. **Phase 2B — Autonomous Life & Attention Pacing:**
   - Hoàn thiện nhịp thở vô thức (Circadian Heartbeat) và cơ chế tự động bắt chuyện khi phát hiện người dùng im lặng hoặc có sự kiện đặc biệt trên màn hình.
2. **Phase 3 — Platform Integrations:**
   - Bộ kết nối trực tiếp đến Discord Bot, Twitch Chat, YouTube Superchat phục vụ mục đích trợ lý cá nhân và AI VTuber livestream.
