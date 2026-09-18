use serde::{Deserialize, Serialize};

/// Token budget definition and category reserves under Skill 16 (Context Engineering).
///
/// Ensures the LLM working context is strictly bounded to prevent context overflow,
/// degraded attention, or ballooning API latencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBudget {
    pub total_tokens: usize,
    pub system_reserve: usize,
    pub personality_reserve: usize,
    pub state_reserve: usize,
    pub relationship_reserve: usize,
    pub memory_reserve: usize,
    pub dialogue_reserve: usize,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self::standard_4k()
    }
}

impl ContextBudget {
    /// Standard 4096 token working window budget
    pub fn standard_4k() -> Self {
        Self {
            total_tokens: 4096,
            system_reserve: 400,
            personality_reserve: 600,
            state_reserve: 300,
            relationship_reserve: 300,
            memory_reserve: 1000,
            dialogue_reserve: 1496,
        }
    }

    /// Compact 2048 token working window budget
    pub fn compact_2k() -> Self {
        Self {
            total_tokens: 2048,
            system_reserve: 300,
            personality_reserve: 400,
            state_reserve: 200,
            relationship_reserve: 200,
            memory_reserve: 500,
            dialogue_reserve: 448,
        }
    }

    pub fn new(total_tokens: usize) -> Self {
        // Automatically allocate reserves proportionally
        let system = (total_tokens as f32 * 0.10).round() as usize;
        let personality = (total_tokens as f32 * 0.15).round() as usize;
        let state = (total_tokens as f32 * 0.08).round() as usize;
        let relationship = (total_tokens as f32 * 0.08).round() as usize;
        let memory = (total_tokens as f32 * 0.25).round() as usize;
        let dialogue = total_tokens.saturating_sub(system + personality + state + relationship + memory);

        Self {
            total_tokens,
            system_reserve: system,
            personality_reserve: personality,
            state_reserve: state,
            relationship_reserve: relationship,
            memory_reserve: memory,
            dialogue_reserve: dialogue,
        }
    }

    /// Check if adding item_tokens would fit within the total budget
    pub fn can_fit(&self, current_used: usize, item_tokens: usize) -> bool {
        current_used + item_tokens <= self.total_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_creation_and_fit() {
        let budget = ContextBudget::standard_4k();
        assert_eq!(budget.total_tokens, 4096);
        assert!(budget.can_fit(2000, 500));
        assert!(!budget.can_fit(4000, 200));
    }

    #[test]
    fn test_compact_budget_proportions() {
        let budget = ContextBudget::compact_2k();
        assert_eq!(budget.total_tokens, 2048);
        assert!(budget.dialogue_reserve > 300);
    }
}
