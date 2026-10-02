//! VisionRouter: Orchestrates on-demand screen understanding with multiplexing and graceful fallbacks.

use crate::resource_manager::{ExecutionMode, ResourceManager};
use crate::vision::provider::{VisionObservation, VisionProvider};
use crate::vision::sensing::CaptureFrame;
use std::sync::Arc;

/// Configuration for VisionRouter.
#[derive(Clone)]
pub struct VisionRouterConfig {
    pub max_image_dimension: u32,
    pub default_prompt: String,
}

impl Default for VisionRouterConfig {
    fn default() -> Self {
        Self {
            max_image_dimension: 1024,
            default_prompt: "Quan sát màn hình và mô tả ngắn gọn bằng tiếng Việt nội dung chính người dùng đang làm hoặc trạng thái ứng dụng.".to_string(),
        }
    }
}

/// Dispatches visual observation requests, enforcing GPU multiplexing and safe fallbacks.
pub struct VisionRouter {
    primary: Arc<dyn VisionProvider>,
    fallback: Option<Arc<dyn VisionProvider>>,
    resource_manager: Option<Arc<ResourceManager>>,
    config: VisionRouterConfig,
}

impl VisionRouter {
    pub fn new(
        primary: Arc<dyn VisionProvider>,
        fallback: Option<Arc<dyn VisionProvider>>,
        resource_manager: Option<Arc<ResourceManager>>,
        config: VisionRouterConfig,
    ) -> Self {
        Self {
            primary,
            fallback,
            resource_manager,
            config,
        }
    }

    /// Process a captured frame into a high-level semantic observation.
    ///
    /// Guarantees that errors in the vision model layer never panic or crash the host runtime.
    pub fn process_frame(&self, frame: &CaptureFrame) -> VisionObservation {
        if frame.data.is_empty() {
            return VisionObservation::fallback("Khung hình chụp rỗng");
        }

        // 1. GPU Multiplexing Guard: Request Vision mode if resource manager is present
        if let Some(ref rm) = self.resource_manager {
            if let Err(e) = rm.set_mode(ExecutionMode::Vision) {
                // If VRAM budget is exceeded or conflict occurs, return degraded observation
                return VisionObservation::fallback(&format!(
                    "Giới hạn phần cứng GPU VRAM ({})",
                    e
                ));
            }
        }

        let prompt = &self.config.default_prompt;

        // 2. Attempt inference with Primary Vision Provider
        let result = match self.primary.observe(&frame.data, prompt) {
            Ok(obs) => obs,
            Err(primary_err) => {
                // 3. Fallback path if configured
                if let Some(ref fallback_prov) = self.fallback {
                    match fallback_prov.observe(&frame.data, prompt) {
                        Ok(fb_obs) => fb_obs,
                        Err(fb_err) => VisionObservation::fallback(&format!(
                            "Cả primary ({}) và fallback ({}) đều lỗi",
                            primary_err, fb_err
                        )),
                    }
                } else {
                    VisionObservation::fallback(&format!("Primary provider lỗi: {}", primary_err))
                }
            }
        };

        // 4. Release GPU Vision Mode back to Idle/Chat baseline
        if let Some(ref rm) = self.resource_manager {
            let _ = rm.set_mode(ExecutionMode::Idle);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vision::provider::MockVisionProvider;

    #[test]
    fn test_vision_router_primary_success() {
        let primary = Arc::new(MockVisionProvider::new("mock-vlm"));
        let canned = VisionObservation::new(
            "Người dùng đang lập trình Rust",
            vec!["IDE".into()],
            0.92,
            12,
            "mock-vlm",
        );
        primary.push_canned_observation(canned.clone());

        let router = VisionRouter::new(primary, None, None, VisionRouterConfig::default());

        let frame = CaptureFrame::new(10, 10, vec![1; 100], 1000);
        let obs = router.process_frame(&frame);

        assert_eq!(obs.description, canned.description);
        assert_eq!(obs.confidence, 0.92);
    }

    #[test]
    fn test_vision_router_falls_back_on_primary_error() {
        let primary = Arc::new(MockVisionProvider::new("primary-vlm"));
        primary.set_simulated_error(Some("Model crashed".into()));

        let fallback = Arc::new(MockVisionProvider::new("fallback-vlm"));
        let canned_fb = VisionObservation::new(
            "Fallback observation thành công",
            vec!["Desktop".into()],
            0.75,
            30,
            "fallback-vlm",
        );
        fallback.push_canned_observation(canned_fb.clone());

        let router =
            VisionRouter::new(primary, Some(fallback), None, VisionRouterConfig::default());

        let frame = CaptureFrame::new(10, 10, vec![1; 100], 1000);
        let obs = router.process_frame(&frame);

        assert_eq!(obs.description, canned_fb.description);
        assert_eq!(obs.model_used, "fallback-vlm");
    }

    #[test]
    fn test_vision_router_never_crashes_when_both_fail() {
        let primary = Arc::new(MockVisionProvider::new("primary-vlm"));
        primary.set_simulated_error(Some("OOM".into()));

        let fallback = Arc::new(MockVisionProvider::new("fallback-vlm"));
        fallback.set_simulated_error(Some("Network timeout".into()));

        let router =
            VisionRouter::new(primary, Some(fallback), None, VisionRouterConfig::default());

        let frame = CaptureFrame::new(10, 10, vec![1; 100], 1000);
        let obs = router.process_frame(&frame);

        assert_eq!(obs.confidence, 0.0);
        assert!(obs.description.contains("Không thể quan sát màn hình"));
    }
}
