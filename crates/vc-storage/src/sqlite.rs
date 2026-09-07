use crate::repository::{CharacterRepository, MemoryRepository};
use vc_core::character::{Character, CharacterId};
use vc_core::memory::{Memory, MemoryId};

pub struct SqliteCharacterRepository {
    pub connection_string: String,
}

impl CharacterRepository for SqliteCharacterRepository {
    fn get_character(&self, _id: CharacterId) -> vc_core::Result<Character> {
        Err(vc_core::CoreError::CharacterNotFound(
            "Sqlite mock: Character not found".into(),
        ))
    }

    fn save_character(&self, _character: Character) -> vc_core::Result<()> {
        Ok(())
    }
}

pub struct SqliteMemoryRepository {
    pub connection_string: String,
}

impl MemoryRepository for SqliteMemoryRepository {
    fn get_memory(&self, _id: MemoryId) -> vc_core::Result<Memory> {
        Err(vc_core::CoreError::MemoryNotFound(
            "Sqlite mock: Memory not found".into(),
        ))
    }

    fn save_memory(&self, _memory: Memory) -> vc_core::Result<()> {
        Ok(())
    }
}
