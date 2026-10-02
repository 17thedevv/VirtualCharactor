Phase 1 — Parallel Development
                    VirtualCharacter
                           │
          ┌────────────────┴────────────────┐
          │                                 │
       DEV A                            DEV B
  Character Core                  Runtime / Infrastructure
          │                                 │
          ↓                                 ↓
 Personality                         Context
 State + Emotion                     Runtime
 Relationship                        LLM
 Memory                              Storage
 Decision
          │                                 │
          └──────────────┬──────────────────┘
                         ↓
                    Integration
DEV A — Character Intelligence

Ownership chính:

crates/vc-core/
├── personality/
├── state/
├── relationship/
├── memory/
└── decision/
A1. Personality [DONE]

Xây Personality thành domain model thực sự (Đã hoàn tất theo docs/design/personality.md).

Phụ trách:

Identity
Traits
Values
Preferences
BehaviorTendencies
CommunicationStyle
DecisionTendencies
Boundaries
Goals

Cần có:

validation
default/baseline
serialization
equality/comparison nếu cần
unit tests

Chưa làm:

prompt generation
LLM personality extraction
personality learning
A2. State + Emotion [DONE]

Đây là phần quan trọng (Đã hoàn tất toàn bộ State Transition, Multi-Axis Emotion, RuleEmotionEngine và đồng bộ Web/CLI).

state/
├── mod.rs
├── emotion.rs
├── cognitive.rs
├── behavior.rs
├── goals.rs
└── session.rs

Phụ trách:

EmotionState
CognitiveState
BehaviorState
Goals
SessionState
CharacterState

Đặc biệt thiết kế:

State Transition
State validation
State mutation
State snapshot
State persistence representation

Emotion cần có nền tảng để sau này thêm:

EmotionEngine
RuleBased
Learned
Hybrid

Nhưng Phase 1 chưa cần ML.

A3. Relationship [DONE]

(Đã hoàn tất toàn bộ Bipartite Relationship, Multi-Actor Isolation, 5 Metrics [closeness, trust, familiarity, affection, tension], Stage Progression, Relationship Transition & Damping, và đồng bộ Web HUD).

Phụ trách:

Relationship
RelationshipId
RelationshipState

và các thuộc tính kiểu:

closeness
trust
familiarity
affection
tension
known_facts / relationship-scoped knowledge

Quan trọng:

CharacterState ≠ RelationshipState

Relationship phải hỗ trợ:

Character A ↔ User X
Character A ↔ User Y

một cách độc lập.

A4. Memory [DONE]

(Đã hoàn tất toàn bộ Domain Memory Model, 4 Tầng Bộ Nhớ, Lifecycle Decay & Reinforcement, MemoryQuery & Actor Isolation, InMemoryMemoryStore và tích hợp Server/Web).

Phụ trách:

Memory
MemoryType
MemoryMetadata
MemoryQuery
MemoryReference
MemoryImportance

Và:

Memory creation
Memory update
Memory retrieval interface
Memory importance
Recency
Memory lifecycle

Nên thiết kế sẵn abstraction cho:

MemoryFormation
MemoryRetrieval
MemoryRanking
MemoryConsolidation
MemoryDecay
MemoryConflictResolution

nhưng chưa cần implementation thông minh.

Ví dụ:

pub trait MemoryRetriever {
    fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<Memory>>;
}
A5. Decision [DONE]

(Đã hoàn tất toàn bộ Decision Domain Module, Action Types, DecisionCandidate, Scoring Heuristics, Inner Monologue Reasoning, BehaviorPolicy, RuleDecisionEngine và tích hợp Server/Web/CLI).

Phụ trách:

Decision
Action
DecisionCandidate
DecisionResult
DecisionEngine

Phase 1 làm deterministic baseline.

Ví dụ:

Input
 ↓
Situation
 ↓
Generate candidates
 ↓
score
 ↓
select

Chưa làm learned model.

Quan trọng nhất là Decision phải trả lời:

"Character nên làm gì?"

chứ không trả về câu text.

Dev A — Definition of Done

Cuối phần A phải có:

✓ Personality domain hoàn chỉnh
✓ State domain hoàn chỉnh
✓ Emotion domain hoàn chỉnh
✓ Relationship domain hoàn chỉnh
✓ Memory foundation
✓ Deterministic DecisionEngine
✓ Unit tests
✓ Domain invariants
✓ Không dependency vào Gemini/DB/runtime
DEV B — Runtime / Context / LLM / Storage

Ownership chính:

crates/vc-core/context/
crates/vc-runtime/
crates/vc-llm/
crates/vc-storage/
B1. Context [DONE]

(Đã hoàn tất toàn bộ Context Module, ContextItem, ContextSource, ContextPriority, ContextBudget, ContextBuilder, ContextPrioritizer, tích hợp WebSocket Live Token Breakdown & Mind Inspector).

Phụ trách:

Context
ContextItem
ContextSource
ContextPriority
ContextBudget

Xây:

ContextBuilder
ContextSelector
ContextPrioritizer

Pipeline:

Personality
State
Relationship
Memory
Conversation
Current Situation
       ↓
Context Builder
       ↓
Optimized Context

Phase 1 chưa cần context algorithm quá thông minh.

Chỉ cần deterministic:

critical > high > medium > low

và có budget.

B2. Runtime [DONE]

(Đã hoàn tất toàn bộ Interaction Lifecycle, CharacterSession & SessionManager, State Feedback, Memory Consolidation, và tích hợp Server/CLI/Web).

Phụ trách:

CharacterSession
SessionId & SessionStatus State Machine (Created, Active, Idle, Completed, Expired, Cancelled)
SessionManager (Thread-safe concurrency, idle timeout detection)
Interaction, InteractionId, InteractionStatus, InteractionOutcome
RuntimeEngine (Orchestrating the 9-stage lifecycle)

Pipeline đã hoàn thiện:

User Input
     ↓
Session Touch / Resolution (SessionManager)
     ↓
Memory Retrieval (Actor-Isolated MemoryQuery)
     ↓
Context Construction & Budget Governance (ContextBuilder)
     ↓
Decision Evaluation (DecisionEngine)
     ↓
LLM Generation (LlmProvider / Gemini / Mock)
     ↓
Emotion & State Feedback Update (apply_delta, decay, sync_behavior)
     ↓
Relationship Evolution (RelationshipTransition)
     ↓
Memory Consolidation (Episodic Memory Formation)
     ↓
InteractionOutcome
B3. LLM [DONE]

(Đã hoàn tất toàn bộ Provider Abstraction, Provider-Neutral Types, MockLlmProvider hoàn chỉnh, và GeminiProvider với Redacted Secrets, GenerationConfig, Usage Metadata & Error Mapping).

Phụ trách:

LlmProvider (Trait trừu tượng thuần túy)
LlmRequest (Builder: prompt, system_instruction, temperature, max_tokens)
LlmResponse (text, LlmUsage, finish_reason)
LlmUsage (prompt_tokens, completion_tokens, total_tokens)
LlmError (RateLimited, AuthenticationFailed, InvalidRequest, NetworkError, Timeout, ModelUnavailable, Other)

Implementation:

MockLlmProvider:
- Hoàn chỉnh cho unit & integration tests mà không phụ thuộc network hay API keys.
- Hỗ trợ hàng đợi câu trả lời kịch bản sẵn (`with_responses`, `push_canned_response`).
- Hỗ trợ mô phỏng lỗi provider (`failing`, `set_simulated_error`) để kiểm thử runtime resilience.
- Ghi nhận lịch sử requests (`recorded_requests`, `last_request`, `request_count`) cho test assertions.

GeminiProvider:
- GeminiConfig cấu hình linh hoạt (model, temperature, max_output_tokens, timeout, max_retries).
- Secrets Management: Masking API key trong format `Debug` (`AIza...[REDACTED]`).
- Hỗ trợ `generationConfig` (temperature, maxOutputTokens) gửi tới Gemini API.
- Trích xuất `usageMetadata` (`promptTokenCount`, `candidatesTokenCount`, `totalTokenCount`) vào `LlmUsage`.
- Cơ chế retry tự động cho lỗi tạm thời (429 RateLimit, 503 Service Unavailable) với exponential backoff.
- Toàn bộ kiểu dữ liệu nội bộ của Gemini được cô lập 100% bên trong `vc-llm::gemini`, tuyệt đối không rò rỉ ra `vc-core` hay `vc-runtime`.
B4. Storage [DONE]

(Đã hoàn tất toàn bộ InMemoryStorage, SqliteStorage Local-First, Schema Auto-Migration, và 4 Repository Traits: CharacterRepository, MemoryRepository, StateRepository, RelationshipRepository).

Phụ trách:

CharacterRepository
MemoryRepository
StateRepository
RelationshipRepository

Đã triển khai:

1. InMemoryStorage:
   - Lưu trữ dạng `Arc<RwLock<HashMap<...>>>` phục vụ testing và chạy không cần ổ đĩa.
   - Hỗ trợ đầy đủ logic truy vấn bộ nhớ đa chiều và cô lập actor (Skill 12).

2. SqliteStorage:
   - Lưu trữ bền vững Local-First trên file SQLite (`rusqlite` bundled).
   - Tự động tạo schema và indexes (`characters`, `relationships`, `memories`).
   - Serialization JSON an toàn, mapping 1-1 giữa Domain model và Persistence model (Skill 21).
   - Đảm bảo an toàn luồng và đồng bộ tuần tự ghi bằng internal mutex.

B5. Integration [DONE]

(Đã hoàn tất tích hợp End-to-End toàn diện giữa Storage, Runtime, Server và CLI).

Pipeline hoạt động:

CLI / Web Client
     ↓
Runtime Engine (9-Stage Lifecycle)
     ↓
Load / Seed Entity (SqliteStorage)
     ↓
Actor-Isolated Memory Retrieval
     ↓
Context Construction & Budget Governance
     ↓
Decision Engine (Candidate Evaluation)
     ↓
LLM Generation (Gemini API / Mock)
     ↓
State & Emotion Transition
     ↓
Relationship Evolution & Damping
     ↓
Memory Formation (Episodic Consolidation)
     ↓
Commit Updates to SQLite (State, Relationship, Memory)
     ↓
Client Response (WebSocket Stream / CLI Display)

Đã kiểm thử:
- Khởi động lại runtime, CLI và server giữ nguyên vẹn 100% cảm xúc, ký ức và mức độ quan hệ.
- Bộ 114 automated tests của workspace pass sạch sẽ.

Dev B — Definition of Done
✓ Context builder
✓ Context budget
✓ Context prioritization
✓ Runtime orchestration
✓ LlmProvider
✓ MockLlmProvider
✓ GeminiProvider
✓ Repository traits
✓ In-memory storage
✓ SQLite foundation
✓ Integration tests
✓ CLI chạy end-to-end
---

## Quy Tắc Phân Quyền Sở Hữu (Ownership Matrix)

| Khu vực / Module | Dev A (Character Intelligence) | Dev B (Runtime & Boundaries) | Ghi Chú |
|---|---|---|---|
| Personality | ✅ | ❌ | Dev A sở hữu |
| State & Emotion | ✅ | ❌ | Dev A sở hữu |
| Relationship Engine | ✅ | ❌ | Dev A sở hữu |
| Memory Domain Types & Logic | ✅ | ❌ | Dev A sở hữu logic |
| Decision Engine Trait & Rules | ✅ | ❌ | Dev A sở hữu |
| Context Builder & Budget | ⚠️ (Review) | ✅ | Dev B sở hữu |
| Runtime & SessionManager | ⚠️ (Review) | ✅ | Dev B sở hữu |
| LLM & Ollama Provider | ❌ | ✅ | Dev B sở hữu |
| Storage & Vector Embeddings | ❌ | ✅ | Dev B sở hữu |
| Voice & Audio Pipeline | ⚠️ (Emotion modulate) | ✅ (Capture/TTS engine) | Phối hợp |
| Avatar Controller | ⚠️ (Expression map) | ✅ (WebSocket stream) | Phối hợp |
| Vision & Screen Sensing | ⚠️ (Attention filter) | ✅ (Win32 capture/OCR) | Phối hợp |
| Server, CLI, Web HUD | Review | ✅ | Dev B sở hữu |

---

## BẢNG NHIỆM VỤ CHI TIẾT TỪNG PHASE (Sprint Backlog)

### PHASE 2 — Local AI (Ollama) & Bộ Nhớ Ngữ Nghĩa 2.0 `[ACTIVE SPRINT]`

#### Nhiệm vụ của Dev B (Infrastructure & Local AI)
- [x] **B2.1: Ollama Provider Cài Đặt Hoàn Chỉnh (`vc-llm`) [DONE]**
  - [x] Tạo `OllamaClient` kết nối tới `http://localhost:11434` qua `/api/chat` làm đường chính.
  - [x] Cấu hình model mặc định: `qwen2.5:3b` trong `OllamaConfig`, không hardcode rải rác.
  - [x] Chuyển đổi `LlmRequest` (system instruction + user prompt + options) sang payload Ollama chuẩn.
  - [x] Map đầy đủ `LlmUsage` (eval_count, prompt_eval_count) và mapping chi tiết lỗi (Connection refused, 404, 400, 500, timeout).
  - [x] Thêm bộ test `crates/vc-llm/tests/ollama_tests.rs` với mock HTTP server độc lập (9/9 tests passed).

- [ ] **B2.2: Model Registry & Model Capabilities (`vc-llm`)**
  - [ ] Định nghĩa `ModelCapability` enum (`Text`, `Vision`, `Embedding`, `ToolCalling`).
  - [ ] Cài đặt `ModelRegistry` quản lý metadata model, kích thước VRAM ước tính và context window.
  - [ ] Hỗ trợ AI Router: tự động chọn local Ollama khi có sẵn, fallback sang Gemini khi model yêu cầu không đáp ứng.
- [ ] **B2.3: Local Text Embeddings trên CPU (`vc-storage`)**
  - [ ] Tích hợp crate `fastembed-rs` (model `bge-small-en-v1.5` / `multilingual`).
  - [ ] Chạy hoàn toàn trên **CPU ONNX Runtime**, sinh vector 384 chiều, **0 MB VRAM**.
- [ ] **B2.4: Tìm Kiếm Ký Ức Theo Vector (Cosine Semantic Search) (`vc-storage`)**
  - [ ] Cập nhật bảng `memories` trong SQLite bổ sung cột lưu trữ vector hoặc bảng vector riêng.
  - [ ] Cài đặt hàm `find_similar_memories(actor_id, query_embedding, top_k, threshold)` tính toán cosine similarity.
  - [ ] Bảo vệ nghiêm ngặt tính riêng tư: chỉ tìm kiếm ký ức thuộc về chính `ActorId` đang tương tác.
- [ ] **B2.5: Giám Sát Tài Nguyên VRAM Cơ Bản (`vc-runtime`)**
  - [ ] Thêm module `hardware` kiểm tra dung lượng VRAM/RAM hệ thống trước khi nạp model nặng.

#### Nhiệm vụ của Dev A (Character Memory & Consolidation)
- [ ] **A2.1: Công Thức Trọng Số & Phân Rã Ký Ức (`vc-core::memory`)**
  - [ ] Cài đặt hàm `apply_time_decay()` làm giảm dần độ ưu tiên của các episodic memory vụn vặt theo thời gian.
  - [ ] Bổ sung trường `importance_score` (1-10) và `last_accessed_at`.
- [ ] **A2.2: Giải Quyết Xung Đột Ký Ức (Conflict Resolution) (`vc-core::memory`)**
  - [ ] Xử lý logic khi người dùng cập nhật thông tin mâu thuẫn với ký ức cũ (ví dụ: "Tôi thích màu đỏ" -> "Bây giờ tôi thích màu xanh").
- [ ] **A2.3: Logic Tổng Hợp Ký Ức (Sleep & Idle Consolidation) (`vc-core::memory`)**
  - [ ] Thiết kế logic nén nhiều mẩu ký ức Episodic trong ngày thành một Semantic Memory cốt lõi.

---

### PHASE 3 — Voice In/Out & Audio Pipeline (Tai & Giọng Nói)
- [ ] **B3.1: Audio Capture & VAD (`vc-runtime`)**
  - [ ] Thu âm microphone qua `cpal` trên Windows WASAPI.
  - [ ] Tích hợp Silero VAD trên CPU phát hiện giọng nói và ngắt audio chunk.
- [ ] **B3.2: Local STT với Whisper (`vc-runtime`)**
  - [ ] Tích hợp `whisper.cpp` (chạy trên CPU threads) lượng tử hóa int8.
  - [ ] Hỗ trợ chế độ Push-to-Talk và Continuous Listening có VAD.
- [ ] **B3.3: Local TTS với Piper & Kokoro (`vc-runtime`)**
  - [ ] Tích hợp Piper TTS (C++ CPU engine) tốc độ siêu nhanh (RTF < 0.2).
  - [ ] Phát âm thanh mượt mà qua loa bằng `rodio`.
- [ ] **A3.1: Emotion-to-Speech Modulator (`vc-core` -> `vc-runtime`)**
  - [ ] Ánh xạ `(valence, arousal)` sang `pitch_modifier`, `speed_modifier`, và `energy_level` tự nhiên.

---

### PHASE 4 — Avatar Thân Thể & Real-Time Lip-Sync
- [ ] **B4.1: Avatar Control Protocol (`vc-server`)**
  - [ ] Thiết lập kênh WebSocket chuyên biệt stream `AvatarCommand` (Expression, Blink, Mouth, Head).
- [ ] **B4.2: Tích hợp Live2D & 3D VRM (`apps/vc-web`)**
  - [ ] Nhúng Live2D Cubism Web SDK và Three.js `@pixiv/three-vrm`.
  - [ ] Cài đặt Auto-Blink ngẫu nhiên và chuyển động mắt tự nhiên (Idle Eye Gaze).
- [ ] **B4.3: Real-Time Audio Lip-Sync (`apps/vc-web` / `vc-runtime`)**
  - [ ] Phân tích biên độ âm thanh RMS/Viseme từ audio TTS để điều khiển nhép miệng chính xác.
- [ ] **A4.1: Mapping Cảm Xúc Sang Blendshapes Avatar (`vc-core`)**
  - [ ] Chuyển đổi 8 trục cảm xúc sang biểu cảm: Joy, Sadness, Anger, Surprise, Shy, Smug.

---

### PHASE 5 — Thị Giác Thích Ứng (Adaptive Screen Perception & VisionRouter)
- [ ] **B5.1: Level 1 - Screen Change Sensing (`vc-runtime`)**
  - [ ] Win32 Desktop Duplication API chụp màn hình nhanh.
  - [ ] Thuật toán so sánh pixel diff trên CPU (0 VRAM). Bỏ qua khi màn hình tĩnh.
- [ ] **B5.2: Level 2 - Fast Text/UI Perception (`vc-runtime`)**
  - [ ] Lấy tên cửa sổ active, chạy OCR siêu nhẹ (Tesseract/RapidOCR trên CPU).
- [ ] **B5.3: Level 3 - VisionProvider & VisionRouter (`vc-runtime` / `vc-llm`)**
  - [ ] Xây dựng trait `VisionProvider` (OllamaQwen3VL, OllamaQwen25VL, CloudGeminiVision, MockVision).
  - [ ] Cài đặt `VisionRouter` với model mặc định `qwen3-vl:2b` (1.9 GB) tối ưu cho GUI/UI element detection và Computer Use.
  - [ ] Hỗ trợ GPU Model Multiplexing trong `ResourceManager` (hoán đổi quyền ưu tiên VRAM giữa Chat LLM và Vision LLM).
- [ ] **A5.1: Perception Scheduler & Visual Memory (`vc-core`)**
  - [ ] Lưu trữ `VisualMemory` mô tả khung cảnh thay vì lưu ảnh thô.


---

### PHASE 6 — Tương Tác Máy Tính An Toàn (Sandboxed Computer Use)
- [ ] **B6.1: Win32 OS Tools (`vc-runtime`)**
  - [ ] Mở ứng dụng trong whitelist, click chuột, gõ phím, cuộn trang.
- [ ] **B6.2: Permission Sandbox & Policy Manager (`vc-runtime`)**
  - [ ] Phân cấp rủi ro (`SafeRead`, `AppControl`, `Dangerous`).
  - [ ] Chặn tuyệt đối xóa file hệ thống, can thiệp registry.
  - [ ] Phím tắt khẩn cấp (Panic Key) ngắt ngay lập tức mọi quyền điều khiển.
- [ ] **B6.3: Action-Verification Loop (`vc-runtime`)**
  - [ ] Chụp màn hình vùng chọn kiểm chứng kết quả sau khi thực hiện hành động.

---

### PHASE 7 — Hệ Thống Chú Ý & Tính Tự Chủ (Attention & Autonomy)
- [ ] **A7.1: World State Domain Model (`vc-core`)**
  - [ ] Theo dõi ứng dụng hiện tại, thời gian trong ngày, sự hiện diện của user.
- [ ] **A7.2: Attention Engine & Salience Scoring (`vc-core`)**
  - [ ] Tính điểm chú ý dựa trên độ khẩn cấp, độ tò mò, quan hệ user và cooldown.
- [ ] **A7.3: Idle Behaviors & Proactive Remarks (`vc-core`)**
  - [ ] Hành vi chủ động bắt chuyện tự nhiên khi có sự kiện đáng chú ý.

---

### PHASE 8 — Mạng Xã Hội (Discord & Livestream Unified Chat)
- [ ] **B8.1: Unified Chat Event Bus (`vc-runtime`)**
  - [ ] Chuẩn hóa tin nhắn đa nền tảng thành `NormalizedChatMessage`.
- [ ] **B8.2: Discord Adapter (`vc-integrations-discord`)**
  - [ ] Tích hợp bot Discord qua `serenity-rs`, hỗ trợ quan hệ riêng từng người bạn.
- [ ] **B8.3: YouTube & Twitch Chat Ingestion (`vc-integrations-stream`)**
  - [ ] Kết nối đọc chat livestream thời gian thực.
- [ ] **B8.4: Chat Priority Engine & Spam Filter (`vc-runtime`)**
  - [ ] Lọc spam, nhận diện câu hỏi hay, ưu tiên SuperChat/Người quen.

---

### PHASE 9 — AI VTuber Runtime & Chế Độ Phát Trực Tiếp (Stream Mode)
- [ ] **B9.1: Stream Mode Orchestrator (`vc-runtime`)**
  - [ ] Vòng lặp tự động: Đọc chat -> Suy nghĩ -> Nói chuyện -> Cử động Avatar -> OBS.
- [ ] **B9.2: Tích hợp OBS Studio (`vc-runtime` / `apps/vc-web`)**
  - [ ] Xuất hình ảnh nền trong suốt qua Spout2 / Web Browser Source vào OBS.
  - [ ] Xuất âm thanh qua Virtual Audio Cable tới microphone của stream.

---

### PHASE 10 — Tự Thích Ứng Dài Hạn & Đa Nhân Vật (Continual Growth)
- [ ] **A10.1: Sleep Memory Consolidation Worker (`vc-runtime`)**
  - [ ] Chạy ngầm khi nhàn rỗi để đúc kết tri thức và tinh chỉnh ký ức dài hạn.
- [ ] **A10.2: Multi-Character Profile Manager (`vc-core`)**
  - [ ] Cho phép chuyển đổi linh hoạt nhiều nhân vật (Aria, Nero, v.v.) dùng chung cùng một Core.