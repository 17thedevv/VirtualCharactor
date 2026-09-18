use serde::{Deserialize, Serialize};
use super::metrics::RelationshipMetrics;

/// Delta changes to be applied to a Character ↔ Actor relationship.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationshipTransition {
    pub familiarity_delta: f32,
    pub closeness_delta: f32,
    pub trust_delta: f32,
    pub affection_delta: f32,
    pub tension_delta: f32,
}

impl Default for RelationshipTransition {
    fn default() -> Self {
        Self::zero()
    }
}

impl RelationshipTransition {
    /// Zero delta.
    pub const fn zero() -> Self {
        Self {
            familiarity_delta: 0.0,
            closeness_delta: 0.0,
            trust_delta: 0.0,
            affection_delta: 0.0,
            tension_delta: 0.0,
        }
    }

    /// Predefined positive interaction: slight boost in familiarity, trust, and affection.
    pub fn positive_interaction() -> Self {
        Self {
            familiarity_delta: 0.05,
            closeness_delta: 0.03,
            trust_delta: 0.03,
            affection_delta: 0.04,
            tension_delta: -0.02,
        }
    }

    /// Predefined deeply vulnerable or supportive interaction: boosts trust and closeness.
    pub fn vulnerable_interaction() -> Self {
        Self {
            familiarity_delta: 0.06,
            closeness_delta: 0.07,
            trust_delta: 0.06,
            affection_delta: 0.06,
            tension_delta: -0.04,
        }
    }

    /// Predefined conflicted interaction: increases tension, minor dampening of closeness and trust.
    pub fn conflicted_interaction() -> Self {
        Self {
            familiarity_delta: 0.02,
            closeness_delta: -0.03,
            trust_delta: -0.04,
            affection_delta: -0.03,
            tension_delta: 0.12,
        }
    }

    /// Scale all deltas by a scalar factor.
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            familiarity_delta: self.familiarity_delta * factor,
            closeness_delta: self.closeness_delta * factor,
            trust_delta: self.trust_delta * factor,
            affection_delta: self.affection_delta * factor,
            tension_delta: self.tension_delta * factor,
        }
    }

    /// Apply damping (e.g. 0.5 for half speed evolution) to enforce relationship stability.
    pub fn damped(&self, damping_factor: f32) -> Self {
        self.scale(damping_factor.clamp(0.0, 1.0))
    }

    /// Combine another transition with this one.
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            familiarity_delta: self.familiarity_delta + other.familiarity_delta,
            closeness_delta: self.closeness_delta + other.closeness_delta,
            trust_delta: self.trust_delta + other.trust_delta,
            affection_delta: self.affection_delta + other.affection_delta,
            tension_delta: self.tension_delta + other.tension_delta,
        }
    }

    /// Apply these transition deltas to a RelationshipMetrics instance safely.
    pub fn apply_to(&self, metrics: &mut RelationshipMetrics) {
        metrics.familiarity.apply_delta(self.familiarity_delta);
        metrics.closeness.apply_delta(self.closeness_delta);
        metrics.trust.apply_delta(self.trust_delta);
        metrics.affection.apply_delta(self.affection_delta);
        metrics.tension.apply_delta(self.tension_delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_transition_application() {
        let mut m = RelationshipMetrics::stranger();
        let trans = RelationshipTransition::positive_interaction();
        trans.apply_to(&mut m);

        assert!(m.familiarity.value() > 0.0);
        assert!(m.trust.value() > 0.15);
        assert!(m.affection.value() > 0.10);
    }

    #[test]
    fn test_scale_and_damp() {
        let trans = RelationshipTransition {
            familiarity_delta: 0.1,
            closeness_delta: 0.1,
            trust_delta: 0.1,
            affection_delta: 0.1,
            tension_delta: 0.1,
        };

        let scaled = trans.scale(0.5);
        assert!((scaled.trust_delta - 0.05).abs() < f32::EPSILON);
    }
}
