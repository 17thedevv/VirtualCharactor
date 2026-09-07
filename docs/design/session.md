# VirtualCharacter Session System

## 1. Tổng quan (Overview)

Tài liệu này định nghĩa hệ thống Session (Phiên làm việc), vòng đời của nó, và mối quan hệ của nó với các khái niệm như Conversation, Interaction, và Message trong kiến trúc VirtualCharacter.

Mục tiêu chính là phân định rõ ranh giới trách nhiệm: Session quản lý **trạng thái runtime tạm thời** và **vòng đời kết nối**, chứ không sở hữu logic về bộ nhớ dài hạn (Memory), cảm xúc cốt lõi (State), hay lý luận (Decision).

---

## 2. Phân định khái niệm (Core Distinctions)

Hệ thống truyền thông của VirtualCharacter được tổ chức theo cấu trúc phân cấp nghiêm ngặt:

1. **Session (Phiên làm việc):**
   - Một khoảng thời gian runtime liên tục mà VirtualCharacter khả dụng để tương tác với một hoặc nhiều Actor.
   - Thường tương đương với việc "User mở app" cho đến khi "User đóng app" hoặc hết timeout.
2. **Conversation (Cuộc hội thoại):**
   - Một luồng giao tiếp logic, liên tục về mặt ngữ cảnh.
   - Một Session chứa một hoặc nhiều Conversation.
3. **Interaction (Lượt tương tác):**
   - Một chu kỳ xử lý duy nhất (Input -> Perception -> Decision -> Action/Response -> Outcome).
   - Một Conversation chứa nhiều Interaction.
4. **Message (Tin nhắn):**
   - Đơn vị dữ liệu giao tiếp thô (raw data) được truyền đi.
   - Một Interaction có thể nhận 1 Message đầu vào và sinh ra 0, 1 hoặc nhiều Message đầu ra (Text, Tool call).

**Lưu ý quan trọng:**
Session ≠ Conversation.
Khi User quay lại vào ngày hôm sau, hệ thống tạo ra một **Session mới** và một **Conversation mới**. Nhân vật nhớ lại User thông qua **Memory Subsystem**, KHÔNG PHẢI bằng cách giữ một Conversation mở vô thời hạn.

---

## 3. Kiến trúc phân cấp (Relationship Model)

Mô hình phân cấp chuẩn mực:

```text
Session (Runtime availability period)
└── Conversation 1 (Current logical thread)
    ├── Interaction 1 (Input A -> Response A)
    │   ├── Input Message
    │   └── Output Message
    ├── Interaction 2 (Input B -> Response B)
    └── ...
```

- Một `Session` sở hữu nhiều `Conversation`. Tuy nhiên tại một thời điểm, thường chỉ có một Conversation đang active.
- `Interaction` luôn thuộc về chính xác một `Conversation`.
- `Message` thuộc về một `Interaction` (và do đó thuộc về một `Conversation`).

---

## 4. Định danh và Trạng thái (Identity & Status)

### 4.1. Định danh (Session Identity)
- `SessionId`: (Bắt buộc) Định danh duy nhất cho Session (UUID). Persistent.
- `CharacterId`: (Bắt buộc) Nhân vật tham gia. Persistent.
- `ParticipantIds`: (Bắt buộc) Danh sách Actor tham gia (Thường là 1 User). Persistent.
- `CreatedAt`: (Bắt buộc) Timestamp.
- `Status`: Trạng thái hiện tại.

### 4.2. Vòng đời (Session State Machine)

Máy trạng thái tối thiểu cần thiết cho Session:

```text
    [Created]
        ↓
    [Active] ←─────┐
        ↓          │ (Resume)
      [Idle] ──────┘
        ↓
   [Completed] / [Expired] / [Cancelled]
```

- **Created**: Session được khởi tạo nhưng chưa có tương tác nào.
- **Active**: Đang có tương tác hoặc user đang trực tuyến.
- **Idle**: User không có hành động nào trong một khoảng thời gian. Memory có thể tận dụng lúc này để chạy background consolidation.
- **Completed**: Phiên kết thúc tự nhiên (User chủ động thoát).
- **Expired**: Hết hạn do idle quá lâu.
- **Cancelled**: Hủy do lỗi hệ thống.

Terminal states (Completed, Expired, Cancelled) **không thể** resume. Nếu user quay lại, tạo Session mới.

---

## 5. Scope và Ownership (Phạm vi Trạng thái)

Điều tối quan trọng là không được nhầm lẫn giữa dữ liệu tạm thời của Session và dữ liệu vĩnh viễn của Character.

| Data Type | Scope (Phạm vi) | Lifetime (Vòng đời) | Owner (Trách nhiệm) |
|---|---|---|---|
| Personality | Character | Long-term (Persistent) | `vc-core::personality` |
| Character State (Emotion) | Character | Dynamic (Persistent) | `vc-core::state` |
| Relationship (Trust, etc) | Relationship | Long-term (Persistent) | `vc-core::relationship` |
| **Session State** | **Session** | **Session (Ephemeral)** | **`vc-runtime`** |
| Conversation History | Conversation | Persistent (Archivable) | `vc-storage` |
| Context Budget | Interaction | Ephemeral | `vc-core::context` |
| Decision Intent | Interaction | Ephemeral | `vc-core::decision` |
| LlmRequest/Response | Interaction | Ephemeral | `vc-llm` |

### 5.1. SessionState là gì?
`SessionState` chứa các thông tin ngữ cảnh **chỉ có ý nghĩa trong phiên hiện tại**:
- `current_topic` (Chủ đề đang nói tới)
- `conversation_mood` (Không khí cuộc trò chuyện: nghiêm túc, đùa giỡn)
- `turn_count` (Số lượt đã nói)

**Tuyệt đối không** để `SessionState` lặp lại các dữ liệu đã có chủ sở hữu khác:
- Không chứa `emotion` (thuộc về `CharacterState`).
- Không chứa `relationship` hay `affection` (thuộc về `RelationshipState`).
- Không chứa `long-term goals` (thuộc về `CharacterState` hoặc `Personality`).

Điều này ngăn chặn tình trạng `CharacterState <-> SessionState` cùng lưu trữ một loại dữ liệu dẫn đến xung đột Source of Truth.

---

## 6. Multi-Session Behavior (Đa phiên)

Một VirtualCharacter có thể hoạt động trong nhiều Session cùng lúc (Ví dụ: chat với User A trên điện thoại, và chat với User B trên web).

```text
Character A
├── Session X (User A)
└── Session Y (User B)
```

**Nguyên tắc Cô lập (Isolation):**
- **Cô lập Memory/Relationship**: Session X **tuyệt đối không** được đọc/ghi RelationshipState hay SessionState của Session Y.
- **Chia sẻ CharacterState**: Cả 2 Session dùng chung Personality của Character A. Tuy nhiên, nếu Character A đang bực mình (Emotional State) vì Session X, sự bực mình đó *có thể* ảnh hưởng đến Session Y (do dùng chung `CharacterState`), trừ khi Emotional State được kiến trúc tách biệt theo từng user (điều này do `vc-core::state` quyết định, không phải Session).

---

## 7. Quan hệ với các Subsystem khác

### 7.1. Session và Memory
Session **KHÔNG** quyết định điều gì sẽ trở thành bộ nhớ dài hạn (Long-term memory).
- Khi Session kết thúc (Completed), runtime có thể emit một sự kiện `SessionCompleted`.
- Subsystem Memory lắng nghe sự kiện này và quyết định xem có cần tổng hợp (Consolidate) các tương tác trong Session thành Memory mới hay không.
- Session chỉ cung cấp "vỏ bọc thời gian", Memory cung cấp "ý nghĩa".

### 7.2. Session và Context
Session **KHÔNG** tự xây dựng LLM Context.
- Khi Interaction bắt đầu, Session cung cấp lịch sử các `Message` gần đây.
- `ContextBuilder` nhận lịch sử này, cắt gọt, kết hợp với Memory và CharacterState để tạo ra Context cuối cùng.

### 7.3. Session và Decision
Session **KHÔNG** quyết định nhân vật sẽ làm gì.
- Session chỉ định nghĩa môi trường ("Đang ở trong conversation nào, topic là gì").
- `DecisionEngine` mới là người trả lời: "Nên làm gì?".

---

## 8. Persistence (Lưu trữ) và Resume (Phục hồi)

### Phân định những gì cần lưu (Persistent):
- Metadata của Session và Conversation (IDs, Timestamps, Participants).
- Danh sách các `Message` (Conversation History).

### Phân định những gì sẽ bị xóa (Ephemeral):
- `Context` đang được build dở dang.
- Các decision đang suy nghĩ.
- Các connection handles (WebSocket, gRPC).

### Flow Resume một Session:
1. Load `Session` từ DB.
2. Kiểm tra nếu `Status == Idle`, chuyển thành `Active`. (Nếu Completed/Expired thì từ chối).
3. Load lịch sử `Message` của `Conversation` đang active.
4. Tái tạo lại `SessionState` tạm thời (hoặc reset tùy rule).
5. Sẵn sàng nhận Interaction mới.

---

## 9. Concurrency & Idempotency (Đồng thời và Lũy đẳng)

- **Concurrency**:
  - Các Interaction trong *cùng một Session* phải được thực thi **tuần tự** (Serialized). Không được xử lý Message thứ 2 khi Message thứ 1 chưa hoàn thành vòng đời của nó.
  - Các Session *khác nhau* có thể thực thi song song, nhưng khi chúng update `CharacterState` dùng chung, `vc-runtime`/`vc-storage` phải đảm bảo cơ chế khóa (locking/optimistic concurrency) ở tầng repository.
- **Idempotency**:
  - Gọi `complete_session()` nhiều lần trên một session đã completed không sinh ra lỗi mới.
  - Gửi lại cùng một raw input (do retry mạng) không được tạo ra 2 Interaction trùng lặp làm hỏng Memory. Hệ thống nên check `InteractionId` hoặc deduplicate message.

---

## 10. API Boundary Căn bản (Dành cho Runtime)

Mặc dù không viết code, `vc-runtime` sẽ cần cung cấp các interface khái niệm sau cho CLI/Transport layer:

- `create_session(character_id, actor_ids)`
- `resume_session(session_id)`
- `end_session(session_id)`
- `append_interaction(session_id, raw_input)` -> Trigger toàn bộ vòng đời thiết kế trong `interaction.md`.

---

## 11. Các nguyên tắc thiết kế bất di bất dịch (Design Principles)

1. Session là trạng thái quản lý vòng đời (lifecycle state), **không phải** bộ nhớ (memory).
2. Session chứa lịch sử hội thoại hiện tại, nhưng Session **không phải** là bản thân lịch sử hội thoại.
3. Không được ghi đè `CharacterState` hoặc `RelationshipState` cố định bằng dữ liệu tạm thời của `SessionState`.
4. Ranh giới Session phải bảo toàn tính riêng tư tuyệt đối (Isolation) giữa các người dùng khác nhau.
5. Khi phân vân giữa việc "giữ Conversation sống mãi" và "tạo Session mới", hãy luôn ưu tiên **tạo Session mới** và dùng **Memory** để nhớ lại chuyện cũ.
