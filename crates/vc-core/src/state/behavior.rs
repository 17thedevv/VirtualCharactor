use serde::{Deserialize, Serialize};
use super::emotion::EmotionScore;

/// Behavioral state — the character's current behavioral tendencies.
///
/// These are dynamic and influenced by emotion. For example, a character
/// whose personality baseline is `playfulness = 0.8` might temporarily
/// have `playfulness = 0.3` when experiencing high sadness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehaviorState {
    /// How playful the character is currently being.
    pub playfulness: EmotionScore,
    /// How serious the character is currently being.
    pub seriousness: EmotionScore,
    /// How verbose/talkative the character is.
    pub verbosity: EmotionScore,
    /// How much the character takes the initiative.
    pub initiative: EmotionScore,
    /// Current activity label.
    pub current_activity: String,
}

impl BehaviorState {
    /// Default engaged behavior.
    pub fn engaged() -> Self {
        Self {
            playfulness: EmotionScore::clamped(0.65),
            seriousness: EmotionScore::clamped(0.40),
            verbosity: EmotionScore::clamped(0.55),
            initiative: EmotionScore::clamped(0.60),
            current_activity: "active_listening".into(),
        }
    }
}

impl Default for BehaviorState {
    fn default() -> Self {
        Self::engaged()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engaged_defaults() {
        let behavior = BehaviorState::engaged();
        assert!(behavior.playfulness.value() > 0.5);
        assert_eq!(behavior.current_activity, "active_listening");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let behavior = BehaviorState::engaged();
        let json = serde_json::to_string(&behavior).expect("serialize");
        let deser: BehaviorState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.current_activity, behavior.current_activity);
    }
}
