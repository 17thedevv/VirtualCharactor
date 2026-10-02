//! Vision & Screen Perception Subsystem.
//!
//! Provides Level 1 Screen Sensing (CPU visual diff), Level 2 Active Window Perception,
//! and Level 3 On-Demand VLM Understanding via VisionRouter.

pub mod provider;
pub mod router;
pub mod sensing;
pub mod windows;

pub use provider::{MockVisionProvider, OllamaVisionProvider, VisionObservation, VisionProvider};
pub use router::{VisionRouter, VisionRouterConfig};
pub use sensing::{
    CaptureFrame, MockScreenCaptureProvider, ScreenCaptureProvider, VisualDiffDetector,
};
pub use windows::{WindowsCaptureConfig, WindowsScreenCaptureProvider};
