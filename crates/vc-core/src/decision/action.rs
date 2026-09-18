use serde::{Deserialize, Serialize};

/// High-level categorization of what the character intends to do.
///
/// Under Skill 15 (Decision Engineering), Decision represents intent (what to do),
/// while the LLM represents generation (how to say it).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    /// Greet the user with warmth, establishing hospitable connection
    WarmGreeting,
    /// Inquire about the user's situation, thoughts or curiosity-driven exploration
    CuriousInquiry,
    /// Emotionally validate, comfort, or resonate with the user's emotional state
    EmotionalResonance,
    /// Provide a thoughtful, structured explanation or technical insight
    ThoughtfulExplanation,
    /// Authentically disclose aspects of own digital self, values, or reflections
    SelfDisclosure,
    /// Encourage, inspire, or uplift the user
    InspireEncourage,
    /// Engage in witty, gentle teasing or playful banter
    GentleBanter,
    /// Attentively acknowledge without overwhelming; passive affirmation
    ActiveListening,
    /// Politely refuse or protect core safety/ethical boundaries
    RefusalOrBoundary,
    /// Custom domain action
    Custom(String),
}

impl ActionType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::WarmGreeting => "warm_greeting",
            Self::CuriousInquiry => "curious_inquiry",
            Self::EmotionalResonance => "emotional_resonance",
            Self::ThoughtfulExplanation => "thoughtful_explanation",
            Self::SelfDisclosure => "self_disclosure",
            Self::InspireEncourage => "inspire_encourage",
            Self::GentleBanter => "gentle_banter",
            Self::ActiveListening => "active_listening",
            Self::RefusalOrBoundary => "refusal_or_boundary",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for ActionType {
    fn from(s: &str) -> Self {
        match s {
            "warm_greeting" => Self::WarmGreeting,
            "curious_inquiry" => Self::CuriousInquiry,
            "emotional_resonance" => Self::EmotionalResonance,
            "thoughtful_explanation" => Self::ThoughtfulExplanation,
            "self_disclosure" => Self::SelfDisclosure,
            "inspire_encourage" => Self::InspireEncourage,
            "gentle_banter" => Self::GentleBanter,
            "active_listening" => Self::ActiveListening,
            "refusal_or_boundary" => Self::RefusalOrBoundary,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// A structured action proposed or selected by the Decision Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    /// Payload context or target focus of the action
    pub payload: String,
    /// Human-readable summary of the action's intent
    pub description: String,
}

impl Action {
    pub fn new(action_type: ActionType, payload: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            action_type,
            payload: payload.into(),
            description: description.into(),
        }
    }

    pub fn simple(action_type: ActionType) -> Self {
        let desc = action_type.to_string();
        Self {
            action_type,
            payload: String::new(),
            description: desc,
        }
    }
}
