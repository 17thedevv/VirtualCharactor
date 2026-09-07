# VirtualCharacter Observability System

## 1. Tổng quan (Overview)

Hệ thống Observability (Khả năng quan sát) định nghĩa cách VirtualCharacter tiết lộ các hoạt động bên trong (runtime behavior) để phục vụ cho việc gỡ lỗi (debugging), chẩn đoán (diagnosis), đánh giá hiệu suất, và ghi nhận luồng thực thi.

Mục tiêu chính của tài liệu này là xác định ranh giới: **Làm sao để runtime có thể quan sát được toàn bộ quá trình quyết định của nhân vật mà không làm ô nhiễm (pollute) Domain Model bằng các SDK logging/metrics bên ngoài.**

---

## 2. Mục tiêu (Objectives)

Hệ thống Observability phải có khả năng trả lời các câu hỏi sau mà không cần debug trực tiếp vào code:
- Tại sao nhân vật lại chọn hành động này? (Decision logic)
- Những ký ức nào đã được truy xuất? (Memory retrieval)
- Trạng thái (Emotion, Trust) trước và sau tương tác là gì?
- LLM đã nhận được Context gì và trả về gì? (LLM payloads)
- Tương tác thất bại ở bước nào và do Subsystem nào gây ra?
- Các thao tác mất bao lâu? (Latency)

---

## 3. Ranh giới Khái niệm (Fundamental Boundaries)

Để bảo vệ `vc-core`, ranh giới Observability được định nghĩa như sau:

```text
       [Domain (vc-core)]
               │ (Trả về Kết quả / Lỗi / Domain Events)
               ↓
    [Runtime Behavior (vc-runtime)]
               │ (Tổng hợp, đo đạc thời gian, gắn InteractionId)
               ↓
    [Observability Output] (Log/Traces)
```

**Nguyên tắc cốt lõi:**
- `vc-core` **không** phụ thuộc vào bất kỳ thư viện logging/tracing nào ở mức Application (như `tracing` exporter, Prometheus, hay OpenTelemetry SDK).
- Nếu `vc-core` gặp lỗi, nó trả về `Result<T, CoreError>`.
- `vc-runtime` là nơi bắt lỗi đó, phân tích nó xảy ra ở bước nào trong lifecycle, và ghi log hoặc đẩy trace ra ngoài.

---

## 4. Observability vs. Domain Events

Tuyệt đối không nhầm lẫn giữa dữ liệu Observability và Domain Events.

| Đặc điểm | Domain Events (VD: `EmotionChanged`) | Observability Data (VD: Interaction Trace) |
|----------|----------------------------------------|--------------------------------------------|
| **Mục đích** | Phục vụ Business Logic và Machine Learning. | Phục vụ lập trình viên Debug và vận hành. |
| **Bản chất** | Là một phần của Dữ liệu hệ thống (Domain). | Dữ liệu phụ trợ, vứt đi hệ thống vẫn chạy. |
| **Lưu trữ** | Lưu vào Database (Event Log) lâu dài. | In ra stdout, log files, hoặc external monitoring. |
| **Độ chi tiết** | Chỉ ghi nhận kết quả cuối cùng (Sự thật). | Ghi cả quá trình (Thời gian chạy, HTTP raw body, Stacktrace). |

---

## 5. Interaction Tracing (Cơ chế cốt lõi)

Đơn vị quan sát chính trong hệ thống là **Interaction Trace**, được định danh bằng `InteractionId`.

Trong suốt vòng đời của một Interaction (đã định nghĩa ở `interaction.md`), `vc-runtime` sẽ mở một "Span" (hoặc Trace Context) liên kết với `InteractionId`. Mọi sự kiện diễn ra trong vòng đời đó đều được gắn tag này.

Một Interaction Trace đầy đủ sẽ chứa:
1. **Input**: Message thô từ người dùng.
2. **State Snapshot**: Trạng thái cảm xúc/quan hệ trước khi xử lý.
3. **Memory Query**: Câu truy vấn và danh sách các Memory được lấy lên (kèm độ liên quan - relevance score).
4. **Context Budget**: Cấu trúc Context cuối cùng được nhồi vào LLM.
5. **Decision Candidates**: Danh sách các hành động được xem xét và quyết định cuối cùng được chọn.
6. **LLM I/O**: Payload gửi đi và raw response trả về (kèm token usage và latency).
7. **Outcome**: Kết quả thành công hay thất bại.
8. **State Transition**: Trạng thái thay đổi sau tương tác.

---

## 6. Triển khai (Phase 1: Structured Logging)

Trong Phase 0/1, hệ thống nhắm tới môi trường Local-First (CLI). Do đó, chúng ta **không** triển khai các hệ thống cồng kềnh như OpenTelemetry hay Prometheus lúc này.

Cơ chế quan sát duy nhất là **Structured Logging** (Log có cấu trúc).
Khuyến nghị sử dụng crate `tracing` của Rust vì nó hỗ trợ Spans tự nhiên.

### Cách thức hoạt động:
- `vc-runtime` tạo một Span cấp cao nhất: `info_span!("interaction", id = %interaction_id)`.
- Khi gọi xuống `vc-llm`, tạo Span con: `debug_span!("llm_invoke", provider = "gemini")`.
- Output log có dạng JSON hoặc Text có cấu trúc để các công cụ phân tích dễ dàng parse.

---

## 7. Bảo mật và Quyền riêng tư (Privacy & Sensitive Data)

Observability mang lại rủi ro rò rỉ dữ liệu nhạy cảm nếu log bừa bãi.

- **Secrets**: KHÔNG BAO GIỜ log API Keys (VD: `GEMINI_API_KEY`).
- **Memory & LLM Responses**: Lịch sử trò chuyện và trí nhớ của nhân vật chứa dữ liệu cá nhân của người dùng. Chúng **không** được log ở level `INFO`. Chỉ được log ở level `DEBUG` hoặc `TRACE` (vốn sẽ bị tắt trong môi trường production sau này).
- `vc-runtime` có trách nhiệm làm mờ (sanitize) dữ liệu nhạy cảm trước khi emit trace nếu cần thiết.

---

## 8. Theo dõi Lỗi (Error & Failure Tracking)

Khi một Interaction thất bại, Observability system phải trả lời được: "Lỗi do subsystem nào?".

- `vc-llm` báo lỗi mạng -> `LlmError`. Trace ghi nhận `subsystem="vc-llm"`, `reason="timeout"`.
- `vc-storage` báo lỗi khóa DB -> `StorageError`. Trace ghi nhận `subsystem="vc-storage"`, `reason="db_locked"`.
- `vc-core` báo lỗi logic -> `DomainError`. Trace ghi nhận `subsystem="vc-core"`, `reason="invalid_state_transition"`.

Nhờ việc gắn mọi hoạt động vào `InteractionId`, Developer có thể dễ dàng grep toàn bộ hành trình của một tương tác bị lỗi trong file log để tìm nguyên nhân gốc (Root Cause).

---

## 9. Nguyên tắc thiết kế tóm tắt

1. **Domain Purity**: Domain không biết đến khái niệm "Trace", "Span", hay "Prometheus".
2. **Runtime Orchestrates Traces**: `vc-runtime` chịu trách nhiệm mở Spans và gắn tags (metadata).
3. **Trace != Event**: Trace dùng để debug (hủy sau khi hết hạn). Domain Event dùng để học (lưu vĩnh viễn).
4. **No Premature External SDKs**: Dùng Structured Logging nội tại trước, không vội tích hợp OpenTelemetry cho đến khi hệ thống trở thành Server phân tán.
5. **Protect Privacy**: Dữ liệu hội thoại nhạy cảm chỉ nằm ở log level thấp (DEBUG/TRACE).
