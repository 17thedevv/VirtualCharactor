use crate::character::CharacterId;
use crate::state::transition::StateTransition;
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
    /// Emitted when the dominant emotion changes after a state transition.
    EmotionChanged {
        character_id: CharacterId,
        previous_dominant: String,
        new_dominant: String,
        transition: StateTransition,
    },
}
