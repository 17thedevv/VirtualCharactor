use crate::context::Context;
use crate::personality::Personality;
use crate::state::CharacterState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DecisionId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionCandidate {
    pub action: Action,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    pub selected_action: Action,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: DecisionId,
    pub result: DecisionResult,
}

/// The domain trait for making decisions.
///
/// Implementations live OUTSIDE vc-core (e.g. vc-runtime for mocks,
/// or a future learned engine crate).
pub trait DecisionEngine {
    fn make_decision(
        &self,
        context: &Context,
        personality: &Personality,
        state: &CharacterState,
    ) -> crate::Result<Decision>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_types() {
        let action = Action {
            action_type: "speak".into(),
            payload: "hello".into(),
        };
        let candidate = DecisionCandidate {
            action: action.clone(),
            confidence: 0.9,
        };
        assert!(candidate.confidence > 0.0);

        let result = DecisionResult {
            selected_action: action,
            reasoning: "test".into(),
        };
        let decision = Decision {
            id: DecisionId(Uuid::new_v4()),
            result,
        };
        assert_eq!(decision.result.selected_action.action_type, "speak");
    }
}
