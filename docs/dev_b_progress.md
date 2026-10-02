# Nhật Ký & Tiến Độ Phát Triển — Developer B
> **Vai trò**: Runtime Orchestration, Infrastructure, Context Engine, LLM Layer, Storage & Persistence  
> **Tài liệu tham chiếu**: [docs/tasks.md](file:///d:/VirtualCharactor/docs/tasks.md), [docs/contracts.md](file:///d:/VirtualCharactor/docs/contracts.md), [docs/design/storage.md](file:///d:/VirtualCharactor/docs/design/storage.md)  
> **Trạng thái cập nhật**: Liên tục (Living Document) — Cập nhật lần cuối: 2026-09-18

---

## I. Tổng Quan Tiến Độ (Progress Dashboard)

| Task Code | Hạng mục | Trạng thái | Mô tả cốt lõi |
| :--- | :--- | :---: | :--- |
| **B1** | **Context Subsystem** | `[DONE]` | Gom nhặt ngữ cảnh từ 5 nguồn, quản trị ngân sách token, cắt tỉa ưu tiên |
| **B2** | **Runtime Orchestrator** | `[DONE]` | Vòng đời tương tác 9 giai đoạn, Session State Machine, State Feedback Loop |
| **B3** | **LLM Provider Layer** | `[DONE]` | Abstraction provider, Gemini REST API, Mock Provider có kịch bản & che giấu secret |
| **B4** | **Storage & Persistence**| `[DONE]` | 4 Repository traits, In-Memory Repository, SQLite Local-First tự động migrate |
| **B5** | **E2E Integration** | `[DONE]` | Cắm SQLite vào `vc-server` & `vc-cli`, kiểm thử dữ liệu sống sót qua restart |

---

## II. Chi Tiết Từng Hạng Mục Dev B Đã Thực Hiện

### 1. Task B1: Context Subsystem & Token Governance (`crates/vc-core/src/context`)
* **Mục tiêu**: Xây dựng cơ chế cấu trúc hóa ngữ cảnh trước khi nạp vào LLM, ngăn chặn việc nhồi nhét thô chat history và kiểm soát chặt chẽ ngân sách token (Context Budget).
* **Nội dung kỹ thuật đã hoàn thành**:
  - **`ContextItem` & `ContextSource`**: Định nghĩa nguồn gốc mục ngữ cảnh (`System`, `User`, `Personality`, `State`, `Relationship`, `Memory`, `Environment`).
  - **`ContextPriority`**: 4 bậc ưu tiên rõ ràng (`Critical > High > Medium > Low`).
  - **`ContextBudget`**: Bộ đếm ước lượng token dựa trên heuristic ký tự/từ (an toàn cho tiếng Việt và tiếng Anh), thiết lập trần tối đa.
  - **`ContextBuilder` & `ContextPrioritizer`**: Gom dữ liệu từ các domain của Dev A, sắp xếp theo độ ưu tiên và tự động cắt bỏ các mục có độ ưu tiên thấp nếu vượt quá ngân sách cho phép, bảo vệ nghiêm ngặt các mục `Critical` (chỉ thị hệ thống, tin nhắn user).
  - **`ContextBreakdown`**: Thống kê số lượng token sử dụng cho từng thành phần (phục vụ hiển thị Live Token Gauge trên Web UI).

### 2. Task B2: Runtime Orchestration & Session Management (`crates/vc-runtime`)
* **Mục tiêu**: Điều phối toàn bộ vòng đời tương tác của nhân vật ảo, kết nối dữ liệu domain của Dev A với hạ tầng của Dev B.
* **Nội dung kỹ thuật đã hoàn thành**:
  - **Interaction Lifecycle (9 giai đoạn)**:
    1. Tiếp nhận input & Resolve/Touch Session.
    2. Truy vấn ký ức liên quan có cô lập Actor (`MemoryQuery`).
    3. Xây dựng ngữ cảnh và kiểm soát ngân sách token (`ContextBuilder`).
    4. Đánh giá quyết định hành động (`DecisionEngine`).
    5. Sinh phản hồi ngôn ngữ (`LlmProvider`).
    6. Cập nhật biến thiên cảm xúc & hành vi (`apply_delta`, `decay`, `sync_behavior`).
    7. Tiến hóa quan hệ theo tương tác (`RelationshipTransition`).
    8. Củng cố ký ức mới (Episodic Memory Formation).
    9. Đóng gói kết quả đầu ra (`InteractionOutcome`).
  - **Session State Machine (`SessionManager`)**:
    - Quản lý các trạng thái: `Created` $\rightarrow$ `Active` $\rightarrow$ `Idle` $\rightarrow$ `Completed` / `Expired` / `Cancelled`.
    - Cơ chế phát hiện idle timeout theo mốc thời gian thực, đảm bảo luồng đồng thời an toàn (Thread-safe concurrency).

### 3. Task B3: LLM Provider Layer & Error Resiliency (`crates/vc-llm`)
* **Mục tiêu**: Tách biệt hoàn toàn tầng gọi mô hình ngôn ngữ khỏi domain core, chống phụ thuộc vào bất kỳ SDK độc quyền nào.
* **Nội dung kỹ thuật đã hoàn thành**:
  - **Provider Abstraction**:
    - Trait `LlmProvider` thuần túy với các kiểu dữ liệu trung lập: `LlmRequest`, `LlmResponse`, `LlmUsage`, `LlmError`.
  - **`MockLlmProvider`**:
    - Hoàn chỉnh cho kiểm thử tự động không cần kết nối mạng hay API key.
    - Hỗ trợ hàng đợi câu trả lời kịch bản sẵn (`with_responses`, `push_canned_response`).
    - Mô phỏng lỗi provider (`failing`, `set_simulated_error`) để test độ kiên cường của Runtime khi gặp sự cố mạng/hạn ngạch.
    - Ghi nhận lịch sử gọi (`recorded_requests`, `last_request`) phục vụ assertion.
  - **`GeminiProvider` (Google Gemini REST API)**:
    - Gọi API trực tiếp qua `ureq`/REST payload.
    - Quản lý cấu hình `GeminiConfig` (model, temperature, max_output_tokens, timeout, retry).
    - **Bảo mật bí mật (Skill 20 / Skill 25)**: Tự động che dấu API key trong format `Debug` (`AIza...[REDACTED]`), không rò rỉ secret ra log.
    - Cơ chế tự động thử lại (retry with exponential backoff) khi gặp lỗi tạm thời (429 Rate Limit, 503 Unavailable).
    - Toàn bộ DTO của Google Gemini được giấu kín 100% bên trong `vc_llm::gemini`.

### 4. Task B4: Storage Subsystem (Local-First Persistence) (`crates/vc-storage`)
* **Mục tiêu**: Xây dựng tầng lưu trữ bền vững tuân thủ triệt để [docs/design/storage.md](file:///d:/VirtualCharactor/docs/design/storage.md), tách rời Domain Object khỏi Persistence Model.
* **Nội dung kỹ thuật đã hoàn thành**:
  - **Bổ sung Core Error**: Thêm `StorageError(String)` vào `vc_core::error::CoreError` để phân loại chính xác lỗi I/O/database.
  - **4 Repository Traits (`Send + Sync`)**:
    - `CharacterRepository`: CRUD Character & lưu trữ cấu hình Personality.
    - `StateRepository`: CRUD trạng thái cảm xúc, nhận thức, hành vi động.
    - `RelationshipRepository`: CRUD quan hệ độc lập giữa Character và từng Actor.
    - `MemoryRepository`: CRUD và truy vấn ký ức với bộ lọc đa chiều & bảo vệ quyền riêng tư actor.
  - **`InMemoryStorage`**:
    - Lưu trữ bằng `Arc<RwLock<HashMap<...>>>`, hỗ trợ test tốc độ cao trong RAM.
  - **`SqliteStorage` (Local-First Database)**:
    - Tích hợp `rusqlite` với cờ `bundled` (tự biên dịch mã nguồn C của SQLite, chạy độc lập trên Windows mà không cần cài đặt thêm phần mềm).
    - Bật chế độ `WAL (Write-Ahead Logging)` và `foreign_keys = ON`.
    - Tự động tạo bảng & indexes khi file database chưa tồn tại:
      - Bảng `characters (id, name, personality_json, state_json, created_at, updated_at)`
      - Bảng `relationships (character_id, actor_id, stage, closeness, trust, familiarity, affection, tension, state_json, updated_at)`
      - Bảng `memories (id, character_id, actor_id, memory_type, importance, content, memory_json, created_at, updated_at)`
    - Đồng bộ ghi an toàn luồng thông qua internal mutex bảo vệ connection.
    - Chuyển đổi hai chiều mượt mà giữa Domain Object và Persistence Model với JSON columns cho các cấu trúc linh hoạt.

### 5. Task B5: Integration & Applications (`apps/vc-server`, `apps/vc-cli`)
* **Mục tiêu**: Ghép nối hạ tầng lưu trữ vào toàn bộ ứng dụng, đảm bảo tính bền vững (Survives Restarts).
* **Nội dung kỹ thuật đã hoàn thành**:
  - **`apps/vc-server`**:
    - Cắm `SqliteStorage` vào `AppState`, đường dẫn mặc định `./data/virtual_character.db` (có thể ghi đè bằng env `VC_DB_PATH`).
    - Lúc khởi động: Tự động phục hồi thực thể nhân vật, cảm xúc, quan hệ và ký ức từ file SQLite; nếu database rỗng sẽ tự động seed nhân vật mặc định Aria.
    - Trong WebSocket tương tác (`ws.rs`): Cứ sau mỗi lượt đối thoại, tự động commit `CharacterState` mới, điểm `Relationship` mới và `Memory` mới vào SQLite.
    - Trong REST API (`routes.rs`): Khi chỉnh sửa tính cách qua endpoint `/api/personality`, cập nhật trực tiếp vào SQLite.
    - Trong Reset (`routes.rs` & `state.rs`): Xóa ký ức cũ và nạp lại baseline mặc định trong SQLite.
  - **`apps/vc-cli`**:
    - Nâng cấp CLI kết nối với file SQLite cục bộ.
    - Tự động nhận diện companion đã lưu, tiếp tục phát triển mối quan hệ qua nhiều phiên chạy CLI khác nhau.
  - **`.gitignore`**: Thêm `data/`, `*.db`, `*.db-wal`, `*.db-shm` để bảo vệ dữ liệu người dùng cục bộ, tránh commit nhầm lên git.

---

## III. Danh Sách Toàn Bộ Các File Dev B Đã Tạo / Đụng Vào

Bảng dưới đây liệt kê chi tiết mọi file trong repository mà Dev B sở hữu toàn quyền hoặc đã chỉnh sửa phục vụ hạ tầng:

| Đường dẫn file | Crate / App | Loại tác vụ | Vai trò & Mục đích |
| :--- | :--- | :---: | :--- |
| **`crates/vc-core/src/error.rs`** | `vc-core` | Chỉnh sửa | Bổ sung biến thể `StorageError(String)` |
| **`crates/vc-core/src/context/mod.rs`** | `vc-core` | Tạo mới/Sở hữu | Module gốc Context, re-export item, builder, budget, prioritizer |
| **`crates/vc-core/src/context/item.rs`** | `vc-core` | Tạo mới/Sở hữu | Định nghĩa `ContextItem`, `ContextSource`, `ContextPriority` |
| **`crates/vc-core/src/context/budget.rs`** | `vc-core` | Tạo mới/Sở hữu | Quản trị ngân sách token `ContextBudget` |
| **`crates/vc-core/src/context/builder.rs`** | `vc-core` | Tạo mới/Sở hữu | Xây dựng pipeline gom ngữ cảnh đa nguồn |
| **`crates/vc-core/src/context/prioritizer.rs`** | `vc-core` | Tạo mới/Sở hữu | Thuật toán cắt tỉa ưu tiên token dưới trần ngân sách |
| **`crates/vc-storage/Cargo.toml`** | `vc-storage` | Chỉnh sửa | Khai báo `rusqlite`, `serde`, `serde_json`, `uuid`, `parking_lot` |
| **`crates/vc-storage/src/lib.rs`** | `vc-storage` | Tạo mới/Sở hữu | Re-export `in_memory`, `sqlite`, và các repository traits |
| **`crates/vc-storage/src/repository.rs`** | `vc-storage` | Tạo mới/Sở hữu | 4 Repository interface chuẩn: Character, State, Relationship, Memory |
| **`crates/vc-storage/src/in_memory.rs`** | `vc-storage` | Tạo mới/Sở hữu | Triển khai bộ nhớ RAM in-memory phục vụ unit test |
| **`crates/vc-storage/src/sqlite.rs`** | `vc-storage` | Tạo mới/Sở hữu | Triển khai SQLite Local-First, auto schema migration, mutex locking |
| **`crates/vc-storage/tests/storage_tests.rs`** | `vc-storage` | Tạo mới/Sở hữu | Bộ test kiểm thử in-memory, SQLite memory và SQLite persistence qua restart |
| **`crates/vc-runtime/src/lib.rs`** | `vc-runtime` | Chỉnh sửa | Khai báo các module runtime, session, interaction |
| **`crates/vc-runtime/src/runtime.rs`** | `vc-runtime` | Chỉnh sửa/Sở hữu | `RuntimeEngine`, điều phối tương tác 9 giai đoạn |
| **`crates/vc-runtime/src/session.rs`** | `vc-runtime` | Tạo mới/Sở hữu | `SessionManager`, `CharacterSession`, phát hiện timeout |
| **`crates/vc-runtime/src/interaction.rs`** | `vc-runtime` | Chỉnh sửa/Sở hữu | `InteractionOutcome`, thống kê token, kết quả tương tác |
| **`crates/vc-runtime/tests/integration.rs`** | `vc-runtime` | Chỉnh sửa/Sở hữu | Bộ kiểm thử tích hợp E2E toàn diện của runtime |
| **`crates/vc-llm/Cargo.toml`** | `vc-llm` | Chỉnh sửa | Khai báo dependencies cho HTTP REST client và serialization |
| **`crates/vc-llm/src/lib.rs`** | `vc-llm` | Chỉnh sửa/Sở hữu | Cấu trúc module LLM provider |
| **`crates/vc-llm/src/provider.rs`** | `vc-llm` | Tạo mới/Sở hữu | Trait `LlmProvider`, builder `LlmRequest`, `LlmResponse`, `LlmError` |
| **`crates/vc-llm/src/mock.rs`** | `vc-llm` | Tạo mới/Sở hữu | `MockLlmProvider` có kịch bản chuỗi, mô phỏng lỗi và ghi log request |
| **`crates/vc-llm/src/gemini.rs`** | `vc-llm` | Tạo mới/Sở hữu | `GeminiProvider` gọi Google Gemini REST API, che giấu API key, retry |
| **`apps/vc-server/Cargo.toml`** | `vc-server` | Chỉnh sửa | Cấu hình dependencies Axum, Tokio, Tower, vc-storage |
| **`apps/vc-server/src/main.rs`** | `vc-server` | Chỉnh sửa/Sở hữu | Điểm khởi chạy HTTP/WebSocket gateway server |
| **`apps/vc-server/src/state.rs`** | `vc-server` | Chỉnh sửa/Sở hữu | `AppState`, cắm `SqliteStorage`, seed dữ liệu Aria ban đầu |
| **`apps/vc-server/src/ws.rs`** | `vc-server` | Chỉnh sửa/Sở hữu | WebSocket streaming handler, commit state/memory/relationship xuống DB |
| **`apps/vc-server/src/routes.rs`** | `vc-server` | Chỉnh sửa/Sở hữu | REST handler, lưu personality vào storage khi cập nhật qua API |
| **`apps/vc-cli/src/main.rs`** | `vc-cli` | Chỉnh sửa/Sở hữu | Giao diện dòng lệnh CLI tương tác bền vững với SQLite |
| **`apps/vc-web/`** | `vc-web` | Hạ tầng Web | Giao diện React Mind Inspector, hiển thị token, cảm xúc, ký ức trực tiếp |
| **`.gitignore`** | Root | Chỉnh sửa | Bổ sung bỏ qua thư mục `data/`, file SQLite `*.db*` |
| **`docs/tasks.md`** | Root | Chỉnh sửa | Đồng bộ đánh dấu các task `B1`, `B2`, `B3`, `B4`, `B5` thành `[DONE]` |

---

## IV. Kết Quả Kiểm Thử Đã Đạt Được (Test Matrix)

Hiện tại toàn bộ workspace đạt trạng thái **Green (114/114 tests passed)**:
```
✓ vc-core tests:               68 passed
✓ vc-llm tests:                10 passed
✓ vc-runtime tests:            17 passed
✓ vc-runtime integration:      16 passed
✓ vc-storage tests:             3 passed
──────────────────────────────────────────
Tổng cộng:                    114 passed, 0 failed
```

---

## V. Cập Nhật Định Hướng Toàn Diện Hệ Thống (Master Documentation Alignment)

Dev B đã hoàn thành rà soát và viết lại/đồng bộ toàn bộ hệ thống tài liệu dự án để chuyển hóa VirtualCharacter từ chatbot thông thường sang **Hệ điều hành AI VTuber / Embodied Character Runtime Cục bộ (Local-First + Cloud Fallback)** tối ưu cho **RTX 3050 Laptop 4GB VRAM**:

1. **`README.md`**:
   - Viết lại 100% trang chủ dự án, định nghĩa rõ ràng triết lý *"Character ≠ LLM"*, sơ đồ kiến trúc 6 trụ cột (Brain, Senses, Attention, Body, Hands, Social), bảng phân bổ tài nguyên tối ưu cho RTX 3050 4GB và hướng dẫn khởi động nhanh.
2. **`docs/TechStack.md`**:
   - Cập nhật toàn bộ công nghệ: Rust 2021, Ollama (Qwen2.5 3B Q4_K_M), Whisper STT (CPU), Piper/Kokoro TTS (CPU), Live2D & 3D VRM, FastEmbed CPU vector search, Win32 Desktop Sensing, và Discord/Stream Ingestion.
3. **`docs/architecture.md`**:
   - Đặc tả kiến trúc tầng, ranh giới crate, luồng dữ liệu thích ứng (Adaptive Event-Driven Sensing), cơ chế bảo vệ VRAM (Resource Manager) và mô hình bảo mật Sandbox cho tương tác máy tính.
4. **`docs/plan.md`**:
   - Viết lại toàn bộ Master Plan theo lộ trình 10 giai đoạn mới (Phase 0 đến Phase 10), thay thế hoàn toàn roadmap cũ (Phase 0-8). Xác định rõ Definition of Done (DoD) cho từng phase.
5. **`docs/tasks.md`**:
   - Cập nhật backlog chi tiết cho active sprint Phase 2 (Local AI Ollama + Semantic Search CPU) và các phase tiếp theo từ Phase 3 đến Phase 10 cho cả Dev A và Dev B.
6. **`docs/contracts.md`**:
   - Mở rộng ma trận hợp đồng ranh giới miền cho các trait tương lai: `ModelRegistry`, `EmbeddingProvider`, `AudioInputProvider`, `TtsProvider`, `AvatarController`, `ScreenCaptureProvider`, và `NormalizedChatMessage`.
7. **`docs/development.md`**:
   - Bổ sung hướng dẫn cài đặt model Ollama cục bộ, giám sát VRAM qua `nvidia-smi`, thiết lập SQLite, chạy test, CLI và server.

---

---

## VI. Chi Tiết Thực Thi Phase 2 — Task B2.1: Ollama Provider [HOÀN TẤT]

Dev B đã hoàn thành triển khai trọn vẹn **Task B2.1** tuân thủ nghiêm ngặt 12 nguyên tắc kiến trúc:
1. **`crates/vc-llm/src/ollama/config.rs`**:
   - Quản lý tập trung endpoint `base_url` (mặc định: `http://127.0.0.1:11434`), model (mặc định: `qwen2.5:3b`), timeout (60s), `keep_alive` ("5m") và max_retries (2).
   - Tuyệt đối không hard-code URL rải rác.
2. **`crates/vc-llm/src/ollama/client.rs`**:
   - `OllamaClient` hoàn toàn độc lập, không import bất kỳ concept nào của `vc-core` (Personality, Emotion, State, Memory, Decision).
   - Xử lý giao tiếp HTTP REST tới `/api/chat` bằng `ureq`.
   - DTO nội bộ: `OllamaChatMessage`, `OllamaChatOptions`, `OllamaChatRequest`, `OllamaChatResponse` được cô lập 100% bên trong crate `vc-llm`.
   - Mapping chi tiết lỗi: Connection Refused -> `ModelUnavailable` (hướng dẫn bật `ollama serve`), 404 -> `ModelUnavailable` (hướng dẫn `ollama pull`), 400 -> `InvalidRequest`, 500/503 -> `Other` / `ModelUnavailable`, Timeout -> `Timeout`.
3. **`crates/vc-llm/src/ollama/chat.rs`**:
   - `OllamaChatProvider` là adapter chuẩn hóa cài đặt trait `LlmProvider`.
   - Tự động chuyển đổi `system_instruction` thành message có `role: "system"` và `prompt` thành `role: "user"`.
   - Trích xuất chính xác token usage (`prompt_eval_count`, `eval_count`) thành `LlmUsage`.
4. **`crates/vc-llm/tests/ollama_tests.rs`**:
   - Xây dựng mock HTTP server độc lập (không cần bật Ollama thật khi test).
   - Kiểm thử toàn diện 9 trường hợp: request mapping, system message, user prompt, options (temperature, num_predict), token usage, JSON error, 404, 400, 500, malformed JSON, và connection refused.
   - Toàn bộ 9/9 tests pass 100%.
5. **Nghiệm thu Thực tế với Model thật (`qwen2.5:3b`) [PASS]**:
   - Đã pull thành công `qwen2.5:3b` (1.9 GB) về Ollama cục bộ.
   - Smoke test với System Instruction Aria: Phản hồi tiếng Việt tự nhiên (*"Chào bạn nha! Đằng ấy đây mà! Bạn hôm nay có muốn thử một trò chơi trí tuệ không?..."*), độ trễ < 2s.
   - Đo đạc VRAM trên RTX 3050 Laptop: Sử dụng 2991 MB / 4096 MB (~2.9GB tổng, còn trống ~1GB VRAM an toàn).

### 📊 Báo Cáo Kiểm Thử Tự Động Toàn Workspace (Phase 2 Baseline):
```
✓ vc-core unittests:           68 passed
✓ vc-llm unittests:            10 passed
✓ vc-llm ollama_tests:          9 passed [MỚI]
✓ vc-runtime unittests:        17 passed
✓ workspace integration tests: 16 passed
✓ vc-storage storage_tests:     3 passed
──────────────────────────────────────────
Tổng cộng:                    123 passed, 0 failed
```


## VII. Chi Tiết Thực Thi Phase 2 — Tasks B2.2, B2.3, B2.4, B2.5 [HOÀN TẤT 100%]

Dev B đã hoàn thành trọn vẹn toàn bộ các nhiệm vụ còn lại của **Phase 2 (Trí Tuệ Cục Bộ & Bộ Nhớ Vector 2.0)**:

1. **Task B2.2: Model Registry & Capabilities (`crates/vc-llm/src/registry.rs`)**:
   - `ModelCapability` enum: `TextGeneration`, `VisionUnderstanding`, `EmbeddingGeneration`, `ToolCalling`.
   - `ModelProfile` struct: quản lý ngân sách VRAM/RAM (`vram_mb_required`, `ram_mb_required`, `context_window`) kèm preset chính thức: `qwen2.5:3b` (2000MB), `qwen2.5vl:3b` (3200MB), `moondream` (1400MB), `gemini-1.5-flash` (0MB).
   - Trait `ModelRegistry` & thread-safe `DefaultModelRegistry`: chuyển đổi linh hoạt Chat Model và Vision Model khi cần.

2. **Task B2.3: Local Text Embeddings trên CPU (`crates/vc-storage/src/embedding.rs`)**:
   - Trait `EmbeddingProvider`: trừu tượng hóa chuẩn cho việc nhúng vector ngữ nghĩa (`embed_text`, `embed_batch`, `dimension`, `model_name`).
   - `MockEmbeddingProvider`: sinh vector giả lập 384 chiều dựa trên token/bigram hashing (DJB2) phục vụ kiểm thử nhanh, độc lập 100% offline.
   - `FastembedProvider` (kích hoạt qua feature flag `local-fastembed`): sử dụng crate `fastembed` với model `bge-small-en-v1.5` trên ONNX Runtime CPU (12 cores / 16 threads của i5-12500H, **0 MB VRAM**).
   - Hàm toán học `cosine_similarity(a, b)`: tính cosine similarity có chặn chuẩn hoá.
   - Cấu trúc `VectorMemoryQuery`: hỗ trợ tìm kiếm top-K kèm ngưỡng `min_similarity` và `actor_id`.

3. **Task B2.4: Tìm Kiếm Ký Ức Theo Vector Trong SQLite (`crates/vc-storage/src/sqlite.rs`)**:
   - Nâng cấp schema bảng `memories` có cột `embedding_json TEXT` kèm auto-migration `ALTER TABLE` cho database cũ.
   - Bổ sung các phương thức vào `MemoryRepository`:
     - `save_memory_with_embedding(&self, memory, embedding)`
     - `get_memory_embedding(&self, id)`
     - `search_similar_memories(&self, character_id, query)`
   - Cài đặt đầy đủ trên cả `SqliteStorage` và `InMemoryStorage`.
   - **Bảo vệ tuyệt đối Actor Privacy Isolation (Skill 12 / 25)**: Actor Alice truy vấn vector không bao giờ thấy ký ức riêng tư của Actor Bob dù độ tương đồng cosine là 1.0!

4. **Task B2.5: Giám Sát Tài Nguyên & Worker Tinh Lọc Giấc Ngủ (`vc-runtime`)**:
   - `ResourceManager` (`crates/vc-runtime/src/resource_manager.rs`):
     - Khóa cứng trần VRAM ở mức **3500 MB** cho RTX 3050 Laptop 4GB (`os_baseline: 1000MB`).
     - Hỗ trợ cơ chế **GPU Model Multiplexing** (`ExecutionMode`: `Chat`, `Vision`, `Idle`): luân phiên hoán đổi Chat và Vision model, không cho phép 2 model chạy cùng lúc trên GPU.
   - `MemoryConsolidator` (`crates/vc-runtime/src/consolidation.rs`):
     - Chạy chu kỳ dọn dẹp khi nhân vật `Idle`: áp dụng suy hao thời gian (decay), đào thải ký ức mờ nhạt (prune) nhưng bảo vệ tuyệt đối ký ức `Critical` và `is_pinned`.
     - Tự động gom 3 hoặc nhiều tương tác rời rạc thành một tri thức khái quát bền vững (`Semantic Memory`).

### 📊 Báo Cáo Kiểm Thử Tự Động Toàn Workspace (Phase 2 Final):
```
✓ vc-core unittests:           68 passed
✓ vc-llm unittests:            14 passed
✓ vc-llm ollama_tests:          9 passed
✓ vc-runtime unittests:        23 passed (+6 mới)
✓ workspace integration tests: 16 passed
✓ vc-storage unittests:         3 passed (+3 mới)
✓ vc-storage storage_tests:     3 passed
✓ vc-storage vector_tests:      4 passed (+4 mới)
──────────────────────────────────────────
Tổng cộng:                    140 passed, 0 failed
```

## VIII. Chi Tiết Thực Thi Phase 3 — Thính Giác & Tiếng Nói Biểu Cảm [HOÀN TẤT 100%]

Dev B đã hoàn thành trọn vẹn toàn bộ các nhiệm vụ của **Phase 3 (Thính Giác Cục Bộ & Tiếng Nói Biểu Cảm)** trong `vc-runtime`:

1. **Task B3.1: Thính Giác Cục Bộ & Voice Activity Detection (`crates/vc-runtime/src/audio/input.rs`)**:
   - Trait `AudioInputProvider`: quản lý vòng đời thu âm (`start_listening`, `stop_listening`, `is_listening`, `poll_transcription`, `process_audio_chunk`).
   - `InputAudioMode`: hỗ trợ 3 chế độ linh hoạt:
     - `PushToTalk`: Nhấn phím để nói (cho gamer/streamer).
     - `VoiceActivity`: Tự động phân đoạn bằng năng lượng RMS/VAD.
     - `WakeWord`: Nhận diện từ khóa kích hoạt ("Aria ơi!").
   - `VadStatus` & `VadConfig`: Nhận diện khoảng lặng (silence gap 600ms) để tự động chốt câu nói và gom buffer PCM gửi sang STT.
   - `MockAudioInputProvider`: Hỗ trợ queue câu nói giả lập và test toàn bộ luồng âm thanh offline 100%.

2. **Task B3.2: Tiếng Nói Biểu Cảm & Bộ Điều Chế Cảm Xúc (`crates/vc-runtime/src/audio/`)**:
   - Trait `TtsProvider` & đối tượng `AudioOutput` (`crates/vc-runtime/src/audio/tts.rs`):
     - Xuất dữ liệu âm thanh (`audio_bytes`), tần số lấy mẫu (`sample_rate`), thời lượng (`duration_ms`).
     - Tự động sinh danh sách **`VisemeCue`** (thời điểm bắt đầu, thời lượng, độ mở khẩu hình môi) sẵn sàng 100% làm dữ liệu Lip-sync cho Avatar Live2D/VRM ở Phase 4.
   - `EmotionAwareVoiceModulator` (`crates/vc-runtime/src/audio/modulator.rs`):
     - Nhận `EmotionState` từ `vc-core` và tự động điều chế:
       - **Joy / Excitement**: Cao độ tăng (+15%), tốc độ nhanh hơn (+10%), năng lượng dồi dào.
       - **Sadness / Melancholy**: Cao độ hạ (-18%), tốc độ chậm rãi (-16%), giọng nói trầm buồn.
       - **Anger / Frustration**: Tốc độ dồn dập (+15%), năng lượng cực đại.
       - **Affection / Intimacy**: Tự động kích hoạt hiệu ứng **thì thầm (`whisper_effect`)** khi tình cảm cao và trạng thái bình yên.
   - `MockTtsProvider` & `crates/vc-runtime/tests/audio_tests.rs`:
     - Kiểm thử tích hợp trọn vẹn chu trình End-to-End Voice Loop:
       $$\text{User Speech} \rightarrow \text{VAD/STT} \rightarrow \text{Runtime Turn} \rightarrow \text{Post-Turn Emotion} \rightarrow \text{Voice Modulator} \rightarrow \text{TTS Audio + Visemes}$$

### 📊 Báo Cáo Kiểm Thử Tự Động Toàn Workspace (Phase 3 Final):
```
✓ vc-core unittests:           68 passed
✓ vc-llm unittests:            14 passed
✓ vc-llm ollama_tests:          9 passed
✓ vc-runtime unittests:        31 passed (+8 audio tests)
✓ vc-runtime audio_tests:       3 passed (+3 mới)
✓ vc-runtime tools_tests:       2 passed (+2 mới - WebSearch & KnowledgeLearner)
✓ workspace integration tests: 16 passed
✓ vc-storage unittests:         3 passed
✓ vc-storage storage_tests:     3 passed
✓ vc-storage vector_tests:      4 passed
──────────────────────────────────────────
Tổng cộng:                    153 passed, 0 failed
```

---

## IX. Công Cụ Tra Cứu Web & Cơ Chế Học Tập Chủ Động (Methods A & B) [HOÀN TẤT 100%]

Triển khai 2 tính năng nâng cao giúp nhân vật am hiểu thế giới thực và ghi nhớ kiến thức do người dùng giảng dạy:

1. **Method A: Real-time Web Search Tool (`crates/vc-runtime/src/tools.rs`)**:
   - `WebSearchTool`: Tự động nhận diện ý định tra cứu thực tế (*"có biết MCK không"*, *"MCK là ai"*, *"thời tiết Hà Nội"*, *"search..."*).
   - Truy vấn song song **DuckDuckGo Lite** và fallback **Wikipedia tiếng Việt** (hoàn toàn miễn phí, 0đ API, 0 credit).
   - Bơm các đoạn trích dẫn thực tế (`snippets`) vào ngữ cảnh LLM (`[Thông Tin Tra Cứu Thực Tế Từ Web / Google]`).
   - Phát sự kiện WebSocket `tool_executing` và `tool_executed` cho Web UI.

2. **Method B: Active Knowledge Learning & Semantic Ingestion (`crates/vc-runtime/src/tools.rs`)**:
   - `KnowledgeLearner`: Phát hiện các câu dạy học hoặc chia sẻ sở thích cá nhân (*"Dạy cho em nè..."*, *"nhớ là..."*, *"mình thích..."*, *"X là Y"*).
   - Tự động hình thành **Semantic Memory** với độ quan trọng **`Critical`** hoặc **`High`** và lưu vĩnh viễn vào SQLite (`data/virtual_character.db`).
   - Cập nhật trực tiếp vào danh sách `known_facts` trong `RelationshipState` hiển thị trên Relationship HUD.
   - Thúc đẩy Aria phản hồi biết ơn, hào hứng xác nhận đã ghi nhớ, và tự động truy xuất lại ký ức này trong các lượt trò chuyện sau.

3. **Bộ Lọc Lời Thoại Thuần Khiết (`sanitize_character_dialogue`)**:
   - Cắt bỏ hoàn toàn các tiền tố/hậu tố tự sự của người thứ ba (như *"Phỏng vấn từ người bạn, Aria đáp lại..."*, *"Aria giữ giọng..."*, *"Với mức độ bộc lộ cảm xúc cao..."*).
   - Đảm bảo Aria nói chuyện 100% tự nhiên ở ngôi thứ nhất.

---

## X. Phase 4: Hiện Thân Ảo & Điều Khiển Thân Thể [HOÀN TẤT 100%]
1. **WebGL 3D VRM Canvas (`apps/vc-web/src/components/avatar/VrmAvatarStage.tsx`)**:
   - Sử dụng `three` và `@pixiv/three-vrm` để render model 3D trực tiếp trong trình duyệt.
   - Nền trong suốt (Transparent Background) sẵn sàng làm Browser Source cho OBS Studio khi livestream.
   - Model avatar 3D chuẩn VRM Public License: `Seed-san.vrm` (10.9 MB, ~150MB VRAM WebGL).
   - Chuyển đổi linh hoạt giữa chế độ 3D VRM Live Stage và 2D Hào quang Cảm xúc (Bioluminescent Orb).
2. **Bộ Điều Khiển Thân Thể (Body Controller)**:
   - Khẩu hình môi (Lip-sync) thời gian thực đồng bộ trực tiếp từ luồng biên độ âm thanh TTS (`audioAmplitude` -> `VRMExpressionPresetName.Aa`).
   - Biểu cảm khuôn mặt tự động chuyển đổi theo `EmotionState` (`Happy`, `Sad`, `Angry`, `Surprised`, `Relaxed`).
   - Tự động chớp mắt (Auto Blink chu kỳ 3-5.5s), thở nhàn rỗi (Idle breathing) và liếc nhìn theo chuột (Look-at target tracking).

---

## XI. Phase 5: Mô Hình Thế Giới, Chú Ý & Thị Giác Thích Ứng [HOÀN TẤT 100%]
1. **Mô Hình Thế Giới (`crates/vc-core/src/state/world.rs`)**:
   - `WorldState`, `WindowContext`, `ActivityType`, `AmbientContext`, `TimeOfDay`.
2. **Bộ Lọc Chú Ý (`crates/vc-runtime/src/attention.rs`)**:
   - Điểm chú ý `AttentionScore` (*Salience + Curiosity + Urgency - CooldownPenalty - BusyPenalty*).
   - Chống phát biểu liên tục: cooldown 30s giữa các lần tự động lên tiếng; người dùng gọi trực tiếp bỏ qua cooldown.
3. **Thị Giác Màn Hình Thích Ứng (`crates/vc-runtime/src/vision/`)**:
   - `VisualDiffDetector`: So sánh sai khác pixel trên CPU, 0 MB VRAM, lọc bỏ biến động < 15%.
   - `VisionRouter` & `ResourceManager`: Kích hoạt On-Demand `qwen2.5vl:3b`, bảo vệ VRAM RTX 3050.

---

## XII. Phase 6: Bàn Tay Tương Tác Máy Tính Có Kiểm Soát [HOÀN TẤT 100%]
1. **Computer Tools (`crates/vc-runtime/src/computer/action.rs`)**:
   - `ComputerAction`, `WindowsComputerExecutor`, `MockComputerExecutor`.
2. **Hộp Cát An Toàn (`crates/vc-runtime/src/computer/policy.rs`)**:
   - `ToolRiskLevel` (`Low`, `Medium`, `High`, `Blocked`). Lệnh phá hoại (`rmdir /s`, format, xóa system32) bị chặn vĩnh viễn (`ToolRiskLevel::Blocked`).
3. **Vòng Lặp Xác Minh Thị Giác (`crates/vc-runtime/src/computer/verifier.rs`)**:
   - `ScreenVerificationLoop`: Chụp lại màn hình sau hành động để xác nhận UI đã phản hồi đúng trước khi báo hoàn tất.

---

## XIII. Phase 7: Tích Hợp Nền Tảng Đa Phương Tiện & Discord [HOÀN TẤT 100%]
1. **Unified Event Bus (`crates/vc-runtime/src/events/mod.rs`)**:
   - Schema `PlatformChatMessage` đa nền tảng (`PlatformType`: Web, Discord, YouTube, Twitch, Cli, System).
   - `PlatformEventBus`: Hệ thống Pub/Sub bất đồng bộ dựa trên `tokio::sync::broadcast`.
2. **Discord Adapter (`crates/vc-runtime/src/platforms/discord.rs`)**:
   - Lọc bot chống lặp vô tận, lọc channel whitelist, nhận diện mention.
   - Bảo mật Actor Privacy Isolation: `actor_id: "discord:<user_id>"` duy trì Relationship và Memory riêng biệt.

---

## XIV. Phase 8: Môi Trường Vận Hành AI VTuber Thực Thụ [HOÀN TẤT 100%]
1. **Livestream Chat Ingestion (`crates/vc-runtime/src/platforms/stream.rs`)**:
   - `YouTubeLiveAdapter` (SuperChat/Donations) & `TwitchChatAdapter` (IRC PRIVMSG).
2. **Bộ Lọc Chat Thông Minh (`crates/vc-runtime/src/stream/priority.rs`)**:
   - `ChatPriorityEngine`: Xếp hạng SuperChat (+100) > Mention (+35) > Question (+25).
   - Chống spam lặp từ (Sliding-window hash deduplication).
   - Gom cụm chủ đề câu hỏi (gaming, food, music, chat).
3. **Stream Director (`crates/vc-runtime/src/stream/director.rs`)**:
   - Nhạc trưởng điều phối nhịp độ livestream tự trị: cân bằng chat, phản ứng sự kiện màn hình game (`ReactToScreen`) và tự động độc thoại phá vỡ khoảng lặng (`SelfInitiatedBanter`).

---

## XV. Phase 9+: Đa Nhân Vật & Học Máy Thích Ứng [HOÀN TẤT 100%]
1. **Multi-Character Profiles (`crates/vc-runtime/src/multicharacter/mod.rs`)**:
   - `MultiCharacterRegistry`: Quản lý song song Aria, Hikari, v.v. với Personality, State, Memory, Relationship độc lập.
2. **Hội Thoại Giữa Các Nhân Vật (C2C Dialogue)**:
   - `CharacterToCharacterDialogue`: Điều phối hội thoại tự trị đa lượt (Turn-taking) giữa 2 nhân vật ảo.
   - Hình thành mối quan hệ tương hỗ tự động giữa các nhân vật.
3. **Học Máy Ngoại Tuyến (Offline Adaptation)**:
   - `PreferenceDataset`: Lưu trữ đánh giá người dùng (`InteractionFeedback`) xuất định dạng JSONL cho DPO/RLHF.
   - `OfflineAdaptationEngine`: Đề xuất vi chỉnh tính cách an toàn (`verbosity_delta`, `initiative_delta`).

---

## XVI. Báo Cáo Kiểm Thử Toàn Diện Workspace (199 Tests PASS)
```
✓ vc-core unittests:            31 passed
✓ vc-llm unittests:             14 passed
✓ vc-llm ollama_tests:           9 passed
✓ vc-runtime unittests:         62 passed
✓ vc-runtime audio_tests:        3 passed
✓ vc-runtime computer_tests:     3 passed
✓ vc-runtime integration:       16 passed
✓ vc-runtime multicharacter:     3 passed
✓ vc-runtime platform_tests:     2 passed
✓ vc-runtime stream_tests:       4 passed
✓ vc-runtime tools_tests:        2 passed
✓ vc-runtime vision_tests:       4 passed
✓ vc-storage unittests:          3 passed
✓ vc-storage storage_tests:      3 passed
✓ vc-storage vector_tests:       4 passed
──────────────────────────────────────────
Tổng cộng:                    199 passed, 0 failed (100% GREEN)
```




