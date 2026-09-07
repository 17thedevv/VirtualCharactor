use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonalityId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub core_identity: String,
    pub background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Traits(pub Vec<String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Values(pub Vec<String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences(pub Vec<String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorTendencies(pub Vec<String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    pub tone: String,
    pub quirks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTendencies {
    pub risk_tolerance: String,
    pub primary_drivers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boundaries(pub Vec<String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_creation() {
        let p = Personality {
            id: PersonalityId(Uuid::new_v4()),
            identity: Identity {
                core_identity: "Friendly AI".into(),
                background: "Created for testing".into(),
            },
            traits: Traits(vec!["helpful".into()]),
            values: Values(vec!["honesty".into()]),
            preferences: Preferences(vec!["conciseness".into()]),
            behavior_tendencies: BehaviorTendencies(vec![]),
            communication_style: CommunicationStyle {
                tone: "polite".into(),
                quirks: vec![],
            },
            decision_tendencies: DecisionTendencies {
                risk_tolerance: "low".into(),
                primary_drivers: vec!["safety".into()],
            },
            boundaries: Boundaries(vec!["no harmful content".into()]),
        };
        assert_eq!(p.identity.core_identity, "Friendly AI");
    }
}
