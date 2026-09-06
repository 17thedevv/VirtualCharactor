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
