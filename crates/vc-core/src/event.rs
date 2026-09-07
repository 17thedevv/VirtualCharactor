use crate::character::CharacterId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacterEvent {
    Spoke {
        character_id: CharacterId,
        content: String,
    },
    Felt {
        character_id: CharacterId,
        emotion: String,
    },
    Acted {
        character_id: CharacterId,
        action: String,
    },
}
