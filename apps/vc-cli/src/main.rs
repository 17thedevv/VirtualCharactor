use std::sync::Arc;
use vc_core::character::CharacterId;
use vc_llm::gemini::GeminiProvider;
use vc_llm::mock::MockLlmProvider;
use vc_llm::provider::LlmProvider;
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

    // 3. Create companion entities for interaction lifecycle demonstration
    let char_id = CharacterId::new();
    let personality = vc_core::personality::Personality::baseline_aria();
    let mut state = vc_core::state::CharacterState::default_aria();
    let mut relationship = vc_core::relationship::Relationship::new_companion(char_id, "cli-user");
    let mut memories = vec![
        vc_core::memory::Memory::new_core("CLI companion initialized in VirtualCharacter runtime.", 1000),
    ];

    // 5. Execute full 9-stage interaction lifecycle via RuntimeEngine (Skill 22)
    println!("\n--- Executing Full B2 Runtime Interaction Lifecycle ---");
    let user_input = "Xin chào Aria! Hôm nay bạn cảm thấy thế nào?";
    println!("User Input: \"{}\"", user_input);

    match runtime.process_interaction(
        char_id,
        "cli-user",
        user_input,
        &personality,
        &mut state,
        &mut relationship,
        &mut memories,
        1050,
    ) {
        Ok(outcome) => {
            println!("✅ Interaction processed successfully!");
            println!("  * Session ID: {}", outcome.session_id);
            println!("  * Interaction ID: {}", outcome.interaction_id);
            println!("  * Decision: {} (confidence: {:.0}%)", outcome.decision.result.selected_action.action_type, outcome.decision.result.confidence * 100.0);
            println!("  * Inner Monologue: \"{}\"", outcome.decision.result.reasoning);
            println!("  * Context Used: {} tokens", outcome.context_breakdown.system_tokens + outcome.context_breakdown.user_tokens + outcome.context_breakdown.personality_tokens);
            println!("  * Character Response: \"{}\"", outcome.response_text);
            let (dom_axis, dom_score) = state.emotion.dominant_emotion();
            println!("  * Updated Dominant Emotion: {} ({:.0}%)", dom_axis.name(), dom_score.value() * 100.0);
            println!("  * Relationship Stage: {:?} (Closeness: {:.0}%, Trust: {:.0}%)", relationship.state.stage, relationship.state.closeness * 100.0, relationship.state.trust * 100.0);
            if let Some(ref mem) = outcome.formed_memory {
                println!("  * Consolidated Memory: \"{}\"", mem.content);
            }
        }
        Err(err) => {
            eprintln!("⚠️ Runtime interaction error: {}", err);
        }
    }

    let _ = runtime.tick(char_id);
    println!("\n--- B2 Runtime Orchestration Complete ---");
}
