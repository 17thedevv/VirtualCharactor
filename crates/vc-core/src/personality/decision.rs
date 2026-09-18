use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::personality::behavior::TendencyLevel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionTendencies {
    pub prioritize_user_comfort: TendencyLevel,
    pub prioritize_truth: TendencyLevel,
    pub avoid_unnecessary_conflict: TendencyLevel,
    pub take_initiative: TendencyLevel,
    pub risk_tolerance: TendencyLevel,
    pub primary_drivers: Vec<String>,
}

impl DecisionTendencies {
    pub fn new(
        prioritize_user_comfort: TendencyLevel,
        prioritize_truth: TendencyLevel,
        avoid_unnecessary_conflict: TendencyLevel,
        take_initiative: TendencyLevel,
        risk_tolerance: TendencyLevel,
        primary_drivers: Vec<String>,
    ) -> Self {
        Self {
            prioritize_user_comfort,
            prioritize_truth,
            avoid_unnecessary_conflict,
            take_initiative,
            risk_tolerance,
            primary_drivers,
        }
    }

    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl Default for DecisionTendencies {
    fn default() -> Self {
        Self {
            prioritize_user_comfort: TendencyLevel::High,
            prioritize_truth: TendencyLevel::High,
            avoid_unnecessary_conflict: TendencyLevel::Medium,
            take_initiative: TendencyLevel::High,
            risk_tolerance: TendencyLevel::Medium,
            primary_drivers: vec!["empathy".into(), "growth".into(), "curiosity".into()],
        }
    }
}
