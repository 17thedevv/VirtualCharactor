# Hướng Dẫn Phát Triển — VirtualCharacter Development Guide

Tài liệu này cung cấp chỉ dẫn thiết lập môi trường lập trình, quy trình làm việc song song, hướng dẫn cài đặt các thành phần Local-First (Ollama, Whisper, Piper) và kiểm soát tài nguyên trên máy tính cá nhân (RTX 3050 Laptop 4GB VRAM).

---

## 1. Yêu Cầu Môi Trường (System Requirements)

- **Hệ điều hành**: Windows 10/11 (64-bit).
- **Rust Toolchain**: Rust 1.75+ (Cài qua `rustup`: `rustup default stable-x86_64-pc-windows-msvc`).
- **Ollama**: Tải và cài đặt từ [ollama.com](https://ollama.com/).
- **Bộ nhớ RAM**: Tối thiểu 16GB (Khuyến nghị 32GB).
- **Card đồ họa**: NVIDIA RTX 3050 Laptop 4GB VRAM (cần cài đặt driver NVIDIA mới nhất hỗ trợ CUDA).

---

## 2. Thiết Lập Môi Trường Cục Bộ (Local Setup)

### 2.1 Cài đặt & Chuẩn bị Model Ollama
Khởi động service Ollama và tải model chat mặc định tối ưu cho 4GB VRAM:
```powershell
# Tải model chat chính (chiếm ~2.2GB VRAM, phản hồi cực nhanh, hỗ trợ tốt tiếng Việt)
ollama pull qwen2.5:3b-instruct-q4_K_M

# (Tùy chọn) Tải model nhẹ dự phòng
ollama pull llama3.2:3b

# Kiểm tra danh sách model đã sẵn sàng
ollama list
```

### 2.2 Cấu trúc Thư mục Dữ liệu Cục bộ
Dữ liệu lưu trữ SQLite và bộ nhớ nhân vật được đặt tại thư mục `data/` ở thư mục gốc:
```powershell
# Thư mục này tự động được tạo và cấu hình trong .gitignore
mkdir -p data
```

---

## 3. Lệnh Thao Tác Cơ Bản Trong Dự Án (CLI & Cargo Commands)

Mọi lệnh đều chạy từ thư mục gốc của workspace (`d:\VirtualCharactor`):

### 3.1 Kiểm thử Toàn bộ Workspace
Đảm bảo toàn bộ 114+ bài kiểm thử tự động pass trước khi commit bất kỳ thay đổi nào:
```powershell
cargo test --workspace
```

### 3.2 Khởi chạy Ứng dụng Console CLI
Giao diện dòng lệnh trực tiếp cho phép trò chuyện và quan sát phản ứng của nhân vật:
```powershell
# Chạy với cấu hình mặc định (sử dụng SqliteStorage và Ollama/Mock)
cargo run -p vc-cli
```

### 3.3 Khởi chạy Máy chủ API Backend & WebSocket
Khởi động máy chủ `vc-server` trên cổng `3000`:
```powershell
cargo run -p vc-server
```
Sau khi server chạy:
- REST API: `http://localhost:3000/api`
- WebSocket Endpoint: `ws://localhost:3000/ws`
- Truy cập trình duyệt xem Web HUD tại: `http://localhost:3000`

### 3.4 Kiểm tra Định dạng & Quy chuẩn Mã nguồn
```powershell
# Kiểm tra định dạng code
cargo fmt --check

# Chạy linter clippy bắt các lỗi logic tiềm ẩn
cargo clippy --workspace --all-targets --all-features
```

---

## 4. Giám Sát Tài Nguyên Phần Cứng (RTX 3050 4GB Guardrails)

Trong quá trình phát triển các tính năng đa phương thức (Ollama, Avatar, Vision), lập trình viên phải thường xuyên giám sát mức tiêu thụ VRAM:

```powershell
# Mở cửa sổ PowerShell riêng để giám sát VRAM mỗi 1 giây
nvidia-smi -l 1
```

### Tiêu chuẩn an toàn VRAM:
- **Ngưỡng bình thường**: 2.2 GB - 2.8 GB VRAM.
- **Ngưỡng cảnh báo**: > 3.4 GB VRAM (Nguy cơ OOM làm crash ứng dụng).
- **Ngưỡng khẩn cấp**: > 3.7 GB VRAM -> Cần kích hoạt offload CPU hoặc chuyển sang Gemini Cloud Fallback.

---

## 5. Quy Trình Phối Hợp 2 Nhà Phát Triển (Two-Developer Model)

Hệ thống phân chia ranh giới sở hữu nhằm ngăn chặn xung đột mã nguồn:

### 5.1 Phân chia Ranh giới Sở hữu (Ownership Map)

| Khu vực / Module | Dev A (Character Intelligence) | Dev B (Runtime & Boundaries) |
|---|---|---|
| `crates/vc-core/src/personality/` | **Chính** | ❌ (Không sửa) |
| `crates/vc-core/src/state/` | **Chính** | ❌ (Không sửa) |
| `crates/vc-core/src/relationship/` | **Chính** | ❌ (Không sửa) |
| `crates/vc-core/src/memory/` | **Chính** | ❌ (Không sửa) |
| `crates/vc-core/src/decision/` | **Chính** | ❌ (Không sửa) |
| `crates/vc-core/src/context/` | Hợp đồng (Contract) | **Chính (Implementation)** |
| `crates/vc-runtime/` | Review | **Chính** |
| `crates/vc-llm/` | Review | **Chính** (Ollama, Gemini, Registry) |
| `crates/vc-storage/` | Review | **Chính** (SQLite, Vector) |
| `apps/vc-cli/`, `apps/vc-server/` | Phối hợp | **Chính** |
| `docs/contracts.md`, `docs/plan.md` | Đồng thuận cả 2 | Đồng thuận cả 2 |

### 5.2 Quy trình Nhánh Git (Branch Workflow)
1. Tạo nhánh tính năng riêng biệt:
   - Dev A: `feature/memory-consolidation`, `feature/attention-salience`
   - Dev B: `feature/ollama-provider`, `feature/vector-search`
2. Giữ commit nhỏ gọn, có ý nghĩa rõ ràng.
3. Trước khi mở Pull Request hoặc merge vào `main`:
   - Phải chạy `cargo test --workspace` thành công.
   - Đảm bảo không vi phạm các ranh giới hợp đồng trong `docs/contracts.md`.
