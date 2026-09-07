use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    pub current_focus: String,
    pub cognitive_load: f32,
}
