//! Integration test: proves the end-to-end mock flow works.
//!
//! Flow: CLI input → Runtime → Character → DecisionEngine → MockLlm → Response
//!
//! No API key. No network. No database.

use std::sync::Arc;

use vc_core::character::CharacterId;
use vc_core::context::{Context, ContextItem, ContextPriority, ContextSource};
use vc_core::decision::DecisionEngine;
use vc_core::personality::*;
use vc_core::state::*;
use vc_llm::mock::MockLlmProvider;
use vc_llm::provider::{LlmProvider, LlmRequest};
use vc_runtime::mock_decision::MockDecisionEngine;
use vc_runtime::runtime::RuntimeEngine;
use uuid::Uuid;

fn build_test_personality() -> Personality {
    Personality {
        id: PersonalityId(Uuid::new_v4()),
        identity: Identity {
            core_identity: "Test Character".into(),
            background: "Integration test entity".into(),
        },
        traits: Traits(vec!["curious".into(), "calm".into()]),
        values: Values(vec!["truth".into()]),
        preferences: Preferences(vec!["brevity".into()]),
        behavior_tendencies: BehaviorTendencies(vec!["analytical".into()]),
        communication_style: CommunicationStyle {
            tone: "neutral".into(),
            quirks: vec![],
        },
        decision_tendencies: DecisionTendencies {
            risk_tolerance: "low".into(),
            primary_drivers: vec!["safety".into()],
        },
        boundaries: Boundaries(vec!["no harmful content".into()]),
    }
}

fn build_test_state() -> CharacterState {
    CharacterState {
        emotion: EmotionState {
            primary_emotion: "calm".into(),
            intensity: 0.3,
        },
        cognition: CognitiveState {
            current_focus: "conversation".into(),
            cognitive_load: 0.2,
        },
        behavior: BehaviorState {
            current_activity: "listening".into(),
        },
        goals: Goals {
            active_goals: vec!["answer user question".into()],
        },
        session: SessionState {
            session_id: "integration-test-1".into(),
            variables: std::collections::HashMap::new(),
        },
    }
}

fn build_test_context() -> Context {
    Context {
        items: vec![
            ContextItem {
                source: ContextSource::User,
                content: "Hello, who are you?".into(),
                priority: ContextPriority::High,
            },
            ContextItem {
                source: ContextSource::System,
                content: "Character is in greeting mode.".into(),
                priority: ContextPriority::Medium,
            },
        ],
    }
}

#[test]
fn test_end_to_end_mock_flow() {
    // 1. Setup infrastructure
    let llm = Arc::new(MockLlmProvider {
        default_response: "I am a test character. Nice to meet you!".into(),
    });
    let runtime = RuntimeEngine::new(llm.clone());
    let decision_engine = MockDecisionEngine;

    // 2. Build domain objects
    let _char_id = CharacterId::new();
    let personality = build_test_personality();
    let state = build_test_state();
    let context = build_test_context();

    // 3. Decision phase: context + personality + state → decision
    let decision = decision_engine
        .make_decision(&context, &personality, &state)
        .expect("MockDecisionEngine should never fail");

    assert_eq!(decision.result.selected_action.action_type, "speak");

    // 4. LLM phase: decision → LLM request → LLM response
    let request = LlmRequest {
        prompt: format!(
            "Action: {}\nPayload: {}",
            decision.result.selected_action.action_type,
            decision.result.selected_action.payload
        ),
        system_instruction: Some("You are a persistent AI character.".into()),
    };

    let response = llm
        .generate_text(request)
        .expect("MockLlmProvider should never fail");

    assert!(!response.text.is_empty());
    assert_eq!(response.text, "I am a test character. Nice to meet you!");

    // 5. Runtime tick (no-op in Phase 0 but proves wiring)
    let tick_result = runtime.tick(CharacterId::new());
    assert!(tick_result.is_ok());
}

#[test]
fn test_context_is_not_conversation_history() {
    // Context contains structured items with priority, not raw chat messages
    let ctx = build_test_context();
    assert_eq!(ctx.items.len(), 2);

    // Items have distinct sources and priorities
    assert!(matches!(ctx.items[0].source, ContextSource::User));
    assert!(matches!(ctx.items[1].source, ContextSource::System));
    assert!(matches!(ctx.items[0].priority, ContextPriority::High));
}

#[test]
fn test_personality_is_not_state() {
    let personality = build_test_personality();
    let state = build_test_state();

    // Personality describes stable tendencies
    assert!(!personality.traits.0.is_empty());

    // State describes current conditions
    assert_eq!(state.emotion.primary_emotion, "calm");

    // They are independent types — modifying one does not affect the other
    let mut state2 = state.clone();
    state2.emotion.primary_emotion = "excited".into();
    assert_ne!(state2.emotion.primary_emotion, "calm");
    // personality unchanged
    assert!(personality.traits.0.contains(&"curious".to_string()));
}

#[test]
fn test_decision_is_not_generation() {
    let decision_engine = MockDecisionEngine;
    let ctx = build_test_context();
    let personality = build_test_personality();
    let state = build_test_state();

    // Decision answers "what should the character do?"
    let decision = decision_engine.make_decision(&ctx, &personality, &state).unwrap();
    assert_eq!(decision.result.selected_action.action_type, "speak");

    // LLM generation answers "how should it be expressed?"
    let llm = MockLlmProvider {
        default_response: "expressed output".into(),
    };
    let response = llm
        .generate_text(LlmRequest {
            prompt: decision.result.selected_action.payload.clone(),
            system_instruction: None,
        })
        .unwrap();

    // These are separate concerns
    assert_ne!(decision.result.selected_action.payload, response.text);
}
