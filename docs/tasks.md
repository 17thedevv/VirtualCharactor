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
A1. Personality

Xây Personality thành domain model thực sự.

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
A2. State + Emotion

Đây là phần quan trọng.

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

A3. Relationship

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

A4. Memory

Đây sẽ là task lớn nhất của Dev A.

Phase 1 chỉ làm foundation + deterministic lifecycle, chưa làm embedding/vector DB.

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
A5. Decision

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
B1. Context

Đây là phần quan trọng nhất của Dev B.

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

B2. Runtime

Xây interaction lifecycle.

Ví dụ:

CharacterSession
Interaction
Runtime

Pipeline ban đầu:

User Input
     ↓
Runtime
     ↓
Load Character
     ↓
Read State
     ↓
Retrieve Memory
     ↓
Build Context
     ↓
Decision
     ↓
LLM
     ↓
Response

Sau này mới thêm:

Outcome
 ↓
State Update
 ↓
Memory Update
B3. LLM

Phụ trách:

LlmProvider
LlmRequest
LlmResponse

Implementation:

MockLlmProvider
GeminiProvider

Phase 1:

Mock

Phải hoàn chỉnh để integration test không phụ thuộc network.

Gemini

Có thể implement basic generation:

Context
 ↓
LlmRequest
 ↓
Gemini
 ↓
LlmResponse

Nhưng Gemini-specific types chỉ được nằm:

vc-llm::gemini

Không được leak vào:

vc-core
vc-runtime domain types
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