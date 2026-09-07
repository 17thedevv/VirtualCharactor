# VirtualCharacter Configuration System

## 1. Tổng quan (Overview)

Configuration System (Hệ thống Cấu hình) định nghĩa cách VirtualCharacter nạp, phân phối và quản lý các thiết lập môi trường và tham số điều hành.

Mục tiêu cốt lõi của tài liệu này là thiết lập ranh giới rõ ràng: **Cấu hình không phải là Dữ liệu Nhân vật (Character Data)**. Hệ thống cấu hình chỉ quản lý "cách máy móc vận hành", chứ không quản lý "nhân vật nghĩ gì".

---

## 2. Phân định Khái niệm (Conceptual Separation)

Để tránh tình trạng "hardcode" sai chỗ hoặc lưu trữ sai loại dữ liệu, kiến trúc phân chia rạch ròi 7 loại dữ liệu:

1. **Character Data (Dữ liệu nhân vật)**: `Personality`, `Identity`, `Core Values`. Đây là dữ liệu thuộc về Domain (Core). Chúng nằm trong Database, **KHÔNG PHẢI** trong file cấu hình (`config.toml`).
2. **Runtime Configuration (Cấu hình điều hành)**: Các tham số cho `vc-runtime`. VD: `max_context_tokens`, `interaction_timeout_ms`, `memory_consolidation_interval`.
3. **Provider Configuration (Cấu hình hạ tầng)**: Thiết lập cho `vc-llm` hoặc `vc-storage`. VD: `gemini_model_name` ("gemini-1.5-pro"), `sqlite_db_path`.
4. **Environment Configuration (Cấu hình môi trường)**: Các cấu hình đặc thù của máy host. VD: `PORT`, `LOG_LEVEL`.
5. **Secrets (Bảo mật)**: Các khóa API (VD: `GEMINI_API_KEY`). **Tuyệt đối không** lưu vào Database, không log ra màn hình, không hardcode.
6. **User Preferences (Tùy chọn của người dùng)**: Sở thích của user đối với nhân vật. Lưu vào Database (gắn với Relationship hoặc User Model), **không** nằm trong cấu hình hệ thống.
7. **Learned/Adaptive State (Trạng thái học được)**: Các thay đổi hành vi do nhân vật tự học. Lưu vào `CharacterState` hoặc `Memory`, **không** phải cấu hình.

*Nguyên tắc vàng:*
```text
Character Data ≠ Runtime Configuration ≠ Environment Configuration ≠ Secrets
```

---

## 3. Quyền sở hữu và Phạm vi theo Crate (Ownership & Scope)

Để bảo vệ quy tắc Dependency Inversion của Phase 0, các crate có các quyền hạn khác nhau đối với Cấu hình.

### 3.1. `vc-core` (Domain)
- **Không bao giờ** đọc biến môi trường (Environment variables).
- **Không bao giờ** đọc file trên ổ cứng.
- Chỉ nhận cấu hình thông qua các `struct` tham số (VD: `CoreConfig`) được truyền vào từ Runtime lúc khởi tạo.

### 3.2. `vc-runtime` (Orchestrator)
- Chịu trách nhiệm nạp (load) cấu hình từ các nguồn (File, Env, CLI).
- Khởi tạo các `struct` cấu hình cho từng subsystem (VD: `LlmConfig`, `StorageConfig`, `CoreConfig`).
- Phân phối cấu hình (Dependency Injection) xuống các crate bên dưới.

### 3.3. `vc-llm` và `vc-storage` (Infrastructure)
- Nhận `Config` (chứa tên model, timeout, connection string) và `Secrets` từ `vc-runtime`.
- Không tự ý đọc `env::var("GEMINI_API_KEY")` sâu bên trong logic. Việc này phải được Inject từ ngoài vào hàm khởi tạo provider.

---

## 4. Thứ tự ưu tiên và Nạp cấu hình (Precedence & Loading Semantics)

Cấu hình được nạp theo thứ tự ưu tiên từ thấp đến cao (cái sau đè cái trước):

1. **Default Values**: Giá trị mặc định được code cứng (hardcode) trong Rust (VD: `timeout = 30s`).
2. **Configuration File**: File cấu hình tại thư mục làm việc hoặc thư mục user (VD: `~/.virtualcharacter/config.toml`). Dùng cho các cài đặt cố định của máy đó.
3. **Environment Variables**: Biến môi trường (VD: `VC_GEMINI_MODEL="gemini-flash"`). Phù hợp cho Docker hoặc CI/CD.
4. **Command Line Arguments**: Tham số dòng lệnh (VD: `vc-cli --model gemini-ultra`). Chỉ tồn tại trong `apps/vc-cli`. Có ưu tiên cao nhất.

---

## 5. Quản lý Secrets (Secrets Management)

Secrets (đặc biệt là API Keys) cần được đối xử đặc biệt:

- **Nguyên tắc In-Memory**: Secret chỉ tồn tại trên RAM trong lúc runtime hoạt động.
- **Không Logging**: Tuyệt đối không in (print/log) nội dung của secret. Nếu cần log, chỉ log độ dài hoặc ẩn dạng `***`. Trong Rust, có thể bọc bằng kiểu `Secret<String>` (ví dụ từ crate `secrecy`) để ngăn chặn việc `Debug` print vô tình làm lộ key.
- **Không Persistence**: Không lưu API Key vào SQLite database của `vc-storage`. Nếu User cần cấu hình API Key qua giao diện ở các Phase sau, nó phải được lưu vào hệ thống an toàn riêng (OS Keychain/Vault) hoặc file cấu hình cục bộ được bảo vệ quyền đọc.

---

## 6. Khởi tạo và Fail-Fast (Validation & Lifecycle)

- **Fail-Fast**: Cấu hình phải được kiểm tra (Validate) **ngay lúc khởi động (startup)**.
- Nếu thiếu `GEMINI_API_KEY` (khi dùng GeminiProvider) hoặc file SQLite không có quyền ghi, app phải báo lỗi rõ ràng và `panic!`/`exit` ngay lập tức.
- Không được để app chạy rồi mới lỗi (crash) giữa chừng lúc đang xử lý Interaction.

### 6.2 Lifecycle (Vòng đời)
- Trong Phase 1, cấu hình là **Bất biến (Immutable)** trong suốt vòng đời của process.
- Việc thay đổi file `config.toml` sẽ yêu cầu khởi động lại ứng dụng. Không hỗ trợ Dynamic Reloading trong Phase 1 để giữ kiến trúc đơn giản.

---

## 7. Các Configuration Structs tham chiếu

Ví dụ thiết kế (không yêu cầu code chính xác, chỉ là định hướng):

```rust
// Khởi tạo bởi vc-runtime
pub struct AppConfig {
    pub llm: LlmConfig,
    pub storage: StorageConfig,
    pub runtime: RuntimeConfig,
}

pub struct LlmConfig {
    pub provider: String, // "gemini" hoặc "mock"
    pub model_name: String,
    pub api_key: Option<SecretString>, // Chỉ có nếu provider không phải mock
}

pub struct StorageConfig {
    pub db_path: PathBuf, // VD: ~/.virtualcharacter/data.db
}

pub struct RuntimeConfig {
    pub max_context_budget: usize, // Dùng cho ContextBuilder
    pub interaction_timeout_ms: u64,
}
```

---

## 8. Nguyên tắc Thiết kế Tóm tắt

1. **Domain Isolation**: Core domain không biết gì về môi trường (OS/Env/Files).
2. **Explicit Injection**: API keys và file paths phải được truyền rõ ràng qua hàm khởi tạo (Constructors), không gọi global state hay giấu trong các hàm sâu.
3. **Fail Early**: Validate toàn bộ config ngay lúc khởi động.
4. **Data ≠ Config**: Không dùng Config để lưu trí nhớ, không dùng Database để lưu API Key.
