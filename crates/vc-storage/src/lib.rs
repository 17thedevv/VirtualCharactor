pub mod embedding;
pub mod in_memory;
pub mod rag;
pub mod repository;
pub mod sqlite;

#[cfg(feature = "local-fastembed")]
pub use embedding::FastembedProvider;
pub use embedding::{
    cosine_similarity, EmbeddingProvider, MockEmbeddingProvider, VectorMemoryQuery,
};

pub use in_memory::InMemoryStorage;
pub use repository::{
    CharacterRepository, MemoryRepository, RelationshipRepository, StateRepository,
};
pub use sqlite::{DialogueRecord, SearchRecord, SqliteStorage};
