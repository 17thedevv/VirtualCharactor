use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::types::MemoryType;

/// Unique identifier for a single Memory instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub Uuid);

impl MemoryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MemoryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MemoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Structured query for retrieving contextually relevant memories without blowing the token budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// Natural language keyword or question to match against memory content and tags
    pub query_text: Option<String>,
    /// Actor scope filter: Strictly restricts private memories to this Actor (Skill 12 Rule).
    /// If None, only shared autobiographical character memories are returned.
    pub actor_id: Option<String>,
    /// Optional filter by memory category
    pub memory_type: Option<MemoryType>,
    /// Minimum threshold of effective importance [0.0, 1.0]
    pub min_importance: Option<f32>,
    /// Maximum number of memory items to return (budget constraint)
    pub limit: usize,
    /// Weight given to keyword/semantic match [0, 1]
    pub semantic_weight: f32,
    /// Weight given to temporal recency [0, 1]
    pub recency_weight: f32,
    /// Weight given to importance score [0, 1]
    pub importance_weight: f32,
}

impl MemoryQuery {
    pub fn new(limit: usize) -> Self {
        Self {
            query_text: None,
            actor_id: None,
            memory_type: None,
            min_importance: None,
            limit: limit.max(1),
            semantic_weight: 0.5,
            recency_weight: 0.2,
            importance_weight: 0.3,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.query_text = Some(text.into());
        self
    }

    pub fn with_actor(mut self, actor_id: impl Into<String>) -> Self {
        self.actor_id = Some(actor_id.into());
        self
    }

    pub fn with_type(mut self, memory_type: MemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    pub fn with_min_importance(mut self, min_importance: f32) -> Self {
        self.min_importance = Some(min_importance.clamp(0.0, 1.0));
        self
    }
}

/// Lightweight scored reference to a retrieved memory item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryReference {
    pub memory_id: MemoryId,
    pub relevance_score: f32,
    pub rationale: Option<String>,
}

impl MemoryReference {
    pub fn new(memory_id: MemoryId, relevance_score: f32) -> Self {
        Self {
            memory_id,
            relevance_score: relevance_score.clamp(0.0, 1.0),
            rationale: None,
        }
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }
}
