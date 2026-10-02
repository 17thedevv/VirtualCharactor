# Hợp Đồng Ranh Giới Miền — VirtualCharacter Public Domain Contracts

Tài liệu này xác định các hợp đồng giao tiếp công khai (Public Domain Contracts) giữa các thành phần trong hệ thống **VirtualCharacter**, bảo vệ tính độc lập của mã nguồn, ngăn chặn trôi dạt phụ thuộc (dependency drift) và cho phép các nhà phát triển làm việc song song hiệu quả.

Mọi thay đổi đối với các hợp đồng được đánh dấu **Stable (Ổn định)** đều phải được thảo luận và đồng thuận trước khi chỉnh sửa.

---

## 1. Bảng Trạng Thái Hợp Đồng (Stability Matrix)

| Nhóm Hợp Đồng | Trait / Type Cốt Lõi | Crate Sở Hữu | Trạng Thái | Ghi Chú Ranh Giới |
|---|---|---|---|---|
| **Personality** | `Personality`, `Trait`, `Values` | `vc-core` | **Stable** | Thuần Rust, zero dependencies. |
| **CharacterState** | `CharacterState`, `CognitiveState` | `vc-core` | **Stable** | Quản lý trạng thái nội tại. |
| **EmotionState** | `EmotionState`, `EmotionAxis` | `vc-core` | **Stable** | 8 trục cảm xúc + phân rã tự nhiên. |
| **Relationship** | `Relationship`, `Stage`, `ActorId` | `vc-core` | **Stable** | Phân cách không gian quan hệ từng người dùng. |
| **Memory Domain** | `Memory`, `MemoryQuery`, `MemoryKind` | `vc-core` | **Stable** | 4 tầng ký ức (Core, Semantic, Episodic, Working). |
| **Decision Engine** | `DecisionEngine`, `DecisionCandidate` | `vc-core` | **Stable** | Sinh quyết định và nhật ký suy nghĩ nội tâm. |
| **Context Assembly** | `ContextBuilder`, `ContextBudget` | `vc-core` | **Stable** | Lắp ráp ngữ cảnh và quản lý ngân sách token. |
| **LLM Provider** | `LlmProvider`, `LlmRequest`, `LlmResponse` | `vc-core` | **Stable** | Trừu tượng hóa hoàn toàn mọi model AI. |
| **Storage Repositories**| `CharacterRepository`, `MemoryRepository`, `StateRepository`, `RelationshipRepository` | `vc-core` / `vc-storage` | **Stable** | Đã triển khai trên InMemory & Sqlite. |
| **Runtime Engine** | `RuntimeEngine`, `InteractionOutcome` | `vc-runtime` | **Stable** | Vòng đời tương tác 9 bước. |
| **Model Registry** | `ModelRegistry`, `ModelCapability` | `vc-llm` | **Active / Evolving** | Phục vụ Phase 2 (Ollama + capabilities). |
| **Embedding Engine** | `EmbeddingProvider`, `VectorQuery` | `vc-storage` | **Active / Evolving** | Vector search trên CPU (FastEmbed). |
| **Voice & Speech** | `AudioInputProvider`, `TtsProvider` | `vc-runtime` | **Design Phase** | Phase 3 (Whisper STT + Piper TTS). |
| **Avatar Body** | `AvatarController`, `AvatarEvent` | `vc-runtime` | **Design Phase** | Phase 4 (Live2D / VRM / Lip-Sync). |
| **Screen Perception**| `ScreenCaptureProvider`, `VisionProvider` | `vc-runtime` | **Design Phase** | Phase 5 (Win32 Sensing + Adaptive Vision). |
| **Computer Tools** | `ComputerTool`, `ToolPermissionPolicy` | `vc-runtime` | **Design Phase** | Phase 6 (Sandboxed OS Actions). |
| **Attention & World** | `AttentionEngine`, `WorldState` | `vc-runtime` | **Design Phase** | Phase 7 (Salience & Proactive Behavior). |
| **Social Event Bus** | `EventBus`, `ChatPlatformProvider` | `vc-runtime` | **Design Phase** | Phase 8-9 (Discord, Stream Ingestion). |

---

## 2. Quy Tắc Hiển Thị & Cô Lập (Visibility & Encapsulation Rules)

| Kiểu dữ liệu / Module | Xuất bản ra ngoài Crate? | Ranh giới kiểm soát |
|---|---|---|
| `DecisionEngine` trait | **Có** | Public contract thuộc `vc-core`. |
| `RuleDecisionEngine` | **Có** | Default rule engine trong `vc-core`. |
| `LlmProvider` trait | **Có** | Public contract thuộc `vc-core`. |
| `LlmRequest` / `LlmResponse` | **Có** | Kiểu dữ liệu trung lập, chuẩn hóa chung cho toàn hệ thống. |
| Chi tiết JSON của Gemini | **TUYỆT ĐỐI KHÔNG** | 100% riêng tư bên trong `vc-llm::gemini`. |
| Chi tiết REST của Ollama | **TUYỆT ĐỐI KHÔNG** | 100% riêng tư bên trong `vc-llm::ollama`. |
| SQLite Schema & SQL Queries | **TUYỆT ĐỐI KHÔNG** | 100% riêng tư bên trong `vc-storage::sqlite`. |
| Model weights / ONNX pointers | **TUYỆT ĐỐI KHÔNG** | Ẩn sau abstraction của từng crate ngoại vi. |
| Win32 HWND / Window Handles | **TUYỆT ĐỐI KHÔNG** | Ẩn sau `ScreenCaptureProvider` trong `vc-runtime`. |

---

## 3. Chi Tiết Các Hợp Đồng Mới (Phase 2 - Phase 9)

### 3.1 Hợp Đồng Local AI & Model Registry (Phase 2)

**Nguyên tắc kiến trúc**: `OllamaClient` chỉ đóng vai trò là một HTTP Transport Backend ngầm, không rò rỉ vào Core. Các dịch vụ của Ollama được chia tách theo từng Trait miền:
- `OllamaChatProvider` cài đặt `LlmProvider` (Chat / Phản hồi văn bản).
- `OllamaVisionProvider` cài đặt `VisionProvider` (Phân tích hình ảnh, GUI elements).
- `OllamaEmbeddingProvider` cài đặt `EmbeddingProvider` (Tạo vector nhúng).

```rust
// Thuộc vc-llm
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelCapability {
    TextGeneration,
    VisionUnderstanding,
    EmbeddingGeneration,
    ToolCalling,
}

#[derive(Debug, Clone)]
pub struct ModelProfile {
    pub name: String,
    pub provider: String, // "ollama", "gemini", etc.
    pub vram_mb_required: u32,
    pub ram_mb_required: u32,
    pub capabilities: Vec<ModelCapability>,
    pub context_window: usize,
}

pub trait ModelRegistry: Send + Sync {
    fn get_active_chat_model(&self) -> ModelProfile;
    fn get_active_vision_model(&self) -> Option<ModelProfile>;
    fn list_available_models(&self) -> Vec<ModelProfile>;
    fn supports_capability(&self, model: &str, cap: ModelCapability) -> bool;
}
```


### 3.2 Hợp Đồng Tìm Kiếm Ngữ Nghĩa (Vector Search - Phase 2)
```rust
// Thuộc vc-storage
pub trait EmbeddingProvider: Send + Sync {
    fn embed_text(&self, text: &str) -> Result<Vec<f32>, StorageError>;
    fn dimension(&self) -> usize;
}

pub struct VectorMemoryQuery {
    pub actor_id: ActorId,
    pub query_vector: Vec<f32>,
    pub top_k: usize,
    pub min_similarity: f32, // Cosine threshold: 0.0 - 1.0
}
```

### 3.3 Hợp Đồng Âm Thanh & Giọng Nói (Phase 3)
```rust
// Thuộc vc-runtime
pub trait AudioInputProvider: Send + Sync {
    fn start_listening(&self) -> Result<(), RuntimeError>;
    fn stop_listening(&self) -> Result<(), RuntimeError>;
    fn poll_transcription(&self) -> Option<String>;
}

pub trait TtsProvider: Send + Sync {
    fn synthesize(&self, text: &str, modulation: &VoiceModulation) -> Result<Vec<u8>, RuntimeError>;
}

#[derive(Debug, Clone)]
pub struct VoiceModulation {
    pub pitch_modifier: f32, // 1.0 = normal, 1.2 = higher (happy)
    pub speed_modifier: f32, // 1.0 = normal, 0.9 = slower (sad)
    pub energy_level: f32,   // 1.0 = normal, 1.4 = excited
}
```

### 3.4 Hợp Đồng Điều Khiển Thân Thể Avatar (Phase 4)
```rust
// Thuộc vc-runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvatarExpression {
    Neutral,
    Joy,
    Sadness,
    Anger,
    Surprise,
    Embarrassed,
    Thinking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarCommand {
    pub expression: AvatarExpression,
    pub eye_blink: bool,
    pub mouth_open_ratio: f32, // 0.0 - 1.0 cho lip-sync
    pub head_tilt: (f32, f32), // (pitch, yaw)
}

pub trait AvatarController: Send + Sync {
    fn dispatch_command(&self, cmd: AvatarCommand) -> Result<(), RuntimeError>;
}
```

### 3.5 Hợp Đồng Thị Giác Thích Ứng & Định Tuyến (Phase 5)
```rust
// Thuộc vc-runtime
#[derive(Debug, Clone)]
pub struct ScreenDiffInfo {
    pub has_significant_change: bool,
    pub diff_ratio: f32, // 0.0 - 1.0
    pub active_window_title: String,
}

pub trait ScreenCaptureProvider: Send + Sync {
    fn sense_screen_changes(&self) -> Result<ScreenDiffInfo, RuntimeError>;
    fn capture_region(&self, x: u32, y: u32, w: u32, h: u32) -> Result<Vec<u8>, RuntimeError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionMode {
    Normal,      // Qwen3-VL 2B (Mặc định cho UI, game, desktop)
    HighQuality, // Qwen2.5-VL 3B (Phân tích chi tiết, tọa độ chính xác)
    UltraLight,  // Moondream (Kiểm tra nhanh trên CPU)
    CloudFallback, // Gemini 1.5 Flash Vision (Khi GPU bận hoặc cần ảnh 4K)
}

#[derive(Debug, Clone)]
pub struct VisionObservation {
    pub summary: String,
    pub detected_elements: Vec<String>,
    pub ui_text: Vec<String>,
    pub confidence: f32,
}

#[async_trait::async_trait]
pub trait VisionProvider: Send + Sync {
    async fn analyze_image(&self, image_bytes: &[u8], prompt: &str) -> Result<VisionObservation, RuntimeError>;
    fn provider_name(&self) -> &'static str;
}
```


### 3.6 Hợp Đồng Tin Nhắn Đa Nền Tảng (Phase 8)
```rust
// Thuộc vc-runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedChatMessage {
    pub platform: String, // "discord", "youtube", "twitch", "web"
    pub channel_id: String,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub timestamp: i64,
    pub is_mention: bool,
    pub priority_score: f32,
}
```

---

## 4. Chính Sách Điều Chỉnh Hợp Đồng (Change Policy)

1. **Bổ sung không phá vỡ (Additive changes)**:
   - Thêm phương thức mới vào trait có kèm `default implementation`, hoặc thêm trường dữ liệu `Option<T>` với `#[serde(default)]` được coi là an toàn.
2. **Thay đổi có tính phá vỡ (Breaking changes)**:
   - Xóa bỏ method, sửa đổi kiểu tham số của các trait đánh dấu **Stable** phải tạo Architecture Decision Record (ADR) trong `docs/adr/` và thông báo trước khi commit.
3. **Tuân thủ quy tắc kiểm thử**:
   - Bất kỳ thay đổi hợp đồng nào cũng phải cập nhật mock objects trong `vc-runtime` và đảm bảo toàn bộ workspace tests tiếp tục pass.
