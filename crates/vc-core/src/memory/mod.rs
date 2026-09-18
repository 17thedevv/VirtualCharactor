pub mod lifecycle;
pub mod metadata;
pub mod query;
pub mod traits;
pub mod types;

use serde::{Deserialize, Serialize};

use crate::character::CharacterId;
use crate::error::{CoreError, Result};

pub use lifecycle::MemoryLifecycle;
pub use metadata::MemoryMetadata;
pub use query::{MemoryId, MemoryQuery, MemoryReference};
pub use traits::{MemoryFormation, MemoryRanking, MemoryRetriever};
pub use types::{MemoryImportance, MemoryType};

/// Rich Memory aggregate in the VirtualCharacter cognitive domain.
///
/// Under Skill 12 (Memory Engineering), memory is retained information with semantic lifecycle,
/// importance evaluation, and strict actor privacy isolation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    pub id: MemoryId,
    /// Optional character identity this memory belongs to
    pub character_id: Option<CharacterId>,
    /// Natural language representation of the remembered information
    pub content: String,
    /// Multi-dimensional classification and metadata
    pub metadata: MemoryMetadata,
    /// Temporal statistics, reinforcement, and decay dynamics
    pub lifecycle: MemoryLifecycle,
}

impl Memory {
    /// Create a new memory instance with full explicit fields.
    pub fn new(
        id: MemoryId,
        character_id: Option<CharacterId>,
        content: impl Into<String>,
        metadata: MemoryMetadata,
        lifecycle: MemoryLifecycle,
    ) -> Self {
        Self {
            id,
            character_id,
            content: content.into(),
            metadata,
            lifecycle,
        }
    }

    /// Helper constructor for creating an Episodic memory.
    pub fn new_episodic(
        content: impl Into<String>,
        importance: MemoryImportance,
        source_actor_id: Option<String>,
        now: u64,
    ) -> Self {
        let metadata = MemoryMetadata::new(MemoryType::Episodic, importance, source_actor_id, now);
        let lifecycle = MemoryLifecycle::new(now, importance == MemoryImportance::Critical);
        Self::new(MemoryId::new(), None, content, metadata, lifecycle)
    }

    /// Helper constructor for creating a Semantic knowledge memory.
    pub fn new_semantic(
        content: impl Into<String>,
        importance: MemoryImportance,
        source_actor_id: Option<String>,
        now: u64,
    ) -> Self {
        let metadata = MemoryMetadata::new(MemoryType::Semantic, importance, source_actor_id, now);
        let lifecycle = MemoryLifecycle::new(now, importance == MemoryImportance::Critical);
        Self::new(MemoryId::new(), None, content, metadata, lifecycle)
    }

    /// Helper constructor for creating a core identity autobiographical memory (pinned & critical).
    pub fn new_core(content: impl Into<String>, now: u64) -> Self {
        let metadata = MemoryMetadata::new(MemoryType::Semantic, MemoryImportance::Critical, None, now);
        let lifecycle = MemoryLifecycle::new(now, true);
        Self::new(MemoryId::new(), None, content, metadata, lifecycle)
    }

    /// Touch / access this memory, updating statistics and reinforcing retention strength.
    pub fn touch(&mut self, now: u64) {
        self.lifecycle.touch(now);
    }

    /// Apply temporal decay, respecting pinning and critical importance protection.
    pub fn apply_decay(&mut self, elapsed_secs: u64) {
        let is_critical = self.metadata.importance == MemoryImportance::Critical;
        self.lifecycle.apply_decay(elapsed_secs, is_critical);
    }

    /// Calculate effective dynamic importance modulated by retention strength.
    pub fn effective_importance(&self) -> f32 {
        self.lifecycle.effective_importance(&self.metadata.importance)
    }

    /// Security Rule (Skill 12): Verify if an actor is authorized to retrieve this memory.
    ///
    /// - Memories with `source_actor_id = None` are global/autobiographical (accessible by all).
    /// - Memories with `source_actor_id = Some(id)` are strictly accessible only by that matching Actor.
    pub fn can_be_retrieved_by(&self, querying_actor: Option<&str>) -> bool {
        match &self.metadata.source_actor_id {
            None => true,
            Some(owner) => match querying_actor {
                Some(caller) => owner == caller,
                None => false,
            },
        }
    }

    /// Validate domain invariants of this memory item.
    pub fn validate(&self) -> Result<()> {
        if self.content.trim().is_empty() {
            return Err(CoreError::ValidationError("Memory content cannot be empty".into()));
        }
        if self.metadata.emotional_valence < -1.0 || self.metadata.emotional_valence > 1.0 {
            return Err(CoreError::ValidationError("Emotional valence must be in [-1.0, 1.0]".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation_and_validation() {
        let mem = Memory::new_episodic("User fixed compiler bug", MemoryImportance::High, Some("user-1".into()), 1000);
        assert_eq!(mem.metadata.memory_type, MemoryType::Episodic);
        assert_eq!(mem.metadata.importance, MemoryImportance::High);
        assert!(mem.validate().is_ok());

        let invalid = Memory::new_episodic("   ", MemoryImportance::Low, None, 1000);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_strict_actor_isolation() {
        let shared_mem = Memory::new_core("Awakened in Rust runtime", 1000);
        let alice_mem = Memory::new_episodic("Alice shared her secret goal", MemoryImportance::High, Some("alice".into()), 1000);
        let bob_mem = Memory::new_episodic("Bob discussed music", MemoryImportance::Medium, Some("bob".into()), 1000);

        // Shared memory is accessible to everyone
        assert!(shared_mem.can_be_retrieved_by(Some("alice")));
        assert!(shared_mem.can_be_retrieved_by(Some("bob")));
        assert!(shared_mem.can_be_retrieved_by(None));

        // Alice's memory can ONLY be retrieved by Alice
        assert!(alice_mem.can_be_retrieved_by(Some("alice")));
        assert!(!alice_mem.can_be_retrieved_by(Some("bob")));
        assert!(!alice_mem.can_be_retrieved_by(None));

        // Bob's memory can ONLY be retrieved by Bob
        assert!(bob_mem.can_be_retrieved_by(Some("bob")));
        assert!(!bob_mem.can_be_retrieved_by(Some("alice")));
    }

    #[test]
    fn test_decay_and_reinforcement() {
        let mut mem = Memory::new_episodic("Casual greeting", MemoryImportance::Low, None, 1000);
        assert_eq!(mem.lifecycle.access_count, 0);

        // Touch reinforces access
        mem.touch(2000);
        assert_eq!(mem.lifecycle.access_count, 1);
        assert_eq!(mem.lifecycle.last_accessed_at, 2000);

        // Decay degrades strength for low importance
        mem.apply_decay(86400 * 30); // 30 days
        assert!(mem.effective_importance() < MemoryImportance::Low.weight());

        // Core memory remains protected
        let mut core = Memory::new_core("Aria Identity", 1000);
        core.apply_decay(86400 * 365); // 1 year
        assert_eq!(core.effective_importance(), 1.0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mem = Memory::new_semantic("Loves clean software architecture", MemoryImportance::Critical, Some("dev-1".into()), 5000);
        let json = serde_json::to_string(&mem).expect("Serialization failed");
        let deserialized: Memory = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(mem.id, deserialized.id);
        assert_eq!(mem.content, deserialized.content);
        assert_eq!(mem.metadata.importance, deserialized.metadata.importance);
        assert_eq!(mem.metadata.source_actor_id, deserialized.metadata.source_actor_id);
    }
}
