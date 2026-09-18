use std::sync::Arc;
use tokio::sync::RwLock;
use vc_core::character::{Character, CharacterId};
use vc_core::memory::{Memory, MemoryImportance};
use vc_core::personality::*;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;
use vc_llm::mock::MockLlmProvider;
use vc_runtime::rule_decision_engine::RuleDecisionEngine;
use vc_runtime::rule_emotion_engine::RuleBasedEmotionEngine;
use vc_runtime::runtime::RuntimeEngine;

#[derive(Clone)]
pub struct AppState {
    pub character: Arc<RwLock<Character>>,
    pub personality: Arc<RwLock<Personality>>,
    pub character_state: Arc<RwLock<CharacterState>>,
    pub relationship: Arc<RwLock<Relationship>>,
    pub memories: Arc<RwLock<Vec<Memory>>>,
    pub runtime: Arc<RuntimeEngine>,
    pub emotion_engine: Arc<RuleBasedEmotionEngine>,
    pub decision_engine: Arc<RuleDecisionEngine>,
}

impl AppState {
    pub fn new() -> Self {
        let char_id = CharacterId::new();

        let character = Character {
            id: char_id,
            name: "Aria".to_string(),
        };

        let personality = Personality::baseline_aria();

        let character_state = CharacterState::default_aria();

        let mut relationship = Relationship::new_companion(char_id, "user-default");
        relationship.add_known_fact("Interested in building intelligent autonomous agents");
        relationship.add_known_fact("Prefers clean architecture and thoughtful interfaces");


        let memories = vec![
            Memory::new_core("First awakened in the VirtualCharacter runtime environment.", 1000),
            Memory::new_semantic(
                "User appreciates thoughtful, self-aware responses.",
                MemoryImportance::Critical,
                Some("user-default".into()),
                1001,
            ),
            Memory::new_episodic(
                "Discussed the beauty of pairing Rust performance with fluid web aesthetics.",
                MemoryImportance::Medium,
                Some("user-default".into()),
                1002,
            ),
        ];

        let llm: Arc<dyn vc_llm::provider::LlmProvider> = match std::env::var("GEMINI_API_KEY") {
            Ok(key) if !key.is_empty() => {
                let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.5-flash-lite".into());
                println!("⚡ vc-server configured with Google Gemini API (model: {})", model);
                Arc::new(vc_llm::gemini::GeminiProvider::with_model(key, model))
            }
            _ => {
                println!("ℹ️ No GEMINI_API_KEY found, using local MockLlmProvider");
                Arc::new(MockLlmProvider {
                    default_response: "Hello! I am Aria. I sense a warm curiosity in our space today. What are we exploring together?".into(),
                })
            }
        };
        let runtime = Arc::new(RuntimeEngine::new(llm));
        let emotion_engine = Arc::new(RuleBasedEmotionEngine::new());
        let decision_engine = Arc::new(RuleDecisionEngine::new());

        Self {
            character: Arc::new(RwLock::new(character)),
            personality: Arc::new(RwLock::new(personality)),
            character_state: Arc::new(RwLock::new(character_state)),
            relationship: Arc::new(RwLock::new(relationship)),
            memories: Arc::new(RwLock::new(memories)),
            runtime,
            emotion_engine,
            decision_engine,
        }
    }

    /// Reset state back to defaults
    pub async fn reset(&self) {
        let fresh = Self::new();
        *self.character.write().await = fresh.character.read().await.clone();
        *self.personality.write().await = fresh.personality.read().await.clone();
        *self.character_state.write().await = fresh.character_state.read().await.clone();
        *self.relationship.write().await = fresh.relationship.read().await.clone();
        *self.memories.write().await = fresh.memories.read().await.clone();
    }
}
