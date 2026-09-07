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
        let personality = vc_core::personality::Personality {
            id: vc_core::personality::PersonalityId(Uuid::new_v4()),
            identity: vc_core::personality::Identity {
                core_identity: "test".into(),
                background: "test".into(),
            },
            traits: vc_core::personality::Traits(vec![]),
            values: vc_core::personality::Values(vec![]),
            preferences: vc_core::personality::Preferences(vec![]),
            behavior_tendencies: vc_core::personality::BehaviorTendencies(vec![]),
            communication_style: vc_core::personality::CommunicationStyle {
                tone: "neutral".into(),
                quirks: vec![],
            },
            decision_tendencies: vc_core::personality::DecisionTendencies {
                risk_tolerance: "low".into(),
                primary_drivers: vec![],
            },
            boundaries: vc_core::personality::Boundaries(vec![]),
        };
        let state = CharacterState {
            emotion: vc_core::state::EmotionState {
                primary_emotion: "calm".into(),
                intensity: 0.5,
            },
            cognition: vc_core::state::CognitiveState {
                current_focus: "none".into(),
                cognitive_load: 0.0,
            },
            behavior: vc_core::state::BehaviorState {
                current_activity: "idle".into(),
            },
            goals: vc_core::state::Goals {
                active_goals: vec![],
            },
            session: vc_core::state::SessionState {
                session_id: "test".into(),
                variables: std::collections::HashMap::new(),
            },
        };

        let decision = engine.make_decision(&context, &personality, &state).unwrap();
        assert_eq!(decision.result.selected_action.action_type, "speak");
        assert_eq!(decision.result.reasoning, "deterministic mock");
    }
}
