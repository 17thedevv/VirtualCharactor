use serde::{Deserialize, Serialize};

/// Emotional state of the character.
///
/// This is intentionally its own module so that a future emotion engine
/// can expand it independently (e.g., compound emotions, decay, triggers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionState {
    pub primary_emotion: String,
    pub intensity: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_defaults() {
        let e = EmotionState {
            primary_emotion: "neutral".into(),
            intensity: 0.0,
        };
        assert_eq!(e.primary_emotion, "neutral");
        assert_eq!(e.intensity, 0.0);
    }

    #[test]
    fn test_emotion_intensity_range() {
        let e = EmotionState {
            primary_emotion: "joy".into(),
            intensity: 0.8,
        };
        assert!(e.intensity >= 0.0 && e.intensity <= 1.0);
    }
}
