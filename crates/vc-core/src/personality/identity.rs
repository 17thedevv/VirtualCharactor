use serde::{Deserialize, Serialize};
use crate::error::{CoreError, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub role: String,
    pub core_identity: String,
    pub background: String,
    pub age_representation: Option<String>,
}

impl Identity {
    pub fn new(
        name: impl Into<String>,
        role: impl Into<String>,
        core_identity: impl Into<String>,
        background: impl Into<String>,
        age_representation: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            role: role.into(),
            core_identity: core_identity.into(),
            background: background.into(),
            age_representation,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CoreError::ValidationError("Identity name cannot be empty".into()));
        }
        if self.core_identity.trim().is_empty() {
            return Err(CoreError::ValidationError("Identity core_identity cannot be empty".into()));
        }
        Ok(())
    }
}
