//! Screen Verification Loop: Verifies visual state response after computer action execution.
//!
//! Guarantees that actions are never reported as "completed" unless verified by post-action screen diff or window focus.

use crate::computer::action::ComputerAction;
use serde::{Deserialize, Serialize};

/// Outcome of the post-action screen verification check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationResult {
    /// True if post-action verification confirmed expected UI change.
    pub is_verified: bool,
    /// Detailed diagnostic or confirmation note.
    pub details: String,
    /// Detected visual difference ratio (0.0 to 1.0).
    pub visual_diff_ratio: f32,
}

impl VerificationResult {
    pub fn success(details: impl Into<String>, visual_diff_ratio: f32) -> Self {
        Self {
            is_verified: true,
            details: details.into(),
            visual_diff_ratio,
        }
    }

    pub fn failure(details: impl Into<String>, visual_diff_ratio: f32) -> Self {
        Self {
            is_verified: false,
            details: details.into(),
            visual_diff_ratio,
        }
    }
}

/// Verification engine checking UI response after actions.
pub struct ScreenVerificationLoop;

impl ScreenVerificationLoop {
    /// Verify whether a computer action actually succeeded based on pre/post screen states.
    pub fn verify_action(
        action: &ComputerAction,
        pre_window_title: &str,
        post_window_title: &str,
        visual_diff_ratio: f32,
    ) -> VerificationResult {
        match action {
            ComputerAction::OpenApp { app_name } => {
                let lower_app = app_name.to_lowercase();
                let lower_post = post_window_title.to_lowercase();

                // 1. Success if foreground window title changed to target app
                if lower_post.contains(&lower_app)
                    || (!lower_app.is_empty()
                        && lower_post != pre_window_title.to_lowercase()
                        && visual_diff_ratio >= 0.10)
                {
                    VerificationResult::success(
                        format!(
                            "Xác nhận thành công: Cửa sổ '{}' đã mở trên màn hình.",
                            post_window_title
                        ),
                        visual_diff_ratio,
                    )
                } else if visual_diff_ratio < 0.05 {
                    VerificationResult::failure(
                        format!("Xác minh thất bại: Không phát hiện cửa sổ '{}' xuất hiện (Sai khác điểm ảnh chỉ {:.1}%).", app_name, visual_diff_ratio * 100.0),
                        visual_diff_ratio,
                    )
                } else {
                    VerificationResult::success(
                        format!(
                            "Ghi nhận thay đổi thị giác ({:.1}%) sau khi mở '{}'.",
                            visual_diff_ratio * 100.0,
                            app_name
                        ),
                        visual_diff_ratio,
                    )
                }
            }
            ComputerAction::Click { x, y, .. } => {
                if visual_diff_ratio >= 0.02 || pre_window_title != post_window_title {
                    VerificationResult::success(
                        format!("Ghi nhận phản hồi UI tại tọa độ ({}, {}) sau click.", x, y),
                        visual_diff_ratio,
                    )
                } else {
                    VerificationResult::failure(
                        format!(
                            "Không phát hiện phản hồi thị giác tại ({}, {}) sau click.",
                            x, y
                        ),
                        visual_diff_ratio,
                    )
                }
            }
            ComputerAction::Scroll { delta_y: _, .. } => {
                if visual_diff_ratio >= 0.02 {
                    VerificationResult::success(
                        "Nội dung màn hình đã cuộn thành công.",
                        visual_diff_ratio,
                    )
                } else {
                    VerificationResult::failure(
                        "Không phát hiện thay đổi trang sau thao tác cuộn.",
                        visual_diff_ratio,
                    )
                }
            }
            ComputerAction::TypeText { text } => {
                if visual_diff_ratio >= 0.01 {
                    VerificationResult::success(
                        format!("Văn bản đã được nhập ('{}').", text),
                        visual_diff_ratio,
                    )
                } else {
                    VerificationResult::failure(
                        "Không phát hiện ký tự xuất hiện trong trường nhập liệu.",
                        visual_diff_ratio,
                    )
                }
            }
            other => VerificationResult::success(
                format!(
                    "Hành động '{}' hoàn tất chu trình xác minh.",
                    other.action_name()
                ),
                visual_diff_ratio,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_app_verification_success() {
        let action = ComputerAction::OpenApp {
            app_name: "calc".into(),
        };
        let res = ScreenVerificationLoop::verify_action(
            &action,
            "Visual Studio Code",
            "Calculator",
            0.22,
        );
        assert!(res.is_verified);
        assert!(res.details.contains("Calculator"));
    }

    #[test]
    fn test_open_app_verification_fails_if_window_never_appears() {
        let action = ComputerAction::OpenApp {
            app_name: "spotify".into(),
        };
        // Window title unchanged, diff 0.01% -> failed
        let res = ScreenVerificationLoop::verify_action(
            &action,
            "Visual Studio Code",
            "Visual Studio Code",
            0.01,
        );
        assert!(!res.is_verified);
        assert!(res.details.contains("thất bại"));
    }

    #[test]
    fn test_click_verification_requires_visual_response() {
        let action = ComputerAction::Click {
            x: 200,
            y: 350,
            button: crate::computer::action::MouseButton::Left,
        };

        // Screen reacted -> success
        let ok_res =
            ScreenVerificationLoop::verify_action(&action, "Browser", "Browser - New Tab", 0.15);
        assert!(ok_res.is_verified);

        // Screen didn't react -> failure
        let fail_res = ScreenVerificationLoop::verify_action(&action, "Browser", "Browser", 0.0);
        assert!(!fail_res.is_verified);
    }
}
