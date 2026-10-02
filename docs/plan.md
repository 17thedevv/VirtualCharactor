# Kế Hoạch Phát Triển Toàn Diện — VirtualCharacter Master Plan

> **Định hướng**: Hệ điều hành Nhân vật Ảo Nhập thể Cục bộ (Local-First Embodied AI Character & AI VTuber Runtime).  
> **Phần cứng mục tiêu**: Intel Core i5-12500H, 32GB RAM, NVIDIA RTX 3050 Laptop GPU (4GB VRAM), Windows 11.  
> **Nguyên tắc cốt lõi**: Local-First, Character ≠ LLM, Tiết kiệm VRAM, Module hóa ranh giới nghiêm ngặt.

---

## 1. Tầm Nhìn Dự Án (Project Vision)

**VirtualCharacter** được định hình là một hệ thống runtime thế hệ mới cho nhân vật AI cá nhân hóa và AI VTuber tự chủ. Mục tiêu cuối cùng là mang lại trải nghiệm nhập vai, có cảm xúc, có thể lắng nghe, nói chuyện, quan sát màn hình máy tính, biểu cảm cơ thể (Live2D/VRM), và tương tác mạng xã hội (Discord, Livestream YouTube/Twitch) tương tự các hiện tượng **Neuro-sama** hay **AIRI**, nhưng chạy mượt mà ngay trên PC cá nhân cấu hình tầm trung.

Hệ thống kiên định với mô hình **"Não bộ tách biệt khỏi LLM"**:
```
┌──────────────────────────────────────────────────────────────┐
│                       CHARACTER BRAIN                        │
│                                                              │
│   Personality + Emotion (8-Axes) + Bipartite Relationship    │
│           + 4-Tier Memory + Goals + Decision Engine          │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                    LOCAL COGNITIVE ENGINE                    │
│                                                              │
│      Ollama (Qwen2.5-3B-Instruct) / FastEmbed on CPU         │
│          [Tùy chọn: Cloud Fallback với Gemini Flash]         │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                   EMBODIED ACTION ROUTER                     │
│                                                              │
│       Speech (Piper TTS) • Avatar (Live2D) • Hands (OS)      │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. Chiến Lược Phân Bổ Tài Nguyên Phần Cứng (RTX 3050 4GB VRAM)

Để ngăn ngừa lỗi Out-Of-Memory (OOM) làm sập hệ thống, kiến trúc chia rõ 3 nhóm xử lý:

| Phân loại | Tài nguyên gánh vác | Thành phần phụ trách |
|---|---|---|
| **[LOCAL - LIGHT]** | **CPU + RAM (32GB)** | Core Brain, SQLite Database, Attention Engine, Event Bus, WebSocket Server, Screen Pixel Diff. |
| **[LOCAL - LIGHT]** | **CPU Threads** | Whisper STT (`whisper.cpp`), Piper TTS (C++ Engine), FastEmbed Text Vectors (`bge-small`). |
| **[LOCAL - MEDIUM]** | **GPU VRAM (Cố định)** | Chat LLM (`qwen2.5:3b-instruct-q4_K_M` chiếm ~2.2GB), Avatar WebGL Canvas (~0.3GB). Tổng ~2.5GB VRAM. |
| **[LOCAL - ON-DEMAND]** | **GPU VRAM (Tạm thời)** | Vision Level 3 VLM (MiniCPM-V / Moondream2 ~0.8GB). Chỉ nạp khi cần, giải phóng sau khi đọc xong. |
| **[OPTIONAL CLOUD]** | **Cloud API** | Gemini 1.5 Flash Vision / Gemini Pro (Kích hoạt khi cần phân tích hình ảnh độ phân giải cao hoặc khi user muốn giải phóng toàn bộ VRAM). |

---

## 3. Lộ Trình 10 Giai Đoạn (Reorganized 10-Phase Roadmap)

```
[Phase 0: Workspace Bootstrap] ──► [Phase 1: Core & Storage] ──► [Phase 2: Local AI & Memory 2.0]
          (HOÀN TẤT)                      (HOÀN TẤT)                        (BẮT ĐẦU NGAY)
                                                                                  │
                                                                                  ▼
[Phase 5: Adaptive Vision]     ◄── [Phase 4: Avatar Body]    ◄── [Phase 3: Voice & Speech]
          │
          ▼
[Phase 6: Computer Use]        ──► [Phase 7: Attention Engine]──► [Phase 8: Social Connectors]
                                                                                  │
                                                                                  ▼
[Phase 10: Continual Growth]   ◄── [Phase 9: AI VTuber Runtime & Stream Mode] ◄───┘
```

---

### PHASE 0: Kiến Trúc Nền Tảng & Hợp Đồng Phân Quyền `[HOÀN TẤT 100%]`
- **Mục tiêu**: Thiết lập cấu trúc Cargo Workspace đa crate (`vc-core`, `vc-runtime`, `vc-llm`, `vc-storage`, `vc-cli`, `vc-server`, `vc-web`).
- **Thành quả**: Phân ranh giới sở hữu Dev A & Dev B, thiết lập quy tắc hợp đồng (`docs/contracts.md`), thiết lập 20 kỹ năng agent (`.agent/skills/`).

---

### PHASE 1: Cognitive Core, SQLite Persistence & Runtime E2E `[HOÀN TẤT 100%]`
- **Mục tiêu**: Xây dựng toàn bộ trí tuệ nhân vật độc lập, lưu trữ dữ liệu bền vững và vòng đời tương tác 9 bước.
- **Thành quả**:
  - **Dev A**: `Personality` domain model, `EmotionState` 8 trục kèm phân rã hàm mũ (decay), `Relationship` đa người dùng (Actor Isolation), `Memory` 4 tầng miền dữ liệu, `RuleDecisionEngine` với nhật ký độc thoại nội tâm (Inner Monologue).
  - **Dev B**: `ContextBuilder` kiểm soát ngân sách token (`ContextBudget`), `RuntimeEngine` điều phối 9 giai đoạn, `SessionManager` đa phiên thread-safe, `LlmProvider` hỗ trợ Gemini REST API và `MockLlmProvider` đầy đủ, `SqliteStorage` lưu file bền vững (`data/virtual_character.db`), máy chủ `vc-server` Axum WebSocket Streaming và Web HUD.
  - **Kiểm thử**: **114 automated tests pass tuyệt đối** trên toàn bộ workspace.

---

### PHASE 2: Local AI (Ollama) & Bộ Nhớ Ngữ Nghĩa 2.0 `[ĐANG THỰC HIỆN]`
- **Mục tiêu**: Đưa VirtualCharacter vào trạng thái **100% Offline-Capable** với Local LLM và vector search cục bộ.
- **Nhiệm vụ cụ thể**:
  1. **Ollama Integration (`vc-llm`)**:
     - Cài đặt `OllamaProvider` kết nối tới `http://localhost:11434`.
     - Hỗ trợ model mặc định: `qwen2.5:3b-instruct-q4_K_M`.
     - Xây dựng `ModelRegistry` và hệ thống nhận biết năng lực model (`ModelCapability`: Text, Vision, Embedding, ToolCalling).
  2. **Memory 2.0 & Vector Embeddings (`vc-storage`)**:
     - Tích hợp `fastembed-rs` tạo vector embedding 384 chiều trên **CPU** (không tốn VRAM).
     - Nâng cấp `SqliteStorage` hỗ trợ tìm kiếm độ tương đồng Cosine (Semantic Search).
     - Phân rã ký ức theo thời gian (Memory Decay) và cơ chế dọn dẹp bộ nhớ đệm.
  3. **Resource Manager nền tảng (`vc-runtime`)**:
     - Giám sát mức chiếm dụng VRAM/RAM trước khi nạp model.
- **Độ sẵn sàng phần cứng**: `[LOCAL - LIGHT & MEDIUM]`.

---

### PHASE 3: Voice In/Out & Tổng Hợp Giọng Nói Cảm Xúc (Audio Pipeline)
- **Mục tiêu**: Trang bị "Tai" (Hearing) và "Miệng" (Voice) cho nhân vật, hoạt động hoàn toàn cục bộ.
- **Nhiệm vụ cụ thể**:
  1. **Hearing Pipeline (Audio Input)**:
     - Thu âm trực tiếp qua Microphone bằng thư viện `cpal`.
     - Tích hợp Silero VAD trên CPU để nhận biết thời điểm nói và ngắt âm thanh.
     - Tích hợp `whisper.cpp` (chạy trên CPU threads) nhận dạng tiếng Việt/Anh độ trễ <500ms.
     - Hỗ trợ cả 2 chế độ: Push-to-Talk (phím tắt) và Voice Activation.
  2. **Speech Pipeline (Audio Output)**:
     - Tích hợp **Piper TTS** chạy trên CPU cho giọng đọc siêu nhanh (RTF < 0.2).
     - Tích hợp **Kokoro 82M** (ONNX CPU/DirectML) cho chất giọng giàu cảm xúc.
     - Phát âm thanh trực tiếp qua loa bằng `rodio`.
  3. **Emotion-to-Speech Modulator**:
     - Điều chỉnh tốc độ, cao độ, năng lượng giọng nói dựa trên tọa độ cảm xúc `(valence, arousal)` hiện tại của nhân vật.
- **Độ sẵn sàng phần cứng**: `[LOCAL - LIGHT]` (100% chạy trên CPU và RAM, 0 MB VRAM).

---

### PHASE 4: Avatar Thân Thể, Biểu Cảm Khuôn Mặt & Real-time Lip-Sync
- **Mục tiêu**: Cung cấp "Thân thể" (Body) cho nhân vật với Live2D và 3D VRM.
- **Nhiệm vụ cụ thể**:
  1. **Avatar Controller Abstraction (`vc-runtime`)**:
     - Định nghĩa `AvatarEvent` (Biểu cảm, chớp mắt, góc nhìn, nhép môi).
     - Ánh xạ tự động từ `EmotionState` sang Blendshape/Parameter của Avatar (Vui, buồn, tức giận, ngượng ngùng).
  2. **Live2D & VRM Renderer (`apps/vc-web` / Client HUD)**:
     - Nhúng Live2D Cubism Web SDK và Three.js `@pixiv/three-vrm`.
     - Tự động chớp mắt ngẫu nhiên (Auto Blink) và cử động mắt tự nhiên (Idle Eye Movement).
  3. **Real-time Lip-Sync**:
     - Phân tích biên độ âm thanh (RMS/Viseme) trực tiếp từ luồng audio phát ra từ TTS để đồng bộ mở miệng chính xác từng mili-giây.
- **Độ sẵn sàng phần cứng**: `[LOCAL - LIGHT]` (~200-300MB VRAM cho WebGL/D3D).

---

### PHASE 5: Thị Giác Thích Ứng & Nhận Thức Màn Hình (Adaptive Vision)
- **Mục tiêu**: Cung cấp "Mắt" (Vision) để nhân vật quan sát màn hình máy tính mà không làm nghẽn tài nguyên.
- **Nhiệm vụ cụ thể**:
  1. **3-Level Perception Architecture**:
     - **Level 1 (Screen Sensing)**: Win32 Desktop Duplication API chụp màn hình, so sánh Pixel Diff trên CPU. Màn hình không đổi -> Bỏ qua.
     - **Level 2 (Fast Perception)**: Lấy tên cửa sổ active, chạy OCR siêu nhẹ (Tesseract/RapidOCR trên CPU) để đọc text giao diện.
     - **Level 3 (Deep Understanding)**: Chỉ gọi VLM khi có yêu cầu hoặc sự kiện quan trọng.
  2. **Perception Scheduler & Token Budget**:
     - Cơ chế cooldown (tối đa 1 lần/vài giây khi có biến động), không bao giờ chụp 30 FPS gửi LLM.
     - Tùy chọn chuyển tiếp ảnh sang Gemini Flash Cloud Fallback khi GPU bận.
- **Độ sẵn sàng phần cứng**: `[LOCAL - LIGHT & ON-DEMAND]`.

---

### PHASE 6: Tương Tác Máy Tính Trong Vùng Kiểm Soát (Sandboxed Computer Use)
- **Mục tiêu**: Cung cấp "Bàn tay" (Hands) cho phép nhân vật hỗ trợ người dùng trên Windows an toàn.
- **Nhiệm vụ cụ thể**:
  1. **Computer Tool Suite**:
     - Mở ứng dụng trong danh sách cho phép (Notepad, Chrome, Minecraft, VSCode).
     - Điều khiển chuột (click, scroll), gõ văn bản, bấm phím tắt qua Win32 API.
  2. **Action-Verification Loop**:
     - Thực thi hành động -> Chụp màn hình vùng chọn kiểm tra -> Báo cáo kết quả lại cho Brain.
  3. **Permission Sandbox & Panic Key**:
     - Phân cấp rủi ro hành động (`ToolRiskLevel`).
     - Chặn tuyệt đối các lệnh phá hoại hệ thống.
     - Phím tắt ngắt khẩn cấp ngay lập tức (`Panic Key`).
- **Độ sẵn sàng phần cứng**: `[LOCAL - LIGHT]` (CPU-based).

---

### PHASE 7: Hệ Thống Chú Ý, Trạng Thái Thế Giới & Tính Tự Chủ (Attention & Autonomy)
- **Mục tiêu**: Nhân vật trở thành thực thể sống động, biết chủ động trò chuyện và quan sát thay vì chỉ hỏi-đáp thụ động.
- **Nhiệm vụ cụ thể**:
  1. **World State Model**:
     - Lưu trữ trạng thái thế giới xung quanh: Ứng dụng hiện tại, thời gian trong ngày, sự hiện diện của user, hoạt động voice.
  2. **Attention Engine**:
     - Tính điểm Salience: Cooldown, độ tò mò, mức độ quan trọng của sự kiện.
     - Lọc bỏ 95% sự kiện bình thường, chỉ kích hoạt suy luận khi điểm chú ý vượt ngưỡng.
  3. **Idle Behavior & Proactive Actions**:
     - Khi user im lặng lâu: Nhân vật có thể thở dài nhẹ, đổi biểu cảm, hoặc bắt chuyện tự nhiên nếu phù hợp.
- **Độ sẵn sàng phần cứng**: `[LOCAL - LIGHT]`.

---

### PHASE 8: Kết Nối Mạng Xã Hội (Discord & Livestream Unified Chat)
- **Mục tiêu**: Kết nối nhân vật vào cộng đồng người dùng qua các nền tảng mạng xã hội.
- **Nhiệm vụ cụ thể**:
  1. **Unified Chat Event Bus**:
     - Chuẩn hóa mọi tin nhắn từ mọi nền tảng thành `ChatMessage { platform, user_id, username, content, timestamp }`.
  2. **Discord Adapter (`vc-integrations-discord`)**:
     - Sử dụng `serenity-rs` kết nối Discord Bot, duy trì bảng quan hệ (Relationship) riêng cho từng bạn bè trong server.
  3. **Livestream Ingestion & Spam Filter**:
     - Đọc tin nhắn từ YouTube Live / Twitch IRC.
     - Lọc bỏ tin nhắn spam hàng loạt, ưu tiên SuperChat, câu hỏi hay hoặc tin nhắn từ người quen.
- **Độ sẵn sàng phần cứng**: `[LOCAL - LIGHT]`.

---

### PHASE 9: AI VTuber Runtime & Chế Độ Phát Trực Tiếp (Stream Mode)
- **Mục tiêu**: Biến nhân vật thành một AI VTuber hoàn chỉnh có thể tự lên sóng livestream phục vụ khán giả.
- **Nhiệm vụ cụ thể**:
  1. **Stream Mode Runtime Profile**:
     - Tự động luân chuyển: Đọc chat -> Lọc câu hay -> Suy nghĩ -> Nói chuyện qua mic ảo -> Cử động Avatar -> OBS Studio.
  2. **OBS Studio Integration**:
     - Xuất hình ảnh Avatar trong suốt (Spout2 / Virtual Camera) và xuất âm thanh qua Virtual Audio Cable tới OBS.
  3. **Audience Interaction & Chat Moderation**:
     - Điều hòa tốc độ nói chuyện, tránh tình trạng nói ngắt quãng hoặc nói không ngừng nghỉ.
- **Độ sẵn sàng phần cứng**: `[LOCAL - MEDIUM]`.

---

### PHASE 10: Tự Thích Ứng Dài Hạn & Đa Nhân Vật (Continual Adaptation & Multi-Character)
- **Mục tiêu**: Tối ưu hóa bộ nhớ ngủ (Sleep Consolidation), tiến hóa tính cách theo thời gian, và hỗ trợ nhiều nhân vật độc lập chạy trên cùng một Core.
- **Nhiệm vụ cụ thể**:
  1. **Memory Sleep Consolidation**:
     - Worker chạy ngầm khi nhàn rỗi tổng hợp các sự kiện trong ngày thành Semantic Knowledge dài hạn.
  2. **Multi-Character Profile Manager**:
     - Cho phép chuyển đổi linh hoạt giữa các nhân vật (Aria, Nero, v.v.) với ngoại hình, giọng nói và tính cách riêng biệt.
- **Độ sẵn sàng phần cứng**: `[LOCAL - LIGHT]`.

---

## 4. Bảng So Sánh Tiến Trình & Khả Năng Thực Thi (Feature Matrix)

| Giai đoạn | Trọng tâm | Trạng thái | Phần cứng (RTX 3050) |
|---|---|---|---|
| **Phase 0** | Workspace, Contracts, Skills | **[HOÀN THÀNH]** | N/A |
| **Phase 1** | Cognitive Core, SQLite, Server/Web | **[HOÀN THÀNH]** (114 tests) | CPU / RAM |
| **Phase 2** | Ollama LLM, Vector Embeddings | **[KẾ HOẠCH NGAY]** | GPU ~2.2GB VRAM + CPU |
| **Phase 3** | Whisper STT, Piper/Kokoro TTS | Sắp tới | CPU Threads (0 VRAM) |
| **Phase 4** | Live2D/VRM Avatar, Lip-Sync | Sắp tới | GPU WebGL ~0.3GB VRAM |
| **Phase 5** | Adaptive Vision, Screen Sensing | Sắp tới | CPU Diff + VLM On-Demand |
| **Phase 6** | Computer Tools, Win32 Sandbox | Sắp tới | CPU |
| **Phase 7** | Attention Engine, World State | Sắp tới | CPU |
| **Phase 8** | Discord Bot, Livestream Chat | Sắp tới | CPU / Network |
| **Phase 9** | AI VTuber Stream Mode, OBS | Tương lai | GPU ~2.5GB VRAM |
| **Phase 10**| Sleep Consolidation, Multi-Char | Tương lai | CPU / SQLite |

---

## 5. Tiêu Chuẩn Hoàn Thành Cho Từng Pha (Definition of Done)

Một Phase chỉ được coi là hoàn tất khi đáp ứng 6 điều kiện tiên quyết:
1. **Contract Definition**: Các trait và data types mới được định nghĩa tường minh, không vi phạm ranh giới kiến trúc.
2. **Automated Testing**: 100% logic mới có unit/integration test; lệnh `cargo test --workspace` phải pass sạch sẽ.
3. **Local-First Verification**: Kiểm thử thực tế chạy offline thành công không cần Internet.
4. **VRAM Safety**: Tổng mức tiêu thụ VRAM không bao giờ vượt ngưỡng 3.5GB trong điều kiện hoạt động bình thường.
5. **No Regressions**: Toàn bộ 114 test có từ trước tiếp tục pass và các chức năng cũ không bị ảnh hưởng.
6. **Documentation**: Cập nhật đầy đủ `docs/tasks.md`, `docs/checklist.md`, và progress logs.
