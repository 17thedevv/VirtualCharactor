use vc_core::character::{Character, CharacterId};
use vc_core::memory::{Memory, MemoryId};

pub trait CharacterRepository {
    fn get_character(&self, id: CharacterId) -> vc_core::Result<Character>;
    fn save_character(&self, character: Character) -> vc_core::Result<()>;
}

pub trait MemoryRepository {
    fn get_memory(&self, id: MemoryId) -> vc_core::Result<Memory>;
    fn save_memory(&self, memory: Memory) -> vc_core::Result<()>;
}
