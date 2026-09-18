use serde::{Deserialize, Serialize};
use super::emotion::EmotionScore;

/// Cognitive state of the character — what it's attending to and how.
///
/// Separate from emotion: cognition is about awareness and processing,
/// not feeling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CognitiveState {
    /// How much attention the character is paying.
    pub attention: EmotionScore,
    /// Level of confusion about the current context.
    pub confusion: EmotionScore,
    /// Active curiosity about the topic (cognitive, not emotional).
    pub curiosity: EmotionScore,
    /// Confidence in understanding the situation.
    pub confidence: EmotionScore,
    /// Depth of focus on the current topic.
    pub focus: EmotionScore,
    /// What topic the character is currently engaged with.
    pub current_topic: Option<String>,
}

impl CognitiveState {
    /// Default attentive state.
    pub fn attentive() -> Self {
        Self {
            attention: EmotionScore::clamped(0.80),
            confusion: EmotionScore::clamped(0.05),
            curiosity: EmotionScore::clamped(0.60),
            confidence: EmotionScore::clamped(0.70),
            focus: EmotionScore::clamped(0.75),
            current_topic: None,
        }
    }
}

impl Default for CognitiveState {
    fn default() -> Self {
        Self::attentive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attentive_defaults() {
        let cog = CognitiveState::attentive();
        assert!(cog.attention.value() > 0.5);
        assert!(cog.confusion.value() < 0.2);
        assert!(cog.current_topic.is_none());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut cog = CognitiveState::attentive();
        cog.current_topic = Some("Rust architecture".into());
        let json = serde_json::to_string(&cog).expect("serialize");
        let deser: CognitiveState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.current_topic, Some("Rust architecture".into()));
    }
}
