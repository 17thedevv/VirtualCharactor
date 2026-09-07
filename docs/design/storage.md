# VirtualCharacter Storage & Persistence System

## 1. Tổng quan (Overview)

Tài liệu này định nghĩa hệ thống lưu trữ (Storage System) của VirtualCharacter. Mục tiêu của kiến trúc này là tách biệt hoàn toàn **Domain Logic** (những gì nhân vật nghĩ và làm) khỏi **Persistence Infrastructure** (cách dữ liệu được ghi vào ổ cứng).

Trong Phase 1, mục tiêu lưu trữ hướng đến **Local-First**, nghĩa là dữ liệu nằm cục bộ trên thiết bị của người dùng hoặc máy chủ chạy runtime trực tiếp (Ví dụ: SQLite), thay vì một hệ thống phân tán khổng lồ.

---

## 2. Ranh giới Khái niệm (Conceptual Boundaries)

Sự nhầm lẫn giữa Domain và Storage là nguyên nhân lớn nhất gây phá vỡ kiến trúc. Hãy ghi nhớ sự khác biệt:

1. **Domain Object**: Là các `struct` nằm trong `vc-core` (VD: `Character`, `Memory`). Chúng là các đối tượng thuần Rust, chứa business logic, rules, và **TUYỆT ĐỐI KHÔNG** chứa các tag ORM (như `#[sqlx(...)]`).
2. **Persistence Model**: Là các `struct` nằm trong `vc-storage`. Chúng ánh xạ 1-1 với cấu trúc bảng trong cơ sở dữ liệu. Dữ liệu từ DB được load vào Persistence Model, sau đó *map* (chuyển đổi) sang Domain Object trước khi trả về cho Runtime.
3. **Repository Trait**: Interface định nghĩa các thao tác CRUD ở mức Domain. (VD: `trait CharacterRepository`). Interface này được định nghĩa ở `vc-storage::repository` (hoặc `vc-core` tùy cách chia trait), nhưng chữ ký hàm của nó **chỉ dùng Domain Object**, không được phép leak các lỗi cụ thể của SQL ra ngoài.
4. **Storage Adapter**: Implementation thực tế (VD: `SqliteRepository`, `InMemoryRepository`) nằm trong `vc-storage`.

---

## 3. Dữ liệu Persistent vs Ephemeral

Không phải mọi dữ liệu trong hệ thống đều được lưu lại.

### 3.1. Dữ liệu Persistent (Bắt buộc lưu trữ)
- **Character Data**: `CharacterId`, `Personality` (Core Traits, Values, Boundaries).
- **State**: `EmotionalState` hiện tại của nhân vật.
- **Relationship**: `RelationshipState` (Trust, Affection) giữa nhân vật và từng Actor.
- **Memory**: Trí nhớ dài hạn (Long-term memories).
- **Conversation History**: Các `Message` đã được trao đổi trong các Session.
- **Event Log**: (Tùy chọn nhưng khuyến nghị) Bản ghi dạng Append-only của các Domain Events để phục vụ Debug và Learning sau này.

### 3.2. Dữ liệu Ephemeral (Sẽ bị xóa/Mất khi khởi động lại)
- **Session State**: Bộ đếm (turn count), topic tạm thời, mood của phiên chat hiện tại. Khi load lại Session, state này được tái tạo thay vì lưu cứng.
- **Context Budget**: Object `Context` được nén và gửi cho LLM. Nó được build động mỗi lần.
- **Decision Engine Temp Data**: Các `Candidate` action đang được đánh giá nhưng chưa chốt.
- **In-flight Requests**: Các `LlmRequest` đang chờ API trả về.

---

## 4. Ranh giới Repository (Repository Boundaries)

Repositories nên được thiết kế theo hướng **Aggregate-Oriented** (Hướng cụm dữ liệu), thay vì mỗi bảng một repository.

- **`CharacterRepository`**: Quản lý `Character`, bao gồm cả việc load/save `Personality` và `CharacterState`. Không nên tách ra làm 3 repository nhỏ lẻ vì State không có ý nghĩa nếu đứng độc lập khỏi Character.
- **`RelationshipRepository`**: Quản lý state giữa một Character và một Actor cụ thể.
- **`MemoryRepository`**: Quản lý kho trí nhớ. Chịu trách nhiệm thực thi các câu query dựa trên semantic/tags/recency.
- **`SessionRepository`**: Lưu trữ metadata của Session và quản lý append lịch sử `Message`.

---

## 5. Transaction & Consistency (Giao dịch và Tính nhất quán)

Trong môi trường Local-First (đặc biệt với SQLite), chúng ta có thể tận dụng lợi thế của local transactions để đảm bảo tính toàn vẹn dữ liệu.

### 5.1. Transaction Boundaries
- Các thao tác cập nhật liên quan đến một Interaction nên được gom vào một Unit of Work.
- Ví dụ: Khi một Interaction kết thúc, nếu `CharacterState` bị trừ đi 0.1 Joy, và một `Memory` mới được tạo ra, cả hai thao tác này nên được commit trong cùng một **Database Transaction**. Nếu một thao tác thất bại, cả hai đều rollback, tránh tình trạng trí nhớ được lưu nhưng cảm xúc bị mất đồng bộ.

### 5.2. Eventual vs Strong Consistency
- **Strong Consistency (Nhất quán mạnh)**: Lưu Message, lưu State update. Runtime phải đợi Storage báo OK thì mới tính là Interaction Completed.
- **Eventual Consistency (Nhất quán độ trễ)**: Các task chạy ngầm như Memory Consolidation (tổng hợp trí nhớ) có thể chạy ở background và lưu sau.

---

## 6. Serialization & Khởi tạo (Serialization & Versioning)

### 6.1. Serialization Strategy
- Các trường dùng để query (VD: `character_id`, `created_at`, `event_type`) phải được lưu thành các Column riêng biệt.
- Các trường có cấu trúc linh hoạt hoặc có khả năng mở rộng liên tục (VD: `Personality.traits`, `Memory.metadata`, `EventPayload`) nên được lưu dưới dạng **JSON string** hoặc **JSONB** (nếu DB hỗ trợ) trong một column. Điều này giúp tránh việc phải sửa schema liên tục trong quá trình phát triển Phase 1.

### 6.2. Schema Versioning (Migrations)
- **Phase 1**: Giữ kiến trúc di trú (migration) đơn giản nhất có thể. Khuyến nghị sử dụng các file `.sql` khởi tạo tự động khi file SQLite chưa tồn tại.
- **Phase tiếp theo**: Khi hệ thống stable, áp dụng các công cụ lightweight migration.

---

## 7. Concurrency & Failure Handling (Xử lý đồng thời và Lỗi)

### 7.1. Concurrency (Xử lý song song)
SQLite hỗ trợ Concurrent Reads rất tốt, nhưng Concurrent Writes sẽ lock database.
- Runtime (Orchestrator) **phải** đảm bảo rằng các Interaction ghi dữ liệu cho cùng một Character được Serialize (Thực thi tuần tự).
- Không được để 2 luồng xử lý Interaction cùng cố gắng update `CharacterState` của Hikari đồng thời.

### 7.2. Failure Handling
Nếu Storage bị lỗi (Disk Full, Lock Timeout, Corrupt):
- `vc-storage` bắt lỗi SQL và chuyển đổi nó thành `CoreError::StorageError("Disk full")`.
- `vc-runtime` nhận lỗi này và **Hủy bỏ (Cancel)** các thao tác tiếp theo của Interaction.
- **Nguyên tắc**: Thà mất một tin nhắn còn hơn lưu một trạng thái bị hỏng (corrupted state).

---

## 8. Định hướng triển khai (Future Implementation)

### 8.1. In-Memory Storage (Giai đoạn đầu)
Trong những ngày đầu của Phase 1, Developer B nên tạo ra các `InMemoryCharacterRepository`, `InMemoryMemoryRepository` bằng `HashMap` và `RwLock`. Điều này cho phép Developer A test toàn bộ orchestration pipeline mà không cần đợi thiết kế xong database schema.

### 8.2. SQLite Storage (Mục tiêu Phase 1)
Sau khi In-Memory chạy ổn định, thay thế bằng `SqliteRepository`.
- Không sử dụng các ORM quá cồng kềnh. Khuyến nghị sử dụng `rusqlite` (đồng bộ, nhẹ, nhanh) hoặc `sqlx` (nếu bắt buộc cần Async I/O cho Runtime).
- Database file nên được lưu tại thư mục data cục bộ của người dùng (VD: `~/.virtualcharacter/data.db`).

---

## 9. Nguyên tắc thiết kế tóm lược

1. **Storage obeys Domain**: `vc-storage` phải tuân theo định nghĩa của `vc-core`, không có chiều ngược lại.
2. **Translate everything**: Luôn ánh xạ từ Persistence Model sang Domain Model. Không bao giờ trả entity có tag của Database cho Runtime xử lý.
3. **Graceful Failures**: Lỗi database phải được bắt và bọc lại, tránh crash runtime.
4. **JSON for flexibility**: Dùng JSON columns cho các cấu trúc thay đổi nhanh trong giai đoạn đầu.
5. **Local-first focus**: Tối ưu hóa cho SQLite, không lo lắng về Redis hay Cassandra lúc này.
