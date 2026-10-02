//! Computer Action definitions and parameters.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Strongly typed computer interaction actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputerAction {
    /// Move mouse cursor to screen coordinates.
    MoveCursor { x: i32, y: i32 },
    /// Click mouse button at coordinates.
    Click { x: i32, y: i32, button: MouseButton },
    /// Type text into focused input field.
    TypeText { text: String },
    /// Press keyboard shortcut (e.g. ["Ctrl"], "c").
    PressHotkey { modifiers: Vec<String>, key: String },
    /// Scroll window or page.
    Scroll { delta_x: i32, delta_y: i32 },
    /// Launch an application.
    OpenApp { app_name: String },
    /// Run shell command (High risk, restricted).
    ExecuteSystemCommand { command: String },
    /// Alter system settings (High risk, restricted).
    ModifySystemSetting { setting: String, value: String },
    /// File deletion operation (High risk, restricted).
    DeleteFile { path: String },
}

impl ComputerAction {
    pub fn action_name(&self) -> &'static str {
        match self {
            ComputerAction::MoveCursor { .. } => "move_cursor",
            ComputerAction::Click { .. } => "click",
            ComputerAction::TypeText { .. } => "type_text",
            ComputerAction::PressHotkey { .. } => "press_hotkey",
            ComputerAction::Scroll { .. } => "scroll",
            ComputerAction::OpenApp { .. } => "open_app",
            ComputerAction::ExecuteSystemCommand { .. } => "execute_system_command",
            ComputerAction::ModifySystemSetting { .. } => "modify_system_setting",
            ComputerAction::DeleteFile { .. } => "delete_file",
        }
    }
}
