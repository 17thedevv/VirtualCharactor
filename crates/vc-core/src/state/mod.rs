pub mod emotion;
pub mod cognitive;
pub mod behavior;
pub mod goals;
pub mod session;

pub use emotion::EmotionState;
pub use cognitive::CognitiveState;
pub use behavior::BehaviorState;
pub use goals::Goals;
pub use session::SessionState;

use serde::{Deserialize, Serialize};

/// The internal state of a character at a given point in time.
///
/// This represents the character's OWN state.
/// Relationship state lives in the `relationship` module because
/// it describes the state BETWEEN entities, not within one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    pub emotion: EmotionState,
    pub cognition: CognitiveState,
    pub behavior: BehaviorState,
    pub goals: Goals,
    pub session: SessionState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_state_creation() {
        let state = CharacterState {
            emotion: EmotionState {
                primary_emotion: "calm".into(),
                intensity: 0.5,
            },
            cognition: CognitiveState {
                current_focus: "listening".into(),
                cognitive_load: 0.1,
            },
            behavior: BehaviorState {
                current_activity: "idle".into(),
            },
            goals: Goals {
                active_goals: vec![],
            },
            session: SessionState {
                session_id: "test-session".into(),
                variables: std::collections::HashMap::new(),
            },
        };
        assert_eq!(state.emotion.primary_emotion, "calm");
        assert!(state.emotion.intensity > 0.0);
    }
}
