pub mod budget;
pub mod builder;
pub mod item;
pub mod prioritizer;

pub use budget::ContextBudget;
pub use builder::ContextBuilder;
pub use item::{estimate_tokens, ContextItem, ContextPriority, ContextSource};
pub use prioritizer::{ContextBreakdown, ContextPrioritizer};

use serde::{Deserialize, Serialize};

/// The assembled, budget-governed snapshot of character reality for a single LLM interaction.
///
/// Under Skill 16 (Context Engineering):
/// - Context represents the specific subset of memories, state, personality, and history relevant right now.
/// - Prioritized deterministically: `Critical` > `High` > `Medium` > `Low`.
/// - Strictly enforces token budget limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub items: Vec<ContextItem>,
    pub total_tokens: usize,
    pub budget: ContextBudget,
    pub breakdown: ContextBreakdown,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            total_tokens: 0,
            budget: ContextBudget::default(),
            breakdown: ContextBreakdown::default(),
        }
    }
}

impl Context {
    /// Creates a Context from a list of items using standard 4K budget.
    pub fn new(items: Vec<ContextItem>) -> Self {
        let budget = ContextBudget::standard_4k();
        ContextPrioritizer::prioritize(items, budget)
    }

    /// Renders the structured context items into a clean, hierarchical prompt string for the LLM.
    pub fn render_for_llm(&self) -> String {
        let mut sections = Vec::new();

        let mut system_items = Vec::new();
        let mut persona_items = Vec::new();
        let mut state_items = Vec::new();
        let mut rel_items = Vec::new();
        let mut memory_items = Vec::new();
        let mut conv_items = Vec::new();
        let mut user_items = Vec::new();
        let mut other_items = Vec::new();

        for item in &self.items {
            match item.source {
                ContextSource::System => system_items.push(&item.content),
                ContextSource::Personality => persona_items.push(&item.content),
                ContextSource::State => state_items.push(&item.content),
                ContextSource::Relationship => rel_items.push(&item.content),
                ContextSource::Memory => memory_items.push(&item.content),
                ContextSource::Conversation => conv_items.push(&item.content),
                ContextSource::User => user_items.push(&item.content),
                _ => other_items.push(&item.content),
            }
        }

        if !system_items.is_empty() {
            sections.push(format!("[Chỉ Thị Cốt Lõi Hệ Thống]:\n{}", system_items.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")));
        }

        if !persona_items.is_empty() {
            sections.push(format!("[Bản Sắc & Tính Cách Cốt Lõi]:\n{}", persona_items.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")));
        }

        if !state_items.is_empty() {
            sections.push(format!("[Trạng Thái Cảm Xúc & Nhận Thức]:\n{}", state_items.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")));
        }

        if !rel_items.is_empty() {
            sections.push(format!("[Mối Quan Hệ & Gắn Kết]:\n{}", rel_items.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")));
        }

        if !memory_items.is_empty() {
            sections.push(format!("[Ký Ức Liên Quan Gần Đây]:\n{}", memory_items.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")));
        }

        if !conv_items.is_empty() {
            sections.push(format!("[Lịch Sử Hội Thoại Gần Nhất]:\n{}", conv_items.iter().map(|s| format!("* {}", s)).collect::<Vec<_>>().join("\n")));
        }

        if !other_items.is_empty() {
            sections.push(format!("[Bối Cảnh Bổ Sung]:\n{}", other_items.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")));
        }

        if !user_items.is_empty() {
            sections.push(format!("[Lời Nhắn Từ Người Bạn]:\n{}", user_items.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")));
        }

        sections.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Memory, MemoryImportance};
    use crate::personality::Personality;
    use crate::relationship::Relationship;
    use crate::state::CharacterState;

    #[test]
    fn test_full_context_builder_pipeline() {
        let personality = Personality::baseline_aria();
        let state = CharacterState::default_aria();
        let rel = Relationship::new_companion(crate::character::CharacterId(personality.id.0), "user-1");
        let memories = vec![
            Memory::new_core("Core awakened memory", 1000),
            Memory::new_semantic("User likes clean code", MemoryImportance::High, Some("user-1".into()), 1001),
        ];

        let context = ContextBuilder::new()
            .with_system_directive("You are Aria, a living virtual companion.")
            .with_personality(personality)
            .with_state(state)
            .with_relationship(rel)
            .with_memories(memories)
            .with_user_input("Chào Aria, hôm nay bạn thấy thế nào?")
            .build(ContextBudget::standard_4k());

        assert!(!context.items.is_empty());
        assert!(context.total_tokens > 50);
        assert!(context.breakdown.total_used > 50);
        assert_eq!(context.breakdown.dropped_items_count, 0);

        let rendered = context.render_for_llm();
        assert!(rendered.contains("You are Aria"));
        assert!(rendered.contains("Chào Aria"));
        assert!(rendered.contains("Ký Ức Liên Quan Gần Đây"));
    }

    #[test]
    fn test_context_serialization_roundtrip() {
        let items = vec![
            ContextItem::new(ContextSource::User, "Hello", ContextPriority::Critical),
        ];
        let ctx = Context::new(items);

        let json = serde_json::to_string(&ctx).expect("Serialization should succeed");
        let deserialized: Context = serde_json::from_str(&json).expect("Deserialization should succeed");

        assert_eq!(ctx.items.len(), deserialized.items.len());
        assert_eq!(ctx.total_tokens, deserialized.total_tokens);
    }
}
