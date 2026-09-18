use super::budget::ContextBudget;
use super::item::{ContextItem, ContextSource};
use super::Context;
use serde::{Deserialize, Serialize};

/// Detailed breakdown of token utilization across categories for observability.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBreakdown {
    pub total_budget: usize,
    pub total_used: usize,
    pub personality_tokens: usize,
    pub state_tokens: usize,
    pub relationship_tokens: usize,
    pub memory_tokens: usize,
    pub dialogue_tokens: usize,
    pub system_tokens: usize,
    pub user_tokens: usize,
    pub dropped_items_count: usize,
}

/// Selector and prioritizer enforcing budget constraints under Skill 16 (Context Engineering).
pub struct ContextPrioritizer;

impl ContextPrioritizer {
    /// Selects items by priority order (Critical > High > Medium > Low) until the token budget is reached.
    ///
    /// Invariant: Items with `Critical` priority are NEVER evicted.
    pub fn prioritize(mut items: Vec<ContextItem>, budget: ContextBudget) -> Context {
        // Sort items by priority descending: Critical (4) > High (3) > Medium (2) > Low (1)
        items.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut selected_items = Vec::new();
        let mut current_tokens = 0;
        let mut dropped_count = 0;

        // First pass: Always retain Critical items
        let mut non_critical = Vec::new();
        for item in items {
            if item.priority.is_critical() {
                current_tokens += item.tokens;
                selected_items.push(item);
            } else {
                non_critical.push(item);
            }
        }

        // Second pass: Fit non-critical items within remaining budget (High -> Medium -> Low)
        for item in non_critical {
            if budget.can_fit(current_tokens, item.tokens) {
                current_tokens += item.tokens;
                selected_items.push(item);
            } else {
                dropped_count += 1;
            }
        }

        // Compute category token breakdown
        let mut breakdown = ContextBreakdown {
            total_budget: budget.total_tokens,
            total_used: current_tokens,
            dropped_items_count: dropped_count,
            ..Default::default()
        };

        for item in &selected_items {
            match item.source {
                ContextSource::Personality => breakdown.personality_tokens += item.tokens,
                ContextSource::State => breakdown.state_tokens += item.tokens,
                ContextSource::Relationship => breakdown.relationship_tokens += item.tokens,
                ContextSource::Memory => breakdown.memory_tokens += item.tokens,
                ContextSource::Conversation => breakdown.dialogue_tokens += item.tokens,
                ContextSource::System => breakdown.system_tokens += item.tokens,
                ContextSource::User => breakdown.user_tokens += item.tokens,
                _ => {}
            }
        }

        Context {
            items: selected_items,
            total_tokens: current_tokens,
            budget,
            breakdown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::item::ContextPriority;

    #[test]
    fn test_prioritizer_retains_critical_and_drops_low_under_budget() {
        let items = vec![
            ContextItem::new(ContextSource::System, "Critical System", ContextPriority::Critical).with_tokens(50),
            ContextItem::new(ContextSource::User, "User query", ContextPriority::Critical).with_tokens(40),
            ContextItem::new(ContextSource::State, "Current emotion", ContextPriority::High).with_tokens(30),
            ContextItem::new(ContextSource::Memory, "Old memory", ContextPriority::Low).with_tokens(100),
        ];

        // Strict tiny budget of 130 tokens
        let budget = ContextBudget::new(130);
        let ctx = ContextPrioritizer::prioritize(items, budget);

        // Critical items (50 + 40 = 90) + High item (30) = 120 tokens.
        // Low item (100) exceeds 130 budget and must be dropped!
        assert_eq!(ctx.items.len(), 3);
        assert_eq!(ctx.total_tokens, 120);
        assert_eq!(ctx.breakdown.dropped_items_count, 1);
        assert!(ctx.items.iter().all(|i| i.priority != ContextPriority::Low));
    }
}
