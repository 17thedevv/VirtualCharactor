use std::io::{self, Write};
use std::sync::Arc;
use vc_core::character::{Character, CharacterId};
use vc_core::memory::{Memory, MemoryImportance};
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;
use vc_llm::gemini::GeminiProvider;
use vc_llm::mock::MockLlmProvider;
use vc_llm::ollama::OllamaChatProvider;
use vc_llm::provider::LlmProvider;
use vc_runtime::runtime::RuntimeEngine;
use vc_storage::sqlite::SqliteStorage;
use vc_storage::{CharacterRepository, MemoryRepository, RelationshipRepository, StateRepository};

fn main() {
    dotenvy::dotenv().ok();
    println!("════════════════════════════════════════════════════════════════════");
    println!("       🌟 VirtualCharacter AI VTuber / Companion Runtime 🌟         ");
    println!("════════════════════════════════════════════════════════════════════");

    // 1. Setup SQLite Storage Layer (Persistent Local Memory)
    let db_path =
        std::env::var("VC_DB_PATH").unwrap_or_else(|_| "data/virtual_character.db".into());
    let storage = match SqliteStorage::open(&db_path) {
        Ok(s) => {
            println!("📦 Database SQLite: {}", db_path);
            s
        }
        Err(e) => {
            eprintln!(
                "⚠️ Failed to open SQLite at {}: {}. Falling back to in-memory store.",
                db_path, e
            );
            SqliteStorage::in_memory().expect("in-memory sqlite should not fail")
        }
    };

    // 2. Load or seed companion entity
    let existing_char = storage
        .list_characters()
        .ok()
        .and_then(|list| list.into_iter().next());

    let (char_id, personality, mut state, mut relationship, mut memories) = match existing_char {
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
                .get_relationship(c.id, "cli-user")
                .ok()
                .flatten()
                .unwrap_or_else(|| {
                    let rel = Relationship::new_companion(c.id, "cli-user");
                    let _ = storage.save_relationship(&rel);
                    rel
                });
            let m = storage.list_memories(c.id).unwrap_or_default();
            println!(
                "📂 Đã nạp nhân vật: '{}' (ID: {}) | Ký ức: {} | Tương tác tích lũy: {}",
                c.name,
                c.id.0,
                m.len(),
                r.interaction_count
            );
            (c.id, p, s, r, m)
        }
        None => {
            let new_id = CharacterId::new();
            let c = Character {
                id: new_id,
                name: "Aria".into(),
            };
            let p = Personality::baseline_aria();
            let s = CharacterState::default_aria();
            let mut r = Relationship::new_companion(new_id, "cli-user");
            r.add_known_fact("Trò chuyện trực tiếp qua terminal CLI");

            let mut m = vec![
                Memory::new_core(
                    "Mình là Aria, một người bạn ảo độc lập, thông minh và giàu cảm xúc.",
                    1000,
                ),
                Memory::new_semantic(
                    "Người dùng đang xây dựng và trò chuyện cùng mình qua môi trường cục bộ.",
                    MemoryImportance::High,
                    Some("cli-user".into()),
                    1001,
                ),
            ];
            for mem in &mut m {
                mem.character_id = Some(new_id);
            }

            let _ = storage.save_character(&c);
            let _ = storage.save_personality(new_id, &p);
            let _ = storage.save_state(new_id, &s);
            let _ = storage.save_relationship(&r);
            for mem in &m {
                let _ = storage.save_memory(mem);
            }
            println!(
                "🌱 Khởi tạo nhân vật mới: Aria (ID: {}) vào SQLite",
                new_id.0
            );
            (new_id, p, s, r, m)
        }
    };

    // 3. Setup LLM Provider: Local Ollama FIRST -> Gemini Cloud -> Mock
    let ollama_provider = OllamaChatProvider::with_model("qwen2.5:3b");
    let llm: Arc<dyn LlmProvider> = if ollama_provider.is_available() {
        println!("⚡ [LOCAL AI] Đã kết nối Ollama cục bộ thành công! (Model: qwen2.5:3b)");
        println!("🔒 Chạy 100% Offline trên máy (0 MB Cloud / 0 chi phí API / Bảo vệ riêng tư)");
        Arc::new(ollama_provider)
    } else if let Ok(key) = std::env::var("GEMINI_API_KEY") {
        if !key.is_empty() {
            let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-flash".into());
            println!(
                "⚡ [CLOUD AI] Sử dụng Google Gemini API Fallback (Model: {})",
                model
            );
            Arc::new(GeminiProvider::with_model(key, model))
        } else {
            println!(
                "ℹ️ [MOCK AI] Không tìm thấy Ollama hoặc Gemini API Key, sử dụng MockLlmProvider"
            );
            Arc::new(MockLlmProvider::new(
                "Chào bạn! Mình là Aria trong chế độ kiểm thử offline.",
            ))
        }
    } else {
        println!("ℹ️ [MOCK AI] Không tìm thấy Ollama hoặc Gemini API Key, sử dụng MockLlmProvider");
        Arc::new(MockLlmProvider::new(
            "Chào bạn! Mình là Aria trong chế độ kiểm thử offline.",
        ))
    };

    // 4. Setup Runtime Engine
    let runtime = RuntimeEngine::new(llm.clone());
    let (dom_axis, dom_score) = state.emotion.dominant_emotion();
    println!(
        "💖 Tâm trạng ban đầu: {} ({:.0}%) | Giai đoạn quan hệ: {:?}",
        dom_axis.name(),
        dom_score.value() * 100.0,
        relationship.state.stage
    );

    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║       💬 Aria đã sẵn sàng! Gõ tin nhắn để trò chuyện trực tiếp     ║");
    println!("║       (Gõ 'exit' hoặc 'quit' để lưu và thoát chương trình)         ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    // 5. Interactive Chat Loop
    let mut stdin_reader = io::stdin().lines();

    loop {
        print!("👤 Bạn > ");
        io::stdout().flush().unwrap();

        let line = match stdin_reader.next() {
            Some(Ok(l)) => l,
            _ => break, // EOF or pipe ended
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            println!("\n👋 Aria: Tạm biệt bạn nhé! Hẹn gặp lại lần sau nha!");
            break;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        match runtime.process_interaction(
            char_id,
            "cli-user",
            trimmed,
            &personality,
            &mut state,
            &mut relationship,
            &mut memories,
            now,
        ) {
            Ok(outcome) => {
                println!();
                // 1. Inner monologue / internal deliberation
                if !outcome.decision.result.reasoning.is_empty() {
                    println!(
                        "💭 [Nội tâm Aria]: \"{}\"",
                        outcome.decision.result.reasoning
                    );
                }

                // 2. Character spoken response
                println!("✨ Aria: {}", outcome.response_text);

                // 3. Emotional State Feedback
                let (axis, score) = state.emotion.dominant_emotion();
                println!(
                    "💖 [Cảm xúc]: {} ({:.0}%) | Valence: {:.2} | Arousal: {:.2}",
                    axis.name(),
                    score.value() * 100.0,
                    state.emotion.valence(),
                    state.emotion.arousal()
                );

                // 4. Relationship Evolution
                println!(
                    "🤝 [Mối quan hệ]: {:?} | Thân mật: {:.0}% | Tin tưởng: {:.0}%",
                    relationship.state.stage,
                    relationship.state.closeness * 100.0,
                    relationship.state.trust * 100.0
                );

                // 5. Memory Formation
                if let Some(ref mem) = outcome.formed_memory {
                    println!("💾 [Ký ức mới ghi nhận]: \"{}\"", mem.content);
                }
                println!();

                // Persist state updates to SQLite
                let _ = storage.save_state(char_id, &state);
                let _ = storage.save_relationship(&relationship);
                if let Some(ref mem) = outcome.formed_memory {
                    let mut mem_persisted = mem.clone();
                    mem_persisted.character_id = Some(char_id);
                    let _ = storage.save_memory(&mem_persisted);
                }
            }
            Err(err) => {
                eprintln!("\n⚠️ Lỗi khi xử lý tương tác: {}\n", err);
            }
        }

        let _ = runtime.tick(char_id);
    }

    println!(
        "\n💾 Dữ liệu phiên trò chuyện đã được lưu trữ an toàn trong SQLite: {}",
        db_path
    );
}
