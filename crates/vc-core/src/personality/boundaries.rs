use serde::{Deserialize, Serialize};
use crate::error::{CoreError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Boundaries {
    pub avoid: Vec<String>,
    pub preserve: Vec<String>,
}

impl Boundaries {
    pub fn new(avoid: Vec<String>, preserve: Vec<String>) -> Self {
        Self { avoid, preserve }
    }

    pub fn should_avoid(&self, topic: &str) -> bool {
        self.avoid.iter().any(|a| a.eq_ignore_ascii_case(topic))
    }

    pub fn should_preserve(&self, principle: &str) -> bool {
        self.preserve.iter().any(|p| p.eq_ignore_ascii_case(principle))
    }

    pub fn validate(&self) -> Result<()> {
        if self.avoid.is_empty() && self.preserve.is_empty() {
            return Err(CoreError::ValidationError(
                "Boundaries must define at least one 'avoid' or 'preserve' rule".into(),
            ));
        }
        for item in &self.avoid {
            if item.trim().is_empty() {
                return Err(CoreError::ValidationError(
                    "Boundary 'avoid' entry cannot be empty".into(),
                ));
            }
        }
        for item in &self.preserve {
            if item.trim().is_empty() {
                return Err(CoreError::ValidationError(
                    "Boundary 'preserve' entry cannot be empty".into(),
                ));
            }
        }
        Ok(())
    }
}
