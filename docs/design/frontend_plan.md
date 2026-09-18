# VirtualCharacter Frontend Design Plan

## 1. Bối Cảnh & Mục Tiêu

`VirtualCharacter` là một **runtime độc lập** bằng Rust với các hệ thống phân tách nghiêm ngặt:
- **Personality** (Cốt cách, giá trị, xu hướng giao tiếp, ranh giới)
- **State & Emotion** (Trạng thái cảm xúc valence/arousal, nhận thức, hành vi)
- **Relationship** (Mức độ tin cậy, gắn kết, lịch sử gắn kết với người dùng)
- **Memory** (Bộ nhớ ngữ nghĩa, tình tiết, suy giảm & củng cố theo thời gian)
- **Decision Engine** (Lập luận nội tâm, đánh giá ứng viên hành động trước khi sinh ngôn ngữ)
- **Context** (Đóng gói thông tin có ngân sách token chuyển cho LLM)

Do đó, **Frontend của VirtualCharacter không thể chỉ là một khung chat thông thường**. Frontend được thiết kế theo mô hình **Dual-Mode UX**:
1. **Interactive Companion Experience (Dành cho người dùng tương tác)**: Trải nghiệm đắm chìm với nhân vật ảo sống động, phản hồi qua hào quang cảm xúc, hội thoại streaming mượt mà và cảm nhận rõ nét mối quan hệ ngày càng phát triển.
2. **Mind Visualizer / Studio Mode (Dành cho nhà phát triển/sáng tạo)**: Kính hiển vi quan sát "não bộ" của nhân vật: radar cảm xúc 2D thời gian thực, chuỗi suy nghĩ/lập luận nội tâm (decision traces), các mảnh ký ức được truy xuất, và công cụ biên tập tính cách trực tiếp.

---

## 2. Ranh Giới Kiến Trúc (Architecture Boundary)

Tuân thủ nguyên tắc **Architecture Governance** (`crates/vc-core` tuyệt đối không phụ thuộc vào web framework, HTTP hay UI):

```
apps/vc-web (Vite + React 19 + TypeScript)
        ▲
        │  Bi-directional WebSocket & REST
        ▼
apps/vc-server (Axum + Tokio + Tower-HTTP)
        │
        ├──> crates/vc-runtime (Điều phối vòng đời Interaction)
        ├──> crates/vc-core    (Các entity Personality, State, Memory, Decision)
        ├──> crates/vc-llm     (Gemini streaming / Mock provider)
        └──> crates/vc-storage (SQLite persistence)
```

- **`crates/vc-core`**: Domain logic độc lập hoàn toàn.
- **`apps/vc-server`**: Crate Gateway bằng Axum, quản lý session, dispatch WebSocket events.
- **`apps/vc-web`**: Ứng dụng web chạy độc lập bằng Vite + React 19 + TypeScript, có thể đóng gói Desktop App với Tauri v2.

---

## 3. Thiết Kế Trải Nghiệm Người Dùng (Dual-Mode UX)

### 3.1. Chế độ 1: "Companion Experience"
- **Visual Stage**:
  - *Phase 1*: Bioluminescent Emotion Core & Reactive Character Avatar (Hào quang năng lượng cảm xúc chuyển màu mượt mà bằng Canvas/SVG theo `EmotionState`).
  - *Phase 2*: 3D VRM Model (Three.js / `@pixiv/three-vrm`) hoặc Live2D với biểu cảm khuôn mặt động.
- **Dialogue Stream**:
  - Streaming token-by-token mượt mà, định dạng markdown tinh tế, bong bóng chat cá nhân hóa theo phong cách nhân vật.
  - Gợi ý chủ đề nhanh (Topic Suggestion Chips).
- **Relationship & Status HUD**:
  - Thanh đo mức độ tin cậy (Trust) và sự gắn kết (Closeness).
  - Đếm số kỷ niệm chung và thời gian tương tác.

### 3.2. Chế độ 2: "Mind Visualizer / Character Studio"
Một thanh điều khiển / Drawer có thể mở bằng phím tắt `~`:
- **2D Emotional Circumplex (Radar Cảm Xúc Valence-Arousal)**: Tọa độ cảm xúc thời gian thực và vệt quỹ đạo di chuyển qua các tương tác.
- **Decision Trace Inspector**: Danh sách ứng viên hành động, điểm tin cậy (`confidence`), và lời độc thoại nội tâm (`reasoning`) trước khi sinh text LLM.
- **Context Window & Token Budget Breakdown**: Phân rã token cho Tính cách, Ký ức, Trạng thái, và Lịch sử hội thoại.
- **Memory Subsystem Browser**: Tìm kiếm, duyệt bộ nhớ theo loại (*Working*, *Episodic*, *Semantic*), điểm quan trọng và độ suy giảm.
- **Personality Tuning Studio**: Biên tập tham số tính cách (Traits, Values, Quirks, Boundaries) và nạp profile JSON/YAML.

---

## 4. Ngôn Ngữ Thiết Kế & Thẩm Mỹ (Design System)

- **Phong cách**: Neo-Cybernetic Glassmorphism (Nền đen sâu đa tầng `#080A10`, kính mờ 18px, viền dạ quang tinh tế).
- **Typography**: `Plus Jakarta Sans` / `Outfit` cho giao diện chính; `Space Grotesk` / `JetBrains Mono` cho telemetry số liệu.
- **Màu sắc cảm xúc**:
  - *Calm / Serene*: `#00F2FE` (Cyan)
  - *Joy / Excited*: `#FF9F43` (Solar Amber)
  - *Melancholy / Thoughtful*: `#7F00FF` (Electric Violet)
  - *Agitated / High Arousal*: `#FF416C` (Coral Red)
  - *Affection / Trust*: `#FF6584` (Soft Rose)

---

## 5. Lộ Trình Triển Khai (Phased Roadmap)

1. **Phase F0: Backend Gateway (`apps/vc-server`)**: Thiết lập server Axum, cấu hình WebSocket `/ws/interaction` và REST API kết nối `vc-runtime`.
2. **Phase F1: Frontend Scaffolding & Design System (`apps/vc-web`)**: Khởi tạo Vite + React 19 + TypeScript, xây dựng token CSS Glassmorphism và layout Dual-Mode.
3. **Phase F2: Streaming Chat & WebSocket Integration**: Tương tác hội thoại thời gian thực, stream token, markdown, thanh nhập liệu.
4. **Phase F3: Emotion Stage & Dynamic Aura**: Hào quang cảm xúc thay đổi màu sắc và nhịp thở theo tâm trạng, widget mối quan hệ.
5. **Phase F4: Mind Visualizer / Telemetry Inspector**: Biểu đồ Radar 2D Valence-Arousal, Decision Trace, Token breakdown.
6. **Phase F5: Character Studio & Memory Explorer**: Trình biên tập tính cách và công cụ quản lý ký ức trực tiếp.
7. **Phase F6: Voice TTS & 3D Avatar (Mở rộng)**: Tích hợp giọng nói TTS với sóng âm thanh phản hồi, hỗ trợ model 3D VRM.
