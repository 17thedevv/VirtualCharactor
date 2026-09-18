use std::sync::Arc;
use vc_core::character::CharacterId;
use vc_core::context::Context;
use vc_core::decision::DecisionEngine;
use vc_llm::gemini::GeminiProvider;
use vc_llm::mock::MockLlmProvider;
use vc_llm::provider::{LlmProvider, LlmRequest};
use vc_runtime::mock_decision::MockDecisionEngine;
use vc_runtime::runtime::RuntimeEngine;

fn main() {
    dotenvy::dotenv().ok();
    println!("VirtualCharacter CLI - Development Mode");

    // 1. Setup Provider: Gemini if key available, else Mock
    let llm: Arc<dyn LlmProvider> = match std::env::var("GEMINI_API_KEY") {
        Ok(key) if !key.is_empty() => {
            let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.7-flash".into());
            println!("⚡ Connecting to Google Gemini API (model: {})", model);
            Arc::new(GeminiProvider::with_model(key, model))
        }
        _ => {
            println!("ℹ️ No GEMINI_API_KEY found, using local MockLlmProvider");
            Arc::new(MockLlmProvider {
                default_response: "Hello from Mock LLM! I am running entirely locally.".into(),
            })
        }
    };

    // 2. Setup Runtime Engine
    let runtime = RuntimeEngine::new(llm.clone());
    println!("Runtime initialized.");

    // 3. Setup Mock Decision Engine (lives in vc-runtime, not vc-core)
    let decision_engine = MockDecisionEngine;

    // 4. Create dummy entities for demonstration
    let char_id = CharacterId::new();
    let _context = Context::default();
    let personality = vc_core::personality::Personality::baseline_aria();
    let state = vc_core::state::CharacterState::default_aria();

    let decision_ctx = vc_core::decision::context::DecisionContext::new(
        "Hello from CLI",
        None,
        personality,
        state,
        None,
        vec![],
    );

    // 5. Execute flow: Context → Decision → LLM → Response
    println!("\n--- Executing Decision Cycle ---");
    let decision = decision_engine
        .make_decision(&decision_ctx)
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
    match llm.generate_text(request) {
        Ok(response) => {
            println!("LLM Response: {}", response.text);
        }
        Err(err) => {
            eprintln!("⚠️ LLM call error: {}", err);
        }
    }

    let _ = runtime.tick(char_id);
    println!("\n--- Phase 0 end-to-end flow complete ---");
}
