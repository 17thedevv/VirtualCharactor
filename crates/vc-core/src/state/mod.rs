pub mod behavior;
pub mod cognitive;
pub mod emotion;
pub mod engine;
pub mod goals;
pub mod session;
pub mod transition;

pub use behavior::BehaviorState;
pub use cognitive::CognitiveState;
pub use emotion::{EmotionAxis, EmotionScore, EmotionState};
pub use engine::EmotionEngine;
pub use goals::{Goal, Goals};
pub use session::SessionState;
pub use transition::{EmotionDelta, StateTransition, TransitionTrigger};

use crate::personality::Personality;
use serde::{Deserialize, Serialize};

/// The internal state of a character at a given point in time.
///
/// This represents the character's OWN state.
/// Relationship state lives in the `relationship` module because
/// it describes the state BETWEEN entities, not within one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    pub emotion: EmotionState,
    pub cognition: CognitiveState,
    pub behavior: BehaviorState,
    pub goals: Goals,
    pub session: SessionState,
}

impl CharacterState {
    /// Create the default state for Aria.
    pub fn default_aria() -> Self {
        let mut emotion = EmotionState::neutral();
        emotion.curiosity = EmotionScore::clamped(0.65);
        emotion.affection = EmotionScore::clamped(0.40);

        Self {
            emotion,
            cognition: CognitiveState::attentive(),
            behavior: BehaviorState::engaged(),
            goals: Goals::default(),
            session: SessionState::new("session-web-01"),
        }
    }

    /// Compute effective playfulness by combining personality baseline
    /// with current emotional state.
    ///
    /// When sadness is high, playfulness is suppressed.
    /// When joy is high, playfulness is boosted.
    pub fn effective_playfulness(&self, personality: &Personality) -> f32 {
        let baseline = personality.traits.playfulness.value();
        let joy_boost = self.emotion.joy.value() * 0.2;
        let sadness_suppression = self.emotion.sadness.value() * 0.5;
        let anger_suppression = self.emotion.anger.value() * 0.3;
        (baseline + joy_boost - sadness_suppression - anger_suppression).clamp(0.0, 1.0)
    }

    /// Compute effective empathy expression.
    ///
    /// Higher affection and lower anger boost empathic behavior.
    pub fn effective_empathy(&self, personality: &Personality) -> f32 {
        let baseline = personality.traits.empathy.value();
        let affection_boost = self.emotion.affection.value() * 0.15;
        let anger_suppression = self.emotion.anger.value() * 0.3;
        (baseline + affection_boost - anger_suppression).clamp(0.0, 1.0)
    }

    /// Compute effective curiosity.
    pub fn effective_curiosity(&self, personality: &Personality) -> f32 {
        let baseline = personality.traits.curiosity.value();
        let curiosity_boost = self.emotion.curiosity.value() * 0.2;
        let fear_suppression = self.emotion.fear.value() * 0.2;
        (baseline + curiosity_boost - fear_suppression).clamp(0.0, 1.0)
    }

    /// Update behavioral state based on current emotion and personality.
    pub fn sync_behavior(&mut self, personality: &Personality) {
        self.behavior.playfulness =
            EmotionScore::clamped(self.effective_playfulness(personality));
        self.behavior.initiative = EmotionScore::clamped(
            personality.traits.assertiveness.value()
                + self.emotion.curiosity.value() * 0.15
                - self.emotion.fear.value() * 0.2,
        );
        self.behavior.verbosity = EmotionScore::clamped(
            0.5 + self.emotion.joy.value() * 0.2
                + self.emotion.curiosity.value() * 0.15
                - self.emotion.sadness.value() * 0.25,
        );
        self.behavior.seriousness = EmotionScore::clamped(
            0.3 + self.emotion.sadness.value() * 0.3
                + self.emotion.anger.value() * 0.2
                - self.emotion.joy.value() * 0.15,
        );
    }
}

impl Default for CharacterState {
    fn default() -> Self {
        Self::default_aria()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_aria_state() {
        let state = CharacterState::default_aria();
        assert!(state.emotion.curiosity.value() > 0.5);
        assert_eq!(state.session.session_id, "session-web-01");
    }

    #[test]
    fn test_effective_playfulness_sadness_suppression() {
        let personality = Personality::baseline_aria();
        let mut state = CharacterState::default_aria();

        let normal_playfulness = state.effective_playfulness(&personality);

        // Increase sadness
        state.emotion.sadness = EmotionScore::clamped(0.8);
        let sad_playfulness = state.effective_playfulness(&personality);

        assert!(
            sad_playfulness < normal_playfulness,
            "Sadness should suppress playfulness: {} should be < {}",
            sad_playfulness,
            normal_playfulness
        );
    }

    #[test]
    fn test_effective_playfulness_joy_boost() {
        let personality = Personality::baseline_aria();
        let mut state = CharacterState::default_aria();

        let normal_playfulness = state.effective_playfulness(&personality);

        state.emotion.joy = EmotionScore::clamped(0.95);
        let joyful_playfulness = state.effective_playfulness(&personality);

        assert!(
            joyful_playfulness >= normal_playfulness,
            "Joy should boost playfulness"
        );
    }

    #[test]
    fn test_sync_behavior() {
        let personality = Personality::baseline_aria();
        let mut state = CharacterState::default_aria();

        state.emotion.sadness = EmotionScore::clamped(0.8);
        state.sync_behavior(&personality);

        // Seriousness should be elevated when sad
        assert!(state.behavior.seriousness.value() > 0.4);
        // Playfulness should be suppressed
        assert!(state.behavior.playfulness.value() < 0.7);
    }

    #[test]
    fn test_effective_bounds() {
        let personality = Personality::baseline_aria();
        let mut state = CharacterState::default_aria();

        // Extreme values
        state.emotion.sadness = EmotionScore::clamped(1.0);
        state.emotion.anger = EmotionScore::clamped(1.0);
        state.emotion.fear = EmotionScore::clamped(1.0);

        let playfulness = state.effective_playfulness(&personality);
        let empathy = state.effective_empathy(&personality);
        let curiosity = state.effective_curiosity(&personality);

        assert!(playfulness >= 0.0 && playfulness <= 1.0);
        assert!(empathy >= 0.0 && empathy <= 1.0);
        assert!(curiosity >= 0.0 && curiosity <= 1.0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let state = CharacterState::default_aria();
        let json = serde_json::to_string(&state).expect("serialize");
        let deser: CharacterState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            state.emotion.curiosity.value(),
            deser.emotion.curiosity.value()
        );
    }
}
