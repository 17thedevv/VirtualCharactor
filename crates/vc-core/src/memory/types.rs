use serde::{Deserialize, Serialize};

/// Categorization of memories based on cognitive psychology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryType {
    /// Specific experiential events and dialogues ("User worked on compiler bug on 2026-09-06")
    Episodic,
    /// Generalized facts, concepts and enduring knowledge ("User loves systems programming with Rust")
    Semantic,
    /// Behavioral patterns, interaction habits and preferences ("User prefers concise, insightful responses")
    Procedural,
    /// Milestone relationship events ("First time character and user shared a personal joke")
    Relationship,
}

impl MemoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Episodic => "Episodic",
            Self::Semantic => "Semantic",
            Self::Procedural => "Procedural",
            Self::Relationship => "Relationship",
        }
    }
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Discretized qualitative importance of a memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MemoryImportance {
    /// Low impact, fleeting detail (e.g. casual small talk)
    Low,
    /// Moderate relevance, useful for ongoing context
    Medium,
    /// High significance, shapes relationship or character understanding
    High,
    /// Critical core insight or identity-defining memory; resistant to decay
    Critical,
}

impl MemoryImportance {
    /// Convert importance to a normalized numerical weight [0.0, 1.0].
    pub fn weight(&self) -> f32 {
        match self {
            Self::Low => 0.25,
            Self::Medium => 0.50,
            Self::High => 0.75,
            Self::Critical => 1.00,
        }
    }

    /// Map a numerical weight [0.0, 1.0] back to MemoryImportance.
    pub fn from_weight(weight: f32) -> Self {
        if weight >= 0.90 {
            Self::Critical
        } else if weight >= 0.65 {
            Self::High
        } else if weight >= 0.40 {
            Self::Medium
        } else {
            Self::Low
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }
}

impl std::fmt::Display for MemoryImportance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_importance_weights() {
        assert_eq!(MemoryImportance::Low.weight(), 0.25);
        assert_eq!(MemoryImportance::Critical.weight(), 1.00);

        assert_eq!(MemoryImportance::from_weight(0.95), MemoryImportance::Critical);
        assert_eq!(MemoryImportance::from_weight(0.70), MemoryImportance::High);
        assert_eq!(MemoryImportance::from_weight(0.50), MemoryImportance::Medium);
        assert_eq!(MemoryImportance::from_weight(0.20), MemoryImportance::Low);
    }
}
