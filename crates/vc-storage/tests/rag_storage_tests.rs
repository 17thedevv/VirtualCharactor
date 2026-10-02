use vc_core::character::CharacterId;
use vc_core::memory::MemoryImportance;
use vc_core::rag::traits::KnowledgeRepository;
use vc_core::rag::types::{DocumentChunk, EmbeddingVector, RagQuery, RagSourceType};
use vc_storage::SqliteStorage;

#[test]
fn test_sqlite_rag_hybrid_storage_and_actor_isolation() {
    let storage = SqliteStorage::in_memory().expect("in-memory db");
    let char_id = CharacterId::new();

    // 1. Create a shared lore chunk (actor_id = None)
    let lore_chunk = DocumentChunk::new(
        "Aria is an intelligent companion who loves programming in Rust.",
        RagSourceType::LoreDocument,
        1000,
    )
    .with_document_id("aria_lore.md")
    .with_character_id(char_id)
    .with_importance(MemoryImportance::High);

    let v_lore = EmbeddingVector::new(vec![1.0, 0.0, 0.0]);
    storage
        .save_chunk(&lore_chunk, Some(&v_lore))
        .expect("save lore chunk");

    // 2. Create Alice's private chunk (actor_id = "alice")
    let alice_chunk = DocumentChunk::new(
        "Alice loves drinking matcha green tea and playing indie games.",
        RagSourceType::EpisodicConversation,
        1005,
    )
    .with_document_id("session-alice")
    .with_character_id(char_id)
    .with_actor_id("alice")
    .with_importance(MemoryImportance::Medium);

    let v_alice = EmbeddingVector::new(vec![0.0, 1.0, 0.0]);
    storage
        .save_chunk(&alice_chunk, Some(&v_alice))
        .expect("save alice chunk");

    // 3. Create Bob's private chunk (actor_id = "bob")
    let bob_chunk = DocumentChunk::new(
        "Bob shared confidential credentials and secret recipes.",
        RagSourceType::SemanticFact,
        1010,
    )
    .with_document_id("session-bob")
    .with_character_id(char_id)
    .with_actor_id("bob")
    .with_importance(MemoryImportance::Critical);

    let v_bob = EmbeddingVector::new(vec![0.0, 0.0, 1.0]);
    storage
        .save_chunk(&bob_chunk, Some(&v_bob))
        .expect("save bob chunk");

    // Check count
    assert_eq!(storage.count_chunks(Some(char_id)).unwrap(), 3);

    // 4. Query as Alice: Target vector close to Alice's preference
    let query_alice = RagQuery::new("matcha tea", 5)
        .with_character(char_id)
        .with_actor("alice")
        .with_vector(EmbeddingVector::new(vec![0.0, 0.9, 0.0]));

    let alice_results = storage.search_hybrid(&query_alice).expect("search ok");
    let alice_chunk_ids: Vec<_> = alice_results.iter().map(|r| r.chunk.id).collect();

    // Alice MUST see her own chunk
    assert!(alice_chunk_ids.contains(&alice_chunk.id));
    // Alice CAN see shared lore chunk
    assert!(alice_chunk_ids.contains(&lore_chunk.id));
    // Alice MUST NEVER see Bob's private chunk! (Actor Isolation)
    assert!(!alice_chunk_ids.contains(&bob_chunk.id));

    // 5. Query as Bob: Keyword query matching "confidential"
    let query_bob = RagQuery::new("confidential secret", 5)
        .with_character(char_id)
        .with_actor("bob");

    let bob_results = storage.search_hybrid(&query_bob).expect("search ok");
    let bob_chunk_ids: Vec<_> = bob_results.iter().map(|r| r.chunk.id).collect();

    // Bob can see his chunk
    assert!(bob_chunk_ids.contains(&bob_chunk.id));
    // Bob CANNOT see Alice's chunk
    assert!(!bob_chunk_ids.contains(&alice_chunk.id));

    // 6. Delete document chunks (e.g. updating lore)
    let deleted = storage
        .delete_document_chunks("aria_lore.md")
        .expect("delete ok");
    assert_eq!(deleted, 1);
    assert_eq!(storage.count_chunks(Some(char_id)).unwrap(), 2);
}
