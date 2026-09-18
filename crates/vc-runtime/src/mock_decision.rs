use vc_core::context::Context;
use vc_core::decision::{Action, Decision, DecisionEngine, DecisionId, DecisionResult};
use vc_core::personality::Personality;
use vc_core::state::CharacterState;
use uuid::Uuid;

/// A deterministic mock decision engine for testing and development.
///
/// This lives in vc-runtime (not vc-core) because it is an implementation,
/// not a domain contract.
pub struct MockDecisionEngine;

impl DecisionEngine for MockDecisionEngine {
    fn make_decision(
        &self,
        _context: &Context,
        _personality: &Personality,
        _state: &CharacterState,
    ) -> vc_core::Result<Decision> {
        Ok(Decision {
            id: DecisionId(Uuid::new_v4()),
            result: DecisionResult {
                selected_action: Action {
                    action_type: "speak".into(),
                    payload: "mock response".into(),
                },
                reasoning: "deterministic mock".into(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_decision_engine_returns_speak() {
        let engine = MockDecisionEngine;
        let context = Context { items: vec![] };
        let personality = vc_core::personality::Personality::baseline_aria();
        let state = CharacterState::default_aria();

        let decision = engine.make_decision(&context, &personality, &state).unwrap();
        assert_eq!(decision.result.selected_action.action_type, "speak");
        assert_eq!(decision.result.reasoning, "deterministic mock");
    }
}
