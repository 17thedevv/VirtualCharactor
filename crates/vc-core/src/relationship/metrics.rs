use serde::{Deserialize, Serialize};
use crate::error::{CoreError, Result};

/// Bounded float score [0.0, 1.0] representing a relationship dimension.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RelationshipScore(f32);

impl RelationshipScore {
    pub const MIN: f32 = 0.0;
    pub const MAX: f32 = 1.0;

    /// Create a new RelationshipScore, validating that 0.0 <= val <= 1.0 and is not NaN.
    pub fn new(val: f32) -> Result<Self> {
        if val.is_nan() || val < Self::MIN || val > Self::MAX {
            return Err(CoreError::ValidationError(format!(
                "RelationshipScore must be between 0.0 and 1.0, got {}",
                val
            )));
        }
        Ok(Self(val))
    }

    /// Create a RelationshipScore clamped safely into [0.0, 1.0].
    pub fn clamped(val: f32) -> Self {
        if val.is_nan() {
            Self(0.0)
        } else {
            Self(val.clamp(Self::MIN, Self::MAX))
        }
    }

    /// Return the raw float value.
    pub fn value(&self) -> f32 {
        self.0
    }

    /// Apply a delta, clamping the result to [0.0, 1.0].
    pub fn apply_delta(&mut self, delta: f32) {
        *self = Self::clamped(self.0 + delta);
    }
}

impl Default for RelationshipScore {
    fn default() -> Self {
        Self(0.0)
    }
}

impl From<RelationshipScore> for f32 {
    fn from(score: RelationshipScore) -> Self {
        score.0
    }
}

/// The 5 core psychological metrics of a Character ↔ Actor relationship.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationshipMetrics {
    /// Familiarity: How well the character recognizes and knows the actor through accumulated interactions [0, 1]
    pub familiarity: RelationshipScore,
    /// Closeness: Psychological proximity, emotional intimacy and comfort [0, 1]
    pub closeness: RelationshipScore,
    /// Trust: Perceived reliability, psychological safety and vulnerability tolerance [0, 1]
    pub trust: RelationshipScore,
    /// Affection: Fondness, positive sentiment and liking [0, 1]
    pub affection: RelationshipScore,
    /// Tension: Interpersonal friction, unresolved conflict, or guardedness [0, 1]
    pub tension: RelationshipScore,
}

impl Default for RelationshipMetrics {
    fn default() -> Self {
        Self::stranger()
    }
}

impl RelationshipMetrics {
    /// Baseline metrics for an unknown Stranger.
    pub fn stranger() -> Self {
        Self {
            familiarity: RelationshipScore::clamped(0.0),
            closeness: RelationshipScore::clamped(0.05),
            trust: RelationshipScore::clamped(0.15), // Neutral cautious baseline trust
            affection: RelationshipScore::clamped(0.10),
            tension: RelationshipScore::clamped(0.0),
        }
    }

    /// Baseline metrics for a Familiar Companion.
    pub fn companion() -> Self {
        Self {
            familiarity: RelationshipScore::clamped(0.60),
            closeness: RelationshipScore::clamped(0.55),
            trust: RelationshipScore::clamped(0.65),
            affection: RelationshipScore::clamped(0.60),
            tension: RelationshipScore::clamped(0.05),
        }
    }

    /// Validate that all metrics are within valid bounds.
    pub fn validate(&self) -> Result<()> {
        RelationshipScore::new(self.familiarity.value())?;
        RelationshipScore::new(self.closeness.value())?;
        RelationshipScore::new(self.trust.value())?;
        RelationshipScore::new(self.affection.value())?;
        RelationshipScore::new(self.tension.value())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_validation() {
        assert!(RelationshipScore::new(0.5).is_ok());
        assert!(RelationshipScore::new(0.0).is_ok());
        assert!(RelationshipScore::new(1.0).is_ok());
        assert!(RelationshipScore::new(-0.1).is_err());
        assert!(RelationshipScore::new(1.1).is_err());
        assert!(RelationshipScore::new(f32::NAN).is_err());
    }

    #[test]
    fn test_score_clamping() {
        assert_eq!(RelationshipScore::clamped(1.5).value(), 1.0);
        assert_eq!(RelationshipScore::clamped(-0.5).value(), 0.0);
        assert_eq!(RelationshipScore::clamped(f32::NAN).value(), 0.0);
    }

    #[test]
    fn test_score_delta() {
        let mut score = RelationshipScore::clamped(0.5);
        score.apply_delta(0.2);
        assert!((score.value() - 0.7).abs() < f32::EPSILON);
        score.apply_delta(0.5);
        assert_eq!(score.value(), 1.0);
        score.apply_delta(-1.5);
        assert_eq!(score.value(), 0.0);
    }
}
