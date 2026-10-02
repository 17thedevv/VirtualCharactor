//! RAG (Retrieval-Augmented Generation) Ingestion and Search Module.

pub mod chunker;
pub mod conversation_archiver;
pub mod lore_indexer;

pub use chunker::{ChunkerOptions, SemanticTextChunker};
pub use conversation_archiver::ConversationArchiver;
pub use lore_indexer::{IndexingReport, LoreIndexer};
