//! Level 1: Screen Sensing & Perceptual Diff Detection.
//!
//! Executes lightweight visual frame comparison on the CPU without allocating GPU VRAM.

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Captured raw image frame or thumbnail from the screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureFrame {
    pub width: u32,
    pub height: u32,
    /// 8-bit grayscale thumbnail or raw RGB pixel buffer
    pub data: Vec<u8>,
    pub timestamp: u64,
    /// Optional encoded image bytes (e.g. BMP / JPEG preview)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoded_image: Option<Vec<u8>>,
}

impl CaptureFrame {
    pub fn new(width: u32, height: u32, data: Vec<u8>, timestamp: u64) -> Self {
        Self {
            width,
            height,
            data,
            timestamp,
            encoded_image: None,
        }
    }

    /// Create a frame with pre-encoded image bytes (e.g. BMP or JPEG).
    pub fn with_encoded(
        width: u32,
        height: u32,
        data: Vec<u8>,
        timestamp: u64,
        encoded: Vec<u8>,
    ) -> Self {
        Self {
            width,
            height,
            data,
            timestamp,
            encoded_image: Some(encoded),
        }
    }

    /// Create an empty or blank frame.
    pub fn blank(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0u8; (width * height) as usize],
            timestamp: 0,
            encoded_image: None,
        }
    }
}

/// Abstract contract for capturing desktop screens.
pub trait ScreenCaptureProvider: Send + Sync {
    /// Capture the current primary desktop screen or region of interest.
    fn capture_screen(&self) -> Result<CaptureFrame>;
    /// Retrieve the currently focused window title and process name.
    fn get_active_window(&self) -> (String, String);
}

/// Deterministic mock capture provider for unit testing without OS display dependencies.
pub struct MockScreenCaptureProvider {
    canned_frames: std::sync::Mutex<Vec<CaptureFrame>>,
    active_window: std::sync::Mutex<(String, String)>,
}

impl MockScreenCaptureProvider {
    pub fn new(initial_window_title: &str, initial_process: &str) -> Self {
        Self {
            canned_frames: std::sync::Mutex::new(Vec::new()),
            active_window: std::sync::Mutex::new((
                initial_window_title.to_string(),
                initial_process.to_string(),
            )),
        }
    }

    pub fn push_frame(&self, frame: CaptureFrame) {
        let mut frames = self.canned_frames.lock().unwrap();
        frames.push(frame);
    }

    pub fn set_active_window(&self, title: &str, process: &str) {
        let mut win = self.active_window.lock().unwrap();
        *win = (title.to_string(), process.to_string());
    }
}

impl ScreenCaptureProvider for MockScreenCaptureProvider {
    fn capture_screen(&self) -> Result<CaptureFrame> {
        let mut frames = self.canned_frames.lock().unwrap();
        if !frames.is_empty() {
            Ok(frames.remove(0))
        } else {
            Ok(CaptureFrame::blank(64, 64))
        }
    }

    fn get_active_window(&self) -> (String, String) {
        let win = self.active_window.lock().unwrap();
        win.clone()
    }
}

/// Perceptual difference detector operating on CPU.
pub struct VisualDiffDetector;

impl VisualDiffDetector {
    /// Compute the pixel change percentage between two identical-dimension thumbnails (0.0 to 1.0).
    ///
    /// Uses absolute luminance difference with a noise gate threshold.
    pub fn compute_diff(frame_a: &CaptureFrame, frame_b: &CaptureFrame) -> f32 {
        if frame_a.data.is_empty() || frame_b.data.is_empty() {
            return 0.0;
        }

        let len = frame_a.data.len().min(frame_b.data.len());
        if len == 0 {
            return 0.0;
        }

        let mut diff_pixels = 0usize;
        let noise_threshold = 12u8; // Ignore sub-12 level noise compression artifacts

        for i in 0..len {
            let val_a = frame_a.data[i];
            let val_b = frame_b.data[i];
            if val_a.abs_diff(val_b) > noise_threshold {
                diff_pixels += 1;
            }
        }

        diff_pixels as f32 / len as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_frames_have_zero_diff() {
        let f1 = CaptureFrame::new(4, 4, vec![100; 16], 100);
        let f2 = CaptureFrame::new(4, 4, vec![100; 16], 101);

        let diff = VisualDiffDetector::compute_diff(&f1, &f2);
        assert_eq!(diff, 0.0);
    }

    #[test]
    fn test_different_frames_report_correct_percentage() {
        let d1 = vec![0u8; 100];
        let mut d2 = vec![0u8; 100];

        // Change 25 pixels beyond noise threshold
        for i in 0..25 {
            d2[i] = 200;
        }

        let f1 = CaptureFrame::new(10, 10, d1, 100);
        let f2 = CaptureFrame::new(10, 10, d2, 101);

        let diff = VisualDiffDetector::compute_diff(&f1, &f2);
        assert!((diff - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_sub_noise_threshold_is_filtered() {
        let d1 = vec![100u8; 50];
        let d2 = vec![105u8; 50]; // 5 units diff, below 12-unit noise threshold

        let f1 = CaptureFrame::new(5, 10, d1, 100);
        let f2 = CaptureFrame::new(5, 10, d2, 101);

        let diff = VisualDiffDetector::compute_diff(&f1, &f2);
        assert_eq!(diff, 0.0);
    }
}
