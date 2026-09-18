use serde::{Deserialize, Serialize};

/// Describes the change to apply to each emotion axis.
///
/// Values can be positive (increase) or negative (decrease).
/// When applied, results are clamped to `[0.0, 1.0]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmotionDelta {
    pub joy: f32,
    pub sadness: f32,
    pub anger: f32,
    pub fear: f32,
    pub surprise: f32,
    pub affection: f32,
    pub embarrassment: f32,
    pub curiosity: f32,
}

impl EmotionDelta {
    /// A delta that changes nothing.
    pub fn zero() -> Self {
        Self {
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            affection: 0.0,
            embarrassment: 0.0,
            curiosity: 0.0,
        }
    }

    /// Check if this delta has any non-zero effect.
    pub fn is_zero(&self) -> bool {
        self.joy.abs() < 0.001
            && self.sadness.abs() < 0.001
            && self.anger.abs() < 0.001
            && self.fear.abs() < 0.001
            && self.surprise.abs() < 0.001
            && self.affection.abs() < 0.001
            && self.embarrassment.abs() < 0.001
            && self.curiosity.abs() < 0.001
    }

    /// Scale all deltas by a factor (e.g., personality modulation).
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            joy: self.joy * factor,
            sadness: self.sadness * factor,
            anger: self.anger * factor,
            fear: self.fear * factor,
            surprise: self.surprise * factor,
            affection: self.affection * factor,
            embarrassment: self.embarrassment * factor,
            curiosity: self.curiosity * factor,
        }
    }

    /// Combine two deltas by addition.
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            joy: self.joy + other.joy,
            sadness: self.sadness + other.sadness,
            anger: self.anger + other.anger,
            fear: self.fear + other.fear,
            surprise: self.surprise + other.surprise,
            affection: self.affection + other.affection,
            embarrassment: self.embarrassment + other.embarrassment,
            curiosity: self.curiosity + other.curiosity,
        }
    }
}

impl Default for EmotionDelta {
    fn default() -> Self {
        Self::zero()
    }
}

/// What triggered a state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionTrigger {
    /// The user sent a message.
    UserMessage(String),
    /// A memory was activated / recalled.
    MemoryActivation(String),
    /// Natural temporal decay.
    TemporalDecay,
    /// A system-level event.
    SystemEvent(String),
}

/// A complete state transition record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub trigger: TransitionTrigger,
    pub emotion_delta: EmotionDelta,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_delta() {
        let delta = EmotionDelta::zero();
        assert!(delta.is_zero());
    }

    #[test]
    fn test_non_zero_delta() {
        let mut delta = EmotionDelta::zero();
        delta.joy = 0.1;
        assert!(!delta.is_zero());
    }

    #[test]
    fn test_scale() {
        let delta = EmotionDelta {
            joy: 0.2,
            sadness: -0.1,
            ..EmotionDelta::zero()
        };
        let scaled = delta.scale(2.0);
        assert!((scaled.joy - 0.4).abs() < 0.001);
        assert!((scaled.sadness - (-0.2)).abs() < 0.001);
    }

    #[test]
    fn test_combine() {
        let a = EmotionDelta {
            joy: 0.1,
            sadness: 0.2,
            ..EmotionDelta::zero()
        };
        let b = EmotionDelta {
            joy: 0.05,
            curiosity: 0.3,
            ..EmotionDelta::zero()
        };
        let combined = a.combine(&b);
        assert!((combined.joy - 0.15).abs() < 0.001);
        assert!((combined.sadness - 0.2).abs() < 0.001);
        assert!((combined.curiosity - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_state_transition_creation() {
        let transition = StateTransition {
            trigger: TransitionTrigger::UserMessage("hello".into()),
            emotion_delta: EmotionDelta {
                joy: 0.15,
                curiosity: 0.10,
                ..EmotionDelta::zero()
            },
            timestamp: 1000,
        };
        assert!(!transition.emotion_delta.is_zero());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let delta = EmotionDelta {
            joy: 0.1,
            sadness: -0.05,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.2,
            affection: 0.15,
            embarrassment: 0.0,
            curiosity: 0.1,
        };
        let json = serde_json::to_string(&delta).expect("serialize");
        let deser: EmotionDelta = serde_json::from_str(&json).expect("deserialize");
        assert!((delta.joy - deser.joy).abs() < 0.001);
        assert!((delta.surprise - deser.surprise).abs() < 0.001);
    }
}
