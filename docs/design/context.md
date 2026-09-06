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
