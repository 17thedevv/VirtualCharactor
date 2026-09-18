use vc_core::decision::action::{Action, ActionType};
use vc_core::decision::context::DecisionContext;
use vc_core::decision::{Decision, DecisionEngine, DecisionId, DecisionResult};
use uuid::Uuid;

/// A deterministic mock decision engine for testing and development.
pub struct MockDecisionEngine;

impl DecisionEngine for MockDecisionEngine {
    fn make_decision(&self, _ctx: &DecisionContext) -> vc_core::Result<Decision> {
        Ok(Decision {
            id: DecisionId(Uuid::new_v4()),
            result: DecisionResult {
                selected_action: Action::simple(ActionType::WarmGreeting),
                confidence: 0.9,
                reasoning: "deterministic mock".into(),
                candidates: vec![],
                policy: None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_core::personality::Personality;
    use vc_core::state::CharacterState;

    #[test]
    fn test_mock_decision_engine_returns_greeting() {
        let engine = MockDecisionEngine;
        let personality = Personality::baseline_aria();
        let state = CharacterState::default_aria();
        let ctx = DecisionContext::new("hello", None, personality, state, None, vec![]);

        let decision = engine.make_decision(&ctx).unwrap();
        assert_eq!(decision.result.selected_action.action_type, ActionType::WarmGreeting);
        assert_eq!(decision.result.reasoning, "deterministic mock");
    }
}
