use serde::{Deserialize, Serialize};
use super::types::{MemoryImportance, MemoryType};

/// Rich metadata annotating a single memory item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// Qualitative importance level of the memory
    pub importance: MemoryImportance,
    /// Type classification (Episodic, Semantic, Procedural, Relationship)
    pub memory_type: MemoryType,
    /// Emotional valence associated with the event [-1.0 (very negative) to 1.0 (very positive)]
    pub emotional_valence: f32,
    /// Semantic search tags or topic keywords
    pub tags: Vec<String>,
    /// Actor scope: If Some(id), this memory is strictly private to this user (Skill 12 Rule).
    /// If None, this is a character-autobiographical or global memory.
    pub source_actor_id: Option<String>,
    /// Epoch timestamp when the memory was initially formed
    pub timestamp: u64,
}

impl MemoryMetadata {
    pub fn new(
        memory_type: MemoryType,
        importance: MemoryImportance,
        source_actor_id: Option<String>,
        timestamp: u64,
    ) -> Self {
        Self {
            importance,
            memory_type,
            emotional_valence: 0.0,
            tags: Vec::new(),
            source_actor_id,
            timestamp,
        }
    }

    /// Add a tag if not already present.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let t = tag.into().trim().to_lowercase();
        if !t.is_empty() && !self.tags.iter().any(|existing| existing == &t) {
            self.tags.push(t);
        }
        self
    }

    /// Set emotional valence clamped safely between [-1.0, 1.0].
    pub fn with_valence(mut self, valence: f32) -> Self {
        self.emotional_valence = if valence.is_nan() { 0.0 } else { valence.clamp(-1.0, 1.0) };
        self
    }
}
