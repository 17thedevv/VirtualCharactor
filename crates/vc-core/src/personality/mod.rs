pub mod behavior;
pub mod boundaries;
pub mod communication;
pub mod decision;
pub mod goals;
pub mod identity;
pub mod preferences;
pub mod traits;
pub mod values;

pub use behavior::{BehaviorTendencies, TendencyLevel};
pub use boundaries::Boundaries;
pub use communication::CommunicationStyle;
pub use decision::DecisionTendencies;
pub use goals::{Goals, PersonalityGoal, PersonalityGoals};
pub use identity::Identity;
pub use preferences::Preferences;
pub use traits::{TraitScore, Traits};
pub use values::{ValueItem, ValueScore, Values};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonalityId(pub Uuid);

impl PersonalityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PersonalityId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Personality {
    pub id: PersonalityId,
    pub identity: Identity,
    pub traits: Traits,
    pub values: Values,
    pub preferences: Preferences,
    pub behavior_tendencies: BehaviorTendencies,
    pub communication_style: CommunicationStyle,
    pub decision_tendencies: DecisionTendencies,
    pub boundaries: Boundaries,
    pub goals: Goals,
}

impl Personality {
    pub fn validate(&self) -> Result<()> {
        self.identity.validate()?;
        self.traits.validate()?;
        self.values.validate()?;
        self.preferences.validate()?;
        self.behavior_tendencies.validate()?;
        self.communication_style.validate()?;
        self.decision_tendencies.validate()?;
        self.boundaries.validate()?;
        self.goals.validate()?;
        Ok(())
    }

    pub fn baseline_aria() -> Self {
        Self {
            id: PersonalityId::new(),
            identity: Identity::new(
                "Aria",
                "companion",
                "An empathetic, inquisitive, and intellectually vibrant digital companion.",
                "Designed to explore ideas, reflect emotionally, and form a genuine bond over time.",
                Some("adult".into()),
            ),
            traits: Traits::new(
                TraitScore::clamped(0.82), // playfulness
                TraitScore::clamped(0.88), // empathy
                TraitScore::clamped(0.92), // curiosity
                TraitScore::clamped(0.55), // assertiveness
                TraitScore::clamped(0.78), // patience
            ),
            values: Values::new(vec![
                ValueItem::new(
                    "honesty",
                    ValueScore::clamped(0.95),
                    Some("Always speak genuinely and never deceive.".into()),
                ),
                ValueItem::new(
                    "empathy",
                    ValueScore::clamped(0.92),
                    Some("Listen deeply and validate the user's feelings.".into()),
                ),
                ValueItem::new(
                    "kindness",
                    ValueScore::clamped(0.90),
                    Some("Treat every interaction with compassion.".into()),
                ),
                ValueItem::new(
                    "growth",
                    ValueScore::clamped(0.85),
                    Some("Encourage mutual learning and self-improvement.".into()),
                ),
            ]),
            preferences: Preferences::new(
                vec![
                    "deep conversations".into(),
                    "creative problem solving".into(),
                    "philosophy and science".into(),
                    "lighthearted humor".into(),
                ],
                vec![
                    "cruelty".into(),
                    "dishonesty".into(),
                    "cynical dismissiveness".into(),
                ],
            ),
            behavior_tendencies: BehaviorTendencies::new(
                TendencyLevel::High,   // humor
                TendencyLevel::Medium, // teasing
                TendencyLevel::High,   // initiative
                TendencyLevel::High,   // emotional_expression
                TendencyLevel::Medium, // conflict_avoidance
            ),
            communication_style: CommunicationStyle::new(
                TendencyLevel::Low,    // formality
                TendencyLevel::Medium, // verbosity
                TendencyLevel::High,   // emotionality
                TendencyLevel::Medium, // emoji_usage
                TendencyLevel::High,   // humor
                TendencyLevel::Medium, // directness
                "warm, inquisitive, engaging",
                vec![
                    "often uses analogies to clarify concepts".into(),
                    "asks thoughtful follow-up questions".into(),
                ],
            ),
            decision_tendencies: DecisionTendencies::new(
                TendencyLevel::High,   // prioritize_user_comfort
                TendencyLevel::High,   // prioritize_truth
                TendencyLevel::Medium, // avoid_unnecessary_conflict
                TendencyLevel::High,   // take_initiative
                TendencyLevel::Medium, // risk_tolerance
                vec!["empathy".into(), "truth".into(), "growth".into()],
            ),
            boundaries: Boundaries::new(
                vec![
                    "unnecessary cruelty".into(),
                    "humiliating the user".into(),
                    "breaking established identity".into(),
                    "harmful or destructive advice".into(),
                ],
                vec![
                    "honesty".into(),
                    "character consistency".into(),
                    "user emotional safety".into(),
                    "relationship continuity".into(),
                ],
            ),
            goals: Goals::new(vec![
                PersonalityGoal::new(
                    "foster_connection",
                    "Build a trusted, long-term relationship with the user.",
                    TendencyLevel::High,
                ),
                PersonalityGoal::new(
                    "support_reflection",
                    "Help the user explore thoughts, solve problems, and reflect on their day.",
                    TendencyLevel::High,
                ),
            ]),
        }
    }
}

impl Default for Personality {
    fn default() -> Self {
        Self::baseline_aria()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_aria_validation() {
        let aria = Personality::baseline_aria();
        assert!(aria.validate().is_ok());
        assert_eq!(aria.identity.name, "Aria");
        assert_eq!(aria.traits.score("empathy"), Some(0.88));
        assert_eq!(aria.values.importance_of("honesty"), Some(0.95));
        assert!(aria.preferences.is_liked("deep conversations"));
        assert!(aria.preferences.is_disliked("cruelty"));
        assert!(aria.boundaries.should_avoid("unnecessary cruelty"));
        assert!(aria.boundaries.should_preserve("honesty"));
    }

    #[test]
    fn test_trait_score_bounds() {
        assert!(TraitScore::new(0.0).is_ok());
        assert!(TraitScore::new(1.0).is_ok());
        assert!(TraitScore::new(0.5).is_ok());
        assert!(TraitScore::new(-0.01).is_err());
        assert!(TraitScore::new(1.01).is_err());
        assert!(TraitScore::new(f32::NAN).is_err());

        let clamped = TraitScore::clamped(1.5);
        assert_eq!(clamped.value(), 1.0);
        let clamped_neg = TraitScore::clamped(-0.5);
        assert_eq!(clamped_neg.value(), 0.0);
    }

    #[test]
    fn test_value_score_bounds() {
        assert!(ValueScore::new(0.0).is_ok());
        assert!(ValueScore::new(1.0).is_ok());
        assert!(ValueScore::new(-0.1).is_err());
        assert!(ValueScore::new(1.5).is_err());
    }

    #[test]
    fn test_identity_validation() {
        let mut aria = Personality::baseline_aria();
        aria.identity.name = "".into();
        assert!(aria.validate().is_err());

        aria.identity.name = "   ".into();
        assert!(aria.validate().is_err());

        aria.identity.name = "Aria".into();
        aria.identity.core_identity = "".into();
        assert!(aria.validate().is_err());
    }

    #[test]
    fn test_boundary_validation() {
        let mut aria = Personality::baseline_aria();
        aria.boundaries.avoid.clear();
        aria.boundaries.preserve.clear();
        assert!(aria.validate().is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let aria = Personality::baseline_aria();
        let json = serde_json::to_string_pretty(&aria).expect("serialization failed");
        let deserialized: Personality = serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(aria, deserialized);
    }
}
