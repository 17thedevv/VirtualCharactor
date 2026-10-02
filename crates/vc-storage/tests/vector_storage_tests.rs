use vc_core::character::CharacterId;
use vc_core::memory::{Memory, MemoryImportance};
use vc_storage::embedding::{EmbeddingProvider, MockEmbeddingProvider, VectorMemoryQuery};
use vc_storage::repository::MemoryRepository;
use vc_storage::{InMemoryStorage, SqliteStorage};

#[test]
fn test_in_memory_vector_search_and_privacy_isolation() {
    let storage = InMemoryStorage::new();
    let embedder = MockEmbeddingProvider::default_384();
    let char_id = CharacterId::new();

    // 1. Create memories with different actor ownerships
    let mut mem_core = Memory::new_core("Aria is a joyful virtual character companion", 1000);
    mem_core.character_id = Some(char_id);

    let mut mem_alice = Memory::new_episodic(
        "Alice loves playing chess every Sunday morning",
        MemoryImportance::High,
        Some("alice".into()),
        1000,
    );
    mem_alice.character_id = Some(char_id);

    let mut mem_bob = Memory::new_episodic(
        "Bob shared confidential guitar song chords with Aria",
        MemoryImportance::High,
        Some("bob".into()),
        1000,
    );
    mem_bob.character_id = Some(char_id);

    // Compute embeddings
    let emb_core = embedder.embed_text(&mem_core.content).unwrap();
    let emb_alice = embedder.embed_text(&mem_alice.content).unwrap();
    let emb_bob = embedder.embed_text(&mem_bob.content).unwrap();

    // Save with embeddings
    storage
        .save_memory_with_embedding(&mem_core, &emb_core)
        .unwrap();
    storage
        .save_memory_with_embedding(&mem_alice, &emb_alice)
        .unwrap();
    storage
        .save_memory_with_embedding(&mem_bob, &emb_bob)
        .unwrap();

    // Verify embedding retrieval
    let retrieved_emb = storage.get_memory_embedding(mem_alice.id).unwrap();
    assert_eq!(retrieved_emb, Some(emb_alice.clone()));

    // 2. Query as Alice for "chess morning"
    let query_vector = embedder.embed_text("chess game morning").unwrap();
    let alice_query = VectorMemoryQuery::new(query_vector.clone(), 10)
        .with_actor_id("alice")
        .with_min_similarity(0.1);

    let alice_results = storage
        .search_similar_memories(char_id, &alice_query)
        .unwrap();

    // Alice should see core memory and Alice's memory, but NEVER Bob's memory!
    let alice_retrieved_ids: Vec<_> = alice_results.iter().map(|(m, _)| m.id).collect();
    assert!(alice_retrieved_ids.contains(&mem_alice.id));
    assert!(
        !alice_retrieved_ids.contains(&mem_bob.id),
        "Actor isolation violated: Alice saw Bob's memory!"
    );

    // 3. Query as Bob for "guitar chords"
    let bob_query_vector = embedder
        .embed_text("confidential guitar song chords")
        .unwrap();
    let bob_query = VectorMemoryQuery::new(bob_query_vector, 10)
        .with_actor_id("bob")
        .with_min_similarity(0.1);

    let bob_results = storage
        .search_similar_memories(char_id, &bob_query)
        .unwrap();
    let bob_retrieved_ids: Vec<_> = bob_results.iter().map(|(m, _)| m.id).collect();
    assert!(bob_retrieved_ids.contains(&mem_bob.id));
    assert!(
        !bob_retrieved_ids.contains(&mem_alice.id),
        "Actor isolation violated: Bob saw Alice's memory!"
    );

    // 4. Anonymous caller (None) can ONLY see core/shared memory
    let anon_query = VectorMemoryQuery::new(query_vector, 10).with_min_similarity(0.1);
    let anon_results = storage
        .search_similar_memories(char_id, &anon_query)
        .unwrap();
    for (m, _) in anon_results {
        assert!(
            m.metadata.source_actor_id.is_none(),
            "Anonymous query returned actor-scoped memory!"
        );
    }
}

#[test]
fn test_sqlite_vector_search_and_privacy_isolation() {
    let storage = SqliteStorage::in_memory().unwrap();
    let embedder = MockEmbeddingProvider::default_384();
    let char_id = CharacterId::new();

    let mut mem_core = Memory::new_core("Core identity knowledge in SQLite", 1000);
    mem_core.character_id = Some(char_id);

    let mut mem_alice = Memory::new_episodic(
        "Alice prefers talking about technology and Rust programming",
        MemoryImportance::High,
        Some("alice".into()),
        1000,
    );
    mem_alice.character_id = Some(char_id);

    let mut mem_bob = Memory::new_episodic(
        "Bob dislikes loud noises and crowded streams",
        MemoryImportance::Medium,
        Some("bob".into()),
        1000,
    );
    mem_bob.character_id = Some(char_id);

    let emb_core = embedder.embed_text(&mem_core.content).unwrap();
    let emb_alice = embedder.embed_text(&mem_alice.content).unwrap();
    let emb_bob = embedder.embed_text(&mem_bob.content).unwrap();

    storage
        .save_memory_with_embedding(&mem_core, &emb_core)
        .unwrap();
    storage
        .save_memory_with_embedding(&mem_alice, &emb_alice)
        .unwrap();
    storage
        .save_memory_with_embedding(&mem_bob, &emb_bob)
        .unwrap();

    // Query as Alice for "Rust programming"
    let q_vec = embedder.embed_text("Rust programming software").unwrap();
    let q = VectorMemoryQuery::new(q_vec, 5)
        .with_actor_id("alice")
        .with_min_similarity(0.05);

    let results = storage.search_similar_memories(char_id, &q).unwrap();
    let retrieved_ids: Vec<_> = results.iter().map(|(m, _)| m.id).collect();

    assert!(retrieved_ids.contains(&mem_alice.id));
    assert!(
        !retrieved_ids.contains(&mem_bob.id),
        "Actor isolation breached in SQLite!"
    );

    // Verify ordering by similarity score descending
    for window in results.windows(2) {
        assert!(
            window[0].1 >= window[1].1,
            "Results must be sorted descending by similarity"
        );
    }
}

#[test]
fn test_sqlite_vector_persistence_across_reopen() {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("vector_test_{}.db", uuid::Uuid::new_v4()));

    let embedder = MockEmbeddingProvider::default_384();
    let char_id = CharacterId::new();
    let mut mem = Memory::new_core(
        "Persistent memory with vector embedding across restarts",
        2000,
    );
    mem.character_id = Some(char_id);
    let emb = embedder.embed_text(&mem.content).unwrap();

    // 1. Open database, save memory with embedding, then drop
    {
        let storage = SqliteStorage::open(&db_path).unwrap();
        storage.save_memory_with_embedding(&mem, &emb).unwrap();
    }

    // 2. Re-open from disk and verify persistence
    {
        let storage = SqliteStorage::open(&db_path).unwrap();
        let loaded_emb = storage.get_memory_embedding(mem.id).unwrap();
        assert_eq!(
            loaded_emb,
            Some(emb.clone()),
            "Embedding should survive database reopen"
        );

        let q = VectorMemoryQuery::new(emb, 5).with_min_similarity(0.9);
        let results = storage.search_similar_memories(char_id, &q).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, mem.id);
        assert!((results[0].1 - 1.0).abs() < 1e-5);
    }

    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn test_vector_similarity_threshold_and_top_k() {
    let storage = SqliteStorage::in_memory().unwrap();
    let char_id = CharacterId::new();

    // Create 5 memories with known synthetic vectors
    for i in 0..5 {
        let mut mem = Memory::new_core(format!("Memory number {}", i), 1000);
        mem.character_id = Some(char_id);

        let mut vec = vec![0.0f32; 10];
        vec[i] = 1.0; // Orthogonal unit vectors
        storage.save_memory_with_embedding(&mem, &vec).unwrap();
    }

    // Query for vector index 0
    let mut query_vec = vec![0.0f32; 10];
    query_vec[0] = 1.0;

    // High threshold: only memory 0 should match (similarity = 1.0, others = 0.0)
    let q = VectorMemoryQuery::new(query_vec, 10).with_min_similarity(0.5);
    let results = storage.search_similar_memories(char_id, &q).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.content, "Memory number 0");
    assert!((results[0].1 - 1.0).abs() < 1e-5);
}
