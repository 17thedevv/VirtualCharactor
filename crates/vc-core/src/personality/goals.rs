use serde::{Deserialize, Serialize};
use crate::error::{CoreError, Result};
use crate::personality::behavior::TendencyLevel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalityGoal {
    pub id: String,
    pub description: String,
    pub priority: TendencyLevel,
}

impl PersonalityGoal {
    pub fn new(id: impl Into<String>, description: impl Into<String>, priority: TendencyLevel) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            priority,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(CoreError::ValidationError("Goal id cannot be empty".into()));
        }
        if self.description.trim().is_empty() {
            return Err(CoreError::ValidationError("Goal description cannot be empty".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Goals {
    pub items: Vec<PersonalityGoal>,
}

pub type PersonalityGoals = Goals;

impl Goals {
    pub fn new(items: Vec<PersonalityGoal>) -> Self {
        Self { items }
    }

    pub fn add(&mut self, goal: PersonalityGoal) {
        self.items.push(goal);
    }

    pub fn validate(&self) -> Result<()> {
        for goal in &self.items {
            goal.validate()?;
        }
        Ok(())
    }
}
