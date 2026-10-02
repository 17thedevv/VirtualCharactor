//! Abstract Domain Repository and Provider Traits for RAG Knowledge Systems.
//!
//! Under Skill 01 (Architecture Governance) & Skill 02 (Contract-First Development):
//! Defines clean interface boundaries so implementations (SQLite, Ollama, ONNX) remain pluggable.

use crate::character::CharacterId;
use crate::error::Result;
use crate::rag::types::{DocumentChunk, EmbeddingVector, RagQuery, RagQueryResult};

/// Abstract provider interface for generating dense vector embeddings.
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding vector for a single text snippet.
    fn embed_text(&self, text: &str) -> Result<EmbeddingVector>;

    /// Generate embeddings for a batch of text snippets.
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        texts.iter().map(|t| self.embed_text(t)).collect()
    }

    /// Expected dimensionality of the vector space (e.g. 768 for nomic-embed-text, 1024 for bge-m3).
    fn dimension(&self) -> usize;

    /// Provider identifier for observability and metrics.
    fn name(&self) -> &'static str {
        "GenericEmbeddingProvider"
    }
}

/// Abstract storage and retrieval repository for RAG document chunks.
pub trait KnowledgeRepository: Send + Sync {
    /// Persist a single document chunk and its optional embedding vector.
    fn save_chunk(&self, chunk: &DocumentChunk, embedding: Option<&EmbeddingVector>) -> Result<()>;

    /// Persist a batch of document chunks and their optional embeddings.
    fn save_chunks_batch(&self, chunks: &[(DocumentChunk, Option<EmbeddingVector>)]) -> Result<()> {
        for (chunk, emb) in chunks {
            self.save_chunk(chunk, emb.as_ref())?;
        }
        Ok(())
    }

    /// Execute a hybrid query (Dense Vector + Sparse FTS) matching character and actor bounds.
    fn search_hybrid(&self, query: &RagQuery) -> Result<Vec<RagQueryResult>>;

    /// Delete all chunks belonging to a specific document (e.g. when updating lore file).
    fn delete_document_chunks(&self, document_id: &str) -> Result<usize>;

    /// Count total indexed chunks, optionally scoped to a character.
    fn count_chunks(&self, character_id: Option<CharacterId>) -> Result<usize>;
}
