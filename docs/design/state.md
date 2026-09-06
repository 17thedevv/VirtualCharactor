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
