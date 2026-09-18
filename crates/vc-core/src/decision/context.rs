use serde::{Deserialize, Serialize};
use crate::memory::Memory;
use crate::personality::Personality;
use crate::relationship::Relationship;
use crate::state::CharacterState;

/// Input context delivered to the Decision Engine to deliberate on the character's next action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    /// Raw dialogue message from the user
    pub user_input: String,
    /// Interacting actor/user identifier
    pub actor_id: Option<String>,
    /// Core character personality identity, traits, and values
    pub personality: Personality,
    /// Current dynamic internal state (emotion, cognition, behavior)
    pub state: CharacterState,
    /// Current relationship bond with this specific actor
    pub relationship: Option<Relationship>,
    /// Retrieved relevant long-term and short-term memories
    pub memories: Vec<Memory>,
}

impl DecisionContext {
    pub fn new(
        user_input: impl Into<String>,
        actor_id: Option<String>,
        personality: Personality,
        state: CharacterState,
        relationship: Option<Relationship>,
        memories: Vec<Memory>,
    ) -> Self {
        Self {
            user_input: user_input.into(),
            actor_id,
            personality,
            state,
            relationship,
            memories,
        }
    }
}
