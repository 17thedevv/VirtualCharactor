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
