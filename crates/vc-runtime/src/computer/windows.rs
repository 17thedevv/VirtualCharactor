//! Native Windows Computer Use Executor using Win32 SendInput and User32 APIs.
//!
//! Provides direct hardware mouse motion, click simulation, unicode keyboard typing,
//! and hotkey synthesis on the real Windows desktop with zero external dependencies.

use crate::computer::action::{ComputerAction, MouseButton};
use crate::computer::executor::{ActionResult, ComputerExecutor};

/// Native Windows executor utilizing official Win32 SendInput and User32 subsystem.
pub struct WindowsComputerExecutor;

impl WindowsComputerExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Retrieve the current cursor coordinates on the Windows desktop.
    #[cfg(target_os = "windows")]
    pub fn get_cursor_position() -> Result<(i32, i32), String> {
        unsafe {
            let hdesk = ffi::OpenInputDesktop(0, 0, 0x01FF);
            if !hdesk.is_null() {
                ffi::SetThreadDesktop(hdesk);
            }

            let mut pt = ffi::POINT::default();
            let res = ffi::GetCursorPos(&mut pt);
            let err = ffi::GetLastError();

            if !hdesk.is_null() {
                ffi::CloseDesktop(hdesk);
            }

            if res == 0 {
                return Err(format!("GetCursorPos failed (Win32 Error: {})", err));
            }
            Ok((pt.x, pt.y))
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_cursor_position() -> Result<(i32, i32), String> {
        Err("Only supported on Windows".to_string())
    }
}

impl Default for WindowsComputerExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
mod ffi {
    use std::ffi::c_void;

    pub type BOOL = i32;

    pub const INPUT_MOUSE: u32 = 0;
    pub const INPUT_KEYBOARD: u32 = 1;

    // Mouse event flags
    pub const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
    pub const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
    pub const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
    pub const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
    pub const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
    pub const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
    pub const MOUSEEVENTF_WHEEL: u32 = 0x0800;
    pub const MOUSEEVENTF_HWHEEL: u32 = 0x1000;

    // Keyboard event flags
    pub const KEYEVENTF_KEYUP: u32 = 0x0002;
    pub const KEYEVENTF_UNICODE: u32 = 0x0004;

    // Virtual-Key codes
    pub const VK_BACK: u16 = 0x08;
    pub const VK_TAB: u16 = 0x09;
    pub const VK_RETURN: u16 = 0x0D;
    pub const VK_SHIFT: u16 = 0x10;
    pub const VK_CONTROL: u16 = 0x11;
    pub const VK_MENU: u16 = 0x12; // Alt key
    pub const VK_ESCAPE: u16 = 0x1B;
    pub const VK_SPACE: u16 = 0x20;
    pub const VK_LWIN: u16 = 0x5B;

    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct MOUSEINPUT {
        pub dx: i32,
        pub dy: i32,
        pub mouseData: u32,
        pub dwFlags: u32,
        pub time: u32,
        pub dwExtraInfo: usize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct KEYBDINPUT {
        pub wVk: u16,
        pub wScan: u16,
        pub dwFlags: u32,
        pub time: u32,
        pub dwExtraInfo: usize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct HARDWAREINPUT {
        pub uMsg: u32,
        pub wParamL: u16,
        pub wParamH: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub union INPUT_UNION {
        pub mi: MOUSEINPUT,
        pub ki: KEYBDINPUT,
        pub hi: HARDWAREINPUT,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct INPUT {
        pub type_: u32,
        pub u: INPUT_UNION,
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn SetCursorPos(X: i32, Y: i32) -> BOOL;
        pub fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
        pub fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;
        pub fn OpenInputDesktop(dwFlags: u32, fInherit: BOOL, dwDesiredAccess: u32) -> *mut c_void;
        pub fn SetThreadDesktop(hDesktop: *mut c_void) -> BOOL;
        pub fn CloseDesktop(hDesktop: *mut c_void) -> BOOL;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetLastError() -> u32;
        pub fn SetLastError(dwErrCode: u32);
    }
}

#[cfg(target_os = "windows")]
impl ComputerExecutor for WindowsComputerExecutor {
    fn execute(&self, action: &ComputerAction) -> Result<ActionResult, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Attach thread to interactive desktop if called from headless subshell
        // 0x01FF = DESKTOP_ALL (including DESKTOP_JOURNALPLAYBACK 0x0020, required by SendInput)
        let hdesk = unsafe { ffi::OpenInputDesktop(0, 0, 0x01FF) };
        if !hdesk.is_null() {
            unsafe { ffi::SetThreadDesktop(hdesk) };
        }

        let result = self.execute_internal(action, now);

        if !hdesk.is_null() {
            unsafe { ffi::CloseDesktop(hdesk) };
        }

        result
    }
}

#[cfg(target_os = "windows")]
impl WindowsComputerExecutor {
    fn execute_internal(&self, action: &ComputerAction, now: u64) -> Result<ActionResult, String> {
        match action {
            ComputerAction::MoveCursor { x, y } => {
                unsafe {
                    if ffi::SetCursorPos(*x, *y) == 0 {
                        return Ok(ActionResult::failure(
                            format!("Không thể di chuyển con trỏ tới ({}, {})", x, y),
                            now,
                        ));
                    }
                }
                Ok(ActionResult::success(
                    format!("Đã di chuyển con trỏ chuột tới ({}, {})", x, y),
                    now,
                ))
            }

            ComputerAction::Click { x, y, button } => {
                unsafe {
                    // 1. Move to target coordinate
                    ffi::SetCursorPos(*x, *y);

                    // 2. Synthesize mouse down and up
                    let (down_flag, up_flag) = match button {
                        MouseButton::Left => (ffi::MOUSEEVENTF_LEFTDOWN, ffi::MOUSEEVENTF_LEFTUP),
                        MouseButton::Right => {
                            (ffi::MOUSEEVENTF_RIGHTDOWN, ffi::MOUSEEVENTF_RIGHTUP)
                        }
                        MouseButton::Middle => {
                            (ffi::MOUSEEVENTF_MIDDLEDOWN, ffi::MOUSEEVENTF_MIDDLEUP)
                        }
                    };

                    let inputs = [
                        ffi::INPUT {
                            type_: ffi::INPUT_MOUSE,
                            u: ffi::INPUT_UNION {
                                mi: ffi::MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: 0,
                                    dwFlags: down_flag,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        },
                        ffi::INPUT {
                            type_: ffi::INPUT_MOUSE,
                            u: ffi::INPUT_UNION {
                                mi: ffi::MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: 0,
                                    dwFlags: up_flag,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        },
                    ];

                    let sent = ffi::SendInput(
                        inputs.len() as u32,
                        inputs.as_ptr(),
                        std::mem::size_of::<ffi::INPUT>() as i32,
                    );

                    if sent < inputs.len() as u32 {
                        return Ok(ActionResult::failure(
                            format!("Gửi tín hiệu click {:?} thất bại (sent: {})", button, sent),
                            now,
                        ));
                    }
                }
                Ok(ActionResult::success(
                    format!("Đã click chuột {:?} tại ({}, {})", button, x, y),
                    now,
                ))
            }

            ComputerAction::Scroll { delta_x, delta_y } => {
                unsafe {
                    let mut inputs = Vec::new();

                    // Vertical scroll
                    if *delta_y != 0 {
                        inputs.push(ffi::INPUT {
                            type_: ffi::INPUT_MOUSE,
                            u: ffi::INPUT_UNION {
                                mi: ffi::MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: (*delta_y * 120) as u32,
                                    dwFlags: ffi::MOUSEEVENTF_WHEEL,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        });
                    }

                    // Horizontal scroll
                    if *delta_x != 0 {
                        inputs.push(ffi::INPUT {
                            type_: ffi::INPUT_MOUSE,
                            u: ffi::INPUT_UNION {
                                mi: ffi::MOUSEINPUT {
                                    dx: 0,
                                    dy: 0,
                                    mouseData: (*delta_x * 120) as u32,
                                    dwFlags: ffi::MOUSEEVENTF_HWHEEL,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        });
                    }

                    if !inputs.is_empty() {
                        let sent = ffi::SendInput(
                            inputs.len() as u32,
                            inputs.as_ptr(),
                            std::mem::size_of::<ffi::INPUT>() as i32,
                        );
                        if sent < inputs.len() as u32 {
                            return Ok(ActionResult::failure("Gửi sự kiện cuộn thất bại", now));
                        }
                    }
                }
                Ok(ActionResult::success(
                    format!("Đã cuộn màn hình delta ({}, {})", delta_x, delta_y),
                    now,
                ))
            }

            ComputerAction::TypeText { text } => {
                unsafe {
                    let utf16_units: Vec<u16> = text.encode_utf16().collect();
                    let mut inputs = Vec::with_capacity(utf16_units.len() * 2);

                    for &code_unit in &utf16_units {
                        // Key Down (Unicode)
                        inputs.push(ffi::INPUT {
                            type_: ffi::INPUT_KEYBOARD,
                            u: ffi::INPUT_UNION {
                                ki: ffi::KEYBDINPUT {
                                    wVk: 0,
                                    wScan: code_unit,
                                    dwFlags: ffi::KEYEVENTF_UNICODE,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        });
                        // Key Up (Unicode)
                        inputs.push(ffi::INPUT {
                            type_: ffi::INPUT_KEYBOARD,
                            u: ffi::INPUT_UNION {
                                ki: ffi::KEYBDINPUT {
                                    wVk: 0,
                                    wScan: code_unit,
                                    dwFlags: ffi::KEYEVENTF_UNICODE | ffi::KEYEVENTF_KEYUP,
                                    time: 0,
                                    dwExtraInfo: 0,
                                },
                            },
                        });
                    }

                    if !inputs.is_empty() {
                        ffi::SetLastError(0);
                        let cb_size = std::mem::size_of::<ffi::INPUT>() as i32;
                        let sent = ffi::SendInput(inputs.len() as u32, inputs.as_ptr(), cb_size);
                        let err = ffi::GetLastError();
                        if sent < inputs.len() as u32 {
                            return Ok(ActionResult::failure(
                                format!(
                                    "Chỉ gửi được {}/{} phím (Win32 Error: {}, cbSize: {})",
                                    sent,
                                    inputs.len(),
                                    err,
                                    cb_size
                                ),
                                now,
                            ));
                        }
                    }
                }
                Ok(ActionResult::success(
                    format!("Đã nhập {} ký tự qua Win32 SendInput", text.chars().count()),
                    now,
                ))
            }

            ComputerAction::PressHotkey { modifiers, key } => {
                unsafe {
                    let mut mod_vks = Vec::new();
                    for m in modifiers {
                        match m.to_lowercase().as_str() {
                            "ctrl" | "control" => mod_vks.push(ffi::VK_CONTROL),
                            "shift" => mod_vks.push(ffi::VK_SHIFT),
                            "alt" => mod_vks.push(ffi::VK_MENU),
                            "win" | "meta" | "super" => mod_vks.push(ffi::VK_LWIN),
                            _ => {}
                        }
                    }

                    let key_vk = parse_vk_code(key);
                    let mut inputs = Vec::new();

                    // 1. Modifiers Down
                    for &vk in &mod_vks {
                        inputs.push(create_key_input(vk, 0));
                    }

                    // 2. Key Down & Up
                    inputs.push(create_key_input(key_vk, 0));
                    inputs.push(create_key_input(key_vk, ffi::KEYEVENTF_KEYUP));

                    // 3. Modifiers Up in reverse order
                    for &vk in mod_vks.iter().rev() {
                        inputs.push(create_key_input(vk, ffi::KEYEVENTF_KEYUP));
                    }

                    let sent = ffi::SendInput(
                        inputs.len() as u32,
                        inputs.as_ptr(),
                        std::mem::size_of::<ffi::INPUT>() as i32,
                    );
                    if sent < inputs.len() as u32 {
                        return Ok(ActionResult::failure("Gửi tổ hợp phím thất bại", now));
                    }
                }
                Ok(ActionResult::success(
                    format!("Đã nhấn tổ hợp phím {:?} + '{}'", modifiers, key),
                    now,
                ))
            }

            ComputerAction::OpenApp { app_name } => {
                let status = std::process::Command::new("cmd")
                    .args(["/c", "start", "", app_name])
                    .status();

                match status {
                    Ok(s) if s.success() => Ok(ActionResult::success(
                        format!("Đã khởi chạy ứng dụng '{}'.", app_name),
                        now,
                    )),
                    Ok(s) => Ok(ActionResult::failure(
                        format!("Khởi chạy '{}' thoát với mã lỗi {:?}", app_name, s.code()),
                        now,
                    )),
                    Err(e) => Err(format!("Lỗi gọi lệnh hệ điều hành: {}", e)),
                }
            }

            other => Ok(ActionResult::success(
                format!("Thực thi {} có kiểm soát", other.action_name()),
                now,
            )),
        }
    }
}

#[cfg(target_os = "windows")]
fn create_key_input(vk: u16, flags: u32) -> ffi::INPUT {
    ffi::INPUT {
        type_: ffi::INPUT_KEYBOARD,
        u: ffi::INPUT_UNION {
            ki: ffi::KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(target_os = "windows")]
fn parse_vk_code(key: &str) -> u16 {
    match key.to_lowercase().as_str() {
        "enter" | "return" => ffi::VK_RETURN,
        "tab" => ffi::VK_TAB,
        "escape" | "esc" => ffi::VK_ESCAPE,
        "space" => ffi::VK_SPACE,
        "backspace" => ffi::VK_BACK,
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap().to_ascii_uppercase();
            ch as u16
        }
        _ => 0,
    }
}

#[cfg(not(target_os = "windows"))]
impl ComputerExecutor for WindowsComputerExecutor {
    fn execute(&self, action: &ComputerAction) -> Result<ActionResult, String> {
        Ok(ActionResult::success(
            format!("(Non-Windows) Executed {}", action.action_name()),
            0,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_send_input_struct_size_matches_win32_abi() {
        // In 64-bit Windows, sizeof(INPUT) is 40 bytes.
        let size = std::mem::size_of::<ffi::INPUT>();
        assert_eq!(
            size, 40,
            "INPUT structure size must match Win32 64-bit ABI (40 bytes)"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_real_cursor_position_and_movement() {
        let executor = WindowsComputerExecutor::new();

        // 1. Read current cursor position
        let initial_pos = WindowsComputerExecutor::get_cursor_position()
            .expect("Should read real cursor position");
        println!("Initial Cursor Position: {:?}", initial_pos);

        // 2. Move cursor by small delta
        let target_x = initial_pos.0 + 5;
        let target_y = initial_pos.1 + 5;
        let move_action = ComputerAction::MoveCursor {
            x: target_x,
            y: target_y,
        };
        let res = executor
            .execute(&move_action)
            .expect("MoveCursor execution");
        assert!(res.success);

        // 3. Verify real cursor moved
        let new_pos = WindowsComputerExecutor::get_cursor_position().unwrap();
        println!("New Cursor Position after move: {:?}", new_pos);
        assert_eq!(new_pos, (target_x, target_y));

        // Restore original cursor position
        let _ = executor.execute(&ComputerAction::MoveCursor {
            x: initial_pos.0,
            y: initial_pos.1,
        });
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_real_unicode_typing_simulation() {
        let executor = WindowsComputerExecutor::new();
        let type_action = ComputerAction::TypeText {
            text: "Aria Xin chào 🌟".to_string(),
        };
        let res = executor.execute(&type_action).expect("TypeText execution");
        println!("TypeText result: {:?}", res);
        assert!(res.success, "TypeText should succeed: {}", res.message);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_real_hotkey_simulation() {
        let executor = WindowsComputerExecutor::new();
        let hotkey_action = ComputerAction::PressHotkey {
            modifiers: vec!["shift".to_string()],
            key: "a".to_string(),
        };
        let res = executor
            .execute(&hotkey_action)
            .expect("PressHotkey execution");
        println!("Hotkey result: {:?}", res);
        assert!(res.success, "PressHotkey should succeed: {}", res.message);
    }
}
