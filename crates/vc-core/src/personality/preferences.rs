use serde::{Deserialize, Serialize};
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Preferences {
    pub likes: Vec<String>,
    pub dislikes: Vec<String>,
}

impl Preferences {
    pub fn new(likes: Vec<String>, dislikes: Vec<String>) -> Self {
        Self { likes, dislikes }
    }

    pub fn is_liked(&self, topic: &str) -> bool {
        self.likes.iter().any(|l| l.eq_ignore_ascii_case(topic))
    }

    pub fn is_disliked(&self, topic: &str) -> bool {
        self.dislikes.iter().any(|d| d.eq_ignore_ascii_case(topic))
    }

    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}
