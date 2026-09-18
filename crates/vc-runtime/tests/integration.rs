//! Integration test: proves the end-to-end mock flow works.
//!
//! Flow: CLI input → Runtime → Character → DecisionEngine → MockLlm → Response
//!
//! No API key. No network. No database.

use std::sync::Arc;

use vc_core::character::CharacterId;
use vc_core::context::{Context, ContextItem, ContextPriority, ContextSource};
use vc_core::decision::DecisionEngine;
use vc_core::personality::Personality;
use vc_core::state::*;
use vc_llm::mock::MockLlmProvider;
use vc_llm::provider::{LlmProvider, LlmRequest};
use vc_runtime::mock_decision::MockDecisionEngine;
use vc_runtime::rule_emotion_engine::RuleBasedEmotionEngine;
use vc_runtime::runtime::RuntimeEngine;

fn build_test_personality() -> Personality {
    Personality::baseline_aria()
}

fn build_test_state() -> CharacterState {
    CharacterState::default_aria()
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
    assert!(personality.traits.curiosity.value() > 0.0);

    // State describes current conditions — dominant emotion
    let (dominant_axis, _score) = state.emotion.dominant_emotion();
    assert_eq!(dominant_axis.name(), "curiosity"); // Aria defaults to curious

    // They are independent types — modifying one does not affect the other
    let mut state2 = state.clone();
    state2.emotion.joy = EmotionScore::clamped(0.99);
    let (new_dominant, _) = state2.emotion.dominant_emotion();
    assert_eq!(new_dominant.name(), "joy");
    // personality unchanged
    assert_eq!(personality.traits.score("curiosity"), Some(0.92));
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

#[test]
fn test_emotion_engine_integration() {
    use vc_core::state::engine::EmotionEngine;

    let engine = RuleBasedEmotionEngine::new();
    let mut state = build_test_state();
    let personality = build_test_personality();

    // Simulate a sad message
    let delta = engine.evaluate(&state.emotion, "Mình đang rất buồn", &personality);
    assert!(delta.sadness > 0.0, "Sadness should increase");

    // Apply delta
    let old_sadness = state.emotion.sadness.value();
    state.emotion.apply_delta(&delta);
    assert!(state.emotion.sadness.value() > old_sadness, "Sadness should have increased");

    // Now simulate a happy message
    let delta2 = engine.evaluate(&state.emotion, "Cảm ơn bạn, mình vui lắm!", &personality);
    assert!(delta2.joy > 0.0, "Joy should increase");

    state.emotion.apply_delta(&delta2);
    assert!(state.emotion.joy.value() > 0.3, "Joy should be above neutral");
}

#[test]
fn test_emotion_decay_over_time() {
    let mut state = build_test_state();

    // Set equal extreme emotions to compare decay rates
    state.emotion.anger = EmotionScore::clamped(0.8);
    state.emotion.surprise = EmotionScore::clamped(0.8);

    let initial_anger = state.emotion.anger.value();
    let initial_surprise = state.emotion.surprise.value();

    // Short decay (10 seconds) — enough to see differential rates
    state.emotion.decay(10);

    assert!(
        state.emotion.anger.value() < initial_anger,
        "Anger should decay: {} -> {}",
        initial_anger,
        state.emotion.anger.value()
    );
    assert!(
        state.emotion.surprise.value() < initial_surprise,
        "Surprise should decay: {} -> {}",
        initial_surprise,
        state.emotion.surprise.value()
    );

    // Surprise decay rate (λ=0.15) > anger decay rate (λ=0.08)
    // So after same time, surprise should have lost more than anger
    let anger_remaining = state.emotion.anger.value();
    let surprise_remaining = state.emotion.surprise.value();
    // Anger should have decayed less (retained more) than surprise
    assert!(
        anger_remaining > surprise_remaining,
        "Anger (slower decay) should retain more than surprise (faster decay): anger={} surprise={}",
        anger_remaining,
        surprise_remaining
    );
}

#[test]
fn test_effective_behavior_personality_emotion_interaction() {
    let personality = build_test_personality();
    let mut state = build_test_state();

    // Normal state → playfulness should be close to personality baseline
    let normal_play = state.effective_playfulness(&personality);
    assert!(normal_play > 0.6, "Normal playfulness should be moderate-high");

    // Sad state → playfulness suppressed
    state.emotion.sadness = EmotionScore::clamped(0.9);
    let sad_play = state.effective_playfulness(&personality);
    assert!(
        sad_play < normal_play,
        "Sadness should suppress playfulness: {} vs {}",
        sad_play,
        normal_play
    );

    // Sync behavior should update behavioral state
    state.sync_behavior(&personality);
    assert!(
        state.behavior.seriousness.value() > 0.4,
        "Seriousness should increase when sad"
    );
}

#[test]
fn test_relationship_evolution_and_multi_actor_isolation() {
    use vc_core::relationship::{Relationship, RelationshipStage, RelationshipTransition};

    let char_id = CharacterId::new();

    // Two independent actors
    let mut rel_alice = Relationship::new_stranger(char_id, "user-alice");
    let rel_bob = Relationship::new_stranger(char_id, "user-bob");

    assert_eq!(rel_alice.state.stage, RelationshipStage::Stranger);
    assert_eq!(rel_bob.state.stage, RelationshipStage::Stranger);

    // Alice has multiple positive interactions
    let positive = RelationshipTransition::positive_interaction();
    for i in 0..5 {
        rel_alice.record_interaction(&positive, 1000 + i);
    }
    assert_eq!(rel_alice.state.stage, RelationshipStage::Acquaintance);

    // Alice shares vulnerable conversations
    let vuln = RelationshipTransition::vulnerable_interaction();
    for i in 5..12 {
        rel_alice.record_interaction(&vuln, 1000 + i);
    }
    rel_alice.add_known_fact("Loves exploring Rust and AI systems");

    // Alice is now CloseFriend
    assert_eq!(rel_alice.state.stage, RelationshipStage::CloseFriend);
    assert!(rel_alice.state.trust > 0.60);
    assert!(rel_alice.state.closeness > 0.50);
    assert_eq!(rel_alice.state.known_facts.len(), 1);

    // Skill 14 Rule: Bob's relationship must remain pure stranger
    assert_eq!(rel_bob.state.stage, RelationshipStage::Stranger);
    assert_eq!(rel_bob.interaction_count, 0);
    assert_eq!(rel_bob.knowledge.facts().len(), 0);
    assert!((rel_bob.state.trust - 0.15).abs() < f32::EPSILON);
}

