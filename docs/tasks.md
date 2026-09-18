Phase 1 — Parallel Development
                    VirtualCharacter
                           │
          ┌────────────────┴────────────────┐
          │                                 │
       DEV A                            DEV B
  Character Core                  Runtime / Infrastructure
          │                                 │
          ↓                                 ↓
 Personality                         Context
 State + Emotion                     Runtime
 Relationship                        LLM
 Memory                              Storage
 Decision
          │                                 │
          └──────────────┬──────────────────┘
                         ↓
                    Integration
DEV A — Character Intelligence

Ownership chính:

crates/vc-core/
├── personality/
├── state/
├── relationship/
├── memory/
└── decision/
A1. Personality [DONE]

Xây Personality thành domain model thực sự (Đã hoàn tất theo docs/design/personality.md).

Phụ trách:

Identity
Traits
Values
Preferences
BehaviorTendencies
CommunicationStyle
DecisionTendencies
Boundaries
Goals

Cần có:

validation
default/baseline
serialization
equality/comparison nếu cần
unit tests

Chưa làm:

prompt generation
LLM personality extraction
personality learning
A2. State + Emotion [DONE]

Đây là phần quan trọng (Đã hoàn tất toàn bộ State Transition, Multi-Axis Emotion, RuleEmotionEngine và đồng bộ Web/CLI).

state/
├── mod.rs
├── emotion.rs
├── cognitive.rs
├── behavior.rs
├── goals.rs
└── session.rs

Phụ trách:

EmotionState
CognitiveState
BehaviorState
Goals
SessionState
CharacterState

Đặc biệt thiết kế:

State Transition
State validation
State mutation
State snapshot
State persistence representation

Emotion cần có nền tảng để sau này thêm:

EmotionEngine
RuleBased
Learned
Hybrid

Nhưng Phase 1 chưa cần ML.

A3. Relationship [DONE]

(Đã hoàn tất toàn bộ Bipartite Relationship, Multi-Actor Isolation, 5 Metrics [closeness, trust, familiarity, affection, tension], Stage Progression, Relationship Transition & Damping, và đồng bộ Web HUD).

Phụ trách:

Relationship
RelationshipId
RelationshipState

và các thuộc tính kiểu:

closeness
trust
familiarity
affection
tension
known_facts / relationship-scoped knowledge

Quan trọng:

CharacterState ≠ RelationshipState

Relationship phải hỗ trợ:

Character A ↔ User X
Character A ↔ User Y

một cách độc lập.

A4. Memory [DONE]

(Đã hoàn tất toàn bộ Domain Memory Model, 4 Tầng Bộ Nhớ, Lifecycle Decay & Reinforcement, MemoryQuery & Actor Isolation, InMemoryMemoryStore và tích hợp Server/Web).

Phụ trách:

Memory
MemoryType
MemoryMetadata
MemoryQuery
MemoryReference
MemoryImportance

Và:

Memory creation
Memory update
Memory retrieval interface
Memory importance
Recency
Memory lifecycle

Nên thiết kế sẵn abstraction cho:

MemoryFormation
MemoryRetrieval
MemoryRanking
MemoryConsolidation
MemoryDecay
MemoryConflictResolution

nhưng chưa cần implementation thông minh.

Ví dụ:

pub trait MemoryRetriever {
    fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<Memory>>;
}
A5. Decision [DONE]

(Đã hoàn tất toàn bộ Decision Domain Module, Action Types, DecisionCandidate, Scoring Heuristics, Inner Monologue Reasoning, BehaviorPolicy, RuleDecisionEngine và tích hợp Server/Web/CLI).

Phụ trách:

Decision
Action
DecisionCandidate
DecisionResult
DecisionEngine

Phase 1 làm deterministic baseline.

Ví dụ:

Input
 ↓
Situation
 ↓
Generate candidates
 ↓
score
 ↓
select

Chưa làm learned model.

Quan trọng nhất là Decision phải trả lời:

"Character nên làm gì?"

chứ không trả về câu text.

Dev A — Definition of Done

Cuối phần A phải có:

✓ Personality domain hoàn chỉnh
✓ State domain hoàn chỉnh
✓ Emotion domain hoàn chỉnh
✓ Relationship domain hoàn chỉnh
✓ Memory foundation
✓ Deterministic DecisionEngine
✓ Unit tests
✓ Domain invariants
✓ Không dependency vào Gemini/DB/runtime
DEV B — Runtime / Context / LLM / Storage

Ownership chính:

crates/vc-core/context/
crates/vc-runtime/
crates/vc-llm/
crates/vc-storage/
B1. Context [DONE]

(Đã hoàn tất toàn bộ Context Module, ContextItem, ContextSource, ContextPriority, ContextBudget, ContextBuilder, ContextPrioritizer, tích hợp WebSocket Live Token Breakdown & Mind Inspector).

Phụ trách:

Context
ContextItem
ContextSource
ContextPriority
ContextBudget

Xây:

ContextBuilder
ContextSelector
ContextPrioritizer

Pipeline:

Personality
State
Relationship
Memory
Conversation
Current Situation
       ↓
Context Builder
       ↓
Optimized Context

Phase 1 chưa cần context algorithm quá thông minh.

Chỉ cần deterministic:

critical > high > medium > low

và có budget.

B2. Runtime [DONE]

(Đã hoàn tất toàn bộ Interaction Lifecycle, CharacterSession & SessionManager, State Feedback, Memory Consolidation, và tích hợp Server/CLI/Web).

Phụ trách:

CharacterSession
SessionId & SessionStatus State Machine (Created, Active, Idle, Completed, Expired, Cancelled)
SessionManager (Thread-safe concurrency, idle timeout detection)
Interaction, InteractionId, InteractionStatus, InteractionOutcome
RuntimeEngine (Orchestrating the 9-stage lifecycle)

Pipeline đã hoàn thiện:

User Input
     ↓
Session Touch / Resolution (SessionManager)
     ↓
Memory Retrieval (Actor-Isolated MemoryQuery)
     ↓
Context Construction & Budget Governance (ContextBuilder)
     ↓
Decision Evaluation (DecisionEngine)
     ↓
LLM Generation (LlmProvider / Gemini / Mock)
     ↓
Emotion & State Feedback Update (apply_delta, decay, sync_behavior)
     ↓
Relationship Evolution (RelationshipTransition)
     ↓
Memory Consolidation (Episodic Memory Formation)
     ↓
InteractionOutcome
B3. LLM [DONE]

(Đã hoàn tất toàn bộ Provider Abstraction, Provider-Neutral Types, MockLlmProvider hoàn chỉnh, và GeminiProvider với Redacted Secrets, GenerationConfig, Usage Metadata & Error Mapping).

Phụ trách:

LlmProvider (Trait trừu tượng thuần túy)
LlmRequest (Builder: prompt, system_instruction, temperature, max_tokens)
LlmResponse (text, LlmUsage, finish_reason)
LlmUsage (prompt_tokens, completion_tokens, total_tokens)
LlmError (RateLimited, AuthenticationFailed, InvalidRequest, NetworkError, Timeout, ModelUnavailable, Other)

Implementation:

MockLlmProvider:
- Hoàn chỉnh cho unit & integration tests mà không phụ thuộc network hay API keys.
- Hỗ trợ hàng đợi câu trả lời kịch bản sẵn (`with_responses`, `push_canned_response`).
- Hỗ trợ mô phỏng lỗi provider (`failing`, `set_simulated_error`) để kiểm thử runtime resilience.
- Ghi nhận lịch sử requests (`recorded_requests`, `last_request`, `request_count`) cho test assertions.

GeminiProvider:
- GeminiConfig cấu hình linh hoạt (model, temperature, max_output_tokens, timeout, max_retries).
- Secrets Management: Masking API key trong format `Debug` (`AIza...[REDACTED]`).
- Hỗ trợ `generationConfig` (temperature, maxOutputTokens) gửi tới Gemini API.
- Trích xuất `usageMetadata` (`promptTokenCount`, `candidatesTokenCount`, `totalTokenCount`) vào `LlmUsage`.
- Cơ chế retry tự động cho lỗi tạm thời (429 RateLimit, 503 Service Unavailable) với exponential backoff.
- Toàn bộ kiểu dữ liệu nội bộ của Gemini được cô lập 100% bên trong `vc-llm::gemini`, tuyệt đối không rò rỉ ra `vc-core` hay `vc-runtime`.
B4. Storage

Phụ trách:

CharacterRepository
MemoryRepository
StateRepository
RelationshipRepository

trước tiên làm:

InMemory implementation

sau đó:

SQLite implementation

Phase 1 không cần tối ưu database.

Mục đích là chứng minh:

Domain
 ↕
Repository
 ↕
Storage

hoạt động.

B5. Integration

Đây là phần Dev B chịu trách nhiệm chính nhưng hai đứa cùng review.

Tạo pipeline:

CLI
 ↓
Runtime
 ↓
Core
 ↓
Mock DecisionEngine
 ↓
Mock LLM
 ↓
Response

rồi nâng lên:

Runtime
 ↓
Real DecisionEngine
 ↓
Context
 ↓
Gemini
Dev B — Definition of Done
✓ Context builder
✓ Context budget
✓ Context prioritization
✓ Runtime orchestration
✓ LlmProvider
✓ MockLlmProvider
✓ GeminiProvider
✓ Repository traits
✓ In-memory storage
✓ SQLite foundation
✓ Integration tests
✓ CLI chạy end-to-end
Quan trọng: 2 đứa KHÔNG được đụng nhau ở đâu?

Đây là ownership matrix tao khuyên ghi thẳng vào docs/development.md:

Area	Dev A	Dev B
Personality	✅	❌
State	✅	❌
Emotion	✅	❌
Relationship	✅	❌
Memory domain	✅	❌
Decision	✅	❌
Context domain types	⚠️	✅
Context builder	❌	✅
Runtime	❌	✅
LLM	❌	✅
Gemini	❌	✅
Storage	❌	✅
CLI	review	✅
Integration tests	review	✅
Architecture/docs	both	

⚠️ ở Context nghĩa là contract đã có, Dev B sở hữu implementation.

Thứ tự chạy song song

Không phải A làm hết rồi B làm.

Ngay sau Phase 0:

DAY 1
────────────────────────────
DEV A
Personality + State

DEV B
Context + LlmProvider
DAY 2
────────────────────────────
DEV A
Emotion + Relationship

DEV B
Mock LLM + Runtime
DAY 3
────────────────────────────
DEV A
Memory foundation

DEV B
Storage + Gemini
DAY 4
────────────────────────────
DEV A
Decision baseline

DEV B
Integration

Sau đó:

              Integration
                   ↓
        ┌──────────┴──────────┐
        ↓                     ↓
   Real Memory          Real Context
        ↓                     ↓
        └──────────┬──────────┘
                   ↓
              Real Decision
                   ↓
                 Gemini
Cách chia branch

Mày:

feature/personality
feature/state
feature/emotion
feature/relationship
feature/memory
feature/decision

Bạn mày:

feature/context
feature/runtime
feature/llm
feature/gemini
feature/storage
feature/integration

Nhưng không nhất thiết mỗi task một branch. Một feature tương đối lớn có thể gom thành một branch.

Ví dụ:

feature/character-state

chứa:

state
emotion
Cực kỳ quan trọng: contract freeze theo từng Phase

Có thể xảy ra trường hợp Dev A đang làm Memory và nhận ra:

MemoryQuery

thiết kế hiện tại chưa đủ.

Không được âm thầm sửa rồi push.

Quy trình:

A phát hiện vấn đề
      ↓
mở issue / ADR
      ↓
hai người review
      ↓
sửa contract
      ↓
update integration tests
      ↓
tiếp tục code
Milestone Phase 1

Tao sẽ đặt 3 mốc.

M1 — Character Core
Personality
State
Emotion
Relationship

chạy độc lập.

M2 — Cognitive Pipeline
Memory
+
Decision
+
Context

chạy với mocks.

M3 — First Real Character
User
 ↓
Memory
 ↓
State
 ↓
Decision
 ↓
Context
 ↓
Gemini
 ↓
Response

Đến M3 mới có thể nói:

VirtualCharacter đã bắt đầu hoạt động như một character, thay vì chỉ là tập domain model.

Tao sẽ chia project thành 2 "mặt trận"
                    ┌─────────────────────┐
                    │   Shared Contracts  │
                    └──────────┬──────────┘
                               │
              ┌────────────────┴────────────────┐
              ↓                                 ↓
       DEV A — Character                 DEV B — Runtime
              │                                 │
 Personality │                                 │ Context
 State       │                                 │ Runtime
 Emotion     │                                 │ LLM
 Relationship│                                 │ Gemini
 Memory      │                                 │ Storage
 Decision    │                                 │ Integration
              │                                 │
              └──────────────┬──────────────────┘
                             ↓
                       End-to-End