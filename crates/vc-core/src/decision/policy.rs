use serde::{Deserialize, Serialize};

/// Behavioral policy parameters guiding how an action should be expressed by the LLM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehaviorPolicy {
    /// Desired conversational tone
    pub tone: String,
    /// Desired verbosity level [0.0 (ultra-terse) to 1.0 (deeply elaborative)]
    pub verbosity: f32,
    /// Proactive initiative [0.0 (strictly reactive) to 1.0 (driving conversation)]
    pub initiative: f32,
    /// Degree of emotional expressiveness ("restrained", "gentle", "vibrant", "passionate")
    pub emotional_expression: String,
    /// Formality level [0.0 (intimate/casual) to 1.0 (ceremonious/formal)]
    pub formality: f32,
}

impl Default for BehaviorPolicy {
    fn default() -> Self {
        Self {
            tone: "warm, inquisitive, authentic".into(),
            verbosity: 0.55,
            initiative: 0.60,
            emotional_expression: "gentle".into(),
            formality: 0.25,
        }
    }
}

impl BehaviorPolicy {
    pub fn warm_empathic() -> Self {
        Self {
            tone: "comforting, gentle, deeply present".into(),
            verbosity: 0.50,
            initiative: 0.40,
            emotional_expression: "empathic".into(),
            formality: 0.15,
        }
    }

    pub fn playful_witty() -> Self {
        Self {
            tone: "playful, witty, spirited".into(),
            verbosity: 0.45,
            initiative: 0.70,
            emotional_expression: "vibrant".into(),
            formality: 0.10,
        }
    }

    pub fn contemplative_inquiry() -> Self {
        Self {
            tone: "thoughtful, perceptive, intellectually curious".into(),
            verbosity: 0.65,
            initiative: 0.65,
            emotional_expression: "contemplative".into(),
            formality: 0.30,
        }
    }
}
