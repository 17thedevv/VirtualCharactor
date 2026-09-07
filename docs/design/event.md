# VirtualCharacter Domain Event System

## 1. Tổng quan (Overview)

Domain Event System (Hệ thống sự kiện miền) cung cấp một cơ chế để ghi nhận và thông báo về những "sự việc có ý nghĩa đã thực sự xảy ra" bên trong VirtualCharacter.

Mục tiêu của hệ thống này là **giảm thiểu sự ghép nối (decoupling)** giữa các subsystem. Ví dụ: Khi một Interaction kết thúc, thay vì `vc-runtime` phải gọi trực tiếp `memory.consolidate()`, runtime chỉ việc phát ra một event `InteractionCompleted`. Các subsystem khác có thể tự động lắng nghe và phản ứng theo cách của riêng chúng.

Tài liệu này xác định ý nghĩa, vòng đời, và ranh giới của Domain Event trong kiến trúc Phase 1.

---

## 2. Domain Event là gì?

Một **Domain Event** là bản ghi mô tả một sự kiện có ý nghĩa đã xảy ra trong quá khứ.

Sự khác biệt cốt lõi:
- **Event vs Command**: Command (Lệnh) là yêu cầu làm một việc gì đó (VD: `UpdateMemory`). Event là thông báo rằng một việc đã hoàn tất (VD: `MemoryUpdated`). Command có thể bị từ chối; Event là sự thật không thể chối cãi.
- **Event vs State**: State là trạng thái hiện tại (VD: `joy = 0.8`). Event là sự dịch chuyển trạng thái (VD: `EmotionChanged { joy_delta: +0.2 }`).
- **Event vs Log**: Log dùng để debug cho lập trình viên (VD: `"LLM connection timeout"`). Event dùng để điều khiển logic nghiệp vụ (business logic) của hệ thống.

---

## 3. Cấu trúc Payload của Event

Một Event tiêu chuẩn cần có đủ ngữ cảnh để các subscriber không phải query lại database ngay lập tức.

### Các trường bắt buộc (Core Fields)
- `EventId`: Định danh duy nhất của event (UUID).
- `Timestamp`: Thời gian xảy ra event.
- `EventType`: Tên loại sự kiện (VD: `InteractionCompleted`).
- `CharacterId`: (Bắt buộc) Nhân vật liên quan đến sự kiện.

### Các trường tùy chọn (Contextual Fields)
- `SessionId` / `InteractionId` / `ActorId`: Nếu event xảy ra trong một bối cảnh cụ thể.
- `Payload`: Dữ liệu đặc thù của event.

*Ví dụ cấu trúc khái niệm:*
```rust
struct DomainEvent {
    id: Uuid,
    timestamp: DateTime,
    character_id: CharacterId,
    event_type: EventType,
    payload: EventPayload, // Chứa data cụ thể
}
```

---

## 4. Các danh mục sự kiện (Event Categories)

Dưới đây là các loại sự kiện tham chiếu mà kiến trúc hỗ trợ:

### 4.1. Interaction & Session Events (Điều phối)
Phát ra bởi `vc-runtime`.
- `SessionStarted`: Session mới được tạo.
- `SessionCompleted`: Session kết thúc, có thể kích hoạt Memory Consolidation.
- `InteractionStarted`: Kích hoạt khi nhận input thô.
- `InteractionCompleted`: Kích hoạt khi response đã được gửi hoặc action đã thực thi xong.
- `InteractionFailed`: Ghi nhận lỗi để hệ thống học hỏi hoặc thử lại.

### 4.2. State Events (Trạng thái)
Phát ra bởi `vc-core::state` và `vc-core::relationship`.
- `EmotionChanged`: Trạng thái cảm xúc thay đổi (chứa `old_state` và `new_state`).
- `RelationshipChanged`: Mức độ thân thiết/tin tưởng thay đổi.

### 4.3. Decision & Memory Events (Nhận thức)
Phát ra bởi `vc-core::decision` và `vc-core::memory`.
- `DecisionMade`: Một quyết định "làm gì" vừa được chốt (trước khi gửi cho LLM).
- `MemoryCreated`: Một trí nhớ dài hạn mới vừa được hình thành.
- `MemoryRecalled`: Một memory cũ vừa được kích hoạt lại (có thể làm tăng `importance` của memory đó).

---

## 5. Vòng đời và Sự kiện (Lifecycle & Orchestration)

Trong Phase 1, hệ thống **KHÔNG** sử dụng Message Broker ngoại vi (như Kafka, Redis, RabbitMQ). Event bus là một **Local In-memory Dispatcher** nằm trong `vc-runtime`.

Vòng đời chuẩn:
1. **Publish**: Một subsystem thay đổi trạng thái thành công và push một `DomainEvent` vào bộ điều phối của runtime.
2. **Dispatch**: Runtime nhận Event và phân phối tuần tự cho các Subsystem đã đăng ký lắng nghe (Subscribers).
3. **React**: Subscriber (VD: `Memory`) nhận `SessionCompleted` và chạy logic tổng hợp bộ nhớ.

```text
[Interaction Orchestrator]
         ↓ (gọi hàm update)
   [vc-core::state] 
         ↓ (trả về event)
[EmotionChanged Event]
         ↓ (dispatch)
    [Event Bus]
         ↓ (lắng nghe)
  [vc-core::memory] (Tạo memory "Hôm nay tôi rất vui")
```

---

## 6. Thứ tự và Xử lý Đồng thời (Ordering & Concurrency)

- **Thứ tự tuyệt đối (Ordering)**: Các event thuộc về cùng một `CharacterId` hoặc `SessionId` **phải** được phát và xử lý theo đúng thứ tự thời gian xảy ra. Không được phép xử lý `InteractionCompleted` trước `InteractionStarted`.
- **Đồng bộ vs Bất đồng bộ**: 
  - Phase 1 ưu tiên xử lý **Đồng bộ (Synchronous)** hoặc tuần tự để tránh race conditions khó debug.
  - Các quá trình nặng (như tóm tắt hội thoại bằng LLM để tạo Memory) có thể được đẩy vào hàng đợi background job, nhưng việc *phát ra event* phải đồng bộ.

---

## 7. Persistence và Lũy đẳng (Persistence & Idempotency)

### 7.1. Persistence (Lưu trữ Event)
Không phải tất cả event đều bị vứt bỏ sau khi dispatch.
- **Event Log**: Hệ thống có thể lưu lại một bản append-only log của tất cả các event. Điều này đóng vai trò như "Nhật ký cuộc đời" (Life Log) của nhân vật, cực kỳ hữu ích cho việc debug (Event Sourcing) và ML Training sau này.
- **Ephemeral Events**: Một số event chỉ dùng để kích hoạt logic local tức thời và không cần lưu vào Database.

### 7.2. Lũy đẳng (Idempotency)
Bản thân việc xử lý Event phải mang tính lũy đẳng (Idempotent).
- Nếu subsystem Memory vô tình nhận lại event `MemoryCreated(id=123)` hai lần, nó phải kiểm tra và bỏ qua lần thứ hai, chứ không được tạo ra 2 memory trùng lặp.
- Các event mang tính tương đối (như `EmotionChanged { delta: +0.1 }`) cần được thiết kế cẩn thận. Tốt nhất payload nên chứa giá trị tuyệt đối (`new_value: 0.8`) để dễ dàng đảm bảo tính lũy đẳng.

---

## 8. Extensibility (Khả năng mở rộng cho Học máy - Learning)

Sức mạnh thực sự của Domain Event System là tạo nền tảng cho **Behavioral Learning** trong các Phase sau.

Thay vì hard-code logic học tập vào thẳng Interaction, chúng ta dựa vào event:
1. `DecisionMade(comfort)`
2. `ActionExecuted(LLM Response)`
3. `InteractionCompleted(Outcome: User tức giận)`

Một **Learning Subsystem** chạy ngầm có thể quét log của 3 event liên tiếp này, nhận ra rằng "Decision comfort trong Context X dẫn đến Outcome tiêu cực", và tự động cập nhật *Decision Weights* cho nhân vật.

---

## 9. Ma trận Trách nhiệm (Responsibility Matrix)

| Subsystem | Trách nhiệm đối với Event |
|-----------|---------------------------|
| `vc-core` | Định nghĩa các struct `DomainEvent`. Trả về event khi state thay đổi. |
| `vc-runtime` | Sở hữu `EventBus` (Dispatcher). Điều phối các event giữa các subsystem. |
| `vc-storage` | Lưu trữ Event Log xuống cơ sở dữ liệu (SQLite/File) nếu cần. |
| Học máy (Future) | Đọc Event Log để phân tích và tinh chỉnh tính cách/quyết định. |

---

## 10. Design Principles (Nguyên tắc thiết kế)

1. **Event là Sự thật**: Không bao giờ thay đổi nội dung của một event đã được phát ra.
2. **Local First**: Không sử dụng Kafka/Redis. Mọi thứ chạy in-memory trong process của Rust.
3. **Decoupled**: Subsystem phát event không cần biết ai đang lắng nghe nó.
4. **Fat Payload**: Event mang đủ dữ liệu cần thiết để subscriber xử lý mà không cần query lại database ngay lập tức.
5. **Cơ sở của học máy**: Event log là kho tàng dữ liệu (training data) của nhân vật.
