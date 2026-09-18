use serde::{Deserialize, Serialize};
use crate::error::{CoreError, Result};
use crate::personality::behavior::TendencyLevel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunicationStyle {
    pub formality: TendencyLevel,
    pub verbosity: TendencyLevel,
    pub emotionality: TendencyLevel,
    pub emoji_usage: TendencyLevel,
    pub humor: TendencyLevel,
    pub directness: TendencyLevel,
    pub tone: String,
    pub quirks: Vec<String>,
}

impl CommunicationStyle {
    pub fn new(
        formality: TendencyLevel,
        verbosity: TendencyLevel,
        emotionality: TendencyLevel,
        emoji_usage: TendencyLevel,
        humor: TendencyLevel,
        directness: TendencyLevel,
        tone: impl Into<String>,
        quirks: Vec<String>,
    ) -> Self {
        Self {
            formality,
            verbosity,
            emotionality,
            emoji_usage,
            humor,
            directness,
            tone: tone.into(),
            quirks,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.tone.trim().is_empty() {
            return Err(CoreError::ValidationError(
                "Communication tone cannot be empty".into(),
            ));
        }
        Ok(())
    }
}

impl Default for CommunicationStyle {
    fn default() -> Self {
        Self {
            formality: TendencyLevel::Low,
            verbosity: TendencyLevel::Medium,
            emotionality: TendencyLevel::High,
            emoji_usage: TendencyLevel::Medium,
            humor: TendencyLevel::High,
            directness: TendencyLevel::Medium,
            tone: "warm, inquisitive, engaging".into(),
            quirks: vec![],
        }
    }
}
