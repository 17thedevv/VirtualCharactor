# Công Nghệ Sử Dụng — VirtualCharacter Tech Stack

Tài liệu này xác định toàn bộ hệ sinh thái công nghệ được lựa chọn cho dự án **VirtualCharacter**, tuân thủ nghiêm ngặt định hướng: **Local-First, Hiệu năng cao (Rust), Tiết kiệm tài nguyên (Tối ưu cho RTX 3050 Laptop 4GB VRAM), và Kiến trúc mô-đun hóa**.

---

## 1. Hệ thống & Ngôn ngữ Cốt Lõi (Core & System Runtime)

| Thành phần | Công nghệ lựa chọn | Lý do & Vai trò kỹ thuật |
|---|---|---|
| **Ngôn ngữ chính** | **Rust (2021 Edition)** | Hiệu năng tiệm cận C++, an toàn bộ nhớ tuyệt đối (Memory Safety không cần GC), kiểm soát chặt chẽ từng byte RAM/VRAM, hỗ trợ đa luồng song song không data race. |
| **Async Runtime** | **Tokio 1.x** | Async runtime công nghiệp cho Rust, quản lý non-blocking IO, timers, và background task scheduling hiệu quả. |
| **Web & API Server** | **Axum 0.7** | Framework web hiệu năng cực cao xây dựng trên `tokio` và `tower`, hỗ trợ RESTful API và WebSocket full-duplex streaming thời gian thực. |
| **Serialization** | **Serde & Serde JSON** | Chuẩn hóa định dạng trao đổi dữ liệu, cấu hình thực thể, trạng thái cảm xúc và schema persistence. |
| **Logging & Tracing** | **tracing & tracing-subscriber** | Phục vụ Observability, xuất log có cấu trúc (structured JSON), hỗ trợ debug luồng xử lý mà không làm rò rỉ dữ liệu cá nhân hay secrets. |

---

## 2. Trí tuệ Nhân tạo Cục bộ (Local AI & LLM Engine)

| Thành phần | Công nghệ lựa chọn | Chiến lược triển khai & Phần cứng |
|---|---|---|
| **Local LLM Engine** | **Ollama** (`http://localhost:11434`) | Cung cấp REST API chuẩn, quản lý nạp/hạ model thông minh, hỗ trợ GPU CUDA offloading. |
| **Model Chat chính (Local)** | **Qwen2.5-3B-Instruct (Q4_K_M)** | Chiếm khoảng **~2.0 - 2.2 GB VRAM**. Khả năng đối thoại tiếng Việt/Anh xuất sắc, hỗ trợ System Prompt tốt, phản hồi nhanh (<800ms) trên RTX 3050. |
| **Model Chat phụ / Nhẹ** | **Llama-3.2-3B / Llama-3.2-1B** | Phương án dự phòng siêu nhẹ khi cần giải phóng thêm VRAM cho tác vụ khác. |
| **LLM Cloud Fallback** | **Google Gemini Flash / Pro** | Tích hợp sẵn trong `vc-llm` qua REST API trực tiếp (không dùng SDK cồng kềnh). Tùy chọn kích hoạt khi người dùng cần suy luận siêu phức tạp hoặc khi offline không khả dụng. |
| **Model Capability System** | `vc-llm::ModelRegistry` | Quản lý siêu dữ liệu model: loại model (Chat, Vision, Embedding), yêu cầu VRAM/RAM, context length, khả năng tool calling. |

---

## 3. Nhận dạng Giọng nói (Hearing / STT Pipeline)

| Thành phần | Công nghệ lựa chọn | Chiến lược tối ưu tài nguyên |
|---|---|---|
| **Audio Capture** | **CPAL (Cross-Platform Audio Library)** | Thu âm từ Microphone theo thời gian thực trên Windows (WASAPI backend), độ trễ cực thấp. |
| **Voice Activity Detection** | **Silero VAD / WebRTC VAD** | Chạy trên **CPU** (<2% tải). Phát hiện chính xác khi nào người dùng bắt đầu nói và dừng nói để ngắt audio chunk, tránh gửi audio rác. |
| **Local STT Engine** | **`whisper.cpp` / `faster-whisper`** | Mô hình `whisper-base` hoặc `whisper-small` lượng tử hóa int8. **Chạy hoàn toàn trên CPU (4-6 threads) hoặc GPU offload một phần nhỏ**. Tốn ~400MB RAM, **0 MB VRAM**, độ trễ chuyển văn bản ~300-500ms. |

---

## 4. Tổng hợp Giọng nói (Voice / TTS Pipeline)

| Thành phần | Công nghệ lựa chọn | Chiến lược tối ưu tài nguyên |
|---|---|---|
| **Local TTS chính** | **Piper TTS** (Fast Neural TTS) | Engine C++ siêu nhẹ, giọng đọc tự nhiên, **chạy 100% trên CPU**, tốc độ tạo audio nhanh gấp 5-10 lần thời gian thực (RTF < 0.2), **0 MB VRAM**. |
| **Local TTS chất lượng cao** | **Kokoro 82M** (ONNX / Local) | Model 82 triệu tham số mang lại cảm xúc biểu cảm cao, hỗ trợ chạy CPU/DirectML. |
| **Phát âm thanh** | **Rodio** | Thư viện audio playback thuần Rust, phát trực tiếp ra loa/tai nghe người dùng. |
| **Cloud TTS Fallback** | **ElevenLabs / Edge TTS** | Tùy chọn online cho chất giọng siêu thực khi kết nối internet khả dụng. |
| **Emotion-to-Speech** | `vc-runtime::VoiceModulator` | Điều chỉnh Pitch (cao độ), Speed (tốc độ) và Energy (năng lượng) của giọng nói dựa trên tọa độ cảm xúc `(valence, arousal)` của Character Core. |

---

## 5. Thân thể & Biểu cảm (Body & Avatar)

| Thành phần | Công nghệ lựa chọn | Vai trò & Kết nối |
|---|---|---|
| **Avatar 2D** | **Live2D Cubism Web SDK** | Hỗ trợ model 2D anime mượt mà, điều khiển qua tham số góc quay đầu, mắt, miệng, biểu cảm. |
| **Avatar 3D** | **Three.js + `@pixiv/three-vrm`** | Hỗ trợ mô hình 3D VRM chuẩn công nghiệp VTuber (blendshapes, eye tracking, bone physics). Đã tích hợp model mẫu `Seed-san.vrm` (10.9 MB, VRM Public License 1.0, VirtualCast Inc.). |
| **Hiển thị & Render** | **WebGL Transparent Canvas** | Chạy trong ứng dụng Web/Desktop HUD. Chỉ chiếm **~150 - 250 MB VRAM**, hoàn toàn độc lập với Core Rust. Tương thích trực tiếp làm OBS Browser Source. |
| **Realtime Lip-Sync** | **Audio Amplitude & Viseme Extraction** | Phân tích biên độ và dải tần số âm thanh từ TTS để điều khiển chuyển động đóng mở miệng theo thời gian thực (`VRMExpressionPresetName.Aa`). |
| **Biểu Cảm & Thần Thái** | **Expression Manager & Natural IK** | Biểu cảm khuôn mặt 5 trạng thái (`Happy`, `Sad`, `Angry`, `Surprised`, `Relaxed`) nội suy mượt mà, tự động chớp mắt (3-5.5s), nhịp thở idle và mắt dõi theo chuột. |
| **Giao thức điều khiển** | **WebSocket (`vc-server`)** | Core Rust bắn event trạng thái cảm xúc, hành vi (`idle`, `talking`, `thinking`) sang Avatar Renderer với độ trễ <10ms. |

---

## 6. Thị giác & Nhận thức Màn hình (Vision & Perception)

Hệ thống áp dụng kiến trúc nhận thức thích ứng 3 tầng (**Adaptive Perception Loop**) kết hợp **VisionRouter**:

| Tầng nhận thức | Công nghệ / Model | Phần cứng | Vai trò |
|---|---|---|---|
| **Level 1: Screen Sensing** | **Win32 Desktop Duplication API / GDI** | **CPU** (0 VRAM) | Chụp màn hình vùng chọn, tính toán sai khác khung hình (Pixel Diff / Hash Diff). Không gọi LLM khi màn hình tĩnh. |
| **Level 2: Fast Perception** | **Tesseract OCR / RapidOCR (ONNX CPU)** | **CPU** (0 VRAM) | Đọc nhanh tiêu đề cửa sổ, text trên màn hình, nhận diện ứng dụng đang hoạt động (Active Window) hoàn toàn trên CPU. |
| **Level 3: Deep Understanding** | **VisionRouter (Ollama / Cloud)** | **GPU (On-Demand) hoặc Cloud** | Chỉ kích hoạt khi Attention Engine phát hiện sự kiện đáng chú ý hoặc khi thực thi Computer Action. |

### Hệ Thống Định Tuyến Thị Giác (VisionRouter):
1. **Mắt chính (Normal Mode)**: **`qwen3-vl:2b`** (File model ~1.9 GB download. *Lưu ý: Mức tiêu thụ VRAM runtime thực tế phụ thuộc vào resolution, context length, KV cache và overhead của Ollama, sẽ được benchmark trực tiếp trên máy, không coi 1.9 GB là VRAM runtime cố định. Khuyến nghị dùng bản Q4 mặc định, tránh dùng bản BF16 ~4.3 GB gây tràn VRAM*). Tối ưu vượt trội cho nhận diện giao diện máy tính (GUI elements), đọc text UI, phục vụ trực tiếp cho Computer Use.
2. **Mắt chất lượng cao (High Quality Mode)**: **`qwen2.5vl:3b`** (File ~3.2 GB) — Tùy chọn khi cần phân tích biểu đồ sâu, trích xuất tọa độ Bounding Box chi tiết.
3. **Mắt siêu nhẹ (Ultra Light Mode)**: **`moondream`** (1.6B) — Chạy trên CPU hoặc VRAM tối thiểu để phân loại nhanh khung cảnh.
4. **Mắt Cloud Fallback**: **Google Gemini 1.5 Flash Vision** — Kích hoạt qua REST API khi cần phân tích ảnh độ phân giải siêu cao hoặc khi người dùng muốn nhường 100% VRAM GPU cho tác vụ khác.


---

## 7. Bộ nhớ & Tìm kiếm Ngữ nghĩa (Memory 2.0 & Storage)

| Thành phần | Công nghệ lựa chọn | Vai trò & Đặc tính |
|---|---|---|
| **Persistent Storage** | **SQLite (`rusqlite` bundled)** | Cơ sở dữ liệu quan hệ nhúng, phi máy chủ, lưu file cục bộ tại `data/virtual_character.db`, hỗ trợ ACID transaction, thread-safe qua Mutex. |
| **Local Text Embeddings** | **`fastembed-rs`** (`bge-small-en-v1.5` / `multilingual`) | Tạo vector nhúng 384 chiều **hoàn toàn trên CPU** qua ONNX Runtime, tốc độ cực nhanh, không phụ thuộc OpenAI/Cloud API. |
| **Vector Search** | **Cosine Similarity / SQLite Virtual Table** | Tìm kiếm ký ức theo độ tương đồng ngữ nghĩa (Semantic Search), lọc theo Actor ID đảm bảo tính riêng tư tuyệt đối. |

---

## 8. Tương tác Máy tính (Sandboxed Computer Interaction)

| Thành phần | Công nghệ lựa chọn | Vai trò & An toàn |
|---|---|---|
| **Hệ điều hành** | **Windows 11 API (via `windows-rs` / `enigo`)** | Tương tác bàn phím, chuột, quản lý cửa sổ ứng dụng. |
| **Quyền hạn & An toàn** | `vc-runtime::ToolPermissionPolicy` | Phân cấp rủi ro (Low / Medium / High / Critical). Chặn hoàn toàn các lệnh nguy hiểm (format disk, delete system files). Yêu cầu người dùng xác nhận cho hành động nhạy cảm. |
| **Action Verification** | Screen Verifier Loop | Chụp ảnh kiểm chứng sau khi thực hiện action (Ví dụ: Click mở Chrome -> Kiểm tra cửa sổ Chrome đã xuất hiện hay chưa). |

---

## 9. Mạng Xã hội & Kênh Livestream (Social & Stream Integration)

| Nền tảng | Công nghệ | Vai trò |
|---|---|---|
| **Discord** | **`serenity-rs`** | Bot Discord độc lập kết nối qua Event Bus, hỗ trợ chat văn bản, nhận diện username, duy trì quan hệ riêng từng bạn bè. |
| **YouTube Live** | **YouTube Live Chat Streaming API / WebSocket** | Thu thập tin nhắn luồng trực tiếp trong phiên livestream. |
| **Twitch** | **Twitch IRC (WebSocket / TCP)** | Thu thập tin nhắn chat từ cộng đồng Twitch. |
| **Chat Priority Engine** | `vc-runtime::ChatPriorityEngine` | Lọc spam, tính điểm ưu tiên (Mention, Người quen, Câu hỏi hay, SuperChat) trước khi đưa vào hàng đợi suy luận của nhân vật. |

---

## 10. Tổng kết Bảng Cân Đối Tài Nguyên (RTX 3050 4GB Laptop Profile)

```
┌──────────────────────────────────────────────────────────────┐
│                    RAM 32GB (CPU THREADS)                    │
│                                                              │
│  [Core Brain & State] ~80 MB     [Whisper STT] ~400 MB       │
│  [Piper TTS]          ~250 MB    [FastEmbed CPU] ~300 MB     │
│  [Desktop Sensing]    ~100 MB    [Axum / Server] ~50 MB      │
│                                                              │
│  -> Tổng RAM sử dụng: ~1.5 GB - 2.5 GB (Dư dả trên 32 GB)    │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                     VRAM 4.0GB (NVIDIA GPU)                  │
│                                                              │
│  [Ollama Chat LLM: Qwen2.5 3B Q4_K_M]       ~ 2.2 GB VRAM    │
│  [Avatar Renderer: Live2D / Three.js VRM]   ~ 0.3 GB VRAM    │
│  [Vision Level 3 VLM: On-Demand Buffer]     ~ 0.8 GB VRAM    │
│  [VRAM Headroom / Dự phòng an toàn]         ~ 0.7 GB VRAM    │
│                                                              │
│  -> TỔNG VRAM: ~3.3 GB / 4.0 GB (100% An toàn, không crash)  │
└──────────────────────────────────────────────────────────────┘
```
