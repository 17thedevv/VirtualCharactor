//! RAG (Retrieval-Augmented Generation) Domain Module for VirtualCharacter.
//!
//! Provides core abstractions for multi-source knowledge ingestion, vector operations,
//! composite memory scoring, and hybrid retrieval.

pub mod scoring;
pub mod traits;
pub mod types;

pub use scoring::{calculate_composite_score, calculate_recency_retention, reciprocal_rank_fusion};
pub use traits::{EmbeddingProvider, KnowledgeRepository};
pub use types::{
    DocumentChunk, DocumentChunkId, EmbeddingVector, RagQuery, RagQueryResult, RagSourceType,
};
