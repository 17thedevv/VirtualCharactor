use serde::{Deserialize, Serialize};
use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ValueScore(f32);

impl ValueScore {
    pub fn new(value: f32) -> Result<Self> {
        if value.is_nan() || value < 0.0 || value > 1.0 {
            return Err(CoreError::ValidationError(format!(
                "ValueScore must be between 0.0 and 1.0, got {}",
                value
            )));
        }
        Ok(Self(value))
    }

    pub fn clamped(value: f32) -> Self {
        if value.is_nan() {
            Self(0.0)
        } else {
            Self(value.clamp(0.0, 1.0))
        }
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn validate(&self) -> Result<()> {
        if self.0.is_nan() || self.0 < 0.0 || self.0 > 1.0 {
            return Err(CoreError::ValidationError(format!(
                "ValueScore must be between 0.0 and 1.0, got {}",
                self.0
            )));
        }
        Ok(())
    }
}

impl Default for ValueScore {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueItem {
    pub name: String,
    pub importance: ValueScore,
    pub description: Option<String>,
}

impl ValueItem {
    pub fn new(name: impl Into<String>, importance: ValueScore, description: Option<String>) -> Self {
        Self {
            name: name.into(),
            importance,
            description,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CoreError::ValidationError("Value name cannot be empty".into()));
        }
        self.importance.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Values {
    pub items: Vec<ValueItem>,
}

impl Values {
    pub fn new(items: Vec<ValueItem>) -> Self {
        Self { items }
    }

    pub fn importance_of(&self, name: &str) -> Option<f32> {
        self.items
            .iter()
            .find(|item| item.name.eq_ignore_ascii_case(name))
            .map(|item| item.importance.value())
    }

    pub fn add(&mut self, item: ValueItem) {
        self.items.push(item);
    }

    pub fn validate(&self) -> Result<()> {
        for item in &self.items {
            item.validate()?;
        }
        Ok(())
    }
}
