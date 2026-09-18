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
    let _context = build_test_context();

    // 3. Decision phase: context + personality + state → decision
    let decision_ctx = vc_core::decision::context::DecisionContext::new(
        "Hello, who are you?",
        None,
        personality.clone(),
        state.clone(),
        None,
        vec![],
    );
    let decision = decision_engine
        .make_decision(&decision_ctx)
        .expect("MockDecisionEngine should never fail");

    assert_eq!(
        decision.result.selected_action.action_type,
        vc_core::decision::action::ActionType::WarmGreeting
    );

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
    let personality = build_test_personality();
    let state = build_test_state();
    let decision_ctx = vc_core::decision::context::DecisionContext::new(
        "Hello, who are you?",
        None,
        personality.clone(),
        state.clone(),
        None,
        vec![],
    );

    // Decision answers "what should the character do?"
    let decision = decision_engine.make_decision(&decision_ctx).unwrap();
    assert_eq!(
        decision.result.selected_action.action_type,
        vc_core::decision::action::ActionType::WarmGreeting
    );

    // LLM generation answers "how should it be expressed?"
    let llm = MockLlmProvider {
        default_response: "expressed output".into(),
    };
    let response = llm
        .generate_text(LlmRequest {
            prompt: decision.result.selected_action.description.clone(),
            system_instruction: None,
        })
        .unwrap();

    // These are separate concerns
    assert_ne!(decision.result.selected_action.description, response.text);
}

#[test]
fn test_rule_decision_engine_rich_scenarios() {
    use vc_core::decision::action::ActionType;
    use vc_core::decision::context::DecisionContext;
    use vc_runtime::rule_decision_engine::RuleDecisionEngine;

    let engine = RuleDecisionEngine::new();
    let personality = build_test_personality();
    let state = build_test_state();

    // 1. Emotional distress -> EmotionalResonance
    let ctx_distress = DecisionContext::new(
        "Hôm nay mình mệt và buồn quá...",
        None,
        personality.clone(),
        state.clone(),
        None,
        vec![],
    );
    let decision_distress = engine.make_decision(&ctx_distress).unwrap();
    assert_eq!(
        decision_distress.result.selected_action.action_type,
        ActionType::EmotionalResonance
    );
    assert!(decision_distress.result.reasoning.to_lowercase().contains("thấu cảm"));
    assert!(decision_distress.result.policy.is_some());

    // 2. Accomplishment -> InspireEncourage
    let ctx_accomplish = DecisionContext::new(
        "Mình vừa hoàn thành xong toàn bộ kiến trúc Phase 1!",
        None,
        personality.clone(),
        state.clone(),
        None,
        vec![],
    );
    let decision_accomplish = engine.make_decision(&ctx_accomplish).unwrap();
    assert_eq!(
        decision_accomplish.result.selected_action.action_type,
        ActionType::InspireEncourage
    );
    assert!(decision_accomplish.result.reasoning.contains("cột mốc ý nghĩa"));

    // 3. Technical question -> ThoughtfulExplanation
    let ctx_tech = DecisionContext::new(
        "Giải thích cho mình kiến trúc memory trong Rust hoạt động như thế nào?",
        None,
        personality.clone(),
        state.clone(),
        None,
        vec![],
    );
    let decision_tech = engine.make_decision(&ctx_tech).unwrap();
    assert_eq!(
        decision_tech.result.selected_action.action_type,
        ActionType::ThoughtfulExplanation
    );
    assert!(decision_tech.result.reasoning.contains("chiều sâu"));
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

#[test]
fn test_memory_system_retrieval_and_actor_isolation() {
    use vc_core::memory::{Memory, MemoryImportance, MemoryQuery};
    use vc_runtime::in_memory_store::InMemoryMemoryStore;

    let now = 200_000;
    let mut memories = vec![
        Memory::new_core("Awakened as Aria, empathetic virtual companion", now - 86400),
        Memory::new_episodic("Alice shared that her favorite compiler is written in Rust", MemoryImportance::High, Some("alice".into()), now - 3600),
        Memory::new_semantic("Alice prefers strict type safety", MemoryImportance::Critical, Some("alice".into()), now - 7200),
        Memory::new_episodic("Bob mentioned he is learning Python for data science", MemoryImportance::Medium, Some("bob".into()), now - 1800),
        Memory::new_episodic("Bob shared his secret password with Aria", MemoryImportance::High, Some("bob".into()), now - 900),
    ];

    // 1. Alice queries memories about 'Rust' or 'compiler'
    let query_alice = MemoryQuery::new(3)
        .with_actor("alice")
        .with_text("Rust compiler");

    let results_alice = InMemoryMemoryStore::retrieve_from_slice(&mut memories, &query_alice, now);
    assert!(results_alice.len() <= 3);
    assert!(results_alice.iter().any(|m| m.content.contains("Alice")));
    // Skill 12 Rule: NEVER retrieve Bob's memories for Alice!
    assert!(!results_alice.iter().any(|m| m.content.contains("Bob")));
    assert!(!results_alice.iter().any(|m| m.content.contains("password")));

    // 2. Bob queries memories about 'Python'
    let query_bob = MemoryQuery::new(2)
        .with_actor("bob")
        .with_text("Python");

    let results_bob = InMemoryMemoryStore::retrieve_from_slice(&mut memories, &query_bob, now);
    assert!(!results_bob.is_empty());
    assert!(results_bob.iter().any(|m| m.content.contains("Python")));
    // Skill 12 Rule: NEVER retrieve Alice's memories for Bob!
    assert!(!results_bob.iter().any(|m| m.content.contains("Alice")));

    // 3. Access reinforcement check: Retrieved memories have incremented access_count
    let alice_accessed = memories.iter().find(|m| m.content.contains("Alice shared")).unwrap();
    assert!(alice_accessed.lifecycle.access_count >= 1);
}


