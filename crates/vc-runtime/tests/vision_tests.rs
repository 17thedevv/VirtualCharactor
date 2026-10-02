//! Component and Integration Tests for Phase 5: World Model, Attention Engine, and Adaptive Vision.

use std::sync::Arc;
use vc_core::personality::Personality;
use vc_core::state::world::{ActivityType, WorldState};
use vc_runtime::attention::{AttentionEngine, AttentionEvent};
use vc_runtime::resource_manager::{ExecutionMode, ResourceManager};
use vc_runtime::vision::{
    CaptureFrame, MockScreenCaptureProvider, MockVisionProvider, ScreenCaptureProvider,
    VisionObservation, VisionRouter, VisionRouterConfig, VisualDiffDetector,
};

#[test]
fn test_screen_sensing_diff_and_attention_filtering() {
    let engine = AttentionEngine::default();
    let world = WorldState::default();
    let personality = Personality::baseline_aria();

    // 1. Unchanged screen (0% diff) -> Ignored completely
    let frame_a = CaptureFrame::new(10, 10, vec![100; 100], 1000);
    let frame_b = CaptureFrame::new(10, 10, vec![100; 100], 1001);
    let diff = VisualDiffDetector::compute_diff(&frame_a, &frame_b);
    assert_eq!(diff, 0.0);

    let event_unchanged = AttentionEvent::ScreenVisualDiff {
        diff_percentage: diff,
        window_title: "VS Code".into(),
        process_name: "Code.exe".into(),
    };
    let eval_unchanged = engine.evaluate(&event_unchanged, &world, &personality, 1001);
    assert!(
        !eval_unchanged.should_capture_vlm,
        "Unchanged screen must NOT invoke VLM!"
    );
    assert!(!eval_unchanged.should_speak);

    // 2. Minor irrelevant change (8% diff < 15% threshold) -> Filtered
    let mut frame_c_data = vec![100; 100];
    for i in 0..8 {
        frame_c_data[i] = 250;
    }
    let frame_c = CaptureFrame::new(10, 10, frame_c_data, 1002);
    let diff_minor = VisualDiffDetector::compute_diff(&frame_a, &frame_c);
    assert_eq!(diff_minor, 0.08);

    let event_minor = AttentionEvent::ScreenVisualDiff {
        diff_percentage: diff_minor,
        window_title: "VS Code".into(),
        process_name: "Code.exe".into(),
    };
    let eval_minor = engine.evaluate(&event_minor, &world, &personality, 1002);
    assert!(
        !eval_minor.should_capture_vlm,
        "Minor diff (<15%) must NOT invoke VLM!"
    );

    // 3. Significant change (30% diff >= 15% threshold) -> Triggers VLM understanding
    let mut frame_d_data = vec![100; 100];
    for i in 0..30 {
        frame_d_data[i] = 250;
    }
    let frame_d = CaptureFrame::new(10, 10, frame_d_data, 1003);
    let diff_major = VisualDiffDetector::compute_diff(&frame_a, &frame_d);
    assert_eq!(diff_major, 0.30);

    let event_major = AttentionEvent::ScreenVisualDiff {
        diff_percentage: diff_major,
        window_title: "Game Window".into(),
        process_name: "Genshin.exe".into(),
    };
    let eval_major = engine.evaluate(&event_major, &world, &personality, 1003);
    assert!(
        eval_major.should_capture_vlm,
        "Major diff (30%) MUST trigger VLM capture!"
    );
}

#[test]
fn test_autonomous_speech_cooldown_and_chatter_prevention() {
    let engine = AttentionEngine::default();
    let mut world = WorldState::default();
    let personality = Personality::baseline_aria();

    // Character spoke at time 500
    world.record_speech(500);

    // Event arrives at 515 (15 seconds later - inside 30s cooldown window)
    let event = AttentionEvent::ScreenVisualDiff {
        diff_percentage: 0.50,
        window_title: "Browser".into(),
        process_name: "chrome.exe".into(),
    };

    let eval = engine.evaluate(&event, &world, &personality, 515);
    assert!(
        !eval.should_speak,
        "Must suppress autonomous speech during cooldown!"
    );
    assert_eq!(eval.score.cooldown_penalty, 1.0);

    // After initial cooldown period (time 535 - 35 seconds later: decaying penalty)
    let eval_after_cooldown = engine.evaluate(&event, &world, &personality, 535);
    assert_eq!(eval_after_cooldown.score.cooldown_penalty, 0.35);

    // After full cooldown period (time 565 - 65 seconds later: zero penalty)
    let eval_full = engine.evaluate(&event, &world, &personality, 565);
    assert_eq!(eval_full.score.cooldown_penalty, 0.0);
}

#[test]
fn test_vision_router_multiplexing_and_graceful_fallbacks() {
    let rm = Arc::new(ResourceManager::new_rtx3050_profile());
    rm.release_gpu_to_idle();
    assert_eq!(rm.current_mode(), ExecutionMode::Idle);

    let primary = Arc::new(MockVisionProvider::new("mock-qwen-vl"));
    primary.push_canned_observation(VisionObservation::new(
        "Người dùng vừa hoàn thành một màn chơi game",
        vec!["Victory Screen".into()],
        0.96,
        18,
        "mock-qwen-vl",
    ));

    let router = VisionRouter::new(
        primary.clone(),
        None,
        Some(rm.clone()),
        VisionRouterConfig::default(),
    );

    let frame = CaptureFrame::new(64, 64, vec![128; 4096], 1000);
    let obs = router.process_frame(&frame);

    assert_eq!(
        obs.description,
        "Người dùng vừa hoàn thành một màn chơi game"
    );
    assert_eq!(obs.confidence, 0.96);
    // Verified GPU mode returned to Idle after inference
    assert_eq!(rm.current_mode(), ExecutionMode::Idle);

    // Test Primary Failure -> Fallback to secondary provider
    primary.set_simulated_error(Some("GPU Out of Memory".into()));

    let fallback = Arc::new(MockVisionProvider::new("gemini-cloud-fallback"));
    fallback.push_canned_observation(VisionObservation::new(
        "Quan sát từ cloud fallback",
        vec!["Editor".into()],
        0.80,
        45,
        "gemini-cloud-fallback",
    ));

    let router_with_fb = VisionRouter::new(
        primary,
        Some(fallback),
        Some(rm.clone()),
        VisionRouterConfig::default(),
    );

    let obs_fb = router_with_fb.process_frame(&frame);
    assert_eq!(obs_fb.description, "Quan sát từ cloud fallback");
    assert_eq!(obs_fb.model_used, "gemini-cloud-fallback");
    assert_eq!(rm.current_mode(), ExecutionMode::Idle);
}

#[test]
fn test_end_to_end_world_model_observation_pipeline() {
    let mock_capture =
        MockScreenCaptureProvider::new("src/main.rs - Visual Studio Code", "Code.exe");
    let (title, process) = mock_capture.get_active_window();

    let mut world = WorldState::default();
    world.update_window(&title, &process, 2000);

    assert_eq!(
        world.active_window.as_ref().unwrap().activity,
        ActivityType::Coding
    );
    assert!(world.is_user_busy());

    let frame = mock_capture.capture_screen().unwrap();
    let primary = Arc::new(MockVisionProvider::new("test-vlm"));
    primary.push_canned_observation(VisionObservation::new(
        "Người dùng đang viết unit test cho Rust runtime",
        vec!["cargo test".into()],
        0.94,
        20,
        "test-vlm",
    ));

    let router = VisionRouter::new(primary, None, None, VisionRouterConfig::default());

    let observation = router.process_frame(&frame);
    world.update_screen_summary(&observation.description, 2005);

    let context_desc = world.context_description();
    assert!(context_desc.contains("Visual Studio Code"));
    assert!(context_desc.contains("Coding"));
    assert!(context_desc.contains("Người dùng đang viết unit test cho Rust runtime"));
}

#[test]
#[cfg(target_os = "windows")]
fn test_real_windows_screen_capture_pipeline() {
    use vc_runtime::vision::WindowsScreenCaptureProvider;

    let capture = WindowsScreenCaptureProvider::new();
    let (title, process) = capture.get_active_window();
    assert!(!process.is_empty(), "Foreground process should be detected");

    let mut world = WorldState::default();
    world.update_window(&title, &process, 3000);
    assert!(world.active_window.is_some());

    let frame1 = capture
        .capture_screen()
        .expect("Real desktop frame 1 capture should succeed");
    assert_eq!(frame1.width, 64);
    assert_eq!(frame1.height, 64);
    assert!(
        frame1.encoded_image.is_some(),
        "Encoded BMP preview should be present"
    );

    std::thread::sleep(std::time::Duration::from_millis(50));

    let frame2 = capture
        .capture_screen()
        .expect("Real desktop frame 2 capture should succeed");
    let diff = VisualDiffDetector::compute_diff(&frame1, &frame2);
    assert!(diff >= 0.0 && diff <= 1.0);
}
