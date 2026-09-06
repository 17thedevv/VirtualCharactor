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
