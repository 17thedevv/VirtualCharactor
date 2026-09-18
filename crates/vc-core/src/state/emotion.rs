use serde::{Deserialize, Serialize};

/// A single emotion intensity, clamped to `[0.0, 1.0]`.
///
/// This is a Value Object: immutable once constructed, enforces its invariant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EmotionScore(f32);

impl EmotionScore {
    /// Create a new score, returning an error if out of range or NaN.
    pub fn new(value: f32) -> crate::Result<Self> {
        if value.is_nan() || value < 0.0 || value > 1.0 {
            return Err(crate::CoreError::ValidationError(format!(
                "EmotionScore must be in [0.0, 1.0], got {}",
                value
            )));
        }
        Ok(Self(value))
    }

    /// Create a score, clamping to `[0.0, 1.0]` without error.
    pub fn clamped(value: f32) -> Self {
        if value.is_nan() {
            Self(0.0)
        } else {
            Self(value.clamp(0.0, 1.0))
        }
    }

    /// The raw f32 value.
    pub fn value(self) -> f32 {
        self.0
    }

    /// Apply a delta (positive or negative) and clamp.
    pub fn apply_delta(self, delta: f32) -> Self {
        Self::clamped(self.0 + delta)
    }

    /// Exponential decay toward a baseline.
    ///
    /// `S(t) = baseline + (S₀ - baseline) × e^(-λ × dt)`
    pub fn decay(self, baseline: f32, lambda: f32, elapsed_secs: f64) -> Self {
        let decayed = baseline + (self.0 - baseline) * (-lambda * elapsed_secs as f32).exp();
        Self::clamped(decayed)
    }
}

impl Default for EmotionScore {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Multi-dimensional emotional state of the character at a point in time.
///
/// Each axis is an `EmotionScore` in `[0.0, 1.0]`. The character can
/// experience multiple emotions simultaneously (e.g., joy=0.7, embarrassment=0.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmotionState {
    pub joy: EmotionScore,
    pub sadness: EmotionScore,
    pub anger: EmotionScore,
    pub fear: EmotionScore,
    pub surprise: EmotionScore,
    pub affection: EmotionScore,
    pub embarrassment: EmotionScore,
    pub curiosity: EmotionScore,
    /// Unix timestamp (seconds) of last update.
    pub last_updated: u64,
}

/// Named emotion axis for iteration and lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionAxis {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Affection,
    Embarrassment,
    Curiosity,
}

impl EmotionAxis {
    /// All axes in canonical order.
    pub const ALL: [EmotionAxis; 8] = [
        EmotionAxis::Joy,
        EmotionAxis::Sadness,
        EmotionAxis::Anger,
        EmotionAxis::Fear,
        EmotionAxis::Surprise,
        EmotionAxis::Affection,
        EmotionAxis::Embarrassment,
        EmotionAxis::Curiosity,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Joy => "joy",
            Self::Sadness => "sadness",
            Self::Anger => "anger",
            Self::Fear => "fear",
            Self::Surprise => "surprise",
            Self::Affection => "affection",
            Self::Embarrassment => "embarrassment",
            Self::Curiosity => "curiosity",
        }
    }

    /// Valence weight for computing overall valence.
    /// Positive emotions get positive weights, negative emotions get negative.
    pub fn valence_weight(self) -> f32 {
        match self {
            Self::Joy => 1.0,
            Self::Affection => 0.8,
            Self::Curiosity => 0.5,
            Self::Surprise => 0.2,
            Self::Embarrassment => -0.3,
            Self::Fear => -0.6,
            Self::Sadness => -0.8,
            Self::Anger => -0.7,
        }
    }

    /// Arousal contribution. High-arousal emotions contribute more.
    pub fn arousal_weight(self) -> f32 {
        match self {
            Self::Surprise => 0.9,
            Self::Anger => 0.85,
            Self::Fear => 0.8,
            Self::Joy => 0.7,
            Self::Curiosity => 0.6,
            Self::Embarrassment => 0.5,
            Self::Affection => 0.4,
            Self::Sadness => 0.3,
        }
    }

    /// Default decay rate (λ). Higher = faster decay.
    pub fn default_decay_rate(self) -> f32 {
        match self {
            // Fast-decaying emotions
            Self::Surprise => 0.15,
            Self::Anger => 0.08,
            Self::Embarrassment => 0.10,
            Self::Fear => 0.07,
            // Medium-decaying emotions
            Self::Joy => 0.04,
            Self::Sadness => 0.03,
            Self::Curiosity => 0.05,
            // Slow-decaying emotions
            Self::Affection => 0.02,
        }
    }
}

impl EmotionState {
    /// Create a neutral baseline state.
    pub fn neutral() -> Self {
        Self {
            joy: EmotionScore::clamped(0.30),
            sadness: EmotionScore::clamped(0.05),
            anger: EmotionScore::clamped(0.02),
            fear: EmotionScore::clamped(0.03),
            surprise: EmotionScore::clamped(0.05),
            affection: EmotionScore::clamped(0.25),
            embarrassment: EmotionScore::clamped(0.02),
            curiosity: EmotionScore::clamped(0.45),
            last_updated: now_secs(),
        }
    }

    /// Get score for a specific axis.
    pub fn get(&self, axis: EmotionAxis) -> EmotionScore {
        match axis {
            EmotionAxis::Joy => self.joy,
            EmotionAxis::Sadness => self.sadness,
            EmotionAxis::Anger => self.anger,
            EmotionAxis::Fear => self.fear,
            EmotionAxis::Surprise => self.surprise,
            EmotionAxis::Affection => self.affection,
            EmotionAxis::Embarrassment => self.embarrassment,
            EmotionAxis::Curiosity => self.curiosity,
        }
    }

    /// Set score for a specific axis.
    pub fn set(&mut self, axis: EmotionAxis, score: EmotionScore) {
        match axis {
            EmotionAxis::Joy => self.joy = score,
            EmotionAxis::Sadness => self.sadness = score,
            EmotionAxis::Anger => self.anger = score,
            EmotionAxis::Fear => self.fear = score,
            EmotionAxis::Surprise => self.surprise = score,
            EmotionAxis::Affection => self.affection = score,
            EmotionAxis::Embarrassment => self.embarrassment = score,
            EmotionAxis::Curiosity => self.curiosity = score,
        }
        self.last_updated = now_secs();
    }

    /// The emotion axis with the highest score.
    pub fn dominant_emotion(&self) -> (EmotionAxis, EmotionScore) {
        EmotionAxis::ALL
            .iter()
            .map(|&axis| (axis, self.get(axis)))
            .max_by(|a, b| a.1.value().partial_cmp(&b.1.value()).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((EmotionAxis::Joy, EmotionScore::default()))
    }

    /// Compute overall valence in `[-1.0, 1.0]`.
    ///
    /// Positive = pleasant emotional state, Negative = unpleasant.
    pub fn valence(&self) -> f32 {
        let mut weighted_sum = 0.0f32;
        let mut weight_sum = 0.0f32;

        for axis in EmotionAxis::ALL {
            let score = self.get(axis).value();
            let weight = axis.valence_weight().abs();
            weighted_sum += score * axis.valence_weight();
            weight_sum += score * weight;
        }

        if weight_sum < 0.001 {
            0.0
        } else {
            (weighted_sum / weight_sum).clamp(-1.0, 1.0)
        }
    }

    /// Compute overall arousal in `[0.0, 1.0]`.
    ///
    /// High arousal = excited/agitated, Low arousal = calm/serene.
    pub fn arousal(&self) -> f32 {
        let mut weighted_sum = 0.0f32;
        let mut weight_sum = 0.0f32;

        for axis in EmotionAxis::ALL {
            let score = self.get(axis).value();
            let weight = axis.arousal_weight();
            weighted_sum += score * weight;
            weight_sum += weight;
        }

        if weight_sum < 0.001 {
            0.0
        } else {
            (weighted_sum / weight_sum).clamp(0.0, 1.0)
        }
    }

    /// Apply an EmotionDelta to produce a new state.
    pub fn apply_delta(&mut self, delta: &super::transition::EmotionDelta) {
        self.joy = self.joy.apply_delta(delta.joy);
        self.sadness = self.sadness.apply_delta(delta.sadness);
        self.anger = self.anger.apply_delta(delta.anger);
        self.fear = self.fear.apply_delta(delta.fear);
        self.surprise = self.surprise.apply_delta(delta.surprise);
        self.affection = self.affection.apply_delta(delta.affection);
        self.embarrassment = self.embarrassment.apply_delta(delta.embarrassment);
        self.curiosity = self.curiosity.apply_delta(delta.curiosity);
        self.last_updated = now_secs();
    }

    /// Decay all axes toward their neutral baseline over elapsed time.
    pub fn decay(&mut self, elapsed_secs: u64) {
        let neutral = Self::neutral();
        let dt = elapsed_secs as f64;

        for axis in EmotionAxis::ALL {
            let baseline = neutral.get(axis).value();
            let lambda = axis.default_decay_rate();
            let decayed = self.get(axis).decay(baseline, lambda, dt);
            self.set(axis, decayed);
        }
    }

    /// Return all axes as name-value pairs for serialization.
    pub fn as_map(&self) -> Vec<(&'static str, f32)> {
        EmotionAxis::ALL
            .iter()
            .map(|&axis| (axis.name(), self.get(axis).value()))
            .collect()
    }
}

impl Default for EmotionState {
    fn default() -> Self {
        Self::neutral()
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_score_bounds() {
        assert!(EmotionScore::new(0.0).is_ok());
        assert!(EmotionScore::new(1.0).is_ok());
        assert!(EmotionScore::new(0.5).is_ok());
        assert!(EmotionScore::new(-0.01).is_err());
        assert!(EmotionScore::new(1.01).is_err());
        assert!(EmotionScore::new(f32::NAN).is_err());
    }

    #[test]
    fn test_emotion_score_clamped() {
        assert_eq!(EmotionScore::clamped(1.5).value(), 1.0);
        assert_eq!(EmotionScore::clamped(-0.5).value(), 0.0);
        assert_eq!(EmotionScore::clamped(f32::NAN).value(), 0.0);
        assert_eq!(EmotionScore::clamped(0.42).value(), 0.42);
    }

    #[test]
    fn test_emotion_score_apply_delta() {
        let score = EmotionScore::clamped(0.5);
        assert!((score.apply_delta(0.3).value() - 0.8).abs() < 0.001);
        assert!((score.apply_delta(-0.3).value() - 0.2).abs() < 0.001);
        // Clamping
        assert_eq!(score.apply_delta(0.8).value(), 1.0);
        assert_eq!(score.apply_delta(-0.8).value(), 0.0);
    }

    #[test]
    fn test_emotion_score_decay() {
        let score = EmotionScore::clamped(0.9);
        let decayed = score.decay(0.3, 0.1, 10.0);
        // Should decay toward baseline 0.3
        assert!(decayed.value() < 0.9);
        assert!(decayed.value() > 0.3);
    }

    #[test]
    fn test_neutral_state() {
        let state = EmotionState::neutral();
        assert!(state.joy.value() > 0.0);
        assert!(state.curiosity.value() > 0.0);
        assert!(state.anger.value() < 0.1);
    }

    #[test]
    fn test_dominant_emotion() {
        let mut state = EmotionState::neutral();
        state.joy = EmotionScore::clamped(0.95);
        let (axis, score) = state.dominant_emotion();
        assert_eq!(axis, EmotionAxis::Joy);
        assert_eq!(score.value(), 0.95);
    }

    #[test]
    fn test_valence_positive() {
        let mut state = EmotionState::neutral();
        state.joy = EmotionScore::clamped(0.9);
        state.affection = EmotionScore::clamped(0.8);
        state.sadness = EmotionScore::clamped(0.0);
        state.anger = EmotionScore::clamped(0.0);
        assert!(state.valence() > 0.0);
    }

    #[test]
    fn test_valence_negative() {
        let mut state = EmotionState::neutral();
        state.joy = EmotionScore::clamped(0.0);
        state.affection = EmotionScore::clamped(0.0);
        state.curiosity = EmotionScore::clamped(0.0);
        state.sadness = EmotionScore::clamped(0.9);
        state.anger = EmotionScore::clamped(0.8);
        assert!(state.valence() < 0.0);
    }

    #[test]
    fn test_arousal() {
        let mut state = EmotionState::neutral();
        state.surprise = EmotionScore::clamped(0.9);
        state.anger = EmotionScore::clamped(0.8);
        let arousal = state.arousal();
        assert!(arousal > 0.3);
    }

    #[test]
    fn test_decay_reduces_extreme_values() {
        let mut state = EmotionState::neutral();
        state.joy = EmotionScore::clamped(0.95);
        state.anger = EmotionScore::clamped(0.80);

        let initial_joy = state.joy.value();
        let initial_anger = state.anger.value();

        state.decay(60); // 60 seconds

        // Both should decay toward neutral
        assert!(state.joy.value() < initial_joy);
        assert!(state.anger.value() < initial_anger);
    }

    #[test]
    fn test_as_map() {
        let state = EmotionState::neutral();
        let map = state.as_map();
        assert_eq!(map.len(), 8);
        assert_eq!(map[0].0, "joy");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let state = EmotionState::neutral();
        let json = serde_json::to_string(&state).expect("serialize");
        let deser: EmotionState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(state.joy.value(), deser.joy.value());
        assert_eq!(state.curiosity.value(), deser.curiosity.value());
    }
}
