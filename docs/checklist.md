# 📋 VirtualCharacter Master Roadmap & Progress Checklist

> **Tài liệu tham chiếu cốt lõi**:
> * **Bản định hướng kiến trúc tổng thể (Master Vision)**: [docs/vision.md](file:///d:/VirtualCharactor/docs/vision.md)
> * Kế hoạch kiến trúc ban đầu: [docs/plan.md](file:///d:/VirtualCharactor/docs/plan.md)
> * Phân chia nhiệm vụ song song Dev A / Dev B: [docs/tasks.md](file:///d:/VirtualCharactor/docs/tasks.md)
> * Hợp đồng giao tiếp miền chung: [docs/contracts.md](file:///d:/VirtualCharactor/docs/contracts.md)
> * Nhật ký & danh sách file của Dev B: [docs/dev_b_progress.md](file:///d:/VirtualCharactor/docs/dev_b_progress.md)
>
> **Trạng thái tổng thể**: `[████████████████████]` **REAL IMPLEMENTATION MODE — 100% PASS** — Nền tảng Core / Storage / LLM / Web HUD / VRM và toàn bộ 6 subsystem I/O vật lý P0 (Native Screen Capture, Win32 Mouse & Keyboard, WASAPI Microphone Input, CPU faster-whisper STT, Windows SAPI Local TTS, Ollama VRAM Eviction) đã được kiểm nghiệm thực tế 100% trên phần cứng thật (Intel i5-12500H + NVIDIA RTX 3050 4GB).


---

## 🟢 PHẦN 1: ĐÃ HOÀN THÀNH (Current Milestones)

### Phase 0: Khởi Tạo Kiến Trúc & Hợp Đồng (Architecture Bootstrap) — `[100% DONE]`
- [x] Thiết lập Rust Workspace đa crate độc lập:
  - [x] `crates/vc-core`: Domain logic thuần túy, không dính dáng hạ tầng.
  - [x] `crates/vc-runtime`: Tầng điều phối tương tác và quản lý phiên.
  - [x] `crates/vc-llm`: Tầng trừu tượng hóa LLM provider và adapter Gemini.
  - [x] `crates/vc-storage`: Tầng trừu tượng hóa kho dữ liệu và adapter SQLite.
  - [x] `apps/vc-cli`: Ứng dụng dòng lệnh kiểm thử nhanh.
  - [x] `apps/vc-server`: Backend Gateway (Axum / Tokio / WebSocket).
  - [x] `apps/vc-web`: Ứng dụng React Mind Inspector HUD.
- [x] Quy tắc phân quyền 2 lập trình viên (`docs/development.md`): Dev A (Character Core) $\leftrightarrow$ Dev B (Runtime & Boundary).
- [x] Đóng băng hợp đồng miền dùng chung (`docs/contracts.md`): Identity types, Domain entities, Trait boundaries.
- [x] Xây dựng Mock ban đầu: `MockLlmProvider`, `MockDecisionEngine`.
- [x] Thiết lập hệ thống kỹ năng chuẩn hóa cho Agent (`.agent/skills/`).

---

### Phase 1: Nền Tảng Domain & Hạ Tầng Cơ Sở (Foundation) — `[100% DONE]`

#### 🧠 1.1. Dev A — Character Intelligence (Trí Tuệ Nhân Vật)
- [x] **A1. Personality Domain (Tính cách nhân vật)**:
  - [x] Cấu trúc vector đặc trưng đa chiều (Traits, Values, Preferences, Boundaries).
  - [x] Giới hạn chỉ số nghiêm ngặt `[-1.0, 1.0]` hoặc `[0.0, 1.0]` với logic validation tự động.
  - [x] Nhân vật mẫu cơ sở: "Aria" (nhân ái, tò mò, ấm áp).
  - [x] Serialization / Deserialization JSON độc lập, tuyệt đối không phụ thuộc vào LLM.
- [x] **A2. State & Emotion Domain (Cảm xúc & Trạng thái)**:
  - [x] Mô hình cảm xúc 8 trục (*Joy, Sadness, Anger, Fear, Surprise, Affection, Embarrassment, Curiosity*).
  - [x] Tính toán tự động trục Valence (Tích cực/Tiêu cực) và Arousal (Mức độ kích thích).
  - [x] Cơ chế suy giảm cảm xúc tự nhiên theo thời gian thực (*Exponential Decay*).
  - [x] Bộ máy cảm xúc luật cứng `RuleBasedEmotionEngine` tính toán biến thiên theo câu nói của user.
  - [x] Đồng bộ cảm xúc sang nhận thức (*CognitiveState: Focus, Attention*) và hành vi (*BehaviorState: Playfulness, Seriousness*).
- [x] **A3. Relationship Domain (Mối quan hệ hai chiều)**:
  - [x] Mô hình Bipartite Relationship độc lập theo cặp `(character_id, actor_id)`.
  - [x] Cơ chế cô lập hoàn toàn giữa các User khác nhau (Actor Privacy Isolation — Skill 14).
  - [x] 5 chỉ số động: *Closeness, Trust, Familiarity, Affection, Tension*.
  - [x] Máy trạng thái 5 bậc quan hệ: *Stranger $\rightarrow$ Acquaintance $\rightarrow$ Friend $\rightarrow$ CloseFriend $\rightarrow$ Companion*.
- [x] **A4. Memory Domain Foundation (Cơ sở ký ức)**:
  - [x] Cấu trúc phân loại 4 tầng: *Core, Semantic, Episodic, Working*.
  - [x] Vòng đời ký ức: Thời gian tạo, mốc truy cập, số lần củng cố (`access_count`).
  - [x] Bộ lọc truy vấn đa chiều `MemoryQuery` (theo Actor, Importance, Recency, Keyword).
- [x] **A5. Decision Domain (Bộ máy ra quyết định độc lập)**:
  - [x] `RuleDecisionEngine`: Phân tích tình huống $\rightarrow$ Sinh ứng viên $\rightarrow$ Chấm điểm $\rightarrow$ Chọn hành động.
  - [x] Các kiểu hành động: *WarmGreeting, ThoughtfulExplanation, EmotionalResonance, PlayfulBanter...*
  - [x] Tạo **Độc thoại nội tâm (Inner Monologue)** giải thích lý do hành vi trước khi gọi LLM sinh văn bản.

#### ⚙️ 1.2. Dev B — Runtime, Infrastructure & Storage (Vận Hành & Hạ Tầng)
- [x] **B1. Context Engine & Token Budget**:
  - [x] Định nghĩa `ContextItem` từ 5 nguồn dữ liệu (`System`, `User`, `Personality`, `State`, `Relationship`, `Memory`).
  - [x] Phân loại 4 cấp ưu tiên: `Critical > High > Medium > Low`.
  - [x] `ContextBudget`: Ước lượng token và tự động cắt tỉa mục ưu tiên thấp khi quá giới hạn cho phép.
  - [x] `ContextBreakdown`: Thống kê token cho từng thành phần phục vụ Web UI Token Gauge.
- [x] **B2. Runtime Orchestration**:
  - [x] Điều phối chu trình tương tác hoàn chỉnh qua 9 giai đoạn (`RuntimeEngine`).
  - [x] Máy trạng thái phiên làm việc `SessionManager` (*Created, Active, Idle, Completed, Expired*).
  - [x] Cơ chế phát hiện và xử lý Timeout khi phiên tương tác ngưng hoạt động.
- [x] **B3. LLM Provider Layer**:
  - [x] Trait `LlmProvider` thuần túy, trung lập với các hãng AI.
  - [x] Tích hợp Google Gemini REST API (`GeminiProvider`) với cơ chế retry tự động (exponential backoff).
  - [x] Cơ chế tự động che giấu API key trong logs (`[REDACTED]` — Skill 20).
  - [x] `MockLlmProvider` có kịch bản chuỗi và mô phỏng lỗi để test offline 100%.
- [x] **B4. Storage Subsystem (Local-First Persistence)**:
  - [x] Bổ sung `StorageError(String)` trong `CoreError`.
  - [x] Định nghĩa 4 Repository traits chuẩn `Send + Sync`: `CharacterRepository`, `StateRepository`, `RelationshipRepository`, `MemoryRepository`.
  - [x] `InMemoryStorage`: Lưu trữ RAM thread-safe phục vụ kiểm thử nhanh.
  - [x] `SqliteStorage`: Tự động tạo bảng (`characters`, `relationships`, `memories`), indexes, bật WAL mode, khóa mutex an toàn luồng ghi.
- [x] **B5. Integration & Applications**:
  - [x] `vc-cli`: CLI chạy trực tiếp trên file database SQLite, dữ liệu sống sót qua nhiều lần chạy.
  - [x] `vc-server`: Backend Axum cung cấp REST API & WebSocket live stream kết nối SQLite.
  - [x] `vc-web`: React Mind Inspector hiển thị trực tiếp trục cảm xúc, độc thoại, token gauge.
  - [x] Toàn bộ **114/114 automated tests** của workspace pass xanh 100%.

---

## 🔵 PHẦN 2: LỘ TRÌNH MỚI CHO AI VTUBER LOCAL-FIRST (Future Roadmap)

### Phase 2: Trí Tuệ Cục Bộ & Bộ Nhớ Vector 2.0 (Local AI Brain & Memory 2.0)

#### 2.1. Tích hợp Ollama Cục Bộ & Model Registry
- [x] **Môi trường Ollama local**:
  - [x] Kiểm tra Ollama đã được cài đặt và chạy được (`ollama version 0.34.2`).
  - [x] Kiểm tra endpoint `http://127.0.0.1:11434` (TCP Port listening & REST API active).
  - [x] Pull model chat mặc định `qwen2.5:3b` (Đã tải thành công 1.9 GB vào Ollama).
  - [x] Xác nhận model có thể chạy local bằng CLI/API với `qwen2.5:3b` (Đã test phản hồi tiếng Việt siêu tốc < 2s).
  - [x] Không hard-code model name ngoài `OllamaConfig`.

- [x] **Ollama Transport Layer**:
  - [x] Tạo `OllamaClient` (độc lập 100%, không import bất kỳ concept nào của `vc-core`).
  - [x] Tạo `OllamaConfig` (quản lý tập trung base_url, model, timeout, max_retries, keep_alive).
  - [x] Hỗ trợ `/api/chat` làm đường chính.
  - [x] `stream: false` cho B2.1.
  - [x] Hỗ trợ `timeout` và `keep_alive`.
  - [x] Không để Ollama DTO (`OllamaChatRequest`, `OllamaChatMessage`) leak ra ngoài `vc-llm`.

- [x] **OllamaChatProvider**:
  - [x] Implement trait `LlmProvider` hiện tại.
  - [x] Map `LlmRequest` (system_instruction + user prompt) $\rightarrow$ `OllamaChatRequest`.
  - [x] Map Ollama response $\rightarrow$ `LlmResponse`.
  - [x] Map token usage (`prompt_eval_count`, `eval_count`) $\rightarrow$ `LlmUsage`.
  - [x] Map lỗi HTTP / network / timeout / invalid response về `LlmError`.
  - [x] Không sửa domain contract của `vc-core`.
  - [x] Không thay đổi contract của `vc-runtime`.

- [x] **Kiểm thử Tự động (Automated Testing)**:
  - [x] Mock HTTP server, không phụ thuộc Ollama thật.
  - [x] Test request mapping (system message, user prompt, options).
  - [x] Test response content mapping & finish reason.
  - [x] Test token usage mapping.
  - [x] Test connection failure & connection refused.
  - [x] Test timeout.
  - [x] Test HTTP 400 bad request, 404 model not found, 500 server error.
  - [x] Test malformed JSON handling.
  - [x] Test Ollama error response (`{"error": "..."}`).
  - [x] Toàn bộ test workspace hiện tại vẫn PASS (123/123 tests pass 100%).

- [x] **Tích hợp & Nghiệm thu Thực tế (Integration & Acceptance)**:
  - [x] `vc-runtime` có thể nhận `Arc<dyn LlmProvider>` với `OllamaChatProvider`.
  - [x] Không cần Gemini API key để chạy local chat.
  - [x] Ollama không được phép truy cập trực tiếp Character Core state.
  - [x] `cargo check --workspace` PASS.
  - [x] `cargo test --workspace` PASS.
  - [x] Test Ollama riêng PASS (`9/9 passed`).
  - [x] Manual smoke test với model thật `qwen2.5:3b` trên máy PASS (Phản hồi tiếng Việt trôi chảy, VRAM ~2.9GB/4GB an toàn).
  - [x] Không có breaking change đối với Core/Runtime.



- [x] **2.2. Vector Embeddings & Semantic Search Trong SQLite**:
  - [x] Tích hợp mô hình sinh Vector Embedding chạy hoàn toàn trên CPU (`fastembed-rs` với `bge-small-en/multilingual` qua feature `local-fastembed`, cùng `MockEmbeddingProvider` cho deterministic test).
  - [x] Nâng cấp bảng `memories` trong SQLite hỗ trợ cột `embedding_json TEXT` và truy vấn Cosine Similarity `search_similar_memories`.
  - [x] Bảo vệ nghiêm ngặt **Actor Privacy Isolation** (Skill 12 / 25): Tuyệt đối không cho phép Actor Alice truy cập ký ức của Actor Bob qua vector similarity.
- [x] **2.3. Giám Sát Tài Nguyên Phần Cứng & Ollama Model Multiplexing**:
  - [x] Xây dựng `ResourceManager`: giám sát VRAM GPU và RAM hệ thống theo thời gian thực (`ResourceSnapshot`).
  - [x] Khóa an toàn: đảm bảo tổng mức VRAM tiêu thụ không bao giờ vượt quá 3.5 GB / 4.0 GB của RTX 3050 (`VramBudgetExceeded`).
  - [x] Hỗ trợ cơ chế **GPU Model Multiplexing** (`ExecutionMode`: `Chat`, `Vision`, `Idle`) và `OllamaClient::unload_model` (`keep_alive: 0`) giải phóng GPU VRAM tức thì.
- [x] **2.4. Worker Giấc Ngủ & Tinh Lọc Ký Ức (Sleep & Consolidation Worker)**:
  - [x] `MemoryConsolidator`: định kỳ chạy chu kỳ dọn dẹp và tinh lọc ký ức khi nhân vật ở trạng thái `Idle`.
  - [x] Tự động tóm tắt chuỗi sự kiện `Episodic Memory` (từ 3 tương tác cùng actor) thành tri thức bền vững `Semantic Memory`.
  - [x] Đào thải (prune/decay) các mẩu thông tin rác có retention strength rơi xuống dưới ngưỡng (bảo vệ tuyệt đối ký ức `Critical` và `is_pinned`).

---

### Phase 3: Thính Giác & Tiếng Nói Biểu Cảm (Voice & Auditory Presence) — `[100% DONE]`
- [x] **3.1. Thính Giác Cục Bộ (Hearing / STT)**:
  - [x] Định nghĩa hợp đồng trait `AudioInputProvider` độc lập (`crates/vc-runtime/src/audio/input.rs`).
  - [x] **Real Native Windows WASAPI Microphone Input** (`CpalAudioInputProvider`):
    - [x] Thu âm trực tiếp từ driver âm thanh phần cứng qua thư viện chuẩn Rust `cpal`.
    - [x] Kiến trúc luồng nền cô lập COM (`mpsc::channel`), đảm bảo an toàn tuyệt đối cho chuẩn COM STA/MTA của Windows WASAPI và thỏa mãn ràng buộc `Send + Sync`.
    - [x] Kiểm nghiệm phần cứng thực tế: Nhận diện chính xác 4 thiết bị đầu vào âm thanh trên máy (`Microphone Array (Realtek(R) Audio)`, `Voicemod Virtual Audio Device`, `Stereo Mix`, `VB-Audio Virtual Cable`).
  - [x] **Real Local CPU Whisper STT** (`WhisperCpuTranscriber`):
    - [x] Chạy mô hình `faster-whisper` (CTranslate2 int8) hoàn toàn trên CPU Intel Core i5-12500H, tiêu thụ 0 MB GPU VRAM.
    - [x] Kiểm nghiệm thực tế: Chuyển đổi chính xác 100% giọng nói mẫu sang văn bản trong 3.5s với độ tin cậy 96.37% ("Hello, I am Aria, your local virtual assistant.").
  - [x] Tích hợp Voice Activity Detection (`VadStatus`, `VadConfig`, RMS energy threshold) để phân đoạn giọng nói và phát hiện khoảng lặng (silence gap).
  - [x] Hỗ trợ chế độ **Push-to-Talk**, **VoiceActivity** và **WakeWord** ("Aria ơi!").
  - [x] `MockAudioInputProvider`: cho phép test luồng âm thanh và nhận diện giọng nói deterministic offline.
- [x] **3.2. Tiếng Nói Biểu Cảm (Emotion-Aware TTS) & Stream Chunker**:
  - [x] Định nghĩa hợp đồng trait `TtsProvider` và đối tượng `AudioOutput` (`crates/vc-runtime/src/audio/tts.rs`).
  - [x] **Real Native Windows SAPI / OneCore TTS** (`WindowsSapiTtsProvider`):
    - [x] Tận dụng giọng đọc máy tính có sẵn trong Windows (`Microsoft Zira Desktop`, `Microsoft David Desktop`), 100% offline, 0đ chi phí, 0 MB GPU VRAM.
    - [x] Kiểm nghiệm thực tế: Tổng hợp 154,582 bytes WAV (22050 Hz) trong 0.51s kèm 176 mốc khẩu hình môi đồng bộ.
  - [x] Tích hợp giọng đọc tiếng Việt truyền cảm `vi-VN-HoaiMyNeural` (Edge-TTS, 0 MB VRAM) qua endpoint `/api/audio/tts`.
  - [x] **AIRI-Style Stream Chunker & Action Stripper** (`TextStreamChunker`):
    - [x] Chia tách luồng token LLM streaming theo dấu câu (`.`, `!`, `?`, `\n`, `。`, `！`, v.v.) để đạt độ trễ phát âm < 500ms (TTFA).
    - [x] Tự động lọc bỏ các chỉ dẫn sân khấu và biểu cảm trong ngoặc (`*mỉm cười*`, `[laughs]`, `(vẫy tay)`) để TTS không phát âm tiếng ngoặc đơn.
  - [x] **Open-LLM-VTuber RMS Volume Slices** (`calculate_rms_volume_slices`):
    - [x] Tính toán mảng năng lượng RMS 20ms trực tiếp từ dữ liệu PCM giúp avatar Live2D / VRM chuyển động khẩu hình mượt mà không cần mô hình neural phoneme.
  - [x] Bộ điều biến giọng nói theo cảm xúc `EmotionAwareVoiceModulator` và sinh mốc khẩu hình `VisemeCue`.
  - [x] `CharacterAudioPlayer` (Web Audio API Analyser) đo biên độ sóng âm cho khẩu hình miệng và soundwave visualizer trên `EmotionStage`.
  - [x] Nút Bật/Tắt âm thanh giọng nói (Mute/Unmute) trên `TopNavbar`.

---

### Phase 4: Hiện Thân Ảo & Điều Khiển Thân Thể (Embodied Avatar & Body Controller) — `[100% DONE]`
- [x] **4.1. Tích Hợp Avatar Live2D / VRM**:
  - [x] Cung cấp Viewport render 3D VRM tương thích chuẩn WebGL (Three.js + `@pixiv/three-vrm`).
  - [x] Nền trong suốt (Transparent Background) sẵn sàng làm Browser Source cho OBS Studio khi stream.
  - [x] Tích hợp model avatar 3D chuẩn VRM Public License: `Seed-san.vrm` (10.9 MB, ~150MB VRAM WebGL).
  - [x] Chuyển đổi linh hoạt giữa chế độ 3D VRM Live Stage và 2D Hào quang Cảm xúc (Bioluminescent Orb).
- [x] **4.2. Bộ Điều Khiển Thân Thể (Body Controller)**:
  - [x] Khẩu hình môi (Lip-sync) thời gian thực đồng bộ trực tiếp từ luồng biên độ âm thanh TTS (`audioAmplitude` -> `VRMExpressionPresetName.Aa`).
  - [x] Biểu cảm khuôn mặt tự động chuyển đổi theo `EmotionState` (`Happy`, `Sad`, `Angry`, `Surprised`, `Relaxed`) với nội suy chuyển động mượt mà.
  - [x] Hành vi sống động tự nhiên: Tự động chớp mắt (Auto Blink chu kỳ 3-5.5s), thở nhàn rỗi (Idle breathing) và liếc nhìn theo chuột/người dùng (Look-at target tracking).

---

### Phase 5: Mô Hình Thế Giới, Chú Ý & Thị Giác Thích Ứng (World Model, Attention & Vision) — `[100% DONE]`
- [x] **5.1. Mô Hình Thế Giới (World Model)**:
  - [x] Cấu trúc `WorldState`: Ghi nhận cửa sổ đang hoạt động (`WindowContext`, `ActivityType`: Coding, Gaming, Browsing, Media...), môi trường âm thanh (`AmbientContext`), mốc thời gian trong ngày (`TimeOfDay`).
  - [x] Sinh mô tả ngữ cảnh tự động cho `ContextBuilder` (`context_description`).
- [x] **5.2. Bộ Lọc Chú Ý & Đời Sống Tự Trị (Attention Engine & Idle Life)**:
  - [x] Tính điểm chú ý đa nhân tố `AttentionScore`: *Salience + Curiosity + Urgency - CooldownPenalty - BusyPenalty*.
  - [x] Chống phát biểu liên tục (Chatter Spurt): Áp dụng hình phạt hồi chiêu tối thiểu 30 giây giữa các lần tự động lên tiếng; Người dùng gọi trực tiếp bỏ qua hồi chiêu tức thì.
  - [x] Tôn trọng trạng thái tập trung của người dùng (`is_user_busy`): Giảm thiểu quấy rầy khi đang viết code hoặc chơi game.
- [x] **5.3. Thị Giác Màn Hình Thích Ứng (Adaptive Screen Perception Loop)**:
  - [x] **Native Windows Screen Capture Provider** (`WindowsScreenCaptureProvider`):
    - [x] Gọi trực tiếp Win32 GDI & User32 FFI native (`GetDC`, `CreateCompatibleDC`, `CreateCompatibleBitmap`, `BitBlt`, `GetDIBits`).
    - [x] Tự động gắn kết Thread Desktop (`OpenInputDesktop` & `SetThreadDesktop`) cho phép chụp màn hình ngay cả khi chạy từ background terminal / IDE subshell.
    - [x] Nhận diện cửa sổ và tiến trình Foreground thời gian thực (`GetForegroundWindow`, `GetWindowTextW`, `QueryFullProcessImageNameW`).
    - [x] Kiểm nghiệm thực tế thành công trên desktop Windows thật: Chụp thành công độ phân giải gốc 2560x1440 (14.7 MB raw BGRA) trong 0.07s, downsample mượt mà sang 64x64 luma thumbnail và encode 640x360 24-bit BMP preview (`scratch/captured_real_desktop.bmp` 691 KB, verified via Python/PIL).
  - [x] **Tầng 1 (Sensing)**: So sánh sai khác khung hình `VisualDiffDetector` (Pixel Diff / Noise Gate) cực nhanh hoàn toàn trên CPU, 0 MB VRAM; Biến động < 15% bị lọc bỏ ngay lập tức.
  - [x] **Tầng 2 (Perception)**: Nhận biết tiêu đề cửa sổ, tiến trình foreground và phân loại hoạt động tự động.
  - [x] **Tầng 3 (Understanding)**:
    - [x] Model Vision cục bộ: Nghiệm thu thực tế `qwen2.5vl:3b` (3.2 GB) trong Ollama qua `OllamaVisionProvider`.
    - [x] `VisionRouter`: Kích hoạt On-Demand kết nối `VisionProvider` và `ResourceManager` (chuyển đổi `ExecutionMode::Vision` và trả về `Idle` an toàn).
    - [x] Cơ chế Fallback an toàn: Khi VLM lỗi/timeout, trả về quan sát suy biến an toàn, tuyệt đối không panic hay làm gián đoạn runtime.

---

### Phase 6: Bàn Tay Tương Tác Máy Tính Có Kiểm Soát (Controlled Computer Use) — `[100% DONE]`
- [x] **6.1. Công Cụ Thao Tác Hệ Thống (Computer Tools)**:
  - [x] **Real Native Windows Input Executor** (`WindowsComputerExecutor`):
    - [x] Điều khiển chuột & bàn phím Win32 native FFI (`SetCursorPos`, `GetCursorPos`, `SendInput`).
    - [x] Cấu trúc `INPUT` 40-byte chuẩn xác tuyệt đối theo Win32 64-bit ABI.
    - [x] Tự động gắn kết quyền desktop tương tác `DESKTOP_ALL` (`0x01FF`) cho phép thực thi `SendInput` từ headless IDE / subshell background process mà không bị lỗi `Win32 Error 5: Access Denied`.
    - [x] Hỗ trợ gõ ký tự Unicode qua cờ `KEYEVENTF_UNICODE`, click chuột trái/phải/giữa và tổ hợp phím tắt (Hotkey simulation: Ctrl/Shift/Alt/Win + Key).
    - [x] Kiểm nghiệm thực tế thành công: Đọc và di chuyển tọa độ con trỏ thật `(1558, 913) -> (1563, 918)`, gõ chuỗi Unicode tiếng Việt "Aria Xin chào 🌟", mô phỏng phím Shift+A.
  - [x] Triển khai các tool cơ bản: `computer.open_app`, `computer.click`, `computer.type`, `computer.press_hotkey`, `computer.scroll` (`ComputerAction`, `WindowsComputerExecutor`, `MockComputerExecutor`).
- [x] **6.2. Hộp Cát An Toàn & Phê Duyệt Hành Động (Action Safety Sandbox)**:
  - [x] Phân loại `ToolRiskLevel` (`Low`, `Medium`, `High`, `Blocked`): Hành động an toàn (cuộn, di chuột) tự động chạy; Hành động rủi ro (click, gõ phím, hotkey, shell) cần kiểm tra và bắt buộc người dùng phê duyệt; Các lệnh phá hoại (`rmdir /s`, format, xóa system32, v.v.) bị chặn vĩnh viễn (`ToolRiskLevel::Blocked`).
  - [x] Tích hợp bộ lọc ứng dụng cho phép (App Whitelist) và danh sách đen lệnh cấm (Command Blacklist).
- [x] **6.3. Vòng Lặp Xác Minh Thị Giác (Screen Verification Loop)**:
  - [x] `ScreenVerificationLoop`: Chụp lại màn hình và kiểm tra trạng thái cửa sổ foreground / visual diff để xác nhận UI đã phản hồi đúng trước khi báo hoàn tất; Ngăn chặn ảo giác hallucination tự nhận hoàn thành.

---

### Phase 7: Tích Hợp Nền Tảng Đa Phương Tiện (Unified Event Bus & Discord) — `[100% DONE]`
- [x] **7.1. Xe Buýt Sự Kiện Hợp Nhất (Unified Platform Event Bus)**:
  - [x] Chuẩn hóa mọi tín hiệu từ bên ngoài thành schema `PlatformChatMessage` đa nền tảng (`PlatformType`: Web, Discord, YouTube, Twitch, Cli, System).
  - [x] `PlatformEventBus`: Hệ thống Pub/Sub bất đồng bộ hiệu năng cao dựa trên `tokio::sync::broadcast` hỗ trợ phát và đăng ký lắng nghe mọi sự kiện luồng chat, phát ngôn nhân vật (`CharacterSpeechOutput`) và thông báo hệ thống.
- [x] **7.2. Discord Integration Adapter**:
  - [x] `DiscordAdapter`: Tiếp nhận payload tin nhắn Discord, lọc tin nhắn bot chống lặp vô tận, lọc whitelist channel và nhận diện mention (@Bot / <@ID>).
  - [x] Phân biệt và duy trì `Relationship` và `Memory` độc lập cho từng thành viên Discord khác nhau (`actor_id: "discord:<user_id>"`), tuân thủ tuyệt đối chuẩn bảo mật Actor Privacy Isolation (Skill 14 & 25).
  - [x] Cơ chế Outgoing Dispatcher gửi phản hồi đến kênh Discord (qua REST API v10 hoặc Mock/Headless Mode kiểm thử tự động offline).


---

### Phase 8: Môi Trường Vận Hành AI VTuber Thực Thụ (Stream Mode & Livestream Director) — `[100% DONE]`
- [x] **8.1. Livestream Chat Ingestion (YouTube & Twitch)**:
  - [x] `YouTubeLiveAdapter`: Tiếp nhận Live Chat snippet và SuperChat / Donation (`superchat_amount`, `currency`), chuẩn hóa thành `PlatformChatMessage` gắn liền với định danh `youtube:<channel_id>`.
  - [x] `TwitchChatAdapter`: Phân tích cú pháp IRC PRIVMSG của Twitch Chat (`twitch:<username>`), nhận diện mention (@Aria / tên kênh), hỗ trợ phản hồi trực tiếp vào phòng stream.
- [x] **8.2. Bộ Lọc Chat Thông Minh (Chat Priority Engine)**:
  - [x] `ChatPriorityEngine`: Công thức chấm điểm đa nhân tố: SuperChat / Donation (+100.0) > Trực tiếp gọi tên Mention (+35.0) > Câu hỏi tò mò (+25.0) > Tin nhắn có chiều sâu (+10.0) > Chat ngắn cụt lủn (-5.0).
  - [x] Bộ lọc chống spam lặp từ (Sliding-window hash deduplication): Phạt nặng tin nhắn duplicate (-80%), chống bão spam làm tràn LLM context.
  - [x] Gom cụm chủ đề câu hỏi (Chat Aggregation): Phân tích tự động các nhóm chủ đề nổi bật (gaming, ẩm thực, âm nhạc, tâm sự) trong phòng chat.
- [x] **8.3. Chế Độ Phát Sóng Trực Tiếp (Stream Director Mode)**:
  - [x] `StreamDirector`: Nhạc trưởng điều phối nhịp độ livestream tự trị:
    - Cân bằng tương tác chat hàng đầu, phản ứng với biến cố màn hình game (`ReactToScreen`) và trạng thái im lặng.
    - Chống nói đè / dồn dập (Enforced speech cooldown tối thiểu giữa các lần phát ngôn).
    - Bộ phát hiện im lặng (Silence detector): Tự động phát động độc thoại / tâm sự giữ nhiệt phòng stream (`SelfInitiatedBanter`) khi không gian bị lắng xuống quá ngưỡng cấu hình.


---

### Phase 9+: Hệ Sinh Thái Nâng Cao & Học Máy Thích Ứng (Ecosystem & Continuous Learning) — `[100% DONE]`
- [x] **9.1. Quản Lý Đa Nhân Vật (Multi-Character Profiles)**:
  - [x] `MultiCharacterRegistry`: Quản lý song song các thực thể nhân vật (`CharacterProfile`) với đầy đủ Personality, State, Memory và Relationship riêng biệt (như Aria, Hikari, v.v.).
  - [x] Chuyển đổi ngữ cảnh nhân vật trực tiếp trong runtime mà không gây rò rỉ hay xáo trộn dữ liệu ký ức giữa các nhân vật.
- [x] **9.2. Hội Thoại Giữa Các Nhân Vật (Character-to-Character Interaction)**:
  - [x] `CharacterToCharacterDialogue`: Điều phối hội thoại tự trị đa lượt (Turn-taking) giữa 2 nhân vật ảo.
  - [x] Tự động hình thành và phát triển mối quan hệ tương hỗ giữa các nhân vật (`target_id: "character:<id>"`).
- [x] **9.3. Học Máy Ngoại Tuyến (Offline Adaptation Model)**:
  - [x] `PreferenceDataset`: Thu thập và lưu trữ có cấu trúc các đánh giá phản hồi (`InteractionFeedback`) với thang điểm Upvote/Downvote và thẻ cảm nhận (too_long, cold, v.v.).
  - [x] Hỗ trợ kết xuất định dạng chuẩn JSONL cho quy trình tinh chỉnh mô hình DPO/RLHF ngoại tuyến.
  - [x] `OfflineAdaptationEngine`: Phân tích thống kê mức độ hài lòng, tự động tính toán các biến thiên thông số tính cách (`verbosity_delta`, `initiative_delta`) để điều chỉnh nhân vật thích ứng an toàn theo năm tháng.


---

### Reference Study Phase 2: Autonomous Life, Attention, Conversation, Emotion, Streaming & Memory — `[RESEARCH COMPLETE - AWAITING IMPLEMENTATION APPROVAL]`
- [x] Nghiên cứu chuyên sâu mã nguồn thực tế AIRI (`D:\AI-ARRI\airi-main`) & Open-LLM-VTuber (`D:\Open-LLM-VTuber-main`).
- [x] Kiểm toán đối kháng và phát hiện các khoảng trống kiến trúc thực tế trong VirtualCharacter (WebSocket monolithic, fake streaming, thiếu Barge-In, thiếu Autonomous Heartbeat loop).
- [x] Hoàn thành tài liệu thiết kế kiến trúc toàn diện: [docs/reference-study-phase2.md](file:///d:/VirtualCharactor/docs/reference-study-phase2.md).
- [ ] Triển khai P0: Parallel Streaming Brain Pipeline (Ollama Stream -> Chunker -> SAPI TTS -> RMS Slices -> VRM Lip-Sync <500ms). *(Chờ phê duyệt)*
- [ ] Triển khai P0: Conversation State Machine & Barge-In Handler với đo lường `heard_text`. *(Chờ phê duyệt)*
- [ ] Triển khai P0: Autonomous Life Engine & Circadian Heartbeat Loop (2.5s) với Subconscious Reflex. *(Chờ phê duyệt)*
- [ ] Triển khai P0: Attention Salience Temporal Gate lọc biến động màn hình trước khi gọi VLM. *(Chờ phê duyệt)*

---

*(Tài liệu này được cập nhật đồng bộ cùng `docs/vision.md`, `docs/reference-study-phase2.md` và `docs/dev_b_progress.md`).*

