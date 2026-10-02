pub mod handler;
pub mod prompt;

pub use handler::process_user_interaction;
pub use prompt::{build_emotion_json, sanitize_character_dialogue};
