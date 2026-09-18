pub mod knowledge;
pub mod metrics;
pub mod stage;
pub mod transition;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::character::CharacterId;
use crate::error::{CoreError, Result};

pub use knowledge::RelationshipKnowledge;
pub use metrics::{RelationshipMetrics, RelationshipScore};
pub use stage::RelationshipStage;
pub use transition::RelationshipTransition;

/// Unique identifier for a relationship instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationshipId(pub Uuid);

impl RelationshipId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RelationshipId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RelationshipId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier of the external participant (User, Actor, NPC) in the relationship.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorId(pub String);

impl ActorId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ActorId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ActorId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Snapshot representation of a relationship's current state.
///
/// This provides direct fields for serialization and backward compatibility
/// while encapsulating the full multi-dimensional metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationshipState {
    pub stage: RelationshipStage,
    pub closeness: f32,
    pub trust: f32,
    pub familiarity: f32,
    pub affection: f32,
    pub tension: f32,
    pub known_facts: Vec<String>,
}

impl RelationshipState {
    pub fn from_parts(metrics: &RelationshipMetrics, knowledge: &RelationshipKnowledge) -> Self {
        Self {
            stage: RelationshipStage::from_metrics(metrics),
            closeness: metrics.closeness.value(),
            trust: metrics.trust.value(),
            familiarity: metrics.familiarity.value(),
            affection: metrics.affection.value(),
            tension: metrics.tension.value(),
            known_facts: knowledge.facts().to_vec(),
        }
    }
}

/// A Bipartite Relationship entity between a Character and an Actor.
///
/// Under Skill 14 (Relationship Engineering), a relationship is strictly isolated per
/// `(character_id, target_id)`. CharacterState does NOT store relationship metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub id: RelationshipId,
    pub character_id: CharacterId,
    pub target_id: String,
    pub relationship_type: String,
    pub metrics: RelationshipMetrics,
    pub knowledge: RelationshipKnowledge,
    pub interaction_count: u64,
    pub last_interaction_ts: u64,
    pub state: RelationshipState,
}

impl Relationship {
    /// Create a new relationship with specified initial metrics.
    pub fn new(
        character_id: CharacterId,
        target_id: impl Into<String>,
        relationship_type: impl Into<String>,
        metrics: RelationshipMetrics,
        knowledge: RelationshipKnowledge,
    ) -> Self {
        let state = RelationshipState::from_parts(&metrics, &knowledge);
        Self {
            id: RelationshipId::new(),
            character_id,
            target_id: target_id.into(),
            relationship_type: relationship_type.into(),
            metrics,
            knowledge,
            interaction_count: 0,
            last_interaction_ts: 0,
            state,
        }
    }

    /// Create an initial relationship for an unfamiliar Stranger.
    pub fn new_stranger(character_id: CharacterId, target_id: impl Into<String>) -> Self {
        Self::new(
            character_id,
            target_id,
            "Stranger",
            RelationshipMetrics::stranger(),
            RelationshipKnowledge::new(),
        )
    }

    /// Create a baseline companion relationship (e.g. for default demo interaction).
    pub fn new_companion(character_id: CharacterId, target_id: impl Into<String>) -> Self {
        Self::new(
            character_id,
            target_id,
            "Companion",
            RelationshipMetrics::companion(),
            RelationshipKnowledge::new(),
        )
    }

    /// Synchronize the cached `state` snapshot with current metrics and knowledge.
    pub fn sync_state(&mut self) {
        self.state = RelationshipState::from_parts(&self.metrics, &self.knowledge);
    }

    /// Apply an interaction transition, recording interaction counters and synchronizing state.
    pub fn record_interaction(&mut self, transition: &RelationshipTransition, timestamp: u64) {
        transition.apply_to(&mut self.metrics);
        self.interaction_count += 1;
        self.last_interaction_ts = timestamp;
        self.sync_state();
    }

    /// Add a fact learned about this actor.
    pub fn add_known_fact(&mut self, fact: impl Into<String>) {
        self.knowledge.add_fact(fact);
        self.sync_state();
    }

    /// Validate invariants of the relationship.
    pub fn validate(&self) -> Result<()> {
        if self.target_id.trim().is_empty() {
            return Err(CoreError::ValidationError(
                "Relationship target_id cannot be empty".into(),
            ));
        }
        self.metrics.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_creation_and_defaults() {
        let char_id = CharacterId::new();
        let rel = Relationship::new_stranger(char_id, "user-alice");

        assert_eq!(rel.character_id, char_id);
        assert_eq!(rel.target_id, "user-alice");
        assert_eq!(rel.state.stage, RelationshipStage::Stranger);
        assert_eq!(rel.interaction_count, 0);
        assert!(rel.validate().is_ok());
    }

    #[test]
    fn test_empty_target_id_fails_validation() {
        let char_id = CharacterId::new();
        let rel = Relationship::new_stranger(char_id, "   ");
        assert!(rel.validate().is_err());
    }

    #[test]
    fn test_interaction_evolution_and_stage_progression() {
        let char_id = CharacterId::new();
        let mut rel = Relationship::new_stranger(char_id, "user-bob");

        assert_eq!(rel.state.stage, RelationshipStage::Stranger);

        // Apply several positive interactions
        let trans = RelationshipTransition::positive_interaction();
        for i in 1..=5 {
            rel.record_interaction(&trans, 1000 + i);
        }

        // Familiarity and trust increase
        assert!(rel.metrics.familiarity.value() >= 0.25);
        assert!(rel.metrics.trust.value() >= 0.25);
        assert_eq!(rel.state.stage, RelationshipStage::Acquaintance);

        // Continue positive and vulnerable interactions
        let vuln = RelationshipTransition::vulnerable_interaction();
        for i in 6..=12 {
            rel.record_interaction(&vuln, 1000 + i);
        }

        assert!(rel.metrics.trust.value() >= 0.6);
        assert!(rel.metrics.closeness.value() >= 0.5);
        assert_eq!(rel.state.stage, RelationshipStage::CloseFriend);
        assert_eq!(rel.interaction_count, 12);
    }

    #[test]
    fn test_strict_multi_actor_isolation() {
        let char_id = CharacterId::new();
        let mut rel_alice = Relationship::new_stranger(char_id, "user-alice");
        let rel_bob = Relationship::new_stranger(char_id, "user-bob");

        // Alice interacts intensely with Aria
        let vuln = RelationshipTransition::vulnerable_interaction();
        for i in 1..=10 {
            rel_alice.record_interaction(&vuln, 5000 + i);
        }
        rel_alice.add_known_fact("Loves quantum computing");

        // Bob's relationship MUST remain completely untouched (Skill 14 Rule)
        assert_eq!(rel_bob.state.stage, RelationshipStage::Stranger);
        assert_eq!(rel_bob.interaction_count, 0);
        assert_eq!(rel_bob.knowledge.facts().len(), 0);
        assert!((rel_bob.metrics.trust.value() - 0.15).abs() < f32::EPSILON);

        // Alice is now CloseFriend
        assert_eq!(rel_alice.state.stage, RelationshipStage::CloseFriend);
        assert_eq!(rel_alice.interaction_count, 10);
        assert_eq!(rel_alice.knowledge.facts(), &["Loves quantum computing"]);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let char_id = CharacterId::new();
        let mut rel = Relationship::new_companion(char_id, "user-carol");
        rel.add_known_fact("Prefers Rust over Python");

        let json = serde_json::to_string(&rel).expect("Serialization failed");
        let deserialized: Relationship = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(rel.id, deserialized.id);
        assert_eq!(rel.target_id, deserialized.target_id);
        assert_eq!(rel.state.closeness, deserialized.state.closeness);
        assert_eq!(rel.state.known_facts, deserialized.state.known_facts);
    }
}
