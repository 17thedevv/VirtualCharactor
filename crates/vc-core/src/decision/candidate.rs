use serde::{Deserialize, Serialize};
use super::action::Action;

/// A candidate action evaluated by the Decision Engine during deliberation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionCandidate {
    /// The action option considered
    pub action: Action,
    /// Model/heuristic confidence in this action choice [0.0, 1.0]
    pub confidence: f32,
    /// Composite multidimensional fit score [0.0, 1.0]
    pub score: f32,
    /// Brief explanation of why this candidate was generated
    pub rationale: String,
}

impl DecisionCandidate {
    pub fn new(action: Action, confidence: f32, score: f32, rationale: impl Into<String>) -> Self {
        Self {
            action,
            confidence: confidence.clamp(0.0, 1.0),
            score: score.clamp(0.0, 1.0),
            rationale: rationale.into(),
        }
    }
}
