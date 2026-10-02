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
use vc_storage::sqlite::SqliteStorage;
use vc_storage::{CharacterRepository, MemoryRepository, RelationshipRepository, StateRepository};

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
    pub storage: Arc<SqliteStorage>,
    pub tts_cache: Arc<RwLock<std::collections::HashMap<String, Vec<u8>>>>,
    pub conversation_fsm: Arc<RwLock<vc_runtime::conversation::ConversationStateMachine>>,
    pub tts_queue: Arc<tokio::sync::Mutex<vc_runtime::audio::OrderedTtsQueue>>,
    pub autonomous_engine: Arc<tokio::sync::Mutex<vc_runtime::autonomous::AutonomousLifeEngine>>,
    pub temporal_gate: Arc<tokio::sync::Mutex<vc_runtime::attention::TemporalAttentionGate>>,
    pub embedder: Arc<dyn vc_core::rag::traits::EmbeddingProvider>,
    pub conversation_archiver: Arc<vc_runtime::rag::ConversationArchiver>,
}

impl AppState {
    pub fn new() -> Self {
        let db_path =
            std::env::var("VC_DB_PATH").unwrap_or_else(|_| "data/virtual_character.db".into());
        let storage = Arc::new(match SqliteStorage::open(&db_path) {
            Ok(s) => {
                println!("📦 SQLite storage initialized at {}", db_path);
                s
            }
            Err(e) => {
                eprintln!(
                    "⚠️ Failed to open SQLite at {}: {}. Falling back to in-memory SQLite.",
                    db_path, e
                );
                SqliteStorage::in_memory().expect("in-memory sqlite should not fail")
            }
        });

        let existing_char = storage
            .list_characters()
            .ok()
            .and_then(|list| list.into_iter().next());

        let (character, personality, character_state, relationship, memories) = match existing_char
        {
            Some(c) => {
                let p = storage
                    .get_personality(c.id)
                    .ok()
                    .flatten()
                    .unwrap_or_else(Personality::baseline_aria);
                let s = storage
                    .get_state(c.id)
                    .ok()
                    .flatten()
                    .unwrap_or_else(CharacterState::default_aria);
                let r = storage
                    .get_relationship(c.id, "user-default")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| {
                        let mut rel = Relationship::new_companion(c.id, "user-default");
                        rel.add_known_fact("Interested in building intelligent autonomous agents");
                        rel.add_known_fact("Prefers clean architecture and thoughtful interfaces");
                        let _ = storage.save_relationship(&rel);
                        rel
                    });
                let m = storage.list_memories(c.id).unwrap_or_default();
                println!(
                    "📂 Loaded persisted character '{}' (ID: {}) from SQLite with {} memories",
                    c.name,
                    c.id.0,
                    m.len()
                );
                (c, p, s, r, m)
            }
            None => {
                let char_id = CharacterId::new();
                let c = Character {
                    id: char_id,
                    name: "Aria".to_string(),
                };
                let p = Personality::baseline_aria();
                let s = CharacterState::default_aria();
                let mut r = Relationship::new_companion(char_id, "user-default");
                r.add_known_fact("Thích trò chuyện vui vẻ, thoải mái, cởi mở");
                r.add_known_fact("Thích được quan tâm và chia sẻ về cuộc sống, sở thích, game");

                let mut m = vec![
                    Memory::new_core("Aria và anh ấy luôn có những phút giây trò chuyện ngọt ngào, ấm áp bên nhau.", 1000),
                    Memory::new_semantic(
                        "Anh ấy thích sự chân thành, ấm áp và phong cách Onee-san dịu dàng, cưng chiều.",
                        MemoryImportance::High,
                        Some("user-default".into()),
                        1001,
                    ),
                    Memory::new_episodic(
                        "Từng cùng nhau trò chuyện vui vẻ và chia sẻ nhiều điều thú vị trong ngày.",
                        MemoryImportance::Medium,
                        Some("user-default".into()),
                        1002,
                    ),
                ];
                for mem in &mut m {
                    mem.character_id = Some(char_id);
                }

                // Persist newly seeded data to SQLite
                let _ = storage.save_character(&c);
                let _ = storage.save_personality(char_id, &p);
                let _ = storage.save_state(char_id, &s);
                let _ = storage.save_relationship(&r);
                for mem in &m {
                    let _ = storage.save_memory(mem);
                }
                println!(
                    "🌱 Seeded new default character Aria (ID: {}) into SQLite",
                    char_id.0
                );
                (c, p, s, r, m)
            }
        };

        let ollama = vc_llm::OllamaChatProvider::with_model("qwen2.5:3b");
        let llm: Arc<dyn vc_llm::provider::LlmProvider> = if ollama.is_available() {
            println!("⚡ [LOCAL AI] vc-server configured with local Ollama (model: qwen2.5:3b)");
            Arc::new(ollama)
        } else if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            if !key.is_empty() {
                let model =
                    std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-flash".into());
                println!(
                    "⚡ [CLOUD AI] vc-server configured with Google Gemini API (model: {})",
                    model
                );
                Arc::new(vc_llm::gemini::GeminiProvider::with_model(key, model))
            } else {
                println!(
                    "ℹ️ [MOCK AI] No Ollama or GEMINI_API_KEY found, using local MockLlmProvider"
                );
                Arc::new(MockLlmProvider::new(
                    "Hello! I am Aria. I sense a warm curiosity in our space today. What are we exploring together?",
                ))
            }
        } else {
            println!("ℹ️ [MOCK AI] No Ollama or GEMINI_API_KEY found, using local MockLlmProvider");
            Arc::new(MockLlmProvider::new(
                "Hello! I am Aria. I sense a warm curiosity in our space today. What are we exploring together?",
            ))
        };
        let emotion_engine = Arc::new(RuleBasedEmotionEngine::new());
        let decision_engine = Arc::new(RuleDecisionEngine::new());
        let runtime = Arc::new(RuntimeEngine::with_engines(
            llm,
            decision_engine.clone(),
            emotion_engine.clone(),
        ));

        let mut initial_tts_cache = std::collections::HashMap::new();
        let precache_file = std::path::Path::new("data/cache_test_voice.mp3");
        if precache_file.exists() {
            if let Ok(bytes) = std::fs::read(precache_file) {
                println!("🎵 [CACHE] Pre-loaded {} bytes of Yae Miko test voice for instant 0ms playback", bytes.len());
                let test_text = "Ara ara~ Chào cưng nhé! Cáo tỷ tỷ Yae Miko đây. Hôm nay muốn tỷ tỷ cưng chiều điều gì nào?";
                initial_tts_cache.insert(format!("yaemiko:{}", test_text), bytes);
            }
        }

        let embedder: Arc<dyn vc_core::rag::traits::EmbeddingProvider> =
            Arc::new(vc_llm::OllamaEmbeddingProvider::default_nomic().with_fallback(true));
        let conversation_archiver = Arc::new(vc_runtime::rag::ConversationArchiver::new(
            embedder.clone(),
            storage.clone(),
        ));

        let lore_indexer = vc_runtime::rag::LoreIndexer::new(embedder.clone(), storage.clone());
        let knowledge_dir = std::path::Path::new("data/knowledge");
        if knowledge_dir.exists() {
            let report = lore_indexer.index_directory(knowledge_dir, Some(character.id));
            if report.files_scanned > 0 {
                println!(
                    "📚 [RAG] Indexed {} knowledge files ({} chunks) from data/knowledge/",
                    report.files_scanned, report.chunks_created
                );
            }
        }

        Self {
            character: Arc::new(RwLock::new(character)),
            personality: Arc::new(RwLock::new(personality)),
            character_state: Arc::new(RwLock::new(character_state)),
            relationship: Arc::new(RwLock::new(relationship)),
            memories: Arc::new(RwLock::new(memories)),
            runtime,
            emotion_engine,
            decision_engine,
            storage,
            tts_cache: Arc::new(RwLock::new(initial_tts_cache)),
            conversation_fsm: Arc::new(RwLock::new(
                vc_runtime::conversation::ConversationStateMachine::new(),
            )),
            tts_queue: Arc::new(tokio::sync::Mutex::new(
                vc_runtime::audio::OrderedTtsQueue::new(),
            )),
            autonomous_engine: Arc::new(tokio::sync::Mutex::new(
                vc_runtime::autonomous::AutonomousLifeEngine::default(),
            )),
            temporal_gate: Arc::new(tokio::sync::Mutex::new(
                vc_runtime::attention::TemporalAttentionGate::default(),
            )),
            embedder: embedder.clone(),
            conversation_archiver,
        }
    }

    /// Reset state back to defaults and persist to database.
    pub async fn reset(&self) {
        let char_id = self.character.read().await.id;
        let fresh_p = Personality::baseline_aria();
        let fresh_s = CharacterState::default_aria();
        let mut fresh_r = Relationship::new_companion(char_id, "user-default");
        fresh_r.add_known_fact("Thích trò chuyện vui vẻ, thoải mái, cởi mở");
        fresh_r.add_known_fact("Thích được quan tâm và chia sẻ về cuộc sống, sở thích, game");

        let mut fresh_m = vec![
            Memory::new_core(
                "Aria và anh ấy luôn có những phút giây trò chuyện ngọt ngào, ấm áp bên nhau.",
                1000,
            ),
            Memory::new_semantic(
                "Anh ấy thích sự chân thành, ấm áp và phong cách Onee-san dịu dàng, cưng chiều.",
                MemoryImportance::High,
                Some("user-default".into()),
                1001,
            ),
            Memory::new_episodic(
                "Từng cùng nhau trò chuyện vui vẻ và chia sẻ nhiều điều thú vị trong ngày.",
                MemoryImportance::Medium,
                Some("user-default".into()),
                1002,
            ),
        ];
        for mem in &mut fresh_m {
            mem.character_id = Some(char_id);
        }

        // Persist to storage
        let _ = self.storage.save_personality(char_id, &fresh_p);
        let _ = self.storage.save_state(char_id, &fresh_s);
        let _ = self.storage.save_relationship(&fresh_r);
        let _ = self.storage.clear_memories(char_id);
        let _ = self.storage.clear_search_records(char_id);
        let _ = self.storage.clear_dialogue_records(char_id);
        for mem in &fresh_m {
            let _ = self.storage.save_memory(mem);
        }

        // Update in-memory representations
        *self.personality.write().await = fresh_p;
        *self.character_state.write().await = fresh_s;
        *self.relationship.write().await = fresh_r;
        *self.memories.write().await = fresh_m;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.conversation_fsm.write().await.reset(now);
        self.tts_queue.lock().await.reset();
        self.autonomous_engine.lock().await.reset_cooldown();
        self.temporal_gate.lock().await.reset();
    }
}
