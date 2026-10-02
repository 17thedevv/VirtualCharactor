//! Conversation Archiver for Episodic Memory RAG (Step 1.8.4).
//!
//! Groups raw dialogue turns into structured conversation episodes,
//! computes vector embeddings, and stores them in the KnowledgeRepository
//! with strict Actor Isolation.

use std::sync::Arc;
use vc_core::character::CharacterId;
use vc_core::error::Result;
use vc_core::memory::MemoryImportance;
use vc_core::rag::traits::{EmbeddingProvider, KnowledgeRepository};
use vc_core::rag::types::{DocumentChunk, RagSourceType};
use vc_storage::DialogueRecord;

/// Automatically archives dialogue history into episodic RAG chunks.
pub struct ConversationArchiver {
    embedder: Arc<dyn EmbeddingProvider>,
    repository: Arc<dyn KnowledgeRepository>,
    turns_per_episode: usize,
}

impl ConversationArchiver {
    pub fn new(
        embedder: Arc<dyn EmbeddingProvider>,
        repository: Arc<dyn KnowledgeRepository>,
    ) -> Self {
        Self {
            embedder,
            repository,
            turns_per_episode: 4, // 2 exchanges (user + character) per episode chunk
        }
    }

    /// Archive a list of dialogue records into episodic RAG memory chunks.
    pub fn archive_dialogues(
        &self,
        character_id: CharacterId,
        actor_id: &str,
        session_id: &str,
        dialogues: &[DialogueRecord],
    ) -> Result<usize> {
        if dialogues.is_empty() {
            return Ok(0);
        }

        let mut chunks_archived = 0;
        let chunks: Vec<&[DialogueRecord]> = dialogues.chunks(self.turns_per_episode).collect();

        for (idx, slice) in chunks.iter().enumerate() {
            let mut episode_text = String::new();
            let mut latest_ts = 0;

            for turn in *slice {
                let speaker = if turn.sender == "user" {
                    "User"
                } else {
                    "Aria"
                };
                episode_text.push_str(&format!("{}: {}\n", speaker, turn.text));
                if turn.created_at > latest_ts {
                    latest_ts = turn.created_at;
                }
            }

            let doc_id = format!("conv_{}_{}", session_id, idx);
            let chunk = DocumentChunk::new(
                episode_text.trim(),
                RagSourceType::EpisodicConversation,
                latest_ts,
            )
            .with_document_id(doc_id)
            .with_character_id(character_id)
            .with_actor_id(actor_id)
            .with_chunk_index(idx)
            .with_importance(MemoryImportance::Medium);

            let emb = self.embedder.embed_text(&chunk.content).ok();
            self.repository.save_chunk(&chunk, emb.as_ref())?;
            chunks_archived += 1;
        }

        Ok(chunks_archived)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_core::rag::types::EmbeddingVector;
    use vc_storage::SqliteStorage;

    struct DummyEmbedder;
    impl EmbeddingProvider for DummyEmbedder {
        fn embed_text(&self, text: &str) -> Result<EmbeddingVector> {
            Ok(EmbeddingVector::new(vec![text.len() as f32, 1.0, 0.0]).normalize())
        }
        fn dimension(&self) -> usize {
            3
        }
    }

    #[test]
    fn test_conversation_archiver_stores_chunks() {
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let embedder = Arc::new(DummyEmbedder);
        let archiver = ConversationArchiver::new(embedder, storage.clone());

        let char_id = CharacterId::new();
        let dialogues = vec![
            DialogueRecord {
                id: "t1".into(),
                character_id: char_id,
                actor_id: "alice".into(),
                sender: "user".into(),
                text: "Tôi vừa hoàn thành dự án Rust".into(),
                created_at: 1000,
            },
            DialogueRecord {
                id: "t2".into(),
                character_id: char_id,
                actor_id: "alice".into(),
                sender: "character".into(),
                text: "Chúc mừng bạn nhé!".into(),
                created_at: 1001,
            },
        ];

        let count = archiver
            .archive_dialogues(char_id, "alice", "sess-1", &dialogues)
            .expect("archive ok");
        assert_eq!(count, 1);
        assert_eq!(storage.count_chunks(Some(char_id)).unwrap(), 1);
    }
}
