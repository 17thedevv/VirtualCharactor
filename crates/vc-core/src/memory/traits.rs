use crate::error::Result;
use super::query::{MemoryQuery, MemoryReference};
use super::Memory;

/// Core interface for retrieving memories according to situational relevance and security constraints.
pub trait MemoryRetriever: Send + Sync {
    /// Retrieve memories that match the query criteria, ordered by relevance.
    fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<Memory>>;
}

/// Interface for evaluating and forming new memories from interaction perception.
pub trait MemoryFormation: Send + Sync {
    /// Evaluate an input string and determine if it warrants long-term memory formation.
    fn evaluate(&self, input: &str, actor_id: Option<&str>, timestamp: u64) -> Option<Memory>;
}

/// Interface for ranking memories based on multidimensional scoring.
pub trait MemoryRanking: Send + Sync {
    /// Rank candidate memories against a query, producing scored memory references.
    fn rank(&self, memories: &[Memory], query: &MemoryQuery) -> Vec<MemoryReference>;
}
