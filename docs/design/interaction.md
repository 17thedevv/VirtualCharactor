# VirtualCharacter Interaction Lifecycle

## 1. Tổng quan

Interaction (Tương tác) là đơn vị cốt lõi của hệ thống VirtualCharacter. Nó không đơn thuần là một request/response HTTP.

Tài liệu này định nghĩa một **Interaction** là gì, cách nó di chuyển qua các subsystem (State, Memory, Context, Decision, LLM), và ranh giới trách nhiệm của từng subsystem trong suốt vòng đời của nó.

Mục tiêu là đảm bảo Developer B (quản lý `vc-runtime`) hiểu rõ cách orchestrate các thành phần do Developer A (quản lý `vc-core`) định nghĩa, mà không làm rò rỉ (leak) logic của nhau.

---

## 2. Các khái niệm nền tảng (Fundamental Concepts)

Để tránh nhầm lẫn, các khái niệm sau phải được phân biệt rõ ràng:

- **Session**: Một khoảng thời gian liên tục mà VirtualCharacter khả dụng cho một Actor (User). Session có thể bắt đầu khi user mở app và kết thúc khi đóng app.
- **Conversation**: Một chuỗi các message có tính liên kết về mặt ngữ cảnh. Một Session có thể có nhiều Conversation.
- **Interaction**: Một đơn vị xử lý đơn lẻ, bắt đầu bằng một kích hoạt (trigger) từ bên ngoài và kết thúc khi hệ thống hoàn tất việc cập nhật trạng thái/phản hồi. (Trọng tâm của tài liệu này).
- **Message**: Dữ liệu thô (raw input) từ Actor gửi đến.
- **Decision**: Quyết định của hệ thống về *việc cần làm* (do `vc-core::decision` đưa ra).
- **Response / Action**: Phản hồi thực tế (text, voice) hoặc hành động (tool call) được sinh ra (bởi `vc-llm` hoặc runtime).
- **Outcome**: Kết quả quan sát được sau khi thực thi Action/Response, dùng để cập nhật State hoặc Memory.

---

## 3. Interaction Identity

Mỗi Interaction cần được định danh để phục vụ tracking, retry, và observability.

Các định danh bao gồm:
- `InteractionId`: (Bắt buộc) Sinh ra ngẫu nhiên (UUID) mỗi khi Interaction bắt đầu. **Ephemeral** (chỉ tồn tại trong lúc xử lý hoặc log).
- `CharacterId`: (Bắt buộc) Xác định VirtualCharacter nào đang xử lý. **Persistent**.
- `ActorId`: (Bắt buộc) Xác định ai đang tương tác (User X, System). **Persistent**.
- `SessionId`: (Tùy chọn ở mức core, nhưng quan trọng ở mức runtime).
- `Timestamp`: (Bắt buộc) Thời điểm kích hoạt.

---

## 4. Interaction Lifecycle (Quy trình tham chiếu)

`vc-runtime` chịu trách nhiệm điều phối quy trình này. Quy trình chuẩn cho một User-initiated Interaction:

```text
                 [1. Raw Input]
                     │
                     ↓
              [2. Validation]
                     │
                     ↓
         [3. Load Character State]
                     │
                     ↓
         [4. Situation Understanding]
                     │
              ┌──────┴──────┐
              ↓             ↓
      [5. Retrieve Memory] [6. Read State]
              │             │
              └──────┬──────┘
                     ↓
           [7. Build Context]
                     │
                     ↓
               [8. Decision]
                     │
              ┌──────┴──────┐
              ↓             ↓
          [9A. LLM]     [9B. Tool/Action]
              │             │
              └──────┬──────┘
                     ↓
               [10. Outcome]
                     │
             ┌───────┴───────┐
             ↓               ↓
      [11. State Update] [12. Memory Update]
             │               │
             └───────┬───────┘
                     ↓
             [13. Finalize]
```

### Chi tiết các bước:
1. **Receive Raw Input**: `vc-runtime` nhận Message.
2. **Validate**: Kiểm tra tính hợp lệ của `CharacterId`, `ActorId`.
3. **Load State**: Truy xuất `CharacterState`, `RelationshipState`, `Personality` từ `vc-storage`.
4. **Situation Understanding**: Phân tích input thô thành Intent/Emotion (Hiện tại có thể dùng rules; tương lai có thể dùng ML model nhỏ).
5. **Memory Retrieval**: Dùng tín hiệu từ bước 4 để query `MemoryRepository`.
6. **Context Construction**: `ContextBuilder` nén và sắp xếp dữ liệu thành một `Context` có budget.
7. **Decision**: Truyền Context vào `DecisionEngine` để nhận lại một `Decision` (VD: action="comfort").
8. **LLM Invocation**: Nếu Decision yêu cầu sinh text, gửi `Context` và `Decision` vào `LlmProvider` để lấy `LlmResponse`.
9. **Outcome**: Ghi nhận kết quả (đã gửi thành công cho User chưa).
10. **State/Memory Update**: Dựa trên Outcome, gọi `vc-core` để thay đổi `EmotionalState` hoặc tạo Memory mới, sau đó persist qua `vc-storage`.

---

## 5. Tương tác với các Subsystem (Cross-Subsystem Boundaries)

Quy tắc quan trọng: **Interaction (Runtime) là Orchestrator, không chứa Domain Logic.**

### 5.1. Với Memory
- Runtime gọi `memory_query` dựa trên tình huống hiện tại.
- Runtime **không** quyết định message nào được lưu thành Long-term memory. Việc đánh giá `ShouldRemember` phải do `vc-core::memory` quyết định trong bước Update.

### 5.2. Với State
- Runtime truyền Input và Outcome vào State subsystem.
- Runtime **không** tự tính toán `joy + 0.1`. Nó phải gọi hàm `transition()` của `vc-core::state`.

### 5.3. Với Relationship
- Interaction đọc `RelationshipState` để quyết định communication style.
- Không được phép rò rỉ (leak) memory của User A sang Interaction của User B.

### 5.4. Với Decision và LLM
- Runtime truyền Data cho `DecisionEngine`.
- Runtime lấy `Decision` đó truyền cho `LlmProvider`.
- Quyết định **làm gì** là của Core. Quyết định **nói thế nào** là của LLM.

---

## 6. Giao dịch, Lỗi và Tính lũy đẳng (Failure & Idempotency)

Hệ thống ưu tiên trạng thái local-first, không yêu cầu Distributed Transaction khổng lồ, nhưng cần xử lý lỗi tinh tế:

- **Interaction Failure vs Subsystem Failure**: 
  - Nếu LLM timeout (Subsystem Failure), Runtime có thể thử lại.
  - Nếu LLM fail hoàn toàn, Runtime có thể trả về lỗi cho User, nhưng KHÔNG được update `Memory` với thông tin sai lệch.
- **Cập nhật dữ liệu (Updates)**: Nên được thực hiện theo cơ chế **Best-effort** ở cuối chu kỳ.
- **Idempotency (Tính lũy đẳng)**: Nếu cùng một `InteractionId` và Input được gửi lại (do retry từ client), hệ thống không được phép sinh ra 2 Memory giống hệt nhau hoặc cộng dồn Emotion 2 lần.

---

## 7. Concurrency & Re-entrancy (Xử lý song song)

Mô hình đồng thời (Concurrency model) được kỳ vọng:

- **Memory Queries**: Có thể chạy song song (Parallel).
- **State Transitions**: Phải được xử lý tuần tự (Serialized) theo từng `CharacterId` + `ActorId` để tránh race condition khi cộng/trừ emotion.
- **Re-entrancy**: Trong **Phase 1**, Character **không** hỗ trợ nested interactions (ví dụ: đang xử lý message A, LLM tự trigger một interaction B ngầm). Hệ thống chỉ xử lý tuần tự (Request -> Response).

---

## 8. Cancellation (Hủy bỏ)

Nếu Interaction bị hủy (User tắt app, LLM timeout quá lâu):
- Các Side-effects (State Update, Memory Creation) **phải bị hủy bỏ** nếu Outcome chưa được xác nhận.
- Cần đảm bảo Character không bị kẹt ở trạng thái "Đang suy nghĩ" vô thời hạn.

---

## 9. Observability (Quan sát)

Cần phân biệt rõ:
- **InteractionResult** (Public): Trả về cho User (Chỉ chứa `Response` Text hoặc Action được phép thấy).
- **InteractionTrace** (Internal/Private): Chứa toàn bộ vòng đời (Context budget đã dùng, Memory đã retrieve, Decision được chọn, Raw LLM Response). `InteractionTrace` liên kết bằng `InteractionId` và chỉ dùng để debug/telemetry. Dữ liệu này **không** tự động biến thành LLM Context cho lượt chat sau.

---

## 10. Responsibility Matrix (Ma trận trách nhiệm)

Đây là kim chỉ nam cho Dev A và Dev B:

| Concern (Vấn đề) | Owner (Trách nhiệm) |
|------------------|---------------------|
| Character Identity | `vc-core` (Dev A) |
| Personality Rules | `vc-core` (Dev A) |
| Emotion & State Semantics | `vc-core` (Dev A) |
| Relationship Semantics | `vc-core` (Dev A) |
| Memory Evaluation (Should remember?) | `vc-core` (Dev A) |
| Context Construction | `vc-core::context` (Dev B) |
| Decision Policy | `vc-core::decision` (Dev A) |
| Memory Retrieval Logic | `vc-storage` / `vc-runtime` (Dev B) |
| LLM Communication & Mapping | `vc-llm` (Dev B) |
| Persistence (DB/SQLite) | `vc-storage` (Dev B) |
| **Interaction Lifecycle & Orchestration** | **`vc-runtime` (Dev B)** |

---

## 11. Các trạng thái của Interaction (Conceptual State Machine)

Interaction trải qua các trạng thái (cho mục đích thiết kế, không bắt buộc implement strict state machine ngay lập tức):

```text
Created
  ↓
Validated
  ↓
Processing
  ├── BuildingContext
  ├── Deciding
  └── Executing (LLM/Tool)
  ↓
AwaitingOutcome
  ↓
Updating (State/Memory)
  ↓
Completed
```

Các trạng thái bất thường:
- `Failed` (Lỗi hệ thống, LLM sập).
- `Cancelled` (Bị ngắt ngang).

---

## 12. Khả năng mở rộng (Extensibility)

Kiến trúc này được thiết kế để không cần đập đi xây lại khi thêm công nghệ mới ở các phase sau:
- **Small ML Perception Model**: Có thể cắm vào bước `Situation Understanding` thay cho rules.
- **Proactive Behavior**: Hệ thống có thể tự kích hoạt một `Interaction` từ Background Event (VD: Cronjob nhắc nhở) thay vì đợi Message từ User.
- **Multimodal Input**: Message ở bước (1) có thể chứa Hình ảnh/Audio, ContextBuilder sẽ xử lý việc nhúng vào `LlmRequest`.

Tài liệu này xác nhận **vòng đời chuẩn mực**, cho phép hai nhà phát triển độc lập triển khai `vc-core` và `vc-runtime` song song mà không dẫm chân lên nhau.
