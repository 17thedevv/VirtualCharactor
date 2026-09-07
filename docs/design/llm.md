# VirtualCharacter LLM Provider System

## 1. Tổng quan

LLM Provider (`vc-llm`) là subsystem đóng vai trò là "cơ quan ngôn ngữ và lập luận" (reasoning engine) của VirtualCharacter.

Nhiệm vụ của nó là nhận các chỉ thị từ hệ thống (Context và Decision) và chuyển đổi chúng thành phản hồi bằng ngôn ngữ tự nhiên hoặc các hành động cụ thể thông qua API của một Large Language Model (như Google Gemini).

Nếu:
- **Personality** định nghĩa nhân vật là ai.
- **State** định nghĩa nhân vật đang cảm thấy thế nào.
- **Memory** định nghĩa nhân vật đã trải qua những gì.
- **Context** định nghĩa những thông tin quan trọng nhất lúc này.
- **Decision** quyết định nhân vật nên làm gì.

Thì **LLM Provider** quyết định:
> **"Làm thế nào để diễn đạt hoặc thực thi quyết định đó một cách phù hợp nhất với Personality và Context?"**

---

## 2. Trách nhiệm của lớp LLM Provider

Lớp `vc-llm` có những trách nhiệm giới hạn và rất cụ thể:

1. **Abstraction**: Cung cấp một interface độc lập với nhà cung cấp (provider-agnostic) cho `vc-runtime`.
2. **Translation**: Chuyển đổi từ `LlmRequest` (chuẩn nội bộ) sang payload đặc thù của nhà cung cấp (ví dụ: REST/gRPC của Gemini).
3. **Execution**: Giao tiếp với API của LLM, quản lý network calls, timeouts, và retries.
4. **Mapping**: Chuyển đổi response đặc thù của LLM thành `LlmResponse` (chuẩn nội bộ) và map các lỗi HTTP/API thành `vc_core::CoreError`.

`vc-llm` **KHÔNG** chịu trách nhiệm:
- Xây dựng Context (đây là việc của `ContextBuilder`).
- Quyết định hành động (đây là việc của `DecisionEngine`).
- Lưu trữ lịch sử hội thoại (đây là việc của `Memory` hoặc `SessionState`).

---

## 3. The `LlmProvider` Contract

Interface cốt lõi được định nghĩa tại `vc-core` (hoặc `vc-llm::provider`) để các thành phần khác sử dụng mà không cần biết LLM bên dưới là gì.

### 3.1. Core Trait

```rust
pub trait LlmProvider {
    fn generate_text(&self, request: LlmRequest) -> vc_core::Result<LlmResponse>;
}
```

*Lưu ý: Trong Phase 1, interface được giữ dạng synchronous để đơn giản hóa kiến trúc. Nếu có yêu cầu strict về async I/O từ `vc-runtime` trong tương lai, trait này sẽ được nâng cấp lên `async fn`.*

### 3.2. LlmRequest

Input tiêu chuẩn gửi đến Provider. Nó độc lập hoàn toàn với Gemini hay OpenAI.

```rust
pub struct LlmRequest {
    /// Nội dung chính cần xử lý (ví dụ: Context + Lịch sử hội thoại)
    pub prompt: String,
    
    /// Hướng dẫn hệ thống (ví dụ: "You are Hikari, a helpful assistant...")
    pub system_instruction: Option<String>,
}
```

### 3.3. LlmResponse

Output tiêu chuẩn nhận từ Provider.

```rust
pub struct LlmResponse {
    /// Chuỗi văn bản LLM sinh ra
    pub text: String,
}
```

---

## 4. Định hướng mở rộng (Future Extensibility)

Trong các Phase tiếp theo, hợp đồng `LlmProvider` sẽ được mở rộng (theo nguyên tắc Additive Changes) để hỗ trợ các tính năng nâng cao mà không phá vỡ kiến trúc cũ:

### 4.1. Structured Output (JSON)
Thêm schema vào `LlmRequest` để yêu cầu LLM trả về JSON có cấu trúc (ví dụ: dùng cho Extracting Memory hoặc Phân tích Cảm xúc).
```rust
// Tương lai
pub struct LlmRequest {
    // ...
    pub response_schema: Option<JsonSchema>,
}
```

### 4.2. Tool Calling (Function Calling)
Cung cấp danh sách các tools mà LLM có thể gọi.
```rust
// Tương lai
pub struct LlmRequest {
    // ...
    pub tools: Vec<ToolDefinition>,
}
pub struct LlmResponse {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
}
```

### 4.3. Metadata & Metrics
Trả về thông tin sử dụng token và độ trễ để phục vụ Context Budgeting.
```rust
// Tương lai
pub struct LlmResponse {
    // ...
    pub tokens_used: usize,
    pub finish_reason: String,
}
```

### 4.4. Streaming
Hỗ trợ `generate_stream` trả về một `Stream` thay vì đợi toàn bộ response (phục vụ cho UX mượt mà hơn).

---

## 5. Các Implementations (Phạm vi của Developer B)

Developer B sẽ chịu trách nhiệm xây dựng các implementation cụ thể cho trait này bên trong crate `vc-llm`.

### 5.1. `MockLlmProvider`
Dùng cho Integration Tests và Local Development trong Phase 1.
- Không yêu cầu API Key.
- Không có network call.
- Có thể trả về response cố định (`default_response`) hoặc mô phỏng lỗi (timeout, rate limit) để test khả năng chịu lỗi của `vc-runtime`.

### 5.2. `GeminiProvider`
Implementation thực tế giao tiếp với Google Gemini API.
- **Configuration**: Cần nhận API Key từ biến môi trường hoặc file config khi khởi tạo (không hardcode).
- **Mapping**: Chuyển đổi `system_instruction` thành trường `system_instruction` của Gemini API, và `prompt` thành `contents`.
- **Safety Settings**: Cấu hình các threshold về an toàn (Harassment, Hate Speech, v.v.) ở mức độ phù hợp với nhân vật.
- **Error Handling**: Bắt các lỗi từ HTTP Client (timeout, 5xx) hoặc từ API (429 Rate Limit) và chuyển đổi thành `vc_core::CoreError::LlmError`.

---

## 6. Integration Pipeline (Runtime)

`LlmProvider` không đứng độc lập mà nằm trong orchestration pipeline của `vc-runtime`.

Luồng dữ liệu (Data Flow):

1. **Runtime** thu thập `CharacterState`, `Personality`, `Relationship` từ `vc-core`.
2. **ContextBuilder** (do Dev B viết) nén lịch sử và tạo ra một `Context` object.
3. **DecisionEngine** (do Dev A viết) đánh giá tình huống và đưa ra một `Decision` (ví dụ: `action_type: "comfort", payload: "User failed interview"`).
4. **Runtime** kết hợp `Context` và `Decision` thành một chuỗi `prompt` và `system_instruction` thuần túy.
5. **Runtime** gọi `LlmProvider::generate_text(LlmRequest)`.
6. LLM trả về `LlmResponse`.
7. **Runtime** lấy text từ response và trả về cho User hoặc gọi các hệ thống khác (ví dụ: TTS, UI).

---

## 7. Ràng buộc Kiến trúc (Architectural Constraints)

Để đảm bảo tính độc lập theo đúng quy tắc Phase 0:

- **Tuyệt đối không rò rỉ (No Leaking)**: Các kiểu dữ liệu đặc thù của Gemini (ví dụ: `GeminiGenerateContentRequest`, `Candidate`, `ContentPart`) **chỉ được phép** tồn tại bên trong module `vc-llm::gemini`. Chúng không được xuất hiện trong chữ ký hàm (function signatures) public.
- **Dependency Direction**: `vc-core` **không bao giờ** import `vc-llm`. `vc-core` chỉ biết về trait (nếu được đặt ở core) hoặc runtime sẽ đảm nhiệm việc dependency injection.
- **Ownership**: `vc-llm` thuộc hoàn toàn về **Developer B**. Developer A chỉ quan tâm đến việc `Decision` của họ sẽ được `vc-runtime` truyền cho LLM như thế nào.
