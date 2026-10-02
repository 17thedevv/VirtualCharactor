# Kiến Trúc Hệ Thống — VirtualCharacter Architecture

Tài liệu này xác định kiến trúc toàn diện của **VirtualCharacter**, chuyển đổi hệ thống từ một "Cognitive Runtime xử lý hội thoại" thành một **Hệ điều hành Nhân vật Ảo Nhập thể Cục bộ (Local-First Embodied AI Character & VTuber Runtime)**.

---

## 1. Triết Lý Kiến Trúc Cốt Lõi (Core Principles)

### 1.1 "Character ≠ LLM" (Nhân vật không phải là LLM)
- Mô hình ngôn ngữ lớn (LLM) chỉ là một **bộ máy chuyển hóa ngôn ngữ và suy luận nhận thức cục bộ** (Cognitive Component).
- Toàn bộ bản sắc nhân vật (**Personality**), trạng thái cảm xúc đa trục (**Emotion 8 trục**), quan hệ xã hội phân cách (**Relationship Engine**), ký ức dài hạn (**4-Tier Memory**), và động cơ hành động (**Decision Engine**) nằm hoàn toàn trong mã nguồn Rust độc lập (`vc-core`).
- Thay thế hoặc nâng cấp LLM (từ Ollama Qwen sang Llama, Gemma, hoặc Gemini Cloud) không làm thay đổi hay xóa nhòa bản sắc của nhân vật.

### 1.2 "Local-First + Optional Cloud Fallback"
- Vận hành bình thường 100% offline không cần Internet khi có Ollama, mô hình nhúng CPU, mô hình giọng nói Piper, và SQLite.
- Dịch vụ Cloud (Gemini, ElevenLabs) chỉ đóng vai trò dự phòng mở rộng (fallback/upgrade), không được trở thành điểm thắt phụ thuộc duy nhất.

### 1.3 "Embodied AIVTuber Runtime" (Thực thể Nhập thể)
- Nhân vật có đầy đủ: **Não (Brain), Giác quan (Senses: Nghe, Nhìn), Cơ thể (Body: Live2D/VRM), Bàn tay (Hands: Computer Tools), và Xã hội (Social: Discord, Stream Chat)**.

---

## 2. Phân Cấp và Ranh Giới Crate (Crate Boundaries)

```
                            ┌────────────────────────┐
                            │   APPS LAYER (Front)   │
                            │  vc-cli   vc-server    │
                            │        vc-web          │
                            └───────────┬────────────┘
                                        │
                                        ▼
                            ┌────────────────────────┐
                            │    ORCHESTRATION       │
                            │       vc-runtime       │
                            │ (Attention, Scheduler, │
                            │  Resource Manager, Bus)│
                            └─────┬────────────┬─────┘
                                  │            │
            ┌─────────────────────┘            └─────────────────────┐
            ▼                                                        ▼
┌────────────────────────┐                                ┌────────────────────────┐
│     COGNITIVE ADAPTERS │                                │    PERSISTENCE LAYER   │
│         vc-llm         │                                │       vc-storage       │
│ (Ollama, Gemini, Mock, │                                │ (SQLite, In-Memory,    │
│     ModelRegistry)     │                                │   Vector Embeddings)   │
└───────────┬────────────┘                                └──────────┬─────────────┘
            │                                                        │
            └─────────────────────┐            ┌─────────────────────┘
                                  ▼            ▼
                            ┌────────────────────────┐
                            │      DOMAIN CORE       │
                            │        vc-core         │
                            │ (Pure Rust, Zero Deps, │
                            │  Entities, Contracts)  │
                            └────────────────────────┘
```

### Chi tiết vai trò từng Crate:

| Crate / Thư mục | Trách nhiệm kiến trúc | Phụ thuộc bên ngoài |
|---|---|---|
| **`crates/vc-core`** | **Trái tim hệ thống**. Chứa các thực thể miền thuần túy: `Personality`, `CharacterState`, `EmotionState`, `Relationship`, `Memory`, `DecisionEngine`, `WorldState`, và các Interface Trait cốt lõi. | **Tuyệt đối 0 phụ thuộc IO/Network/DB**; chỉ dùng standard library, `serde`, `uuid`. |
| **`crates/vc-runtime`** | **Nhạc trưởng điều phối (Orchestrator)**. Quản lý vòng đời tương tác 9 bước, `SessionManager`, `AttentionEngine`, `ResourceManager` (bảo vệ 4GB VRAM), `VoiceModulator`, và `EventBus`. | Phụ thuộc: `vc-core`, `vc-llm`, `vc-storage`. |
| **`crates/vc-llm`** | **Bộ chuyển đổi trí tuệ**. Cung cấp `OllamaProvider`, `GeminiProvider`, `MockLlmProvider`, `ModelRegistry`, và `ModelCapability`. Cô lập 100% định dạng JSON của các nhà cung cấp bên ngoài. | Phụ thuộc: `vc-core`, `reqwest`, `tokio`. |
| **`crates/vc-storage`** | **Lưu trữ dữ liệu bền vững**. Cài đặt các repository: `CharacterRepository`, `MemoryRepository`, `StateRepository`, `RelationshipRepository` trên nền SQLite và In-Memory. Quản lý schema migration và vector cosine search. | Phụ thuộc: `vc-core`, `rusqlite`, `fastembed`. |
| **`apps/vc-server`** | **Cổng giao tiếp đa phương thức**. Xây dựng trên Axum, cung cấp REST API và WebSocket Full-Duplex cho Frontend Web HUD, đồng thời đóng vai trò Gateway nhận webhook/event từ Discord và Livestream. | Phụ thuộc: `vc-runtime`, `vc-core`, `axum`, `tokio`. |
| **`apps/vc-cli`** | **Ứng dụng Console cục bộ**. Dùng để debug nhanh, tương tác bằng bàn phím mà không cần mở trình duyệt hay UI đồ họa. | Phụ thuộc: `vc-runtime`, `vc-core`, `vc-storage`, `vc-llm`. |
| **`apps/vc-web`** | **Web HUD & Body Renderer**. Giao diện người dùng thời gian thực, hiển thị chỉ số cảm xúc 8 trục, quan hệ, lịch sử ký ức, và nhúng Avatar Live2D / 3D VRM Canvas. | Vanilla TS / Vite, kết nối qua WebSocket tới `vc-server`. |

---

## 3. Kiến Trúc 6 Trụ Cột Nhập Thể (Embodied Pillars)

```
                       ┌─────────────────────────────────┐
                       │       SENSES (Giác Quan)        │
                       │  • Win32 Screen Sensing (CPU)   │
                       │  • Whisper Audio In (CPU)       │
                       │  • Multi-Platform Chat Events   │
                       └────────────────┬────────────────┘
                                        │
                                        ▼
                       ┌─────────────────────────────────┐
                       │       ATTENTION ENGINE          │
                       │  • Salience Calculation         │
                       │  • Urgency & Interest Filter    │
                       │  • Token & Frequency Budget     │
                       └────────────────┬────────────────┘
                                        │
                                        ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              BRAIN (Bộ Não)                                     │
│                                                                                 │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │                              WORLD STATE                                │   │
│   │ Current App • Active Window • User Present • Voice Activity • Chat Rate │   │
│   └────────────────────────────────────┬────────────────────────────────────┘   │
│                                        │                                        │
│   ┌───────────────────┐       ┌────────┴──────────┐       ┌─────────────────┐   │
│   │    PERSONALITY    │       │  EMOTION (8 Axes) │       │  RELATIONSHIP   │   │
│   │ Core Values/Style │       │ Valence, Arousal  │       │ Trust, Affinity │   │
│   └─────────┬─────────┘       └────────┬──────────┘       └────────┬────────┘   │
│             │                          │                           │            │
│             └──────────────────────────┼───────────────────────────┘            │
│                                        ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │                              4-TIER MEMORY                              │   │
│   │    Working Memory  •  Episodic Memory  •  Semantic Memory  •  Core      │   │
│   └────────────────────────────────────┬────────────────────────────────────┘   │
│                                        ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │                             DECISION ENGINE                             │   │
│   │               Candidates • Inner Monologue • Trace Log                  │   │
│   └────────────────────────────────────┬────────────────────────────────────┘   │
└────────────────────────────────────────┼────────────────────────────────────────┘
                                         │
                                         ▼
                       ┌─────────────────────────────────┐
                       │         ACTION PLANNER          │
                       │    (Speech, Tool, Expression)   │
                       └──────┬──────────┬──────────┬────┘
                              │          │          │
                              ▼          ▼          ▼
                        ┌──────────┐┌──────────┐┌──────────┐
                        │   BODY   ││  HANDS   ││  VOICE   │
                        │ Live2D/  ││ Win32 UI ││ Piper    │
                        │ VRM Mesh ││ Sandbox  ││ Speech   │
                        └──────────┘└──────────┘└──────────┘
```

---

## 4. Chu Trình Nhận Thức & Hành Động An Toàn (Safe Computer Perception Loop)

Tuyệt đối không để Model Vision tự ý quyết định click chuột hay ra lệnh điều khiển máy tính. VLM **chỉ là giác quan quan sát (Perception)**, không phải là cơ quan ra quyết định hành vi. Chu trình vận hành diễn ra nghiêm ngặt như sau:

```
                      SCREEN (Màn hình Desktop)
                                 │
                                 ▼
                     SENSING (CPU Level 1 & 2)
                 (Win32 Pixel Diff & Fast OCR CPU)
                                 │
                                 ▼
                         ATTENTION ENGINE
                   (Đánh giá độ khẩn cấp & Salience)
                                 │ (Nếu có biến cố đáng chú ý)
                                 ▼
                           VISION ROUTER
                    (Normal / High-Quality / Cloud)
                                 │
                                 ▼
                          VISION PROVIDER
                       (qwen3-vl:2b / Gemini)
                                 │
                                 ▼
                        VISION OBSERVATION
                (Dữ liệu quan sát thuần túy khách quan)
                                 │
                                 ▼
                         DECISION ENGINE
             (Bộ Não Core: Personality, Emotion, Goals)
                                 │
                                 ▼
                      TOOL PERMISSION POLICY
                 (Hàng rào an toàn / Sandbox Sandbox)
                                 │ (Chỉ thực thi khi được phép)
                                 ▼
                         COMPUTER ACTION
                (Win32 Mouse / Keyboard / App Launch)
                                 │
                                 ▼
                       SCREEN VERIFICATION
                  (Chụp kiểm chứng lại màn hình)
                                 │
                                 ▼
                        VISION OBSERVATION
                 (Cập nhật kết quả vào Memory Core)
```

### Phân tách Rõ ràng giữa Backend và Domain Providers:
Hệ thống không gom tất cả vào một monolithic provider mà tách biệt thành các hợp đồng chức năng chuyên biệt:

```
  Domain Trait Contracts                  External Backend Adapters
┌─────────────────────────┐               ┌─────────────────────────┐
│       LlmProvider       │ ◄──────────── │    OllamaChatProvider   │
│    (Chat & Sinh từ)     │ ◄──────────── │    GeminiChatProvider   │
└─────────────────────────┘               └─────────────────────────┘
┌─────────────────────────┐               ┌─────────────────────────┐
│      VisionProvider     │ ◄──────────── │   OllamaVisionProvider  │
│ (Quan sát & Trích xuất) │ ◄──────────── │   GeminiVisionProvider  │
└─────────────────────────┘               └─────────────────────────┘
┌─────────────────────────┐               ┌─────────────────────────┐
│    EmbeddingProvider    │ ◄──────────── │  OllamaEmbeddingProvider│
│  (Vector nhúng ngữ nghĩa)│ ◄──────────── │  FastEmbedCpuProvider   │
└─────────────────────────┘               └─────────────────────────┘
```
Nhờ kiến trúc này, **Ollama** chỉ là một backend client quản lý kết nối HTTP (`http://localhost:11434`), còn `Character Core` chỉ giao tiếp qua các trait trung lập `LlmProvider`, `VisionProvider`, `EmbeddingProvider`. Khi chuyển đổi model Vision (từ Qwen3-VL sang Gemini Vision), logic nhân vật hoàn toàn không bị ảnh hưởng.


---

## 5. Chiến Lược Quản Lý Tài Nguyên RTX 3050 Laptop (4GB VRAM)

Máy tính xách tay với GPU 4GB VRAM là môi trường rất nhạy cảm với hiện tượng OOM (Out Of Memory). Kiến trúc hệ thống áp dụng cơ chế điều phối tài nguyên nghiêm ngặt:

```
                  ┌─────────────────────────────────────────┐
                  │      RESOURCE MANAGER (vc-runtime)      │
                  │   Giám sát VRAM / RAM theo thời gian    │
                  └────────────────────┬────────────────────┘
                                       │
            ┌──────────────────────────┴──────────────────────────┐
            ▼                                                     ▼
┌───────────────────────────┐                         ┌───────────────────────────┐
│     ƯU TIÊN VRAM (GPU)    │                         │    CHUYỂN TẢI SANG CPU    │
│  • Chat LLM (Ollama 3B)   │                         │  • STT: whisper.cpp       │
│    ~2.2 GB VRAM cố định   │                         │    (4-6 CPU threads)      │
│  • Avatar Live2D / VRM    │                         │  • TTS: Piper TTS (CPU)   │
│    ~0.3 GB VRAM           │                         │  • Embeddings: FastEmbed  │
│  • Level 3 VLM (On-Demand)│                         │    (ONNX Runtime CPU)     │
│    ~0.8 GB (nếu cần nhìn) │                         │  • Screen Pixel Diff/OCR  │
│  ───────────────────────  │                         │  • Core Brain & SQLite    │
│  Tổng: <= 3.3 GB / 4.0 GB │                         │  ───────────────────────  │
│  -> Còn dư 700MB VRAM an toàn                       │  Tổng RAM: ~2.5 GB / 32 GB│
└───────────────────────────┘                         └───────────────────────────┘
```

### Cơ chế On-Demand VLM & GPU Model Multiplexing:
Không chạy đồng thời Chat LLM và Vision LLM trên GPU cùng một lúc để tránh cạn kiệt 4GB VRAM. Thay vào đó, `ResourceManager` điều phối theo chế độ đa hợp (**Model Multiplexing**):
- **Chế độ Chat thường trực**: Giữ `qwen2.5:3b-instruct` trong VRAM phục vụ suy nghĩ và hội thoại tức thì. Màn hình chỉ cảm biến bằng Pixel Diff trên CPU (0 VRAM).
- **Chế độ Vision (Khi có biến cố/Action)**:
  - `VisionRouter` điều hướng yêu cầu:
    - *Normal Mode*: Kích hoạt `qwen3-vl:2b` (1.9 GB) trích xuất GUI elements, text UI, xác định vị trí nút bấm hỗ trợ Computer Use.
    - *High Quality Mode*: Kích hoạt `qwen2.5vl:3b` (3.2 GB) khi cần bounding box chi tiết.
    - *Ultra Light Mode*: Dùng `moondream` trên CPU.
    - *Cloud Fallback*: Chuyển tiếp ảnh sang Gemini 1.5 Flash Vision (0 MB VRAM cục bộ).
  - Sau khi trích xuất xong `VisionObservation` (dạng text/JSON), VRAM được giải phóng hoặc trả quyền ưu tiên lại cho Chat LLM.


---

## 6. Mô Hình An Toàn & Bảo Mật (Sandboxed Security)

Tương tác máy tính (Computer Use) tuyệt đối không được cấp quyền điều khiển trực tiếp cho LLM:
1. **Ranh giới cô lập**: LLM chỉ xuất ra đề xuất hành động dạng JSON (`ToolCallRequest`).
2. **Policy Enforcement**: `ToolPermissionPolicy` kiểm tra danh mục quyền hạn:
   - `SafeRead`: Chụp ảnh màn hình vùng chọn, đọc danh sách cửa sổ đang mở. (Tự động cấp phép).
   - `ApplicationControl`: Mở ứng dụng trong danh sách trắng (Notepad, Chrome, Minecraft). (Cho phép kèm thông báo).
   - `Dangerous`: Chạy terminal command, xóa file, tải file không xác định, can thiệp registry. (Mặc định **CHẶN** hoàn toàn).
3. **Emergency Stop**: Người dùng có thể nhấn phím tắt khẩn cấp (Panic Key: `Ctrl + Shift + Esc` hoặc phím cấu hình) để ngắt ngay lập tức mọi quyền điều khiển ngoại vi của nhân vật.

---

## 7. Ranh Giới Hợp Đồng (Contract Stability Matrix)

Mọi sự phát triển song song đều phải tuân theo các hợp đồng giao tiếp tại `docs/contracts.md`:
- `vc-core` không bao giờ phụ thuộc vào chi tiết cài đặt của `vc-runtime`, `vc-llm`, hay `vc-storage`.
- Các provider AI mới (Ollama, Gemini, tương lai là Local VLM) chỉ cần cài đặt trait `LlmProvider`.
- Các nguồn tương tác bên ngoài (Discord, Twitch, YouTube, Microphone, Keyboard) đều được chuẩn hóa thành `DomainEvent` trước khi gửi tới `EventBus`.
