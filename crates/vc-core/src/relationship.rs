use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::character::CharacterId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationshipId(pub Uuid);

/// The state of a relationship between a character and another entity.
///
/// This is NOT part of CharacterState because it describes the bond
/// BETWEEN two entities, not the internal state of one character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipState {
    pub closeness: f32,
    pub trust: f32,
    pub known_facts: Vec<String>,
}

/// A relationship between a character and another entity (user, NPC, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: RelationshipId,
    pub character_id: CharacterId,
    pub target_id: String,
    pub relationship_type: String,
    pub state: RelationshipState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_creation() {
        let rel = Relationship {
            id: RelationshipId(Uuid::new_v4()),
            character_id: CharacterId::new(),
            target_id: "user-123".into(),
            relationship_type: "friend".into(),
            state: RelationshipState {
                closeness: 0.5,
                trust: 0.7,
                known_facts: vec!["likes coffee".into()],
            },
        };
        assert_eq!(rel.state.closeness, 0.5);
        assert_eq!(rel.state.known_facts.len(), 1);
    }
}
