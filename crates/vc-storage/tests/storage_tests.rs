use std::time::SystemTime;
use uuid::Uuid;
use vc_core::character::{Character, CharacterId};
use vc_core::memory::{Memory, MemoryImportance, MemoryQuery};
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;
use vc_storage::in_memory::InMemoryStorage;
use vc_storage::sqlite::SqliteStorage;
use vc_storage::{CharacterRepository, MemoryRepository, RelationshipRepository, StateRepository};

#[test]
fn test_in_memory_storage_full_lifecycle() {
    let storage = InMemoryStorage::new();
    let char_id = CharacterId::new();

    // 1. Character
    let character = Character {
        id: char_id,
        name: "TestAria".into(),
    };
    storage.save_character(&character).unwrap();
    let loaded_char = storage.get_character(char_id).unwrap();
    assert_eq!(loaded_char.name, "TestAria");

    // 2. Personality
    let personality = Personality::baseline_aria();
    storage.save_personality(char_id, &personality).unwrap();
    let loaded_pers = storage.get_personality(char_id).unwrap().unwrap();
    assert_eq!(
        loaded_pers.identity.core_identity,
        personality.identity.core_identity
    );

    // 3. State
    let mut state = CharacterState::default_aria();
    state.session.session_id = "test-session".into();
    storage.save_state(char_id, &state).unwrap();
    let loaded_state = storage.get_state(char_id).unwrap().unwrap();
    assert_eq!(loaded_state.session.session_id, "test-session");

    // 4. Relationship
    let rel = Relationship::new_companion(char_id, "user-alice");
    storage.save_relationship(&rel).unwrap();
    let loaded_rel = storage
        .get_relationship(char_id, "user-alice")
        .unwrap()
        .unwrap();
    assert_eq!(loaded_rel.target_id, "user-alice");

    // 5. Memory & Actor Isolation
    let mem1 = Memory::new_core("Core memory", 1000);
    let mut mem2 = Memory::new_episodic(
        "Alice shared a secret",
        MemoryImportance::High,
        Some("user-alice".into()),
        1010,
    );
    mem2.character_id = Some(char_id);

    storage.save_memory(&mem1).unwrap();
    storage.save_memory(&mem2).unwrap();

    let query_alice = MemoryQuery::new(5).with_actor("user-alice");
    let results_alice = storage.query_memories(char_id, &query_alice, 1050).unwrap();
    assert_eq!(results_alice.len(), 1);
    assert!(results_alice[0].content.contains("Alice"));

    let query_bob = MemoryQuery::new(5).with_actor("user-bob");
    let results_bob = storage.query_memories(char_id, &query_bob, 1050).unwrap();
    assert!(
        results_bob.is_empty(),
        "Alice memory must be isolated from Bob"
    );
}

#[test]
fn test_sqlite_in_memory_full_lifecycle() {
    let storage = SqliteStorage::in_memory().expect("in-memory sqlite should initialize");
    let char_id = CharacterId::new();

    // 1. Character & Personality
    let character = Character {
        id: char_id,
        name: "AriaSqlite".into(),
    };
    storage.save_character(&character).unwrap();
    let loaded_char = storage.get_character(char_id).unwrap();
    assert_eq!(loaded_char.name, "AriaSqlite");

    let personality = Personality::baseline_aria();
    storage.save_personality(char_id, &personality).unwrap();
    let loaded_pers = storage.get_personality(char_id).unwrap().unwrap();
    assert_eq!(
        loaded_pers.traits.curiosity.value(),
        personality.traits.curiosity.value()
    );

    // 2. State
    let mut state = CharacterState::default_aria();
    state.emotion.joy = vc_core::state::EmotionScore::clamped(0.95);
    storage.save_state(char_id, &state).unwrap();
    let loaded_state = storage.get_state(char_id).unwrap().unwrap();
    assert!((loaded_state.emotion.joy.value() - 0.95).abs() < 0.001);

    // 3. Relationships (Multi-Actor)
    let mut rel_alice = Relationship::new_companion(char_id, "user-alice");
    rel_alice.add_known_fact("Likes Rust and SQLite");
    storage.save_relationship(&rel_alice).unwrap();

    let rel_bob = Relationship::new_stranger(char_id, "user-bob");
    storage.save_relationship(&rel_bob).unwrap();

    let loaded_alice = storage
        .get_relationship(char_id, "user-alice")
        .unwrap()
        .unwrap();
    assert_eq!(loaded_alice.knowledge.facts().len(), 1);

    let all_rels = storage.list_relationships(char_id).unwrap();
    assert_eq!(all_rels.len(), 2);

    // 4. Memories & Query Retrieval
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut mem_alice = Memory::new_episodic(
        "Discussed database persistence in Rust",
        MemoryImportance::Critical,
        Some("user-alice".into()),
        now,
    );
    mem_alice.character_id = Some(char_id);
    storage.save_memory(&mem_alice).unwrap();

    let query_alice = MemoryQuery::new(3)
        .with_actor("user-alice")
        .with_text("persistence");
    let retrieved = storage
        .query_memories(char_id, &query_alice, now + 10)
        .unwrap();
    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].id, mem_alice.id);
    assert_eq!(retrieved[0].lifecycle.access_count, 1);

    // Query by Bob should be isolated
    let query_bob = MemoryQuery::new(3)
        .with_actor("user-bob")
        .with_text("persistence");
    let retrieved_bob = storage
        .query_memories(char_id, &query_bob, now + 10)
        .unwrap();
    assert!(
        retrieved_bob.is_empty(),
        "Actor isolation violated in SQLite"
    );
}

#[test]
fn test_sqlite_file_persistence_across_restarts() {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("test_vc_{}.db", Uuid::new_v4()));

    let char_id = CharacterId::new();
    let personality = Personality::baseline_aria();
    let state = CharacterState::default_aria();

    // Session 1: Create storage and write data
    {
        let storage = SqliteStorage::open(&db_path).expect("Should open sqlite file");
        let character = Character {
            id: char_id,
            name: "PersistentAria".into(),
        };
        storage.save_character(&character).unwrap();
        storage.save_personality(char_id, &personality).unwrap();
        storage.save_state(char_id, &state).unwrap();

        let mut rel = Relationship::new_companion(char_id, "user-persisted");
        rel.add_known_fact("Survives app restarts");
        storage.save_relationship(&rel).unwrap();

        let mut mem = Memory::new_core("Core identity initialized", 1000);
        mem.character_id = Some(char_id);
        storage.save_memory(&mem).unwrap();
    } // storage dropped, connection closed

    // Session 2: Reopen same database file and verify persistence
    {
        let storage2 = SqliteStorage::open(&db_path).expect("Should reopen existing sqlite file");
        let loaded_char = storage2.get_character(char_id).unwrap();
        assert_eq!(loaded_char.name, "PersistentAria");

        let loaded_pers = storage2.get_personality(char_id).unwrap().unwrap();
        assert_eq!(
            loaded_pers.traits.empathy.value(),
            personality.traits.empathy.value()
        );

        let loaded_state = storage2.get_state(char_id).unwrap().unwrap();
        assert_eq!(
            loaded_state.emotion.dominant_emotion().0.name(),
            "curiosity"
        );

        let loaded_rel = storage2
            .get_relationship(char_id, "user-persisted")
            .unwrap()
            .unwrap();
        assert!(loaded_rel
            .knowledge
            .facts()
            .contains(&"Survives app restarts".to_string()));

        let memories = storage2.list_memories(char_id).unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].content, "Core identity initialized");
    }

    // Cleanup
    let _ = std::fs::remove_file(db_path);
}
