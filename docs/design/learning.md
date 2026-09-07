# VirtualCharacter Learning & Adaptation System

## 1. Tổng quan (Overview)

Tài liệu này định nghĩa hệ thống Học tập, Thích nghi và Cá nhân hóa của VirtualCharacter.

Mục tiêu cốt lõi của tài liệu này là tách biệt hoàn toàn **Runtime Adaptation** (Thích nghi thời gian thực thông qua ghi nhớ) và **Model Training** (Huấn luyện mô hình Machine Learning offline).

Hệ thống VirtualCharacter phải hoạt động hoàn hảo và có tính xác định (deterministic) ngay cả khi không có bất kỳ mô hình Machine Learning (ML) tùy chỉnh nào. Các mô hình ML (Learned Components) là một tính năng bổ trợ (Opt-in), không bao giờ được phép làm hỏng (corrupt) tính cách cốt lõi của nhân vật.

---

## 2. Phân biệt Khái niệm (Fundamental Distinctions)

Tuyệt đối không nhầm lẫn các hành vi sau:

- **Runtime Adaptation (Thích nghi Runtime)**: Nhân vật thay đổi cách cư xử ngay lập tức dựa trên việc lưu/đọc State, Memory, và Relationship. (VD: User nói "Tôi ghét câu trả lời dài". Hệ thống lưu `Memory` này lại, lần sau Context Builder sẽ báo cho LLM biết). Hệ thống **không** tính toán lại bất kỳ "weights" toán học nào.
- **Model Training (Huấn luyện Model)**: Sử dụng lịch sử dữ liệu để huấn luyện một mô hình (VD: Random Forest, Neural Network) để dự đoán xem User thích gì. Việc này **luôn luôn** diễn ra offline (ngoại tuyến).
- **Model Evaluation**: Đánh giá xem Model v2 có tốt hơn Rule-based v1 không.
- **Model Deployment**: Đưa Model đã huấn luyện vào chạy trong Runtime.

### 2.1. Phân loại Khả năng Thích nghi
- **State Adaptation**: Rất nhanh, tạm thời (Ví dụ: Đang vui chuyển sang buồn).
- **User Preference Adaptation**: Dài hạn, áp dụng cho một user cụ thể (Ví dụ: Thích xưng hô "anh-em").
- **Personality Adaptation**: Rất chậm, bị ràng buộc nghiêm ngặt (Ví dụ: Trở nên cởi mở hơn sau hàng ngàn tương tác).
- **Learned Model Adaptation**: Ngoại tuyến, được phiên bản hóa (versioned).

---

## 3. Triết lý Học tập (Learning Philosophy)

1. **Không cập nhật Model Online**: Hệ thống **nghiêm cấm** việc tự động cập nhật weights (online gradient update) sau mỗi tương tác trong môi trường Production. Việc này gây ra sự bất ổn định (instability), quên lãng thảm khốc (catastrophic forgetting), và trôi dạt tính cách (personality drift).
2. **Deterministic Baseline**: Luôn phải có một quy tắc xác định (Rule-based) làm nền tảng.
3. **Training Data != Runtime Data**: Dữ liệu thô từ runtime phải trải qua bộ lọc bảo mật/riêng tư trước khi trở thành Dữ liệu huấn luyện (Training Dataset).

Vòng đời lý tưởng của ML trong hệ thống:
```text
Interaction → Outcome → Learning Signal → [Offline Filtering & Redaction] → Dataset → Offline Training → Evaluation → Versioning → Deployment
```

---

## 4. Phân cấp Cá nhân hóa (Personalization Hierarchy)

Mô hình cá nhân hóa phải hoạt động theo một hệ thống phân cấp ưu tiên nghiêm ngặt để tránh việc "Sở thích của User A làm hỏng tính cách chung của Nhân vật".

1. **Core Personality** (Cao nhất): Bất biến. (VD: "Tôi là Hikari, một AI trung thực").
2. **Character Preferences**: Sở thích chung của nhân vật.
3. **Relationship State**: Mức độ thân thiết cụ thể với User A.
4. **User Preferences**: Sở thích riêng của User A (VD: "User A ghét emoji"). Dữ liệu này **chỉ** áp dụng cho User A.
5. **Current State & Context** (Thấp nhất): Cảm xúc nhất thời.

---

## 5. Bảo vệ Tính cách Cốt lõi (Core Personality Protection)

Các giá trị cốt lõi (Core values) và danh tính (Identity) **không được phép** bị ghi đè chỉ vì một hoặc vài tương tác.

Nếu hệ thống sau này hỗ trợ tự động thay đổi tính cách (Personality adaptation), nó phải tuân thủ:
- **Evidence Threshold (Ngưỡng bằng chứng)**: Được cấu hình bởi policy của subsystem (chưa hard-code thành kiến trúc).
- **Rate of Change (Tốc độ thay đổi)**: Cực chậm.
- **Rollback**: Có khả năng quay ngược lại phiên bản tính cách cũ.

---

## 6. Learned Components (Các Component có thể học)

Một "Learned Component" là một mô hình ML nhỏ, chạy cục bộ, dự đoán kết quả hoặc xếp hạng (ranking) danh sách thay vì viết code if/else thủ công.

**Ví dụ:**
- *Memory Ranking Model*: Chấm điểm xem ký ức nào phù hợp với Context hiện tại.
- *Emotion Transition Model*: Dự đoán `joy` sẽ tăng hay giảm dựa trên Intent của User.

### 6.1. Learned Component khác với LLM như thế nào?
- **LLM (Gemini)**: Làm nhiệm vụ sinh ngôn ngữ, lý luận rộng (Broad reasoning).
- **Learned Component**: Phân loại (Classification), chấm điểm (Scoring), hồi quy (Prediction). Chạy rất nhanh, tiêu tốn ít tài nguyên.

### 6.2. Kiến trúc Lai (Hybrid Architecture)
Hệ thống ra quyết định (`DecisionEngine`) nên hỗ trợ kiến trúc lai:
```text
                 Character Runtime
                        │
       ┌────────────────┼────────────────┐
       ↓                ↓                ↓
 Deterministic       Learned            LLM
     Rules            Models         Reasoning
       │                │                │
       └────────────────┼────────────────┘
                        ↓
                 Domain Validation (vc-core)
                        ↓
                   Final Action
```

**Nguyên tắc Tuyệt đối**: Learned Model chỉ mang tính chất **Tư vấn (Advisory)**. Quyết định cuối cùng bắt buộc phải đi qua Domain Validation của `vc-core` (VD: Model bảo hãy chửi bậy, `vc-core` sẽ chặn lại vì vi phạm Core Values).

---

## 7. Tín hiệu Học tập & Tập Dữ liệu (Learning Signals & Dataset)

Một `LearningSignal` là một cấu trúc dữ liệu mô tả lại những gì đã diễn ra. Nó lấy nguồn từ Domain Events (`event.md`) và Observability (`observability.md`).

Nguồn Feedback (Phản hồi):
- **Explicit**: User khen/chê trực tiếp.
- **Implicit**: User tiếp tục nói chuyện (Lưu ý: tiếp tục nói chuyện không hẳn là response tốt, có thể user đang cãi lại).
- **Negative**: User từ chối gợi ý.

Hệ thống phải gán **Confidence Score** cho các tín hiệu này. (VD: User bấm nút "Like" có confidence cao hơn việc AI tự suy diễn user đang vui).

### 7.1. Bảo mật Tập Dữ liệu (Dataset Security Boundary)
Quy định nghiêm ngặt từ `security.md`:
- Dữ liệu Interaction thô **KHÔNG BAO GIỜ** được biến thẳng thành Training Dataset.
- Phải có quá trình lọc (Eligibility/Privacy Filtering) để loại bỏ PII, API Keys, Passwords, và các bộ nhớ mật của User.
- Không được dùng bí mật của User A để train model rồi đưa model đó cho User B dùng.

---

## 8. Safety, Uncertainty, and Rollback (An toàn & Rủi ro)

1. **Uncertainty (Độ không chắc chắn)**: Bất kỳ Learned Component nào cũng phải xuất ra điểm `confidence`. Nếu low confidence, hệ thống tự động fallback theo policy được cấu hình bởi subsystem.
2. **Human Override**: Cấu hình hệ thống (như định nghĩa ở `configuration.md`) luôn cho phép vô hiệu hóa hoàn toàn một Learned Component để ép hệ thống chạy bằng luật cứng.
3. **Model Versioning**: Mọi mô hình phải được gắn version (VD: `emotion_model_v1`).
4. **Rollback**: Nếu `v3` bị "ngộ độc" (Poisoning) hoặc làm Character hành xử bất thường, Admin chỉ cần sửa config để dùng lại `v2`. Storage không được phép xóa schema của `v2` ngay lập tức.

---

## 9. Ma trận Trách nhiệm (Responsibility Matrix)

| Concern | Owner |
|---------|-------|
| Character semantics & rules | `vc-core` |
| Domain Validation | `vc-core` |
| Memory/State semantics | `Memory` & `State` |
| Runtime adaptation (Memory ghi nhớ) | `Runtime` & Domain subsystems |
| Thu thập Learning signals | Learning layer (Future) |
| Tạo Dataset (Lọc dữ liệu) | Learning pipeline (Offline) |
| Huấn luyện mô hình (Training) | External/Offline environment |
| Khởi chạy mô hình (Inference) | Learned component adapter (Runtime) |
| Quyết định Persistence | `vc-storage` |
| Giao tiếp LLM | `vc-llm` |

---

## 10. Tóm tắt Nguyên tắc Bất di bất dịch (Design Principles)

1. **Runtime adaptation không phải là Model training**.
2. **Learned models là Optional (Tùy chọn)**. Hệ thống luôn phải chạy được với Deterministic baseline.
3. **Core personality được bảo vệ** khỏi việc trôi dạt vô kiểm soát (Catastrophic drift).
4. **Learning data phải được lọc bảo mật**. Không học bí mật của user.
5. **Model chỉ mang tính tư vấn**. Domain Validation có quyền phủ quyết (Veto).
6. **Mọi mô hình đều có version và có thể rollback**.
7. **Huấn luyện mô hình luôn là Offline/Batched**, không bao giờ là Per-Interaction Online Gradient Update.
