use vc_core::character::{Character, CharacterId};

pub trait CharacterUseCase {
    fn get_character(&self, id: CharacterId) -> vc_core::Result<Character>;
    fn create_character(&self, name: String) -> vc_core::Result<CharacterId>;
}
