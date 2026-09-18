use serde::{Deserialize, Serialize};

/// Priority tier for context items under Skill 16 (Context Engineering).
///
/// Deterministic sorting rule:
/// `Critical` (never evicted) > `High` > `Medium` > `Low` (evicted first when budget exceeded).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ContextPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl ContextPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn is_critical(&self) -> bool {
        matches!(self, Self::Critical)
    }
}

/// Category/origin of a context item for tracing and observability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextSource {
    /// Core system directives, safety bounds, persona instructions
    System,
    /// User input or prompt
    User,
    /// Stable traits and identity values
    Personality,
    /// Dynamic emotion and cognitive state
    State,
    /// Actor-specific relationship metrics and stage
    Relationship,
    /// Retrieved relevant memories (episodic/semantic)
    Memory,
    /// Recent dialogue turns
    Conversation,
    /// Environmental situation or active scene
    Situation,
    /// Ambient environment or simulator metadata
    Environment,
    /// Custom extension source
    Custom(String),
}

impl ContextSource {
    pub fn as_str(&self) -> &str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Personality => "personality",
            Self::State => "state",
            Self::Relationship => "relationship",
            Self::Memory => "memory",
            Self::Conversation => "conversation",
            Self::Situation => "situation",
            Self::Environment => "environment",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for ContextSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A discrete unit of information assembled into the LLM context snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextItem {
    pub source: ContextSource,
    pub content: String,
    pub priority: ContextPriority,
    pub tokens: usize,
    /// Optional identifier linking back to domain origin (e.g. MemoryId, ActorId, RuleId)
    pub source_id: Option<String>,
}

impl ContextItem {
    pub fn new(source: ContextSource, content: impl Into<String>, priority: ContextPriority) -> Self {
        let content_str = content.into();
        let tokens = estimate_tokens(&content_str);
        Self {
            source,
            content: content_str,
            priority,
            tokens,
            source_id: None,
        }
    }

    pub fn with_tokens(mut self, tokens: usize) -> Self {
        self.tokens = tokens;
        self
    }

    pub fn with_source_id(mut self, id: impl Into<String>) -> Self {
        self.source_id = Some(id.into());
        self
    }
}

/// Heuristic token estimator for fast deterministic budget calculations.
/// Approximately 4 characters per token for English / 2.5 characters per token for Vietnamese.
pub fn estimate_tokens(text: &str) -> usize {
    let char_count = text.chars().count();
    if char_count == 0 {
        return 0;
    }
    // Blend estimate: 1 token ~ 3.5 chars, with a minimum of 1
    std::cmp::max(1, (char_count as f32 / 3.5).ceil() as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(ContextPriority::Critical > ContextPriority::High);
        assert!(ContextPriority::High > ContextPriority::Medium);
        assert!(ContextPriority::Medium > ContextPriority::Low);
        assert!(ContextPriority::Critical.is_critical());
        assert!(!ContextPriority::High.is_critical());
    }

    #[test]
    fn test_item_creation_and_token_estimation() {
        let item = ContextItem::new(
            ContextSource::Personality,
            "Aria là người bạn đồng hành ảo ấm áp, tò mò.",
            ContextPriority::High,
        )
        .with_source_id("aria-core-id");

        assert_eq!(item.source, ContextSource::Personality);
        assert_eq!(item.priority, ContextPriority::High);
        assert!(item.tokens > 5);
        assert_eq!(item.source_id.as_deref(), Some("aria-core-id"));
    }
}
