//! Integration tests for Phase 6: Controlled Computer Use & Screen Verification.

use vc_core::state::world::WorldState;
use vc_runtime::computer::{
    ComputerAction, ComputerExecutor, MockComputerExecutor, ScreenVerificationLoop,
    ToolPermissionPolicy, ToolRiskLevel,
};

#[test]
fn test_computer_use_permission_and_risk_gating() {
    let policy = ToolPermissionPolicy::default();

    // 1. Low risk: scroll
    let scroll = ComputerAction::Scroll {
        delta_x: 0,
        delta_y: 10,
    };
    assert_eq!(policy.assess_risk(&scroll), ToolRiskLevel::Low);
    assert!(policy.is_permitted(&scroll, false).is_ok());

    // 2. High risk: execute command
    let cmd = ComputerAction::ExecuteSystemCommand {
        command: "powershell -Command Get-Process".into(),
    };
    assert_eq!(policy.assess_risk(&cmd), ToolRiskLevel::High);
    // Unapproved -> Gated
    assert!(policy.is_permitted(&cmd, false).is_err());
    // User approved -> Allowed
    assert!(policy.is_permitted(&cmd, true).is_ok());

    // 3. Blocked: destructive operations
    let blocked_cmd = ComputerAction::ExecuteSystemCommand {
        command: "format C: /fs:NTFS".into(),
    };
    assert_eq!(policy.assess_risk(&blocked_cmd), ToolRiskLevel::Blocked);
    assert!(
        policy.is_permitted(&blocked_cmd, true).is_err(),
        "Blocked actions must NEVER be permitted!"
    );
}

#[test]
fn test_computer_execution_and_verification_pipeline() {
    let executor = MockComputerExecutor::new();
    let mut world = WorldState::default();
    let policy = ToolPermissionPolicy::default();

    let open_app = ComputerAction::OpenApp {
        app_name: "calculator".into(),
    };

    // 1. Permission check
    assert!(policy.is_permitted(&open_app, false).is_ok());

    // 2. Execute action
    let result = executor.execute(&open_app).unwrap();
    assert!(result.success);
    assert_eq!(executor.action_count(), 1);

    // 3. Screen Verification Loop (Simulating Calculator opened with 25% visual diff)
    let pre_title = "Desktop";
    let post_title = "Calculator";
    let visual_diff = 0.25;

    let verification =
        ScreenVerificationLoop::verify_action(&open_app, pre_title, post_title, visual_diff);
    assert!(
        verification.is_verified,
        "Screen verification must pass when app opens!"
    );
    assert!(verification.details.contains("Calculator"));

    // 4. Update WorldState with verified window
    world.update_window(post_title, "Calculator.exe", 1000);
    assert_eq!(world.active_window.as_ref().unwrap().title, "Calculator");
}

#[test]
fn test_unverified_action_prevents_false_success_report() {
    let executor = MockComputerExecutor::new();
    let action = ComputerAction::OpenApp {
        app_name: "Spotify".into(),
    };

    let _ = executor.execute(&action).unwrap();

    // Verification check where window never changed and screen didn't change (0.0% diff)
    let verification = ScreenVerificationLoop::verify_action(
        &action,
        "Visual Studio Code",
        "Visual Studio Code",
        0.0,
    );
    assert!(
        !verification.is_verified,
        "Must NOT report success if screen never responded!"
    );
    assert!(verification.details.contains("thất bại"));
}
