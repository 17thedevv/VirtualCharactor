pub mod action;
pub mod candidate;
pub mod context;
pub mod policy;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;

pub use action::{Action, ActionType};
pub use candidate::DecisionCandidate;
pub use context::DecisionContext;
pub use policy::BehaviorPolicy;

/// Unique identifier for a deliberation decision instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DecisionId(pub Uuid);

impl DecisionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DecisionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DecisionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The structured outcome of decision deliberation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionResult {
    /// The optimal action selected by the engine
    pub selected_action: Action,
    /// Confidence in this selection [0.0, 1.0]
    pub confidence: f32,
    /// Inner monologue reasoning explaining why this action was chosen
    pub reasoning: String,
    /// All candidate actions evaluated with their respective scores
    pub candidates: Vec<DecisionCandidate>,
    /// Optional behavioral styling policy to guide LLM response tone & verbosity
    pub policy: Option<BehaviorPolicy>,
}

/// Full Decision entity tracking the deliberated choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub id: DecisionId,
    pub result: DecisionResult,
}

impl Decision {
    pub fn new(result: DecisionResult) -> Self {
        Self {
            id: DecisionId::new(),
            result,
        }
    }
}

/// Domain trait for deciding the character's intent and action plan.
///
/// Under Skill 15 (Decision Engineering), the DecisionEngine evaluates situations
/// and selects an action BEFORE the LLM generates the final natural-language response.
pub trait DecisionEngine: Send + Sync {
    fn make_decision(&self, ctx: &DecisionContext) -> Result<Decision>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_creation_and_serialization() {
        let action = Action::new(ActionType::WarmGreeting, "welcoming user", "Warm greeting");
        let candidate = DecisionCandidate::new(action.clone(), 0.95, 0.92, "Direct friendly greeting detected");
        let result = DecisionResult {
            selected_action: action,
            confidence: 0.95,
            reasoning: "User greeted warmly; welcoming them is high priority.".into(),
            candidates: vec![candidate],
            policy: Some(BehaviorPolicy::warm_empathic()),
        };

        let decision = Decision::new(result);
        assert_eq!(decision.result.selected_action.action_type, ActionType::WarmGreeting);
        assert!(decision.result.confidence >= 0.9);

        let json = serde_json::to_string(&decision).expect("Serialization failed");
        let deserialized: Decision = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(decision.id, deserialized.id);
        assert_eq!(decision.result.confidence, deserialized.result.confidence);
    }
}
