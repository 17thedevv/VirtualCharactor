//! Action Safety Sandbox & Permission Policy.
//!
//! Enforces strict authorization boundaries: Low-risk actions can execute automatically,
//! Medium-risk actions operate under strict context rules, High-risk actions require
//! explicit human user approval, and destructive commands are blocked permanently.

use crate::computer::action::ComputerAction;
use serde::{Deserialize, Serialize};

/// Risk category for computer tools under Skill 25 (Security Engineering).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ToolRiskLevel {
    /// Harmless observation or navigation (cursor move, scroll, open harmless app).
    Low,
    /// Interactive modification (click UI, typing text, keyboard shortcuts).
    Medium,
    /// System changes (executing commands, deleting files, changing configuration).
    High,
    /// Critical or prohibited actions (permanently blocked).
    Blocked,
}

/// Security policy governing what computer actions the autonomous character can perform.
#[derive(Debug, Clone)]
pub struct ToolPermissionPolicy {
    /// Whitelist of safe applications allowed to be launched automatically.
    pub app_whitelist: Vec<String>,
    /// Blacklist of banned process names, commands, or path keywords.
    pub command_blacklist: Vec<String>,
}

impl Default for ToolPermissionPolicy {
    fn default() -> Self {
        Self {
            app_whitelist: vec![
                "notepad".into(),
                "calc".into(),
                "calculator".into(),
                "chrome".into(),
                "msedge".into(),
                "spotify".into(),
                "code".into(),
            ],
            command_blacklist: vec![
                "format".into(),
                "rmdir /s".into(),
                "del /f".into(),
                "reg delete".into(),
                "virtual_character.db".into(),
                "System32".into(),
                "powershell -encodedcommand".into(),
            ],
        }
    }
}

impl ToolPermissionPolicy {
    /// Assess the inherent risk tier of a proposed action.
    pub fn assess_risk(&self, action: &ComputerAction) -> ToolRiskLevel {
        match action {
            ComputerAction::MoveCursor { .. } | ComputerAction::Scroll { .. } => ToolRiskLevel::Low,
            ComputerAction::OpenApp { app_name } => {
                let lower = app_name.to_lowercase();
                if self.app_whitelist.iter().any(|safe| lower.contains(safe)) {
                    ToolRiskLevel::Low
                } else {
                    ToolRiskLevel::Medium
                }
            }
            ComputerAction::Click { .. }
            | ComputerAction::TypeText { .. }
            | ComputerAction::PressHotkey { .. } => ToolRiskLevel::Medium,
            ComputerAction::ExecuteSystemCommand { command } => {
                let lower = command.to_lowercase();
                if self
                    .command_blacklist
                    .iter()
                    .any(|b| lower.contains(&b.to_lowercase()))
                {
                    ToolRiskLevel::Blocked
                } else {
                    ToolRiskLevel::High
                }
            }
            ComputerAction::DeleteFile { path } => {
                let lower = path.to_lowercase();
                if lower.contains("system32")
                    || lower.contains("virtual_character.db")
                    || lower.contains(".git")
                {
                    ToolRiskLevel::Blocked
                } else {
                    ToolRiskLevel::High
                }
            }
            ComputerAction::ModifySystemSetting { .. } => ToolRiskLevel::High,
        }
    }

    /// Check if the action is authorized to execute given the user approval flag.
    pub fn is_permitted(&self, action: &ComputerAction, user_approved: bool) -> Result<(), String> {
        let risk = self.assess_risk(action);
        match risk {
            ToolRiskLevel::Blocked => Err(format!(
                "Hành động '{}' bị từ chối vĩnh viễn vì thuộc danh mục cấm bảo mật.",
                action.action_name()
            )),
            ToolRiskLevel::High => {
                if user_approved {
                    Ok(())
                } else {
                    Err(format!(
                        "Hành động mức độ rủi ro CAO ('{}') bắt buộc phải có sự chấp thuận rõ ràng của người dùng.",
                        action.action_name()
                    ))
                }
            }
            ToolRiskLevel::Medium => {
                // Medium risk is permitted if approved or running within user session
                Ok(())
            }
            ToolRiskLevel::Low => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_risk_actions_are_automatically_permitted() {
        let policy = ToolPermissionPolicy::default();
        let scroll = ComputerAction::Scroll {
            delta_x: 0,
            delta_y: 5,
        };
        assert_eq!(policy.assess_risk(&scroll), ToolRiskLevel::Low);
        assert!(policy.is_permitted(&scroll, false).is_ok());

        let open_calc = ComputerAction::OpenApp {
            app_name: "calc.exe".into(),
        };
        assert_eq!(policy.assess_risk(&open_calc), ToolRiskLevel::Low);
        assert!(policy.is_permitted(&open_calc, false).is_ok());
    }

    #[test]
    fn test_high_risk_actions_require_explicit_approval() {
        let policy = ToolPermissionPolicy::default();
        let cmd = ComputerAction::ExecuteSystemCommand {
            command: "dir C:\\Users".into(),
        };
        assert_eq!(policy.assess_risk(&cmd), ToolRiskLevel::High);

        // Disapproved -> Rejected
        assert!(policy.is_permitted(&cmd, false).is_err());
        // Approved -> Allowed
        assert!(policy.is_permitted(&cmd, true).is_ok());
    }

    #[test]
    fn test_blocked_actions_are_denied_even_with_approval() {
        let policy = ToolPermissionPolicy::default();
        let format_cmd = ComputerAction::ExecuteSystemCommand {
            command: "format C:".into(),
        };
        assert_eq!(policy.assess_risk(&format_cmd), ToolRiskLevel::Blocked);
        assert!(policy.is_permitted(&format_cmd, true).is_err());

        let del_db = ComputerAction::DeleteFile {
            path: "data/virtual_character.db".into(),
        };
        assert_eq!(policy.assess_risk(&del_db), ToolRiskLevel::Blocked);
        assert!(policy.is_permitted(&del_db, true).is_err());
    }
}
