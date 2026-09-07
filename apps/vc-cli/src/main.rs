use vc_core::character::CharacterId;
use vc_core::context::Context;
use vc_core::decision::DecisionEngine;
use vc_llm::mock::MockLlmProvider;
use vc_llm::provider::{LlmProvider, LlmRequest};
use vc_runtime::mock_decision::MockDecisionEngine;
use vc_runtime::runtime::RuntimeEngine;
use std::sync::Arc;

fn main() {
    println!("VirtualCharacter CLI - Development Mode");

    // 1. Setup Mock Provider
    let llm = Arc::new(MockLlmProvider {
        default_response: "Hello from Mock LLM! I am running entirely locally.".into(),
    });

    // 2. Setup Runtime Engine
    let runtime = RuntimeEngine::new(llm.clone());
    println!("Runtime initialized.");

    // 3. Setup Mock Decision Engine (lives in vc-runtime, not vc-core)
    let decision_engine = MockDecisionEngine;

    // 4. Create dummy entities for demonstration
    let char_id = CharacterId::new();
    let context = Context { items: vec![] };
    let personality = vc_core::personality::Personality {
        id: vc_core::personality::PersonalityId(uuid::Uuid::new_v4()),
        identity: vc_core::personality::Identity {
            core_identity: "Test Character".into(),
            background: "Created for Phase 0 verification".into(),
        },
        traits: vc_core::personality::Traits(vec!["helpful".into()]),
        values: vc_core::personality::Values(vec!["honesty".into()]),
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
    let state = vc_core::state::CharacterState {
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
            session_id: "cli-session-1".into(),
            variables: std::collections::HashMap::new(),
        },
    };

    // 5. Execute flow: Context → Decision → LLM → Response
    println!("\n--- Executing Decision Cycle ---");
    let decision = decision_engine
        .make_decision(&context, &personality, &state)
        .unwrap();
    println!(
        "Decision: action={}, reasoning={}",
        decision.result.selected_action.action_type, decision.result.reasoning
    );

    let request = LlmRequest {
        prompt: format!(
            "Execute action: {}",
            decision.result.selected_action.payload
        ),
        system_instruction: Some("You are a persistent AI character.".into()),
    };

    println!("Sending to LLM...");
    let response = llm.generate_text(request).unwrap();
    println!("LLM Response: {}", response.text);

    let _ = runtime.tick(char_id);
    println!("\n--- Phase 0 end-to-end flow complete ---");
}
