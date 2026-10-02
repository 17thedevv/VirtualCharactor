//! Core Domain Data Types for RAG (Retrieval-Augmented Generation).
//!
//! Under Skill 01 (Architecture Governance):
//! Strictly decoupled from external network I/O, SQLite, or specific HTTP embedding providers.

use crate::character::CharacterId;
use crate::memory::MemoryImportance;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an individual document chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentChunkId(pub Uuid);

impl DocumentChunkId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DocumentChunkId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DocumentChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Source categorization of a RAG knowledge item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RagSourceType {
    /// Past episodic conversation turns and interaction summaries.
    EpisodicConversation,
    /// Learned semantic facts or user preferences.
    SemanticFact,
    /// Curated static knowledge, lore, world setting, or external documents.
    LoreDocument,
    /// Explicit real-time user teaching ("I am a Rust developer").
    UserTeaching,
}

impl RagSourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EpisodicConversation => "EpisodicConversation",
            Self::SemanticFact => "SemanticFact",
            Self::LoreDocument => "LoreDocument",
            Self::UserTeaching => "UserTeaching",
        }
    }
}

impl std::fmt::Display for RagSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Dense vector embedding representation with vector operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingVector(pub Vec<f32>);

impl EmbeddingVector {
    pub fn new(vector: Vec<f32>) -> Self {
        Self(vector)
    }

    pub fn dimension(&self) -> usize {
        self.0.len()
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.0
    }

    pub fn dot_product(&self, other: &EmbeddingVector) -> f32 {
        if self.0.len() != other.0.len() {
            return 0.0;
        }
        self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum()
    }

    pub fn magnitude(&self) -> f32 {
        self.0.iter().map(|v| v * v).sum::<f32>().sqrt()
    }

    pub fn cosine_similarity(&self, other: &EmbeddingVector) -> f32 {
        let mag_a = self.magnitude();
        let mag_b = other.magnitude();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        let dot = self.dot_product(other);
        (dot / (mag_a * mag_b)).clamp(-1.0, 1.0)
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 {
            return self.clone();
        }
        Self(self.0.iter().map(|v| v / mag).collect())
    }
}

/// An individual chunk of text ingested into the RAG knowledge store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// Unique chunk identifier.
    pub id: DocumentChunkId,
    /// Parent document identifier (e.g. "world_lore.md" or "session-xyz").
    pub document_id: Option<String>,
    /// Associated character identifier.
    pub character_id: Option<CharacterId>,
    /// Actor identifier for strict isolation (Skill 25: Security & Privacy).
    /// If None, this chunk is public/shared knowledge (e.g. world lore).
    pub actor_id: Option<String>,
    /// Source category.
    pub source_type: RagSourceType,
    /// Raw text content of the chunk.
    pub content: String,
    /// Sequence index within the source document.
    pub chunk_index: usize,
    /// Discretized qualitative importance.
    pub importance: MemoryImportance,
    /// Timestamp when chunk was created (UNIX epoch seconds).
    pub created_at: u64,
}

impl DocumentChunk {
    pub fn new(content: impl Into<String>, source_type: RagSourceType, created_at: u64) -> Self {
        Self {
            id: DocumentChunkId::new(),
            document_id: None,
            character_id: None,
            actor_id: None,
            source_type,
            content: content.into(),
            chunk_index: 0,
            importance: MemoryImportance::Medium,
            created_at,
        }
    }

    pub fn with_document_id(mut self, doc_id: impl Into<String>) -> Self {
        self.document_id = Some(doc_id.into());
        self
    }

    pub fn with_character_id(mut self, char_id: CharacterId) -> Self {
        self.character_id = Some(char_id);
        self
    }

    pub fn with_actor_id(mut self, actor_id: impl Into<String>) -> Self {
        self.actor_id = Some(actor_id.into());
        self
    }

    pub fn with_importance(mut self, importance: MemoryImportance) -> Self {
        self.importance = importance;
        self
    }

    pub fn with_chunk_index(mut self, index: usize) -> Self {
        self.chunk_index = index;
        self
    }
}

/// Query parameter structure for searching the RAG knowledge store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RagQuery {
    /// Raw query text (used for FTS5 sparse keyword matching and embedding).
    pub text: String,
    /// Pre-computed dense vector for query (if already embedded).
    pub query_vector: Option<EmbeddingVector>,
    /// Character scope boundary.
    pub character_id: Option<CharacterId>,
    /// Target actor boundary for privacy isolation.
    /// If provided, returns chunks belonging to this actor PLUS shared chunks (actor_id == None).
    pub actor_id: Option<String>,
    /// Maximum number of items to retrieve.
    pub top_k: usize,
    /// Minimum dense similarity threshold (0.0 to 1.0).
    pub min_similarity: Option<f32>,
    /// Filter by specific source types if desired.
    pub source_types: Option<Vec<RagSourceType>>,
}

impl RagQuery {
    pub fn new(text: impl Into<String>, top_k: usize) -> Self {
        Self {
            text: text.into(),
            query_vector: None,
            character_id: None,
            actor_id: None,
            top_k,
            min_similarity: None,
            source_types: None,
        }
    }

    pub fn with_vector(mut self, vector: EmbeddingVector) -> Self {
        self.query_vector = Some(vector);
        self
    }

    pub fn with_character(mut self, char_id: CharacterId) -> Self {
        self.character_id = Some(char_id);
        self
    }

    pub fn with_actor(mut self, actor_id: impl Into<String>) -> Self {
        self.actor_id = Some(actor_id.into());
        self
    }

    pub fn with_min_similarity(mut self, min: f32) -> Self {
        self.min_similarity = Some(min);
        self
    }

    pub fn with_source_types(mut self, types: Vec<RagSourceType>) -> Self {
        self.source_types = Some(types);
        self
    }
}

/// Result returned from a hybrid RAG query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RagQueryResult {
    /// The retrieved document chunk.
    pub chunk: DocumentChunk,
    /// Dense vector cosine similarity score [0.0, 1.0], if computed.
    pub dense_score: Option<f32>,
    /// Sparse keyword match score (e.g. BM25 / FTS5 rank), if computed.
    pub sparse_score: Option<f32>,
    /// Reciprocal Rank Fusion (RRF) score [0.0, 1.0].
    pub rrf_score: Option<f32>,
    /// Multi-factor composite score considering similarity, importance, and recency decay.
    pub final_score: f32,
}

impl RagQueryResult {
    pub fn new(
        chunk: DocumentChunk,
        dense_score: Option<f32>,
        sparse_score: Option<f32>,
        rrf_score: Option<f32>,
        final_score: f32,
    ) -> Self {
        Self {
            chunk,
            dense_score,
            sparse_score,
            rrf_score,
            final_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_vector_math() {
        let v1 = EmbeddingVector::new(vec![1.0, 0.0, 0.0]);
        let v2 = EmbeddingVector::new(vec![0.0, 1.0, 0.0]);
        let v3 = EmbeddingVector::new(vec![1.0, 1.0, 0.0]);

        assert_eq!(v1.dot_product(&v2), 0.0);
        assert_eq!(v1.cosine_similarity(&v2), 0.0);

        // Identical vectors have cosine similarity = 1.0
        assert!((v1.cosine_similarity(&v1) - 1.0).abs() < 1e-5);

        // v1 and v3: 1 / sqrt(2) ≈ 0.7071
        let sim = v1.cosine_similarity(&v3);
        assert!((sim - 0.7071).abs() < 1e-3);

        // Normalize
        let norm_v3 = v3.normalize();
        assert!((norm_v3.magnitude() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_document_chunk_builder() {
        let char_id = CharacterId::new();
        let chunk = DocumentChunk::new("Aria likes tea", RagSourceType::SemanticFact, 1000)
            .with_document_id("facts.md")
            .with_character_id(char_id)
            .with_actor_id("user-1")
            .with_importance(MemoryImportance::High);

        assert_eq!(chunk.content, "Aria likes tea");
        assert_eq!(chunk.source_type, RagSourceType::SemanticFact);
        assert_eq!(chunk.actor_id.as_deref(), Some("user-1"));
        assert_eq!(chunk.importance, MemoryImportance::High);
    }
}
