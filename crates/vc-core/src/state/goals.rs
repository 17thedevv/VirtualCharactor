use serde::{Deserialize, Serialize};

/// A single active goal the character is pursuing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goal {
    /// Short identifier for the goal.
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// Priority in `[0.0, 1.0]`.
    pub priority: f32,
    /// Progress toward completion in `[0.0, 1.0]`.
    pub progress: f32,
}

impl Goal {
    pub fn new(id: impl Into<String>, description: impl Into<String>, priority: f32) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            priority: priority.clamp(0.0, 1.0),
            progress: 0.0,
        }
    }
}

/// Collection of active goals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goals {
    pub active_goals: Vec<Goal>,
}

impl Goals {
    pub fn new(goals: Vec<Goal>) -> Self {
        Self { active_goals: goals }
    }

    pub fn empty() -> Self {
        Self {
            active_goals: Vec::new(),
        }
    }
}

impl Default for Goals {
    fn default() -> Self {
        Self::new(vec![
            Goal::new(
                "establish_rapport",
                "Build a meaningful rapport with the user",
                0.8,
            ),
            Goal::new(
                "understand_perspectives",
                "Understand the user's perspectives and interests",
                0.7,
            ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_creation() {
        let goal = Goal::new("test", "A test goal", 0.5);
        assert_eq!(goal.id, "test");
        assert_eq!(goal.priority, 0.5);
        assert_eq!(goal.progress, 0.0);
    }

    #[test]
    fn test_goal_priority_clamping() {
        let goal = Goal::new("test", "Overclamped", 1.5);
        assert_eq!(goal.priority, 1.0);
    }

    #[test]
    fn test_default_goals() {
        let goals = Goals::default();
        assert_eq!(goals.active_goals.len(), 2);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let goals = Goals::default();
        let json = serde_json::to_string(&goals).expect("serialize");
        let deser: Goals = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.active_goals.len(), goals.active_goals.len());
    }
}
