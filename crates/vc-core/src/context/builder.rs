use super::budget::ContextBudget;
use super::item::{ContextItem, ContextPriority, ContextSource};
use super::prioritizer::ContextPrioritizer;
use super::Context;
use crate::memory::{Memory, MemoryImportance};
use crate::personality::Personality;
use crate::relationship::Relationship;
use crate::state::CharacterState;

/// Fluent builder for gathering multi-source character information and assembling
/// a budget-governed, prioritized Context snapshot.
#[derive(Debug, Default, Clone)]
pub struct ContextBuilder {
    system_directives: Vec<String>,
    personality: Option<Personality>,
    state: Option<CharacterState>,
    relationship: Option<Relationship>,
    memories: Vec<Memory>,
    dialogue_turns: Vec<(String, String)>,
    user_input: Option<String>,
    situation: Option<String>,
    custom_items: Vec<ContextItem>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system_directive(mut self, directive: impl Into<String>) -> Self {
        self.system_directives.push(directive.into());
        self
    }

    pub fn with_personality(mut self, personality: Personality) -> Self {
        self.personality = Some(personality);
        self
    }

    pub fn with_state(mut self, state: CharacterState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_relationship(mut self, relationship: Relationship) -> Self {
        self.relationship = Some(relationship);
        self
    }

    pub fn with_memories(mut self, memories: impl IntoIterator<Item = Memory>) -> Self {
        self.memories.extend(memories);
        self
    }

    pub fn with_user_input(mut self, user_input: impl Into<String>) -> Self {
        self.user_input = Some(user_input.into());
        self
    }

    pub fn with_dialogue_turn(mut self, sender: impl Into<String>, text: impl Into<String>) -> Self {
        self.dialogue_turns.push((sender.into(), text.into()));
        self
    }

    pub fn with_situation(mut self, situation: impl Into<String>) -> Self {
        self.situation = Some(situation.into());
        self
    }

    pub fn with_custom_item(mut self, item: ContextItem) -> Self {
        self.custom_items.push(item);
        self
    }

    /// Converts all collected inputs into raw discrete ContextItems with initial priority assignments.
    pub fn build_raw_items(&self) -> Vec<ContextItem> {
        let mut items = Vec::new();

        // 1. System Directives (Always Critical)
        for directive in &self.system_directives {
            items.push(ContextItem::new(
                ContextSource::System,
                directive.clone(),
                ContextPriority::Critical,
            ));
        }

        // 2. Personality Identity (Critical) & Traits (High)
        if let Some(ref p) = self.personality {
            items.push(
                ContextItem::new(
                    ContextSource::Personality,
                    format!("Tên: {} — {}", p.identity.name, p.identity.core_identity),
                    ContextPriority::Critical,
                )
                .with_source_id(p.id.0.to_string()),
            );

            items.push(
                ContextItem::new(
                    ContextSource::Personality,
                    format!(
                        "Nét tính cách nổi bật: Tò mò: {:.0}%, Thấu cảm: {:.0}%, Hóm hỉnh: {:.0}%, Kiên nhẫn: {:.0}%",
                        p.traits.curiosity.value() * 100.0,
                        p.traits.empathy.value() * 100.0,
                        p.traits.playfulness.value() * 100.0,
                        p.traits.patience.value() * 100.0,
                    ),
                    ContextPriority::High,
                )
                .with_source_id(p.id.0.to_string()),
            );

            if !p.communication_style.quirks.is_empty() {
                items.push(
                    ContextItem::new(
                        ContextSource::Personality,
                        format!("Thói quen giao tiếp: {}", p.communication_style.quirks.join("; ")),
                        ContextPriority::Medium,
                    )
                    .with_source_id(p.id.0.to_string()),
                );
            }
        }

        // 3. User Input (Critical - must never be dropped)
        if let Some(ref input) = self.user_input {
            items.push(ContextItem::new(
                ContextSource::User,
                format!("Lời nhắn của người dùng: \"{}\"", input),
                ContextPriority::Critical,
            ));
        }

        // 4. Character State (Dominant Emotion is High, details are Medium)
        if let Some(ref s) = self.state {
            let (dominant, intensity) = s.emotion.dominant_emotion();
            items.push(ContextItem::new(
                ContextSource::State,
                format!(
                    "Cảm xúc chủ đạo: {} ({:.0}%), Sắc thái: Valence {:.2}, Arousal {:.2}",
                    dominant.name(),
                    intensity.value() * 100.0,
                    s.emotion.valence(),
                    s.emotion.arousal()
                ),
                ContextPriority::High,
            ));

            items.push(ContextItem::new(
                ContextSource::State,
                format!(
                    "Nhận thức & Hành vi: Chú ý {:.0}%, Hiếu kỳ {:.0}%, Tập trung: {}",
                    s.cognition.attention.value() * 100.0,
                    s.cognition.curiosity.value() * 100.0,
                    s.cognition.current_topic.as_deref().unwrap_or("Hội thoại tự nhiên")
                ),
                ContextPriority::Medium,
            ));
        }

        // 5. Relationship (Stage & Metrics are High, known facts are Medium)
        if let Some(ref r) = self.relationship {
            items.push(
                ContextItem::new(
                    ContextSource::Relationship,
                    format!(
                        "Giai đoạn quan hệ: {} | Tin cậy: {:.0}%, Gần gũi: {:.0}%, Thiện cảm: {:.0}%",
                        r.state.stage.as_str(),
                        r.state.trust * 100.0,
                        r.state.closeness * 100.0,
                        r.state.affection * 100.0,
                    ),
                    ContextPriority::High,
                )
                .with_source_id(r.target_id.clone()),
            );

            if !r.state.known_facts.is_empty() {
                items.push(
                    ContextItem::new(
                        ContextSource::Relationship,
                        format!("Thông tin đã biết về người bạn: {}", r.state.known_facts.join("; ")),
                        ContextPriority::Medium,
                    )
                    .with_source_id(r.target_id.clone()),
                );
            }
        }

        // 6. Retrieved Memories (Priority mapped from MemoryImportance)
        for m in &self.memories {
            let priority = match m.metadata.importance {
                MemoryImportance::Critical => ContextPriority::Critical,
                MemoryImportance::High => ContextPriority::High,
                MemoryImportance::Medium => ContextPriority::Medium,
                MemoryImportance::Low => ContextPriority::Low,
            };

            items.push(
                ContextItem::new(
                    ContextSource::Memory,
                    format!("[Ký ức {:?}]: {}", m.metadata.memory_type, m.content),
                    priority,
                )
                .with_source_id(m.id.0.to_string()),
            );
        }

        // 7. Recent Dialogue Turns
        let turns_len = self.dialogue_turns.len();
        for (idx, (sender, text)) in self.dialogue_turns.iter().enumerate() {
            let is_latest = idx + 1 == turns_len;
            let priority = if is_latest {
                ContextPriority::High
            } else if idx + 3 >= turns_len {
                ContextPriority::Medium
            } else {
                ContextPriority::Low
            };

            items.push(ContextItem::new(
                ContextSource::Conversation,
                format!("{}: {}", sender, text),
                priority,
            ));
        }

        // 8. Situation (High)
        if let Some(ref sit) = self.situation {
            items.push(ContextItem::new(
                ContextSource::Situation,
                format!("Tình huống hiện tại: {}", sit),
                ContextPriority::High,
            ));
        }

        // 9. Custom Items
        items.extend(self.custom_items.clone());

        items
    }

    /// Assembles raw items and applies budget-constrained prioritization.
    pub fn build(self, budget: ContextBudget) -> Context {
        let raw_items = self.build_raw_items();
        ContextPrioritizer::prioritize(raw_items, budget)
    }
}
