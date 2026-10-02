use vc_core::character::{Character, CharacterId};
use vc_core::error::Result;
use vc_core::memory::{Memory, MemoryId, MemoryQuery};
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;

/// Repository interface for Character entity and its static personality.
pub trait CharacterRepository: Send + Sync {
    /// Retrieve a character by its ID.
    fn get_character(&self, id: CharacterId) -> Result<Character>;

    /// Persist or update a character entity.
    fn save_character(&self, character: &Character) -> Result<()>;

    /// Retrieve the personality profile of a character.
    fn get_personality(&self, character_id: CharacterId) -> Result<Option<Personality>>;

    /// Persist or update the personality profile of a character.
    fn save_personality(&self, character_id: CharacterId, personality: &Personality) -> Result<()>;

    /// List all characters managed by the storage.
    fn list_characters(&self) -> Result<Vec<Character>>;
}

/// Repository interface for dynamic CharacterState persistence.
pub trait StateRepository: Send + Sync {
    /// Retrieve current runtime state (emotion, cognitive, behavior, goals, session) of a character.
    fn get_state(&self, character_id: CharacterId) -> Result<Option<CharacterState>>;

    /// Persist or update character dynamic state.
    fn save_state(&self, character_id: CharacterId, state: &CharacterState) -> Result<()>;
}

/// Repository interface for bipartite Relationship persistence.
pub trait RelationshipRepository: Send + Sync {
    /// Retrieve relationship between a character and a specific external actor.
    fn get_relationship(
        &self,
        character_id: CharacterId,
        actor_id: &str,
    ) -> Result<Option<Relationship>>;

    /// Persist or update a relationship entity.
    fn save_relationship(&self, relationship: &Relationship) -> Result<()>;

    /// List all relationships formed by a character.
    fn list_relationships(&self, character_id: CharacterId) -> Result<Vec<Relationship>>;
}

/// Repository interface for long-term and episodic memory persistence.
pub trait MemoryRepository: Send + Sync {
    /// Retrieve a single memory by its unique ID.
    fn get_memory(&self, id: MemoryId) -> Result<Memory>;

    /// Persist or update a memory record.
    fn save_memory(&self, memory: &Memory) -> Result<()>;

    /// Retrieve all memories associated with a character.
    fn list_memories(&self, character_id: CharacterId) -> Result<Vec<Memory>>;

    /// Query memories using structured query filters with strict actor isolation and token budgeting.
    fn query_memories(
        &self,
        character_id: CharacterId,
        query: &MemoryQuery,
        now: u64,
    ) -> Result<Vec<Memory>>;

    /// Remove a memory record by ID.
    fn delete_memory(&self, id: MemoryId) -> Result<()>;

    /// Delete all memories associated with a character (e.g. for reset).
    fn clear_memories(&self, character_id: CharacterId) -> Result<()>;

    /// Persist or update a memory record along with its precomputed dense vector embedding.
    fn save_memory_with_embedding(&self, memory: &Memory, _embedding: &[f32]) -> Result<()> {
        self.save_memory(memory)
    }

    /// Retrieve stored vector embedding for a memory, if available.
    fn get_memory_embedding(&self, _id: MemoryId) -> Result<Option<Vec<f32>>> {
        Ok(None)
    }

    /// Search memories by vector similarity with strict actor isolation and minimum similarity threshold.
    fn search_similar_memories(
        &self,
        _character_id: CharacterId,
        _query: &crate::embedding::VectorMemoryQuery,
    ) -> Result<Vec<(Memory, f32)>> {
        Ok(Vec::new())
    }
}
