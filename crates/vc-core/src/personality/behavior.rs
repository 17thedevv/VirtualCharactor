use serde::{Deserialize, Serialize};
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TendencyLevel {
    Low,
    Medium,
    High,
}

impl Default for TendencyLevel {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehaviorTendencies {
    pub humor: TendencyLevel,
    pub teasing: TendencyLevel,
    pub initiative: TendencyLevel,
    pub emotional_expression: TendencyLevel,
    pub conflict_avoidance: TendencyLevel,
}

impl BehaviorTendencies {
    pub fn new(
        humor: TendencyLevel,
        teasing: TendencyLevel,
        initiative: TendencyLevel,
        emotional_expression: TendencyLevel,
        conflict_avoidance: TendencyLevel,
    ) -> Self {
        Self {
            humor,
            teasing,
            initiative,
            emotional_expression,
            conflict_avoidance,
        }
    }

    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl Default for BehaviorTendencies {
    fn default() -> Self {
        Self {
            humor: TendencyLevel::Medium,
            teasing: TendencyLevel::Medium,
            initiative: TendencyLevel::Medium,
            emotional_expression: TendencyLevel::Medium,
            conflict_avoidance: TendencyLevel::Medium,
        }
    }
}
