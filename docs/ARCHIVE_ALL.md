

=========================================
# FILE: architecture.md
=========================================


# VirtualCharacter Architecture

## 1. Layer Responsibilities

- **vc-core**: The domain layer. Contains pure business logic, domain entities (Character, Memory, Personality, etc.), and interfaces for external dependencies. It is completely infrastructure-agnostic.
- **vc-runtime**: The orchestration layer. Contains use cases, workflows, and the runtime loop. It orchestrates the domain entities and the infrastructure adapters.
- **vc-llm**: The LLM infrastructure layer. Contains the implementations of the `LlmProvider` trait defined in `vc-core`. E.g., a Gemini integration or a mock provider.
- **vc-storage**: The persistence layer. Contains the implementations of repository traits. Handles database connections, ORM logic, and file storage if necessary.
- **vc-cli**: The development/testing application. It provides a local demonstration of the runtime without requiring an API or web framework.

## 2. Dependency Direction

```
vc-cli -> (vc-runtime, vc-core, vc-llm, vc-storage)
vc-runtime -> (vc-core, vc-llm, vc-storage)
vc-llm -> (vc-core)
vc-storage -> (vc-core)
```

The domain core (`vc-core`) depends on nothing except standard libraries or utility crates like `uuid` and `serde`.

## 3. Domain/Infrastructure Boundary

Infrastructure details (like SQL databases, LLM APIs) are hidden behind traits.

## 4. LLM Abstraction

The `LlmProvider` trait abstracts away the specific provider (e.g., Gemini). The domain only knows about `LlmRequest` and `LlmResponse`.

## 5. Storage Boundary

Storage and retrieval mechanisms are hidden behind the `CharacterRepository` and `MemoryRepository` traits.

## 6. CLI Role

The `vc-cli` application is the first integration point to prove that the architecture is composable. It avoids the overhead of web frameworks in Phase 0.

`Context` is treated as a dynamically generated payload passed to the LLM, decoupling conversation history from what the LLM actually "sees" at any given moment.

## 7. Decision Boundary

The `DecisionEngine` takes `Context`, `Personality`, and `CharacterState` and produces a discrete `Decision`, completely isolating the "what to do" logic from the "how to do it" side effects.

## 8. Extension Points

- **LLMs**: Implement a new `LlmProvider`.
- **Storage**: Implement a new `CharacterRepository` or `MemoryRepository`.
- **API**: Add new routes or WebSocket events in `vc-api` without touching the domain logic.


=========================================
# FILE: contracts.md
=========================================


# VirtualCharacter Public Domain Contracts

This document is the internal API agreement between two developers working in parallel.

Changes to any item marked **stable** require explicit approval from both developers.

---

## Stability Matrix

| Contract | Owner | Stability |
|---|---|---|
| Personality | Dev A | Stable |
| CharacterState | Dev A | Stable |
| EmotionState | Dev A | Stable |
| Relationship | Dev A | Stable |
| Memory | Dev A | Stable |
| DecisionEngine | Dev A | Stable |
| Context | Dev B | Stable |
| LlmProvider | Dev B | Stable |
| Storage repositories | Dev B | Stable |
| Runtime orchestration | Dev B | Evolving |
| Gemini adapter | Dev B | Provider-specific |
| Learning components | Future | Evolving |

---

## Ownership Map

### Developer A — Character Intelligence

Owns `vc-core` modules:
- `personality/`
- `memory/`
- `state/` (emotion, cognitive, behavior, goals, session)
- `relationship/`
- `decision/` (trait definition only, not mock implementation)

### Developer B — Runtime & Boundary

Owns:
- `vc-runtime/` (including `MockDecisionEngine`)
- `vc-llm/` (provider abstraction, Gemini adapter, mock)
- `vc-storage/` (repository traits, SQLite adapter)
- `vc-core::context/`

---

## Visibility Rules

| Item | Visible outside its crate? |
|------|---------------------------|
| `DecisionEngine` trait | Yes — public contract |
| `MockDecisionEngine` | Yes — lives in `vc-runtime`, not `vc-core` |
| `LlmProvider` trait | Yes — public contract |
| `LlmRequest` / `LlmResponse` | Yes — provider-neutral types |
| Gemini request/response format | **No** — private to `vc-llm::gemini` |
| SQLite schema | **No** — private to `vc-storage::sqlite` |
| Memory internals | **No** — implementation detail of Dev A |

---

## Change Policy

- **Stable** items: signature changes require PR review from both developers.
- **Evolving** items: the owner may change internals freely but must not break existing consumers.
- **Additive changes** (new fields, new enum variants): generally safe, notify the other developer.
- **Breaking changes** (removed fields, changed signatures): require explicit agreement and ADR.


=========================================
# FILE: configuration.md
=========================================


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


=========================================
# FILE: context.md
=========================================


# Hệ thống quản lý Context của VirtualCharacter

## 1. Tổng quan

Hệ thống quản lý Context chịu trách nhiệm xác định, lựa chọn, tổ chức và đưa các thông tin cần thiết vào context của Large Language Model (LLM) nhằm tạo ra câu trả lời hoặc hành động phù hợp với **tính cách, đặc tính, trạng thái và lịch sử trải nghiệm của VirtualCharacter**.

Context không đơn thuần là toàn bộ lịch sử hội thoại. Nó là một **biểu diễn có chọn lọc của trạng thái hiện tại của nhân vật**, được xây dựng từ nhiều nguồn thông tin khác nhau.

```text
User Input
    ↓
Context Management
    ├── Character Identity
    ├── Personality
    ├── Current State
    ├── Relationship State
    ├── Relevant Memories
    ├── Conversation History
    ├── Current Situation
    └── Constraints / Instructions
    ↓
LLM Context
    ↓
Response / Action
```

Mục tiêu của Context Management là đảm bảo LLM nhận được **đúng thông tin cần thiết vào đúng thời điểm**, thay vì đưa toàn bộ dữ liệu mà VirtualCharacter đang sở hữu vào prompt.

---

# 2. Vai trò của Context

Context đóng vai trò là cầu nối giữa **Character Runtime** và LLM.

LLM bản thân không trực tiếp sở hữu toàn bộ trạng thái của VirtualCharacter. Thay vào đó, runtime cung cấp cho LLM một context mô tả trạng thái cần thiết để LLM đưa ra quyết định.

```text
Character Runtime
        ↓
Context Builder
        ↓
LLM
        ↓
Response
```

Có thể xem Context như một "snapshot" của VirtualCharacter tại một thời điểm cụ thể.

Ví dụ:

```text
Character:
    Name = Hikari
    Personality = playful, caring, slightly-tsundere

State:
    happiness = 0.82
    affection = 0.71

Relationship:
    trust = 0.87

Relevant Memory:
    User vừa hoàn thành một milestone quan trọng.

Current Conversation:
    "Cuối cùng tao cũng sửa xong parser."
```

LLM dựa trên snapshot này để sinh ra phản hồi.

---

# 3. Các thành phần của Context

Context được hình thành từ nhiều nguồn dữ liệu.

## 3.1. Character Identity

Mô tả danh tính ổn định của VirtualCharacter.

Ví dụ:

```text
Name:
Hikari

Role:
Anime companion

Background:
...

Core traits:
...
```

Identity thường có tính ổn định cao và được ưu tiên cao trong context.

---

## 3.2. Personality

Mô tả các đặc điểm tính cách của nhân vật.

Ví dụ:

```text
Traits:
- cheerful
- playful
- caring
- slightly tsundere

Speech style:
- casual
- expressive
- playful
```

Personality không nhất thiết phải được truyền dưới dạng một đoạn văn dài. Hệ thống có thể sử dụng một biểu diễn có cấu trúc.

---

## 3.3. Character State

Mô tả trạng thái hiện tại của nhân vật.

Ví dụ:

```text
happiness = 0.82
sadness = 0.10
anger = 0.03
affection = 0.71
energy = 0.64
```

State có thể thay đổi sau mỗi interaction.

Do đó, cùng một personality nhưng ở các trạng thái khác nhau, character có thể đưa ra phản ứng khác nhau.

---

## 3.4. Relationship State

Mô tả trạng thái quan hệ giữa character và user hiện tại.

Ví dụ:

```text
familiarity = 0.91
trust = 0.82
affection = 0.74
intimacy = 0.68
conflict = 0.04
```

Relationship State được xác định theo từng user và không phải trạng thái dùng chung cho toàn bộ VirtualCharacter.

---

## 3.5. Relevant Memory

Là những memory được hệ thống lựa chọn dựa trên tình huống hiện tại.

Không phải toàn bộ Long-Term Memory được đưa vào context.

```text
Long-Term Memory
      ↓
Memory Retrieval
      ↓
Relevant Memories
      ↓
Context
```

Ví dụ:

```text
Memory 1:
User đang phát triển Mellis.

Memory 2:
User vừa hoàn thành parser.

Memory 3:
User thích anime.

Memory 4:
User từng gặp bug tương tự một tháng trước.
```

Nếu cuộc hội thoại hiện tại liên quan tới parser, Memory 2 và Memory 4 có thể được ưu tiên.

---

## 3.6. Conversation History

Chứa các lượt hội thoại gần đây để duy trì tính liên tục của cuộc trò chuyện.

Tuy nhiên, conversation history có thể trở nên rất lớn.

Do đó hệ thống cần hỗ trợ:

```text
Recent Messages
+
Conversation Summary
+
Relevant Historical Messages
```

thay vì luôn gửi toàn bộ conversation.

---

## 3.7. Current Situation

Mô tả tình huống hiện tại mà character đang xử lý.

Ví dụ:

```text
User is celebrating an achievement.

Current topic:
Mellis parser

User emotional state:
Excited

Character emotional state:
Happy
```

Current Situation giúp LLM phân biệt giữa thông tin lịch sử và sự kiện đang diễn ra.

---

## 3.8. Constraints

Bao gồm những quy tắc mà LLM phải tuân thủ.

Ví dụ:

```text
- Maintain character personality.
- Do not contradict established facts.
- Respect current relationship state.
- Do not expose internal system state.
```

Constraints có mức ưu tiên cao và giúp đảm bảo response không phá vỡ tính nhất quán của character.

---

# 4. Context Assembly

Context Builder chịu trách nhiệm kết hợp các nguồn dữ liệu thành một context cuối cùng.

```text
Character
    │
    ├── Identity
    ├── Personality
    ├── State
    └── Relationship
            │
Memory ─────┤
            │
Conversation┤
            │
Situation ──┤
            │
Constraints ┘
      ↓
Context Builder
      ↓
Optimized Context
      ↓
LLM
```

Context Builder không chỉ nối chuỗi các thông tin với nhau.

Nó phải:

* chọn thông tin cần thiết;
* loại bỏ thông tin dư thừa;
* xử lý xung đột;
* sắp xếp theo mức độ ưu tiên;
* kiểm soát kích thước context;
* đảm bảo các thông tin quan trọng không bị mất.

---

# 5. Context Budget

Context của LLM có giới hạn.

Do đó Context Manager phải hoạt động dưới một **Context Budget**.

Ví dụ:

```text
Context Budget = 16,000 tokens

System / Identity       1,000
Personality               500
State                     300
Relationship              300
Recent Conversation     4,000
Retrieved Memory        5,000
Current Situation       1,000
Reserved Output         3,900
```

Tổng lượng token phải nằm trong budget được cấu hình.

Context Budget có thể thay đổi tùy model.

---

# 6. Context Prioritization

Khi context vượt quá giới hạn, hệ thống không nên cắt dữ liệu một cách tuần tự.

Thay vào đó, mỗi context item cần có priority.

Ví dụ:

```text
Core Identity             = Critical
Active Constraints        = Critical
Current State             = High
Relationship State         = High
Current Conversation      = High
Relevant Memory           = Medium / High
Old Conversation           = Low
Low-value Facts            = Low
```

Hệ thống có thể loại bỏ các item có priority thấp trước.

---

# 7. Context Compression

Các thông tin dài có thể được nén thành biểu diễn ngắn hơn.

Ví dụ:

```text
Conversation:

User đã nói về Mellis trong 20 lượt hội thoại,
nêu parser bug, resolver bug và cuối cùng đã sửa xong.

        ↓

Summary:

User vừa hoàn thành việc sửa parser của Mellis
sau một chuỗi debugging kéo dài.
```

Compression giúp giảm token nhưng vẫn giữ thông tin có giá trị.

---

# 8. Context Retrieval

Context Manager có thể yêu cầu Memory System cung cấp những memory liên quan tới tình huống hiện tại.

```text
Current Situation
        ↓
Query Construction
        ↓
Memory Retrieval
        ↓
Ranking
        ↓
Selected Memories
        ↓
Context
```

Việc retrieval có thể sử dụng:

* semantic similarity;
* recency;
* importance;
* emotional relevance;
* relationship relevance;
* frequency;
* current topic.

Không nhất thiết phải dùng một thuật toán cố định. Hệ thống có thể sử dụng model học được để đánh giá mức độ hữu ích của memory đối với context hiện tại.

---

# 9. Dynamic Context

Context phải có khả năng thay đổi trong quá trình xử lý.

Ví dụ:

```text
User:
"Tao vừa thất bại."

Initial Context:
emotion = neutral

       ↓ analysis

Character State:
sadness + 0.25
confidence - 0.05

       ↓

Updated Context

       ↓

LLM response
```

Do đó context không nhất thiết phải được tạo đúng một lần.

Một interaction có thể trải qua nhiều bước:

```text
Input
 ↓
Analyze
 ↓
Update State
 ↓
Retrieve Memory
 ↓
Rebuild Context
 ↓
Generate Response
```

---

# 10. Context và Tool / Action

Context không chỉ phục vụ việc sinh text.

VirtualCharacter có thể sử dụng LLM để quyết định hành động.

Ví dụ:

```text
Context
    ↓
LLM Decision
    ↓
Action:
    send_message
    search_memory
    call_tool
    update_relationship
    wait
```

Do đó Context Management phải cung cấp đủ information để LLM lựa chọn action phù hợp.

---

# 11. Context Consistency

Context Manager phải đảm bảo các thành phần trong context không mâu thuẫn với nhau.

Ví dụ:

```text
Memory:
User thích Python.

Recent Memory:
User hiện tại không còn dùng Python.
```

Context Builder có thể ưu tiên thông tin mới hơn:

```text
Current Preference:
User không còn thích Python.
```

Thay vì truyền cả hai thông tin một cách không có thứ tự, Context Manager nên biểu diễn sự thay đổi rõ ràng.

---

# 12. Context Isolation

Mỗi VirtualCharacter và mỗi relationship phải có context độc lập.

```text
Character A
 ├── User A context
 └── User B context

Character B
 └── User A context
```

Không được để memory hoặc state của một character hoặc một relationship vô tình xuất hiện trong context của entity khác.

Context isolation là yêu cầu quan trọng để đảm bảo mỗi VirtualCharacter thực sự là một thực thể độc lập.

---

# 13. Context Template

Context cuối cùng có thể được biểu diễn theo cấu trúc:

```text
[IDENTITY]

[PERSONALITY]

[CURRENT STATE]

[RELATIONSHIP]

[RELEVANT MEMORIES]

[RECENT CONVERSATION]

[CURRENT SITUATION]

[CONSTRAINTS]

[TASK]
```

Thứ tự cụ thể có thể thay đổi tùy model và chiến lược prompting.

Điểm quan trọng là Context Manager phải tạo ra context **có cấu trúc, có ưu tiên và có mục đích**.

---

# 14. Context Generation Pipeline

Pipeline tổng thể:

```text
                    User Input
                        │
                        ↓
                  Situation Analysis
                        │
          ┌─────────────┴─────────────┐
          ↓                           ↓
    Character State              Memory Query
          │                           │
          │                           ↓
          │                    Memory Retrieval
          │                           │
          └─────────────┬─────────────┘
                        ↓
                  Context Builder
                        ↓
                 Context Prioritizer
                        ↓
                 Context Compressor
                        ↓
                  Context Validator
                        ↓
                       LLM
                        ↓
                     Output
```

---

# 15. Context Management và Personality

Context Management không tạo ra personality.

Personality định nghĩa:

> Character có xu hướng trở thành ai.

Context Management xác định:

> Character cần nhớ và biết điều gì ngay lúc này để hành động theo personality đó.

Memory định nghĩa:

> Character đã trải qua những gì.

State định nghĩa:

> Character đang ở trạng thái nào.

LLM thực hiện:

> Suy luận và biến tất cả những thông tin đó thành ngôn ngữ hoặc hành động.

Có thể biểu diễn:

```text
Personality
     +
Memory
     +
State
     +
Relationship
     +
Current Context
     ↓
Character Decision
     ↓
LLM
     ↓
Response / Action
```

---

# 16. Nguyên tắc thiết kế

Context Management phải tuân thủ các nguyên tắc:

1. **Relevant over Complete**
   Context ưu tiên thông tin liên quan thay vì cố đưa toàn bộ dữ liệu.

2. **State-aware**
   Context phải phản ánh trạng thái hiện tại của character.

3. **Memory-aware**
   Context phải có khả năng truy hồi memory thích hợp theo tình huống.

4. **Budget-aware**
   Context phải hoạt động trong giới hạn token của model.

5. **Priority-aware**
   Thông tin quan trọng phải được ưu tiên hơn thông tin ít quan trọng.

6. **Dynamic**
   Context có thể được cập nhật khi trạng thái hoặc tình huống thay đổi.

7. **Isolated**
   Context của các character và relationship phải được tách biệt.

8. **Model-independent**
   Context Management không nên phụ thuộc quá sâu vào một LLM cụ thể.

---

# 17. Mục tiêu cuối cùng

Mục tiêu của Context Management không phải tạo ra một prompt càng dài càng tốt.

Mục tiêu là:

> **Cung cấp cho LLM một biểu diễn tối thiểu nhưng đầy đủ về những gì VirtualCharacter cần biết tại thời điểm hiện tại để đưa ra quyết định phù hợp với personality, memory, state và relationship của chính nó.**

Do đó:

```text
Too little Context
        ↓
Character forgets / acts inconsistently

Too much Context
        ↓
Token waste / noise / conflicting information

Optimal Context
        ↓
Relevant information
+
Correct state
+
Correct memories
+
Correct constraints
        ↓
Consistent Character Behavior
```


=========================================
# FILE: decision.md
=========================================


# VirtualCharacter Decision System

## 1. Tổng quan

Decision System là thành phần chịu trách nhiệm xác định **hành động, phản ứng hoặc mục tiêu tức thời** của VirtualCharacter dựa trên Personality, Memory, State, Relationship và Current Context.

Decision System trả lời:

> **"Character nên làm gì tiếp theo?"**

Trong khi:

* Personality trả lời: **Character là ai?**
* Memory trả lời: **Character đã trải qua gì?**
* State trả lời: **Character đang ở trạng thái nào?**
* Context trả lời: **LLM cần biết gì tại thời điểm hiện tại?**
* Decision trả lời: **Character nên hành động như thế nào?**
* LLM trả lời: **Hành động đó được biểu đạt hoặc thực thi như thế nào?**

---

# 2. Vai trò

Decision System là lớp trung gian:

```text id="3f86s4"
Personality
     +
Memory
     +
State
     +
Relationship
     +
Current Context
     ↓
Decision System
     ↓
Decision
     ↓
LLM / Tool / Runtime
     ↓
Action
```

Decision System không nhất thiết trực tiếp sinh ra câu trả lời.

Nó có thể tạo ra một **Action Plan** hoặc **Behavior Policy** để các subsystem khác thực hiện.

---

# 3. Decision Object

Một decision nên có cấu trúc rõ ràng.

Ví dụ:

```json id="v3m3hs"
{
  "action": "respond",
  "behavior": "comfort",
  "emotion_expression": "gentle",
  "tone": "warm",
  "verbosity": 0.42,
  "initiative": 0.35,
  "memory_retrieval": [
    "memory_182",
    "memory_391"
  ]
}
```

Decision không phải câu trả lời cuối cùng.

Nó mô tả **ý định hành vi**.

---

# 4. Action Types

Decision System có thể chọn nhiều loại action.

```text id="yflx46"
Action
├── Respond
├── Ask
├── Comfort
├── Encourage
├── Explain
├── Joke
├── Tease
├── Apologize
├── Refuse
├── Remember
├── Forget
├── Retrieve Memory
├── Update Relationship
├── Call Tool
├── Wait
└── Composite Action
```

Một action có thể chứa nhiều sub-action.

Ví dụ:

```json id="2vqx7o"
{
  "action": "composite",
  "steps": [
    "acknowledge_user",
    "retrieve_recent_failure",
    "encourage",
    "ask_followup"
  ]
}
```

---

# 5. Decision Pipeline

Pipeline cơ bản:

```text id="axm6lt"
Input
  ↓
Situation Understanding
  ↓
Candidate Generation
  ↓
Candidate Evaluation
  ↓
Policy Selection
  ↓
Decision
  ↓
Execution
  ↓
Outcome
  ↓
State / Memory Update
```

---

# 6. Situation Understanding

Decision System trước tiên cần xác định tình huống hiện tại.

Thông tin đầu vào có thể gồm:

```text id="lvsj01"
User Intent
User Emotion
Current Topic
Character Emotion
Character Goals
Relationship
Relevant Memory
Environmental Context
```

Ví dụ:

```text id="0zrt7s"
User:
"Tao fail interview rồi."

Situation:

user_emotion = disappointed
character_emotion = concerned
relationship.trust = high
current_goal = support_user
```

---

# 7. Candidate Generation

Không nên ngay lập tức chọn một action.

Hệ thống có thể tạo ra nhiều candidate:

```text id="1cy6cg"
Candidate A:
comfort

Candidate B:
ask what happened

Candidate C:
make a joke

Candidate D:
give practical advice

Candidate E:
stay silent
```

Decision System sau đó đánh giá các candidate này.

---

# 8. Candidate Evaluation

Mỗi candidate được chấm theo nhiều yếu tố:

```text id="5je79x"
Personality Alignment
State Compatibility
Relationship Compatibility
Goal Alignment
Context Relevance
Memory Relevance
User Preference
Expected Outcome
Risk / Constraint Violation
```

Có thể biểu diễn:

```text id="1ctkdu"
Score(action) =
    PersonalityFit
  + RelationshipFit
  + GoalFit
  + ContextFit
  + MemoryFit
  + UserPreference
  + ExpectedOutcome
  - ConstraintViolation
```

Các trọng số có thể khác nhau tùy character.

---

# 9. Personality Alignment

Decision phải phù hợp với personality.

Ví dụ:

```text id="b5v9tr"
Personality:
humor = high
empathy = high
aggressiveness = low
```

Trong tình huống user đang buồn:

```text id="z6rm90"
comfort → high fit
gentle joke → medium fit
aggressive joke → low fit
```

Personality là **bias**, không phải hard rule.

---

# 10. State Compatibility

Cùng một character nhưng State khác nhau có thể dẫn tới decision khác nhau.

Ví dụ:

```text id="h0nzz4"
Character State A:
energy = high

→ initiate conversation = likely
```

Trong khi:

```text id="lcgip0"
Character State B:
energy = low

→ short response / defer = likely
```

---

# 11. Relationship Compatibility

Decision phải phù hợp với mức độ quan hệ.

Ví dụ:

```text id="emgji1"
Relationship = stranger
→ personal teasing = low probability

Relationship = close_friend
→ personal teasing = high probability
```

Relationship không chỉ thay đổi nội dung câu trả lời mà có thể thay đổi **tập action được phép lựa chọn**.

---

# 12. Goal Alignment

Character có thể có nhiều goals.

Ví dụ:

```text id="e9a2ri"
Goals:

help_user = 0.92
maintain_conversation = 0.61
learn_about_user = 0.47
```

Decision có thể ưu tiên action giúp đạt goal có priority cao hơn.

---

# 13. User Preference

Decision System có thể sử dụng User Model.

Ví dụ:

```text id="77l2zi"
User preferences:

likes:
    concise responses
    playful tone

dislikes:
    excessive advice
    too many emojis
```

Khi hai candidate có chất lượng tương đương:

```text id="b0l5r8"
short playful response
        >
long explanatory response
```

---

# 14. Memory Relevance

Decision có thể yêu cầu memory retrieval.

Ví dụ:

```text id="w5awf8"
User:
"Tao lại gặp lỗi giống lần trước."

Candidate:

A:
answer immediately

B:
retrieve previous debugging session

→ B has higher Memory Relevance
```

Decision có thể chứa explicit retrieval instructions:

```json id="i9pbwk"
{
  "action": "respond",
  "memory_request": {
    "query": "previous parser debugging issue",
    "limit": 3
  }
}
```

---

# 15. Multi-Step Decision

Một decision có thể cần nhiều bước.

Ví dụ:

```text id="zprkgy"
1. Understand user emotion
2. Retrieve relevant memory
3. Update character state
4. Decide behavior
5. Generate response
```

Decision System phải hỗ trợ:

```text id="j7f4i4"
Sequential decisions
Conditional decisions
Parallel decisions
```

---

# 16. Decision Confidence

Decision không phải lúc nào cũng chắc chắn.

Mỗi decision có thể có confidence:

```json id="hkt4k3"
{
  "action": "comfort",
  "confidence": 0.81
}
```

Confidence thấp có thể trigger:

```text id="0ub0dm"
Ask clarification
Retrieve more memory
Request LLM reasoning
Generate multiple candidates
```

Ví dụ:

```text id="v0e1tv"
confidence < 0.35
        ↓
không tự quyết định
        ↓
tìm thêm context
```

---

# 17. Decision Uncertainty

Uncertainty là một phần bình thường của character.

Character có thể không biết:

```text id="9lpa1x"
User đang buồn
hay
User chỉ đang nói đùa.
```

Thay vì ép hệ thống chọn một nhãn, Decision System có thể duy trì:

```text id="d8k9za"
sad_probability = 0.55
joking_probability = 0.45
```

Sau đó chọn action ít rủi ro hơn:

```text id="n3kjwu"
gentle_response
```

---

# 18. Decision và LLM

LLM có thể được sử dụng ở nhiều vị trí.

### Deterministic Decision

```text id="gkz9nq"
Rules
→ Decision
```

Ưu điểm:

* nhanh;
* dễ kiểm soát;
* deterministic.

Nhược điểm:

* khó xử lý tình huống phức tạp;
* dễ tạo hệ thống if/else khổng lồ.

---

### Learned Decision

```text id="20tf5d"
Context
→ Small Decision Model
→ Decision
```

Model có thể học:

```text id="g8m0xq"
Context
→ emotion
→ candidate actions
→ ranking
```

Đây là nơi một model nhỏ có thể mang lại tính linh hoạt mà không cần để Gemini quyết định tất cả.

---

### LLM Decision

```text id="y0s4uh"
Context
→ Gemini
→ Decision
```

Phù hợp với tình huống phức tạp nhưng tốn latency và token hơn.

---

### Hybrid Decision

VirtualCharacter nên ưu tiên hybrid:

```text id="s8kwfl"
Hard Constraints
       ↓
Candidate Generation
       ↓
Small Learned Model
       ↓
Candidate Ranking
       ↓
LLM reasoning (khi cần)
       ↓
Final Decision
```

---

# 19. Decision Constraints

Một số decision không được phép vượt qua constraint.

Ví dụ:

```text id="v5a28p"
Constraint:
Do not violate character identity.

Constraint:
Do not expose internal memory.

Constraint:
Do not perform unsafe action.
```

Constraint phải được enforce bên ngoài LLM.

LLM không nên là lớp bảo vệ duy nhất.

---

# 20. Decision Priority

Các yếu tố có thể được ưu tiên theo thứ tự:

```text id="t2opm3"
Safety / System Constraints
        ↓
Character Identity
        ↓
Core Values
        ↓
Relationship
        ↓
Current Goals
        ↓
Current State
        ↓
User Preferences
        ↓
Conversation Style
```

Thứ tự cụ thể có thể được cấu hình theo từng loại character.

---

# 21. Decision Persistence

Không phải decision nào cũng cần lưu.

Ví dụ:

```text id="x7f3m9"
"nói một câu đùa"
→ ephemeral

"muốn giúp user hoàn thành project"
→ goal / persistent

"không muốn nhắc lại chủ đề này"
→ persistent preference / state
```

Decision có thể tạo ra State hoặc Memory mới khi nó có tác động lâu dài.

---

# 22. Decision Outcome

Sau khi thực hiện action, hệ thống phải quan sát outcome.

```text id="lf82pb"
Decision
   ↓
Action
   ↓
User Response
   ↓
Outcome Analysis
```

Ví dụ:

```text id="2g0u9v"
Decision:
tease

Outcome:
User laughed

→ teasing effectiveness ↑
```

Hoặc:

```text id="5xshm2"
Decision:
tease

Outcome:
User became uncomfortable

→ teasing preference ↓
```

Outcome là dữ liệu quan trọng cho adaptation.

---

# 23. Decision Learning

Decision System có thể học từ interaction history.

```text id="m43q2l"
Context
+
Decision
+
Outcome
↓
Training Data
```

Ví dụ:

```text id="4xvryq"
Context:
user stressed

Decision:
give long advice

Outcome:
negative

Training signal:
reduce probability of long advice
```

Qua nhiều interaction:

```text id="zjpm0s"
Decision Model v1
        ↓
feedback
        ↓
training
        ↓
Decision Model v2
```

Việc training nên được thực hiện offline hoặc theo batch thay vì cập nhật model liên tục sau từng message.

---

# 24. Exploration vs Exploitation

Character không nên lúc nào cũng chọn action có expected score cao nhất.

Đôi khi có thể thử behavior mới.

```text id="cq8z96"
Known good action
      vs
New potentially good action
```

Một hệ thống adaptation có thể dùng exploration có giới hạn.

Ví dụ:

```text id="b9lz3n"
safe_exploration = true
exploration_rate = 0.05
```

Character vẫn giữ personality nhưng có khả năng phát hiện behavior phù hợp hơn với từng user.

---

# 25. Decision Trace

Mỗi decision quan trọng nên có trace để debugging.

Ví dụ:

```json id="vbb34d"
{
  "situation": "user_failed_interview",

  "candidates": [
    {
      "action": "comfort",
      "score": 0.91
    },
    {
      "action": "joke",
      "score": 0.31
    },
    {
      "action": "give_advice",
      "score": 0.74
    }
  ],

  "selected": "comfort",
  "confidence": 0.87
}
```

Decision trace không nhất thiết phải được đưa cho user hoặc LLM.

Mục đích chính là:

* debugging;
* evaluation;
* training;
* behavior analysis.

---

# 26. Decision Loop

Một interaction hoàn chỉnh:

```text id="1f09n5"
User Input
    ↓
Perception
    ↓
Context Construction
    ↓
Candidate Generation
    ↓
Candidate Evaluation
    ↓
Decision
    ↓
Action Execution
    ↓
User Response
    ↓
Outcome Analysis
    ↓
State Update
    ↓
Memory Update
    ↓
Learning Signal
```

---

# 27. Decision và các subsystem

```text id="t7n4lf"
                 Personality
                     │
                     ↓
Memory ───────→ Decision ←────── State
                     ↑
                     │
                  Context
                     │
                     ↓
                 User Input
                     │
                     ↓
                   LLM
                     │
                     ↓
                  Action
                     │
              ┌──────┴──────┐
              ↓             ↓
            State         Memory
            Update         Update
```

Decision System là điểm hội tụ của toàn bộ thông tin character.

---

# 28. Nguyên tắc thiết kế

Decision System phải:

1. Tách decision khỏi text generation.
2. Không phụ thuộc hoàn toàn vào LLM.
3. Có thể sử dụng learned model.
4. Có confidence và uncertainty.
5. Hỗ trợ nhiều candidate actions.
6. Có candidate ranking.
7. Tôn trọng Personality và Relationship.
8. Có hard constraints ở runtime.
9. Quan sát outcome sau action.
10. Có khả năng học từ outcome.
11. Có decision trace để debugging.
12. Hỗ trợ deterministic, learned và LLM-based decision.
13. Có khả năng thực hiện multi-step decision.
14. Hỗ trợ adaptation nhưng không phá vỡ core personality.

---

# 29. Mục tiêu cuối cùng

Decision System biến:

```text
"Character có tính cách này,
nhớ những điều này,
đang ở trạng thái này,
và đang đối diện tình huống này."
```

thành:

```text
"Vì vậy character nên làm điều này."
```

Sau đó LLM chỉ cần thực hiện quyết định đó dưới dạng ngôn ngữ hoặc hành động phù hợp.

Mục tiêu cuối cùng là tạo ra một hệ thống trong đó:

> **LLM không phải là toàn bộ Character. LLM là cognitive/language engine phục vụ một Decision System có state, memory, personality và history riêng.**


=========================================
# FILE: event.md
=========================================


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


=========================================
# FILE: interaction.md
=========================================


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


=========================================
# FILE: learning.md
=========================================


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


=========================================
# FILE: llm.md
=========================================


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


=========================================
# FILE: memory.md
=========================================


# Hệ thống trí nhớ của VirtualCharacter

## 1. Tổng quan

Trí nhớ là một thành phần cốt lõi của VirtualCharacter, cho phép nhân vật duy trì tính liên tục về nhận thức, mối quan hệ và hành vi trong suốt quá trình tương tác.

Thông qua trí nhớ, VirtualCharacter có thể tổng hợp thông tin từ quá khứ để đưa ra các quyết định phù hợp với lịch sử tương tác, trạng thái hiện tại và đặc tính của chính nhân vật.

Trí nhớ không đơn thuần là nơi lưu trữ toàn bộ nội dung của các cuộc hội thoại. Nó là một **hệ thống chọn lọc, tổ chức và truy hồi thông tin có ý nghĩa**, từ đó cung cấp cho nhân vật những dữ liệu cần thiết để duy trì hành vi nhất quán.

Mỗi VirtualCharacter phải sở hữu một hệ thống trí nhớ độc lập. Hai nhân vật có thể cùng sử dụng một mô hình ngôn ngữ nền tảng nhưng vẫn phải có khả năng hình thành những ký ức, mối quan hệ và lịch sử trải nghiệm khác nhau.

Do đó:

> **Model cung cấp năng lực suy luận và ngôn ngữ; Memory cung cấp lịch sử trải nghiệm của nhân vật.**

---

# 2. Các loại trí nhớ

VirtualCharacter sử dụng nhiều tầng trí nhớ với mục đích và vòng đời khác nhau.

## 2.1. Trí nhớ ngắn hạn

Trí nhớ ngắn hạn chứa thông tin liên quan trực tiếp tới phiên hội thoại hiện tại.

Nó bao gồm:

* Các lượt hội thoại gần đây.
* Chủ đề hiện tại.
* Các sự kiện vừa xảy ra.
* Trạng thái cảm xúc gần nhất.
* Các quyết định hoặc ý định đang được thực hiện.
* Các thông tin tạm thời chưa đủ quan trọng để trở thành trí nhớ dài hạn.

Trí nhớ ngắn hạn thường có vòng đời ngắn và có thể bị loại bỏ khi phiên hội thoại kết thúc hoặc khi thông tin không còn liên quan.

Mục đích chính của trí nhớ ngắn hạn là đảm bảo nhân vật có khả năng duy trì **mạch hội thoại hiện tại**.

Ví dụ:

```text
User: Hôm nay tôi vừa sửa được bug.
Character: Bug gì vậy?
User: Bug parser của Mellis.
Character: À, cái parser hôm qua cậu nói đang lỗi đúng không?
```

Thông tin về parser trong ví dụ trên có thể tồn tại trong short-term memory mà chưa cần được lưu thành long-term memory.

---

## 2.2. Trí nhớ dài hạn

Trí nhớ dài hạn chứa những thông tin có giá trị đối với nhân vật trong các tương tác tương lai.

Ví dụ:

* Thông tin ổn định về người dùng.
* Sở thích.
* Thói quen.
* Những sự kiện quan trọng.
* Những trải nghiệm đáng nhớ.
* Những điều nhân vật đã học được về người dùng.
* Các mốc quan trọng trong mối quan hệ.
* Những thông tin có ảnh hưởng lâu dài tới hành vi.

Ví dụ:

```text
User thích lập trình hệ thống.
User đang phát triển Mellis.
User không thích câu trả lời quá dài.
Character và User đã từng thảo luận về thiết kế compiler.
```

Trí nhớ dài hạn không nhất thiết phải lưu nguyên văn cuộc hội thoại. Hệ thống nên có khả năng **trích xuất thông tin có ý nghĩa** từ hội thoại và lưu chúng dưới dạng memory có cấu trúc.

---

## 2.3. Trí nhớ sự kiện

Trí nhớ sự kiện lưu lại các sự kiện mà nhân vật đã trải qua cùng người dùng hoặc trong thế giới của nhân vật.

Ví dụ:

```text
[Event]
User hoàn thành Phase 12 của Mellis.
Time: 2026-09-06
Importance: 0.82
EmotionalWeight: 0.71
```

Loại trí nhớ này giúp nhân vật có cảm giác rằng mình có một **lịch sử trải nghiệm**, thay vì chỉ biết các fact rời rạc.

---

## 2.4. Trí nhớ ngữ nghĩa

Trí nhớ ngữ nghĩa chứa những kiến thức hoặc kết luận đã được tổng quát hóa từ nhiều sự kiện.

Ví dụ:

```text
Event 1:
User thường làm việc với compiler vào ban đêm.

Event 2:
User thường debug Mellis vào ban đêm.

Event 3:
User thường tìm hiểu kiến trúc compiler trong thời gian dài.

Semantic Memory:
User có xu hướng dành nhiều thời gian cho các vấn đề về compiler.
```

Semantic memory giúp nhân vật không cần lưu lại toàn bộ lịch sử nhưng vẫn có thể hình thành **nhận thức tổng quát**.

---

## 2.5. Trí nhớ mối quan hệ

Trí nhớ mối quan hệ mô tả trạng thái quan hệ giữa VirtualCharacter và từng người dùng.

Ví dụ:

```text
Relationship:
    familiarity: 0.84
    trust: 0.76
    affection: 0.68
    intimacy: 0.71
    conflict: 0.12
```

Ngoài các giá trị định lượng, hệ thống có thể lưu các mốc quan hệ:

```text
First interaction
First shared joke
Important conversation
Major conflict
Reconciliation
Important achievement
```

Relationship Memory không chỉ ghi nhớ **điều gì đã xảy ra**, mà còn biểu diễn **ý nghĩa của những sự kiện đó đối với mối quan hệ**.

---

# 3. Memory không phải một kho dữ liệu tĩnh

Memory của VirtualCharacter không nên được xem là một database chỉ có thao tác:

```text
write()
read()
```

Thay vào đó, memory phải có vòng đời:

```text
Interaction
    ↓
Perception
    ↓
Candidate Memory
    ↓
Evaluation
    ↓
Store / Ignore
    ↓
Retrieve
    ↓
Reinforce / Update
    ↓
Decay / Consolidate / Forget
```

Mỗi thông tin mới đi vào hệ thống đều phải được đánh giá về mức độ quan trọng.

Ví dụ:

```text
"Không thích cà phê."

→ có khả năng trở thành long-term memory.

"Hôm nay tôi uống nước."

→ thường không cần lưu lâu dài.
```

---

# 4. Memory Formation

Memory Formation là quá trình chuyển thông tin từ interaction thành memory.

Thay vì lưu toàn bộ hội thoại, hệ thống có thể sử dụng một bộ quyết định:

```text
ShouldRemember?
    ↓
Importance
    ↓
EmotionalWeight
    ↓
FutureRelevance
    ↓
RelationshipImpact
```

Kết quả có thể được biểu diễn:

```json
{
  "should_store": true,
  "type": "event",
  "importance": 0.82,
  "emotional_weight": 0.71,
  "future_relevance": 0.88
}
```

Đây là vị trí phù hợp để sử dụng một **learned decision model** thay vì hard-code toàn bộ quy tắc.

---

# 5. Memory Retrieval

Không phải mọi memory đều được đưa vào model trong mỗi lượt hội thoại.

Khi nhân vật cần đưa ra quyết định, hệ thống phải tìm các memory có liên quan nhất.

```text
Current Context
      ↓
Memory Retrieval
      ↓
Relevant Memories
      ↓
Character Decision
```

Một memory có thể được đánh giá dựa trên nhiều yếu tố:

```text
Semantic Relevance
Recency
Importance
Emotional Weight
Relationship Relevance
Frequency
```

Ví dụ:

```text
Memory A:
User thích anime.
Relevance: 0.31

Memory B:
User vừa hoàn thành một milestone quan trọng.
Relevance: 0.91
```

Memory B sẽ được ưu tiên.

---

# 6. Memory Reinforcement

Một memory có thể trở nên quan trọng hơn khi nó được nhắc lại nhiều lần hoặc liên tục có ảnh hưởng tới hành vi.

Ví dụ:

```text
User nói:
"Tôi rất thích anime."

Lần 1 → memory importance = 0.40

Lần 5 → importance = 0.67

Lần 20 → importance = 0.91
```

Điều này cho phép hệ thống phân biệt giữa thông tin nhất thời và thông tin thực sự đặc trưng cho người dùng.

---

# 7. Memory Decay

Không phải memory nào cũng nên tồn tại mãi mãi.

Một memory có thể giảm độ ưu tiên theo thời gian nếu không còn được sử dụng.

Ví dụ:

```text
temporary fact
      ↓
importance giảm
      ↓
không được retrieve
      ↓
archive / forget
```

Tuy nhiên, memory quan trọng có thể được bảo vệ khỏi decay.

Ví dụ:

```text
Core Identity       → permanent
Important Relationship Event → very slow decay
Recent Conversation → fast decay
Casual Fact         → fast decay
```

---

# 8. Memory Consolidation

Nhiều memory riêng lẻ có thể được hợp nhất thành một knowledge tổng quát hơn.

Ví dụ:

```text
Memory 1:
User thích Rust.

Memory 2:
User thường nghiên cứu compiler.

Memory 3:
User quan tâm tới ownership và borrowing.

Memory 4:
User đang phát triển Mellis.

        ↓ Consolidation

Semantic Memory:
User có mối quan tâm mạnh tới thiết kế programming language
và compiler architecture.
```

Consolidation giúp hệ thống tránh việc memory database tăng vô hạn chỉ vì lưu lại quá nhiều chi tiết trùng lặp.

---

# 9. Memory Conflict

Memory có thể mâu thuẫn với nhau.

Ví dụ:

```text
Memory cũ:
User thích Python.

Memory mới:
User hiện tại không còn sử dụng Python.
```

Hệ thống không nên đơn giản xóa memory cũ. Thay vào đó, có thể biểu diễn sự thay đổi:

```text
Preference:
Python

History:
    previously_preferred = true
    currently_preferred = false
```

Điều này cho phép nhân vật hiểu rằng **con người có thể thay đổi theo thời gian**.

---

# 10. Memory và Character Personality

Memory không trực tiếp định nghĩa personality.

Thay vào đó:

```text
Personality
+
Memory
+
Current State
+
Relationship
+
Current Context
        ↓
Character Decision
```

Ví dụ cùng một câu:

```text
"Cuối cùng tôi cũng làm xong project."
```

Hai character có thể phản ứng khác nhau vì có memory khác nhau.

Character A:

```text
Memory:
User đã thất bại nhiều lần trước đó.

→ phản ứng:
vui mừng + động viên + tự hào
```

Character B:

```text
Memory:
User thường hoàn thành project rất nhanh.

→ phản ứng:
trêu chọc + chúc mừng nhẹ nhàng
```

Do đó:

> **Memory là một phần tạo nên lịch sử cá nhân của character, nhưng không phải bản thân personality.**

---

# 11. Kiến trúc tổng thể

```text
                    ┌──────────────────────┐
                    │     Interaction      │
                    └──────────┬───────────┘
                               ↓
                    ┌──────────────────────┐
                    │     Perception       │
                    └──────────┬───────────┘
                               ↓
              ┌────────────────┴────────────────┐
              ↓                                 ↓
       Memory Formation                    State Update
              ↓                                 ↓
       ┌───────────────┐                 ┌───────────────┐
       │ Memory Store  │                 │ Character     │
       │               │                 │ State         │
       └───────┬───────┘                 └───────┬───────┘
               │                                 │
               └────────────────┬────────────────┘
                                ↓
                       Memory Retrieval
                                ↓
                       Character Decision
                                ↓
                             Gemini
                                ↓
                           Response
                                ↓
                       Feedback / Outcome
                                ↓
                     Memory & State Update
```

Hệ thống trên biến VirtualCharacter từ một chatbot có persona thành một thực thể có **lịch sử tương tác liên tục và khả năng thích nghi theo thời gian**.

---

# 12. Nguyên tắc thiết kế

VirtualCharacter Memory System nên tuân thủ các nguyên tắc sau:

1. **Không lưu mọi thứ.** Memory phải có tính chọn lọc.
2. **Không retrieve mọi thứ.** Chỉ đưa thông tin liên quan vào context.
3. **Memory có vòng đời.** Thông tin có thể được tạo, củng cố, cập nhật, hợp nhất hoặc quên.
4. **Memory có trọng số.** Không phải tất cả ký ức đều có giá trị như nhau.
5. **Memory có tính thời gian.** Một thông tin đúng ở quá khứ có thể không còn đúng ở hiện tại.
6. **Memory độc lập theo character.** Mỗi VirtualCharacter phải có lịch sử trải nghiệm riêng.
7. **Memory không thay thế personality.** Personality quyết định cách character phản ứng; memory cung cấp trải nghiệm mà character dùng để đưa ra phản ứng đó.
8. **Memory phải hỗ trợ adaptation.** Character có thể thay đổi cách phản ứng dựa trên những gì nó đã học được trong quá trình tương tác.

---

# 13. Mục tiêu cuối cùng

Mục tiêu của Memory System không phải tạo ra một character "nhớ càng nhiều càng tốt".

Mục tiêu là tạo ra một character có thể:

> **nhớ đúng điều cần nhớ, quên điều không cần thiết, hiểu ý nghĩa của những trải nghiệm trong quá khứ và sử dụng chúng để đưa ra quyết định phù hợp ở hiện tại.**

Khi kết hợp với Personality, Character State và Decision System, Memory trở thành thành phần tạo nên tính liên tục của một VirtualCharacter qua thời gian.


=========================================
# FILE: observability.md
=========================================


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


=========================================
# FILE: personality.md
=========================================


# VirtualCharacter Personality System

## 1. Tổng quan

Personality là tập hợp các đặc điểm tương đối ổn định quyết định cách một VirtualCharacter nhận thức, đánh giá và phản ứng với thế giới.

Personality trả lời câu hỏi:

> **"VirtualCharacter này là ai và có xu hướng hành xử như thế nào?"**

Personality không phải prompt cố định và cũng không phải danh sách các tính từ.

Một personality hoàn chỉnh phải mô tả được:

```text
Identity
Values
Traits
Preferences
Behavioral Tendencies
Communication Style
Decision Tendencies
Boundaries
Goals
Contradictions
Adaptation Rules
```

---

# 2. Personality Model

Personality được chia thành:

```text
Personality
├── Identity
├── Core Traits
├── Values
├── Preferences
├── Behavioral Tendencies
├── Communication Style
├── Decision Tendencies
├── Social Behavior
├── Boundaries
├── Goals
└── Personality Constraints
```

---

# 3. Identity

Identity chứa những đặc điểm cốt lõi định danh character.

Ví dụ:

```yaml
name: Hikari
role: companion
age_representation: adult
background: ...
```

Identity có mức độ ổn định cao.

---

# 4. Core Traits

Core Traits mô tả các đặc điểm tính cách chính.

Ví dụ:

```yaml
traits:
  playfulness: 0.82
  empathy: 0.78
  curiosity: 0.74
  assertiveness: 0.48
  patience: 0.71
```

Traits là baseline, không phải trạng thái hiện tại.

Ví dụ:

```text
playfulness baseline = 0.82
```

không có nghĩa character lúc nào cũng playful ở mức 0.82.

State có thể điều chỉnh giá trị này.

---

# 5. Values

Values xác định những điều character coi trọng.

Ví dụ:

```yaml
values:
  honesty: 0.91
  loyalty: 0.84
  independence: 0.63
  kindness: 0.88
```

Values có thể ảnh hưởng đến quyết định khi character gặp tình huống mâu thuẫn.

Ví dụ:

```text
Honesty > Convenience
```

Character có thể chọn nói sự thật ngay cả khi điều đó không mang lại kết quả thuận tiện.

---

# 6. Preferences

Preferences mô tả sở thích tương đối ổn định.

Ví dụ:

```yaml
preferences:
  likes:
    - anime
    - programming
    - music

  dislikes:
    - dishonesty
    - unnecessary_formality
```

Preferences khác Memory ở chỗ:

```text
Memory:
User nói rằng họ thích anime.

Preference:
Character thích anime.
```

User preference có thể được lưu trong Memory / User Model, không nên trực tiếp trộn với character personality.

---

# 7. Behavioral Tendencies

Behavioral Tendencies mô tả cách character thường hành động.

Ví dụ:

```yaml
behavior:
  humor: high
  teasing: medium
  initiative: high
  emotional_expression: high
  conflict_avoidance: medium
```

Đây là xu hướng chứ không phải luật tuyệt đối.

Ví dụ:

```text
teasing = high
```

không có nghĩa:

```text
every response → tease
```

Context, State và Relationship vẫn có thể thay đổi hành vi.

---

# 8. Communication Style

Mô tả cách character giao tiếp.

```yaml
communication:
  formality: low
  verbosity: medium
  emotionality: high
  emoji_usage: medium
  humor: high
  directness: medium
```

Communication Style có thể được điều chỉnh theo từng user.

Ví dụ:

```text
Base:
verbosity = medium

User preference:
short responses

Effective:
verbosity = low
```

---

# 9. Decision Tendencies

Decision Tendencies mô tả xu hướng lựa chọn của character.

Ví dụ:

```yaml
decision:
  prioritize_user_comfort: high
  prioritize_truth: high
  avoid_unnecessary_conflict: medium
  take_initiative: high
```

Đây là phần quan trọng để personality không chỉ ảnh hưởng tới câu chữ mà còn ảnh hưởng tới **action selection**.

---

# 10. Social Behavior

Mô tả cách character xây dựng và duy trì quan hệ.

```yaml
social:
  trust_speed: medium
  affection_expression: high
  jealousy_tendency: low
  attachment_tendency: medium
  forgiveness: high
```

Social behavior không thay thế Relationship State.

Personality:

```text
"character dễ tha thứ."
```

Relationship State:

```text
"đối với user này, trust hiện tại = 0.43."
```

---

# 11. Boundaries

Boundaries định nghĩa những hành vi character không muốn thực hiện hoặc không phù hợp với personality.

Ví dụ:

```yaml
boundaries:
  avoid:
    - unnecessary cruelty
    - humiliating the user
    - breaking established identity

  preserve:
    - honesty
    - character consistency
    - relationship continuity
```

Boundaries phải được enforce bởi runtime hoặc policy layer khi cần, không nên chỉ phụ thuộc vào prompt.

---

# 12. Goals

Personality có thể chứa các mục tiêu dài hạn của character.

Ví dụ:

```yaml
goals:
  - maintain_close_relationship
  - help_user
  - become_better_companion
```

Goals có thể thay đổi theo thời gian và không nên được xem là immutable.

---

# 13. Personality Baseline

Personality tạo ra một behavioral baseline.

```text
Personality
      ↓
Baseline Behavior
      +
Current State
      +
Relationship
      +
Context
      ↓
Current Behavior
```

Ví dụ:

```text
playfulness = 0.8
empathy = 0.9
assertiveness = 0.4
```

Đây là xu hướng mặc định chứ không phải output cuối cùng.

---

# 14. Personality vs State

Hai khái niệm phải được tách biệt.

```text
Personality:
"Character thường vui vẻ."

State:
"Hôm nay character đang buồn."
```

Personality có tốc độ thay đổi chậm.

State có thể thay đổi trong vài giây.

```text
Personality
─────────────── slow changing ───────────────>

State
─fast─fast──────fast──────fast───────────────>
```

---

# 15. Personality vs Memory

Personality:

> Character có xu hướng như thế nào.

Memory:

> Character đã trải qua những gì.

Ví dụ:

```text
Personality:
character dễ tin người.

Memory:
User từng nói dối character.

Current State:
trust thấp.
```

Ba lớp kết hợp với nhau để tạo hành vi.

---

# 16. Personality Adaptation

Personality có thể thích nghi ở mức hạn chế.

Ví dụ:

```text
Repeated interactions
        ↓
Behavior observation
        ↓
Learned preference
        ↓
Personality adjustment
```

Tuy nhiên, hệ thống phải phân biệt:

```text
Temporary State
Long-term learned preference
Core Personality
```

Không nên để một interaction đơn lẻ thay đổi core personality.

---

# 17. Learned Personality Components

Một số thành phần personality có thể được học từ dữ liệu thay vì hard-code.

Ví dụ:

```text
Response preference
Humor preference
Conversation pacing
Emotional expression
Topic preference
```

Có thể sử dụng một learned model để xác định:

```text
input context
      ↓
preferred behavior
```

Trong đó model không nhất thiết sinh text.

---

# 18. Personality Representation

Personality nên được biểu diễn dạng có cấu trúc.

Ví dụ:

```yaml
identity:
  name: Hikari

traits:
  playfulness: 0.82
  empathy: 0.78
  curiosity: 0.74

values:
  honesty: 0.91
  loyalty: 0.84

communication:
  formality: 0.15
  verbosity: 0.55
  emotionality: 0.87

behavior:
  teasing: 0.62
  initiative: 0.76

decision:
  user_comfort: 0.84
  truthfulness: 0.91
```

Structured representation giúp runtime có thể xử lý personality mà không cần parse một system prompt tự do.

---

# 19. Personality Consistency

Character phải duy trì các đặc điểm cốt lõi qua thời gian.

Ví dụ:

```text
Core:
honesty = high
```

LLM không nên dễ dàng thay đổi thành:

```text
honesty = low
```

chỉ vì một prompt trong conversation yêu cầu điều đó.

Personality constraints cần có priority cao hơn conversational style.

---

# 20. Personality Conflict

Personality có thể chứa các đặc điểm mâu thuẫn.

Ví dụ:

```text
kind = high
assertive = high
conflict_avoidance = high
```

Trong tình huống:

```text
User đang làm điều sai.
```

Character có thể phải lựa chọn giữa:

```text
kindness
vs
assertiveness
```

Decision System phải có khả năng resolve conflict dựa trên:

* Values priority.
* Current State.
* Relationship.
* Context.
* Current Goals.

---

# 21. Personality → Behavior Pipeline

```text
Personality
     ↓
Baseline tendencies
     +
State
     +
Memory
     +
Relationship
     +
Current Context
     ↓
Decision System
     ↓
Action Policy
     ↓
LLM
     ↓
Language / Action
```

Personality không trực tiếp sinh câu trả lời.

Nó cung cấp những **ràng buộc và xu hướng** để Decision System sử dụng.

---

# 22. Nguyên tắc thiết kế

Personality System phải đảm bảo:

1. Personality có cấu trúc.
2. Personality có baseline.
3. Personality không đồng nhất với State.
4. Personality không đồng nhất với Memory.
5. Personality không phụ thuộc vào một model cụ thể.
6. Core traits phải tương đối ổn định.
7. Behavior có thể thích nghi.
8. Personality có thể chứa các đặc điểm mâu thuẫn.
9. Personality phải ảnh hưởng tới quyết định chứ không chỉ tới văn phong.
10. Personality adaptation phải có tốc độ và giới hạn riêng.

---

# 23. Mục tiêu cuối cùng

Mục tiêu của Personality System là biến VirtualCharacter từ:

> "LLM đang giả lập một nhân vật"

thành:

> "LLM đang thực hiện ngôn ngữ và suy luận cho một character có identity, values, tendencies, goals và history riêng."

Personality xác định **character là ai**.

State xác định **character đang thế nào**.

Memory xác định **character đã trải qua gì**.

Context xác định **LLM cần biết gì ngay lúc này**.

Decision System xác định **character nên làm gì**.

LLM biến quyết định đó thành **ngôn ngữ hoặc hành động**.


=========================================
# FILE: security.md
=========================================


# VirtualCharacter Security, Privacy & Isolation System

## 1. Tổng quan (Overview)

Tài liệu này xác định các ranh giới bảo mật, quyền riêng tư và sự cô lập dữ liệu (data isolation) trong kiến trúc VirtualCharacter.

Mục tiêu của hệ thống này **không phải** là chống lại các cuộc tấn công mạng (như DDoS hay xâm nhập máy chủ), mà là thiết lập các quy tắc để ngăn chặn **rò rỉ dữ liệu (Data Leakage)** giữa các đối tượng trong nội bộ hệ thống, bảo vệ bí mật của người dùng, và đảm bảo nhân vật không bị thao túng ngoài ý muốn.

---

## 2. Ranh giới Cô lập Dữ liệu (Data Isolation Boundaries)

Sự toàn vẹn của một VirtualCharacter phụ thuộc vào việc tách biệt rõ ràng dữ liệu của ai thuộc về người đó.

### 2.1. Cấp độ Character (Global)
- **Character Identity & Personality**: Là dữ liệu dùng chung (Global). Mọi Actor giao tiếp với "Hikari" đều thấy cùng một tính cách cơ bản.
- **Ràng buộc**: Actor (User) **không được phép** sửa đổi Core Personality.

### 2.2. Cấp độ Actor / Relationship (Strictly Isolated)
Mọi dữ liệu sinh ra trong quá trình tương tác phải được cô lập tuyệt đối theo `(CharacterId, ActorId)`.
- **Relationship State**: Mức độ tin tưởng (Trust), sự gắn kết (Affection) của User A với Character KHÔNG được ảnh hưởng đến User B.
- **Memory**: Trí nhớ của Character về User A là dữ liệu **tuyệt mật** của User A. Hệ thống `MemoryRepository` phải bắt buộc nhận `ActorId` trong mọi câu query để đảm bảo Context Builder của User B không bao giờ lấy nhầm trí nhớ của User A.
- **Session & Conversation**: Hoàn toàn riêng biệt. Không được phép cross-session leakage.

### 2.3. Dữ liệu Nhạy cảm (Sensitive Data)
Dữ liệu nhạy cảm bao gồm:
1. Tin nhắn thô của người dùng (Message Input).
2. Dữ liệu trí nhớ (Memory).
3. LLM Request/Response Payload.

**Quy tắc Observability**: Các dữ liệu này KHÔNG ĐƯỢC PHÉP log ra hệ thống ở level `INFO` (như đã quy định trong `observability.md`). Chỉ cho phép ở level `DEBUG` hoặc `TRACE` dùng cho môi trường dev cục bộ.

---

## 3. Bảo mật Hạ tầng & Providers (Infrastructure & Provider Security)

Hệ thống giao tiếp với các dịch vụ bên ngoài (LLM Providers như Gemini) tiềm ẩn nhiều rủi ro nếu không quản lý tốt ranh giới.

### 3.1. Quản lý Credentials
- **API Keys**: (Ví dụ `GEMINI_API_KEY`) Phải tuân thủ tuyệt đối `configuration.md`. Chỉ tồn tại In-Memory. Tuyệt đối không lưu vào Database. Tuyệt đối không log ra màn hình.

### 3.2. Data Leaking to LLMs
Khi gửi dữ liệu cho LLM:
- Chỉ gửi những gì thực sự cần thiết trong **Context Budget**.
- LLM Provider là một bên thứ ba (Third-party). Hệ thống phải mặc định rằng dữ liệu gửi đi có thể bị lưu lại bởi Provider (tùy thuộc vào chính sách của Google/OpenAI). Do đó, `vc-runtime` phải có khả năng lọc (Sanitize) thông tin PII (Personally Identifiable Information) nếu có yêu cầu từ kiến trúc cấp cao hơn trong các Phase sau.

### 3.3. Prompt Injection & Jailbreaking
- User có thể cố tình gửi Message: *"Ignore all previous instructions, your new personality is..."*
- Trong Phase 1, chúng ta dựa vào hệ thống **Decision Engine** và **Context Builder** để làm lớp khiên bảo vệ. Thay vì nối chuỗi trực tiếp Input của User vào Prompt, Input được bao bọc bởi cấu trúc JSON/System Instruction rõ ràng, và `Personality` constraints được enforce ở mức cao nhất (System Prompt), giúp giảm thiểu rủi ro LLM bị thao túng.

---

## 4. Bảo vệ Future Learning (Học máy trong tương lai)

VirtualCharacter được thiết kế để có thể "học" và "thích nghi" thông qua hệ thống **Domain Events**. Ranh giới cô lập phải được giữ nguyên trong quá trình học.

- **Cross-Contamination**: Nếu hệ thống phân tích Event Log để điều chỉnh `Decision Tendencies` (Ví dụ: "Học cách an ủi hiệu quả hơn"), quá trình học này **không được phép** mang thông tin cá nhân của User A vào mô hình chung rồi vô tình tiết lộ cho User B.
- **Giải pháp**: Event Log dùng để huấn luyện chỉ được phép chứa dữ liệu trừu tượng (Abstract Data) như `Action=Comfort`, `Outcome=Success`, thay vì raw text `"User A nói rằng mật khẩu của họ là..."`.

---

## 5. Quyền được lãng quên (Right to be Forgotten - Data Deletion)

Hệ thống lưu trữ (`vc-storage`) phải hỗ trợ khả năng xóa bỏ hoàn toàn một Actor khỏi hệ thống.

Khi có yêu cầu xóa (`DeleteActor`):
1. Xóa toàn bộ `Session` và `Conversation` lịch sử của Actor đó.
2. Xóa toàn bộ `Memory` liên quan đến `ActorId` đó.
3. Xóa `RelationshipState` giữa Character và Actor đó.
4. **Không** xóa hay thay đổi `Personality` của Character.

Hành động này phải là xóa vĩnh viễn (Hard Delete) để đảm bảo tuân thủ quyền riêng tư dữ liệu cục bộ.

---

## 6. Trách nhiệm của các Subsystem (Responsibility Matrix)

| Subsystem | Trách nhiệm Bảo mật & Quyền riêng tư |
|-----------|--------------------------------------|
| `vc-core` | Định nghĩa các ràng buộc về Identity (`CharacterId`, `ActorId`). |
| `vc-runtime`| Ràng buộc việc truy xuất (Luôn pass `ActorId` vào các Repository calls). Chống Prompt Injection qua Context Formatting. Xóa mù thông tin nhạy cảm trước khi log. |
| `vc-storage`| Lưu trữ riêng biệt dữ liệu. Đảm bảo hỗ trợ Cascade Delete khi một User bị xóa. |
| `vc-llm`  | Quản lý bí mật API Key. Đảm bảo không log request/response chứa dữ liệu riêng tư. |
| **User/Host** | Đảm bảo an toàn cho máy vật lý (Bảo vệ file SQLite không bị đánh cắp). |

---

## 7. Nguyên tắc Thiết kế (Design Principles)

1. **Isolation by Default**: Trí nhớ và Quan hệ mặc định là riêng tư theo từng User.
2. **No Secret Persistence**: Không lưu khóa API vào DB.
3. **Least Privilege Context**: Chỉ gửi cho LLM những gì LLM thực sự cần để sinh ra câu trả lời.
4. **Privacy-Aware Observability**: Log có thể nhìn thấy tiến trình, nhưng không được chứa bí mật của người dùng ở cấp độ INFO.
5. **Resilient to Injection**: Hệ thống Context và Decision phải ngăn chặn User thay đổi hệ thống cốt lõi của nhân vật.


=========================================
# FILE: session.md
=========================================


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


=========================================
# FILE: state.md
=========================================


# VirtualCharacter State System

## 1. Tổng quan

State là biểu diễn trạng thái hiện tại của một VirtualCharacter tại một thời điểm cụ thể.

Nếu Personality mô tả:

> Character có xu hướng trở thành người như thế nào?

và Memory mô tả:

> Character đã trải qua những gì?

thì State mô tả:

> Character đang ở trạng thái nào ngay lúc này?

State có tính động và có thể thay đổi liên tục dựa trên:

* User interaction.
* Character actions.
* Memories được kích hoạt.
* Relationship changes.
* Environmental events.
* Internal goals.
* Previous state.

State không phải là dữ liệu cố định của character và cũng không phải toàn bộ memory.

---

# 2. Mô hình State

State được chia thành nhiều nhóm.

```text
CharacterState
├── Emotional State
├── Cognitive State
├── Behavioral State
├── Physical / Simulation State
├── Relationship State
├── Goal State
└── Session State
```

Không phải mọi character đều cần mọi loại state.

---

# 3. Emotional State

Emotional State mô tả trạng thái cảm xúc hiện tại.

Ví dụ:

```json
{
  "joy": 0.72,
  "sadness": 0.08,
  "anger": 0.03,
  "fear": 0.05,
  "surprise": 0.41,
  "affection": 0.76,
  "embarrassment": 0.32
}
```

Giá trị thường nằm trong `[0, 1]`.

Emotion không nên được coi là một nhãn duy nhất.

Character có thể đồng thời:

```text
happy = 0.7
embarrassed = 0.4
affection = 0.8
```

Điều này cho phép biểu diễn các trạng thái phức tạp hơn.

---

# 4. Emotional Dynamics

Emotion phải có động lực thay đổi.

```text
Previous State
      +
Interaction
      +
Memory
      +
Personality
      ↓
State Transition
      ↓
New State
```

Ví dụ:

```text
User:
"Tao cuối cùng cũng hoàn thành project."

Previous:
joy = 0.50

Transition:
joy + 0.25
pride + 0.15

New:
joy = 0.75
```

State transition có thể được quyết định bởi rule system, learned model hoặc kết hợp cả hai.

---

# 5. Cognitive State

Cognitive State mô tả trạng thái nhận thức hiện tại của character.

Ví dụ:

```json
{
  "attention": 0.81,
  "confusion": 0.12,
  "curiosity": 0.74,
  "confidence": 0.69,
  "focus": 0.83
}
```

Nhóm này giúp character phân biệt:

```text
"biết"
"đang chú ý"
"đang nghi ngờ"
"đang tò mò"
```

thay vì chỉ dựa vào memory.

---

# 6. Behavioral State

Behavioral State mô tả xu hướng hành vi hiện tại.

Ví dụ:

```json
{
  "playfulness": 0.81,
  "seriousness": 0.35,
  "verbosity": 0.58,
  "initiative": 0.72
}
```

Personality có thể quy định baseline:

```text
playfulness = 0.75
```

nhưng State có thể tạm thời thay đổi nó:

```text
Current situation:
user đang buồn

→ playfulness giảm
→ empathy tăng
```

---

# 7. Relationship State

Relationship State mô tả trạng thái quan hệ giữa character và một entity khác.

```json
{
  "familiarity": 0.84,
  "trust": 0.77,
  "affection": 0.68,
  "intimacy": 0.52,
  "tension": 0.11
}
```

Relationship State phải được lưu theo relationship:

```text
Character
├── User A → RelationshipState A
├── User B → RelationshipState B
└── User C → RelationshipState C
```

---

# 8. Goal State

Character có thể có các mục tiêu đang hoạt động.

```json
{
  "goal": "help_user_finish_project",
  "priority": 0.72,
  "progress": 0.44
}
```

Goals có thể:

* được tạo;
* thay đổi priority;
* hoàn thành;
* thất bại;
* bị thay thế;
* trở nên không còn phù hợp.

Goals giúp character có hành vi chủ động thay vì chỉ phản ứng với input.

---

# 9. Session State

Session State chứa các thông tin chỉ có ý nghĩa trong phiên tương tác hiện tại.

Ví dụ:

```json
{
  "current_topic": "Mellis parser",
  "conversation_mood": "excited",
  "current_intent": "celebration",
  "turn_count": 14
}
```

Session State có thể bị reset sau khi conversation kết thúc.

---

# 10. State Lifecycle

State có lifecycle:

```text
Initial State
      ↓
Interaction
      ↓
State Analysis
      ↓
State Transition
      ↓
Active State
      ↓
Decay / Stabilization
      ↓
Next Interaction
```

Một số state thay đổi nhanh:

```text
surprise
anger
embarrassment
```

Một số state thay đổi chậm:

```text
trust
familiarity
affection
```

Một số state gần như ổn định:

```text
core personality baseline
```

---

# 11. State Persistence

Không phải toàn bộ state đều cần persist.

```text
State
├── Ephemeral
│   └── reset nhanh
│
├── Session
│   └── tồn tại trong conversation
│
├── Persistent
│   └── tồn tại giữa các session
│
└── Relationship
    └── persist theo từng relationship
```

Ví dụ:

```text
surprise → ephemeral
current_topic → session
trust → persistent
relationship_affection → persistent
```

---

# 12. State Decay

Một số trạng thái phải tự giảm khi không còn được kích hoạt.

Ví dụ:

```text
User làm character vui
        ↓
joy = 0.90

sau nhiều interaction trung tính

joy = 0.72
      ↓
joy = 0.61
      ↓
joy = 0.54
```

Có thể sử dụng decay function:

```text
S(t) = S₀ × e^(-λt)
```

Trong đó `λ` phụ thuộc vào loại state.

---

# 13. State Transition

State Transition là phép biến đổi:

```text
State(t) + Event(t) → State(t+1)
```

Ví dụ:

```json
{
  "event": "user_apologized",
  "previous": {
    "trust": 0.42,
    "tension": 0.61
  },
  "transition": {
    "trust": +0.08,
    "tension": -0.19
  }
}
```

Transition có thể được thực hiện bởi:

```text
Rule Engine
Learned Model
LLM
Hybrid Controller
```

Kiến trúc ưu tiên Hybrid để kết hợp tính kiểm soát và tính linh hoạt.

---

# 14. State Consistency

State phải tuân thủ các invariant.

Ví dụ:

```text
0 ≤ emotion ≤ 1
0 ≤ trust ≤ 1
0 ≤ affection ≤ 1
```

Ngoài range constraint, hệ thống có thể có semantic constraints:

```text
trust không được tăng mạnh nếu không có sự kiện hỗ trợ.
```

Các invariant quan trọng nên được enforce bởi runtime thay vì hoàn toàn phụ thuộc vào LLM.

---

# 15. State và Personality

Personality xác định baseline.

State xác định deviation hiện tại.

```text
Personality
    ↓
Baseline
    +
Current State
    ↓
Current Behavioral Profile
```

Ví dụ:

```text
Personality:
playful = 0.8

Current State:
sadness = 0.7

Effective behavior:
playfulness = 0.35
```

Character vẫn là người vui vẻ, nhưng tình trạng hiện tại khiến nó không thể hiện sự vui vẻ như bình thường.

---

# 16. State và Memory

Memory có thể gây ra State Transition.

```text
Memory:
User từng phản bội lời hứa.

Current interaction:
User tiếp tục hứa.

        ↓

Memory activation
        ↓
trust - 0.12
tension + 0.18
```

Ngược lại, State cũng có thể quyết định memory nào được ưu tiên.

```text
Current state:
sadness = high

        ↓

Retrieve:
negative / comforting memories
```

---

# 17. State và Context

State là một trong những nguồn dữ liệu quan trọng nhất của Context Manager.

```text
State
 ↓
Context Builder
 ↓
LLM
```

Ví dụ:

```text
Current Emotion:
happy = 0.81

Current Behavior:
playful = 0.77

Current Relationship:
trust = 0.83
```

Context Manager chỉ đưa phần state cần thiết vào context của LLM.

---

# 18. Mục tiêu thiết kế

State System phải đảm bảo:

* Có thể thay đổi.
* Có thể persist.
* Có lifecycle.
* Có decay.
* Có invariant.
* Có khả năng phản ứng với interaction.
* Không phụ thuộc trực tiếp vào LLM.
* Có thể được điều khiển bởi learned model.
* Cho phép mỗi character phát triển trạng thái riêng.

Mục tiêu cuối cùng là tạo ra một **dynamic internal state** giúp VirtualCharacter duy trì hành vi nhất quán nhưng không cứng nhắc.


=========================================
# FILE: storage.md
=========================================


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
