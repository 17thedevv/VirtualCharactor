//! Computer action execution backends (Mock and Native).

use crate::computer::action::ComputerAction;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Output outcome of a computer action execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub timestamp: u64,
}

impl ActionResult {
    pub fn success(message: impl Into<String>, timestamp: u64) -> Self {
        Self {
            success: true,
            message: message.into(),
            timestamp,
        }
    }

    pub fn failure(message: impl Into<String>, timestamp: u64) -> Self {
        Self {
            success: false,
            message: message.into(),
            timestamp,
        }
    }
}

/// Abstract contract for executing authorized computer actions.
pub trait ComputerExecutor: Send + Sync {
    /// Execute the action and return outcome.
    fn execute(&self, action: &ComputerAction) -> Result<ActionResult, String>;
}

/// Deterministic mock computer executor for safe testing without modifying the real desktop.
pub struct MockComputerExecutor {
    recorded_actions: RwLock<Vec<ComputerAction>>,
    simulated_failure: RwLock<Option<String>>,
}

impl Default for MockComputerExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl MockComputerExecutor {
    pub fn new() -> Self {
        Self {
            recorded_actions: RwLock::new(Vec::new()),
            simulated_failure: RwLock::new(None),
        }
    }

    pub fn set_simulated_failure(&self, err_msg: Option<String>) {
        *self.simulated_failure.write().unwrap() = err_msg;
    }

    pub fn last_action(&self) -> Option<ComputerAction> {
        let list = self.recorded_actions.read().unwrap();
        list.last().cloned()
    }

    pub fn action_count(&self) -> usize {
        let list = self.recorded_actions.read().unwrap();
        list.len()
    }
}

impl ComputerExecutor for MockComputerExecutor {
    fn execute(&self, action: &ComputerAction) -> Result<ActionResult, String> {
        {
            let mut recorded = self.recorded_actions.write().unwrap();
            recorded.push(action.clone());
        }

        {
            let failure = self.simulated_failure.read().unwrap();
            if let Some(ref msg) = *failure {
                return Ok(ActionResult::failure(msg, 1000));
            }
        }

        Ok(ActionResult::success(
            format!(
                "Hành động '{}' thực thi thành công (Mô phỏng).",
                action.action_name()
            ),
            1000,
        ))
    }
}

pub use crate::computer::windows::WindowsComputerExecutor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_executor_records_actions() {
        let executor = MockComputerExecutor::new();
        let action = ComputerAction::MoveCursor { x: 500, y: 300 };

        let res = executor.execute(&action).unwrap();
        assert!(res.success);
        assert_eq!(executor.action_count(), 1);
        assert_eq!(executor.last_action(), Some(action));
    }

    #[test]
    fn test_mock_executor_simulated_failure() {
        let executor = MockComputerExecutor::new();
        executor.set_simulated_failure(Some("Màn hình bị khóa".into()));

        let action = ComputerAction::Click {
            x: 100,
            y: 100,
            button: crate::computer::action::MouseButton::Left,
        };

        let res = executor.execute(&action).unwrap();
        assert!(!res.success);
        assert!(res.message.contains("Màn hình bị khóa"));
    }
}
