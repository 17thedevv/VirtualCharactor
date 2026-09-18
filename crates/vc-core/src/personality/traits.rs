use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TraitScore(f32);

impl TraitScore {
    pub fn new(value: f32) -> Result<Self> {
        if value.is_nan() || value < 0.0 || value > 1.0 {
            return Err(CoreError::ValidationError(format!(
                "TraitScore must be between 0.0 and 1.0, got {}",
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
}

impl Default for TraitScore {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Traits {
    pub playfulness: TraitScore,
    pub empathy: TraitScore,
    pub curiosity: TraitScore,
    pub assertiveness: TraitScore,
    pub patience: TraitScore,
    #[serde(default)]
    pub custom: HashMap<String, TraitScore>,
}

impl Traits {
    pub fn new(
        playfulness: TraitScore,
        empathy: TraitScore,
        curiosity: TraitScore,
        assertiveness: TraitScore,
        patience: TraitScore,
    ) -> Self {
        Self {
            playfulness,
            empathy,
            curiosity,
            assertiveness,
            patience,
            custom: HashMap::new(),
        }
    }

    pub fn score(&self, name: &str) -> Option<f32> {
        match name.to_lowercase().as_str() {
            "playfulness" => Some(self.playfulness.value()),
            "empathy" => Some(self.empathy.value()),
            "curiosity" => Some(self.curiosity.value()),
            "assertiveness" => Some(self.assertiveness.value()),
            "patience" => Some(self.patience.value()),
            other => self.custom.get(other).map(|s| s.value()),
        }
    }

    pub fn set_custom(&mut self, name: impl Into<String>, score: TraitScore) {
        self.custom.insert(name.into(), score);
    }

    pub fn validate(&self) -> Result<()> {
        self.playfulness.validate()?;
        self.empathy.validate()?;
        self.curiosity.validate()?;
        self.assertiveness.validate()?;
        self.patience.validate()?;
        for (k, v) in &self.custom {
            if k.trim().is_empty() {
                return Err(CoreError::ValidationError("Custom trait name cannot be empty".into()));
            }
            v.validate()?;
        }
        Ok(())
    }
}

impl TraitScore {
    pub fn validate(&self) -> Result<()> {
        if self.0.is_nan() || self.0 < 0.0 || self.0 > 1.0 {
            return Err(CoreError::ValidationError(format!(
                "TraitScore must be between 0.0 and 1.0, got {}",
                self.0
            )));
        }
        Ok(())
    }
}
