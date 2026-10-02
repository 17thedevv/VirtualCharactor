//! Native Windows Screen Capture & Active Window Sensing using Win32 GDI.
//!
//! Provides hardware-accelerated GDI desktop frame capture, CPU downsampling,
//! and foreground application tracking with zero external dependencies.

use crate::error::{Result, RuntimeError};
use crate::vision::sensing::{CaptureFrame, ScreenCaptureProvider};

/// Configuration options for Windows Screen Capture.
#[derive(Debug, Clone)]
pub struct WindowsCaptureConfig {
    /// Thumbnail width used for fast CPU visual diff calculation (e.g., 64).
    pub thumb_width: u32,
    /// Thumbnail height used for fast CPU visual diff calculation (e.g., 64).
    pub thumb_height: u32,
    /// Optional maximum width for encoded preview BMP (e.g., 640). If 0, full resolution is kept.
    pub preview_max_width: u32,
    /// Whether to generate encoded BMP bytes inside `CaptureFrame.encoded_image`.
    pub include_encoded_bmp: bool,
}

impl Default for WindowsCaptureConfig {
    fn default() -> Self {
        Self {
            thumb_width: 64,
            thumb_height: 64,
            preview_max_width: 640,
            include_encoded_bmp: true,
        }
    }
}

/// Native Windows Screen Capture Provider communicating directly with Win32 GDI and User32 APIs.
pub struct WindowsScreenCaptureProvider {
    config: WindowsCaptureConfig,
}

impl WindowsScreenCaptureProvider {
    pub fn new() -> Self {
        Self {
            config: WindowsCaptureConfig::default(),
        }
    }

    pub fn with_config(config: WindowsCaptureConfig) -> Self {
        Self { config }
    }

    /// Retrieve configuration.
    pub fn config(&self) -> &WindowsCaptureConfig {
        &self.config
    }
}

impl Default for WindowsScreenCaptureProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
mod ffi {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type HDC = *mut c_void;
    pub type HBITMAP = *mut c_void;
    pub type HGDIOBJ = *mut c_void;
    pub type HANDLE = *mut c_void;
    pub type BOOL = i32;

    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;
    pub const SRCCOPY: u32 = 0x00CC0020;
    pub const CAPTUREBLT: u32 = 0x40000000;
    pub const DIB_RGB_COLORS: u32 = 0;
    pub const BI_RGB: u32 = 0;
    pub const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct BITMAPINFOHEADER {
        pub biSize: u32,
        pub biWidth: i32,
        pub biHeight: i32,
        pub biPlanes: u16,
        pub biBitCount: u16,
        pub biCompression: u32,
        pub biSizeImage: u32,
        pub biXPelsPerMeter: i32,
        pub biYPelsPerMeter: i32,
        pub biClrUsed: u32,
        pub biClrImportant: u32,
    }

    #[repr(C)]
    pub struct BITMAPINFO {
        pub bmiHeader: BITMAPINFOHEADER,
        pub bmiColors: [u32; 1],
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn SetProcessDPIAware() -> BOOL;
        pub fn GetDC(hWnd: HWND) -> HDC;
        pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> i32;
        pub fn GetSystemMetrics(nIndex: i32) -> i32;
        pub fn GetForegroundWindow() -> HWND;
        pub fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
        pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut u32) -> u32;
        pub fn OpenInputDesktop(dwFlags: u32, fInherit: BOOL, dwDesiredAccess: u32) -> *mut c_void;
        pub fn SetThreadDesktop(hDesktop: *mut c_void) -> BOOL;
        pub fn CloseDesktop(hDesktop: *mut c_void) -> BOOL;
    }

    #[link(name = "gdi32")]
    extern "system" {
        pub fn DeleteDC(hdc: HDC) -> BOOL;
        pub fn CreateCompatibleDC(hDC: HDC) -> HDC;
        pub fn CreateCompatibleBitmap(hDC: HDC, cx: i32, cy: i32) -> HBITMAP;
        pub fn SelectObject(hDC: HDC, h: HGDIOBJ) -> HGDIOBJ;
        pub fn BitBlt(
            hdc: HDC,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            hdcSrc: HDC,
            x1: i32,
            y1: i32,
            rop: u32,
        ) -> BOOL;
        pub fn GetDIBits(
            hdc: HDC,
            hbm: HBITMAP,
            start: u32,
            cLines: u32,
            lpvBits: *mut c_void,
            lpbmi: *mut BITMAPINFO,
            usage: u32,
        ) -> i32;
        pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn SetLastError(dwErrCode: u32);
        pub fn GetLastError() -> u32;
        pub fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: BOOL, dwProcessId: u32) -> HANDLE;
        pub fn QueryFullProcessImageNameW(
            hProcess: HANDLE,
            dwFlags: u32,
            lpExeName: *mut u16,
            lpdwSize: *mut u32,
        ) -> BOOL;
        pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    }
}

#[cfg(target_os = "windows")]
impl WindowsScreenCaptureProvider {
    /// Capture raw 32-bit BGRA pixels from the Windows primary desktop screen.
    ///
    /// Returns `(width, height, raw_bgra_bytes)`.
    pub fn capture_raw_bgra(&self) -> Result<(u32, u32, Vec<u8>)> {
        unsafe {
            ffi::SetProcessDPIAware();

            // Ensure the calling thread is attached to the interactive input desktop (required for subshells / services)
            let hdesk = ffi::OpenInputDesktop(0, 0, 0x0001 | 0x0080 | 0x0100);
            if !hdesk.is_null() {
                ffi::SetThreadDesktop(hdesk);
            }

            let hdc_screen = ffi::GetDC(std::ptr::null_mut());
            if hdc_screen.is_null() {
                if !hdesk.is_null() {
                    ffi::CloseDesktop(hdesk);
                }
                return Err(RuntimeError::VisionError(
                    "GDI GetDC(Desktop) returned NULL".into(),
                ));
            }

            let width = ffi::GetSystemMetrics(ffi::SM_CXSCREEN);
            let height = ffi::GetSystemMetrics(ffi::SM_CYSCREEN);
            if width <= 0 || height <= 0 {
                ffi::ReleaseDC(std::ptr::null_mut(), hdc_screen);
                if !hdesk.is_null() {
                    ffi::CloseDesktop(hdesk);
                }
                return Err(RuntimeError::VisionError(format!(
                    "Invalid screen dimensions detected: {}x{}",
                    width, height
                )));
            }

            let hdc_mem = ffi::CreateCompatibleDC(hdc_screen);
            if hdc_mem.is_null() {
                ffi::ReleaseDC(std::ptr::null_mut(), hdc_screen);
                if !hdesk.is_null() {
                    ffi::CloseDesktop(hdesk);
                }
                return Err(RuntimeError::VisionError(
                    "CreateCompatibleDC failed".into(),
                ));
            }

            let hbm = ffi::CreateCompatibleBitmap(hdc_screen, width, height);
            if hbm.is_null() {
                ffi::DeleteDC(hdc_mem);
                ffi::ReleaseDC(std::ptr::null_mut(), hdc_screen);
                if !hdesk.is_null() {
                    ffi::CloseDesktop(hdesk);
                }
                return Err(RuntimeError::VisionError(
                    "CreateCompatibleBitmap failed".into(),
                ));
            }

            let old_obj = ffi::SelectObject(hdc_mem, hbm);

            ffi::SetLastError(0);
            let mut blt_res = ffi::BitBlt(
                hdc_mem,
                0,
                0,
                width,
                height,
                hdc_screen,
                0,
                0,
                ffi::SRCCOPY | ffi::CAPTUREBLT,
            );

            if blt_res == 0 {
                // Fallback to standard SRCCOPY
                ffi::SetLastError(0);
                blt_res = ffi::BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, ffi::SRCCOPY);
            }

            if blt_res == 0 {
                let err_code = ffi::GetLastError();
                ffi::SelectObject(hdc_mem, old_obj);
                ffi::DeleteObject(hbm);
                ffi::DeleteDC(hdc_mem);
                ffi::ReleaseDC(std::ptr::null_mut(), hdc_screen);
                if !hdesk.is_null() {
                    ffi::CloseDesktop(hdesk);
                }
                return Err(RuntimeError::VisionError(format!(
                    "BitBlt screen capture failed ({}x{}, Win32 Error: {})",
                    width, height, err_code
                )));
            }

            let mut bmi = ffi::BITMAPINFO {
                bmiHeader: ffi::BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<ffi::BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // Negative height indicates top-down DIB
                    biPlanes: 1,
                    biBitCount: 32, // 32-bit BGRA
                    biCompression: ffi::BI_RGB,
                    biSizeImage: (width * height * 4) as u32,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [0],
            };

            let mut raw_pixels = vec![0u8; (width * height * 4) as usize];
            let lines = ffi::GetDIBits(
                hdc_mem,
                hbm,
                0,
                height as u32,
                raw_pixels.as_mut_ptr() as *mut _,
                &mut bmi,
                ffi::DIB_RGB_COLORS,
            );

            // Resource cleanup
            ffi::SelectObject(hdc_mem, old_obj);
            ffi::DeleteObject(hbm);
            ffi::DeleteDC(hdc_mem);
            ffi::ReleaseDC(std::ptr::null_mut(), hdc_screen);
            if !hdesk.is_null() {
                ffi::CloseDesktop(hdesk);
            }

            if lines == 0 {
                return Err(RuntimeError::VisionError(
                    "GetDIBits failed to copy frame buffer".into(),
                ));
            }

            Ok((width as u32, height as u32, raw_pixels))
        }
    }

    /// Capture desktop and encode as standard 24-bit BMP image bytes, optionally downscaled.
    pub fn capture_bmp(&self, max_width: u32) -> Result<Vec<u8>> {
        let (src_w, src_h, raw_bgra) = self.capture_raw_bgra()?;

        if max_width > 0 && src_w > max_width {
            let scale = max_width as f32 / src_w as f32;
            let dst_w = max_width;
            let dst_h = ((src_h as f32 * scale).round() as u32).max(1);
            let downscaled = downsample_bgra(src_w, src_h, dst_w, dst_h, &raw_bgra);
            Ok(encode_bmp_24bit(dst_w, dst_h, &downscaled))
        } else {
            Ok(encode_bmp_24bit(src_w, src_h, &raw_bgra))
        }
    }
}

#[cfg(target_os = "windows")]
impl ScreenCaptureProvider for WindowsScreenCaptureProvider {
    fn capture_screen(&self) -> Result<CaptureFrame> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let (src_w, src_h, raw_bgra) = self.capture_raw_bgra()?;

        // Generate downscaled 8-bit grayscale thumbnail for CPU VisualDiffDetector
        let thumb_w = self.config.thumb_width.max(1);
        let thumb_h = self.config.thumb_height.max(1);
        let mut thumb_luma = Vec::with_capacity((thumb_w * thumb_h) as usize);

        for ty in 0..thumb_h {
            let sy = ty * src_h / thumb_h;
            let row_offset = (sy * src_w * 4) as usize;
            for tx in 0..thumb_w {
                let sx = tx * src_w / thumb_w;
                let px = row_offset + (sx * 4) as usize;
                let b = raw_bgra[px] as u32;
                let g = raw_bgra[px + 1] as u32;
                let r = raw_bgra[px + 2] as u32;
                // ITU-R BT.601 luminance
                let luma = ((r * 299 + g * 587 + b * 114) / 1000) as u8;
                thumb_luma.push(luma);
            }
        }

        // Generate encoded BMP preview if requested
        let encoded_bmp = if self.config.include_encoded_bmp {
            let max_w = self.config.preview_max_width;
            if max_w > 0 && src_w > max_w {
                let scale = max_w as f32 / src_w as f32;
                let dst_w = max_w;
                let dst_h = ((src_h as f32 * scale).round() as u32).max(1);
                let downscaled = downsample_bgra(src_w, src_h, dst_w, dst_h, &raw_bgra);
                Some(encode_bmp_24bit(dst_w, dst_h, &downscaled))
            } else {
                Some(encode_bmp_24bit(src_w, src_h, &raw_bgra))
            }
        } else {
            None
        };

        if let Some(encoded) = encoded_bmp {
            Ok(CaptureFrame::with_encoded(
                thumb_w, thumb_h, thumb_luma, now, encoded,
            ))
        } else {
            Ok(CaptureFrame::new(thumb_w, thumb_h, thumb_luma, now))
        }
    }

    fn get_active_window(&self) -> (String, String) {
        unsafe {
            let hwnd = ffi::GetForegroundWindow();
            if hwnd.is_null() {
                return ("Desktop".to_string(), "explorer.exe".to_string());
            }

            // Window Title
            let mut title_buf = [0u16; 512];
            let len = ffi::GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
            let title = if len > 0 {
                String::from_utf16_lossy(&title_buf[..len as usize])
            } else {
                "Desktop".to_string()
            };

            // Process Name
            let mut pid = 0u32;
            ffi::GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                return (title, "unknown".to_string());
            }

            let h_proc = ffi::OpenProcess(ffi::PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h_proc.is_null() {
                return (title, "unknown".to_string());
            }

            let mut path_buf = [0u16; 1024];
            let mut size = 1024u32;
            let success =
                ffi::QueryFullProcessImageNameW(h_proc, 0, path_buf.as_mut_ptr(), &mut size);
            ffi::CloseHandle(h_proc);

            let process_name = if success != 0 && size > 0 {
                let full_path = String::from_utf16_lossy(&path_buf[..size as usize]);
                std::path::Path::new(&full_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                "unknown".to_string()
            };

            (title, process_name)
        }
    }
}

/// Fallback implementation on non-Windows platforms.
#[cfg(not(target_os = "windows"))]
impl WindowsScreenCaptureProvider {
    pub fn capture_raw_bgra(&self) -> Result<(u32, u32, Vec<u8>)> {
        Err(RuntimeError::VisionError(
            "Windows GDI capture is only supported on Windows OS".into(),
        ))
    }

    pub fn capture_bmp(&self, _max_width: u32) -> Result<Vec<u8>> {
        Err(RuntimeError::VisionError(
            "Windows GDI capture is only supported on Windows OS".into(),
        ))
    }
}

#[cfg(not(target_os = "windows"))]
impl ScreenCaptureProvider for WindowsScreenCaptureProvider {
    fn capture_screen(&self) -> Result<CaptureFrame> {
        Ok(CaptureFrame::blank(
            self.config.thumb_width,
            self.config.thumb_height,
        ))
    }

    fn get_active_window(&self) -> (String, String) {
        ("Non-Windows OS".into(), "none".into())
    }
}

/// Downsample top-down 32-bit BGRA pixels into target dimensions using nearest-neighbor interpolation.
pub fn downsample_bgra(src_w: u32, src_h: u32, dst_w: u32, dst_h: u32, src: &[u8]) -> Vec<u8> {
    let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
    for dy in 0..dst_h {
        let sy = dy * src_h / dst_h;
        let src_row = (sy * src_w * 4) as usize;
        let dst_row = (dy * dst_w * 4) as usize;
        for dx in 0..dst_w {
            let sx = dx * src_w / dst_w;
            let src_px = src_row + (sx * 4) as usize;
            let dst_px = dst_row + (dx * 4) as usize;
            dst[dst_px..dst_px + 4].copy_from_slice(&src[src_px..src_px + 4]);
        }
    }
    dst
}

/// Encode 32-bit top-down BGRA pixel data into standard 24-bit bottom-up BMP bytes.
pub fn encode_bmp_24bit(width: u32, height: u32, bgra_pixels: &[u8]) -> Vec<u8> {
    let row_stride = ((width * 3 + 3) / 4) * 4;
    let image_size = row_stride * height;
    let file_size = 54 + image_size;

    let mut bmp = Vec::with_capacity(file_size as usize);

    // BITMAPFILEHEADER (14 bytes)
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&0u16.to_le_bytes());
    bmp.extend_from_slice(&0u16.to_le_bytes());
    bmp.extend_from_slice(&54u32.to_le_bytes());

    // BITMAPINFOHEADER (40 bytes)
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(width as i32).to_le_bytes());
    // Positive height for standard bottom-up DIB (maximum compatibility)
    bmp.extend_from_slice(&(height as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes());
    bmp.extend_from_slice(&24u16.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&image_size.to_le_bytes());
    bmp.extend_from_slice(&2835i32.to_le_bytes());
    bmp.extend_from_slice(&2835i32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());

    // Pixel array (bottom-up: starting from bottom row up to top row)
    let pad_len = (row_stride - width * 3) as usize;
    let pad_bytes = [0u8; 4];

    for y in (0..height).rev() {
        let src_row = (y * width * 4) as usize;
        for x in 0..width {
            let px = src_row + (x * 4) as usize;
            let b = bgra_pixels[px];
            let g = bgra_pixels[px + 1];
            let r = bgra_pixels[px + 2];
            bmp.push(b);
            bmp.push(g);
            bmp.push(r);
        }
        if pad_len > 0 {
            bmp.extend_from_slice(&pad_bytes[..pad_len]);
        }
    }

    bmp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bmp_encoding_valid_header() {
        let width = 2;
        let height = 2;
        // 2x2 BGRA pixels (4 pixels * 4 bytes = 16 bytes)
        let bgra = vec![
            255, 0, 0, 255, // Blue
            0, 255, 0, 255, // Green
            0, 0, 255, 255, // Red
            255, 255, 255, 255, // White
        ];

        let bmp = encode_bmp_24bit(width, height, &bgra);
        assert!(bmp.len() >= 54);
        assert_eq!(&bmp[0..2], b"BM");

        // Stride for 2 pixels: 2 * 3 = 6 bytes -> padded to 8 bytes.
        // Image size: 8 * 2 = 16 bytes. Total file size: 54 + 16 = 70 bytes.
        assert_eq!(bmp.len(), 70);

        // Verify biWidth and biHeight in BITMAPINFOHEADER (offset 18)
        let w = i32::from_le_bytes(bmp[18..22].try_into().unwrap());
        let h = i32::from_le_bytes(bmp[22..26].try_into().unwrap());
        assert_eq!(w, 2);
        assert_eq!(h, 2);
    }

    #[test]
    fn test_downsample_bgra_dimensions() {
        let src_w = 4;
        let src_h = 4;
        let src = vec![128u8; (src_w * src_h * 4) as usize];

        let dst = downsample_bgra(src_w, src_h, 2, 2, &src);
        assert_eq!(dst.len(), (2 * 2 * 4) as usize);
        assert_eq!(dst[0], 128);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_real_windows_screen_capture_and_active_window() {
        let provider = WindowsScreenCaptureProvider::new();

        // 1. Verify Active Window detection
        let (title, process) = provider.get_active_window();
        assert!(!process.is_empty(), "Process name should not be empty");
        println!(
            "Real Active Window: Title='{}', Process='{}'",
            title, process
        );

        // 2. Verify Raw BGRA Capture
        let (raw_w, raw_h, raw_bgra) = provider
            .capture_raw_bgra()
            .expect("Raw BGRA capture should succeed");
        assert!(
            raw_w >= 640,
            "Screen width should be at least 640, got {}",
            raw_w
        );
        assert!(
            raw_h >= 480,
            "Screen height should be at least 480, got {}",
            raw_h
        );
        assert_eq!(raw_bgra.len(), (raw_w * raw_h * 4) as usize);

        // 3. Verify ScreenCaptureProvider trait implementation
        let frame = provider
            .capture_screen()
            .expect("Real Windows screen capture should succeed");
        assert_eq!(frame.width, 64);
        assert_eq!(frame.height, 64);
        assert_eq!(frame.data.len(), 4096);

        // Verify captured frame contains actual display data (non-zero pixels)
        let has_non_zero = frame.data.iter().any(|&b| b > 0);
        assert!(
            has_non_zero,
            "Captured screen frame should not be all-black zeroes"
        );

        // 4. Verify BMP encoding
        assert!(
            frame.encoded_image.is_some(),
            "Encoded BMP preview should be generated"
        );
        let bmp_data = frame.encoded_image.unwrap();
        assert!(bmp_data.len() > 54, "BMP must be larger than header");
        assert_eq!(&bmp_data[0..2], b"BM");

        // 5. Verify direct BMP capture with max_width constraint
        let preview_bmp = provider
            .capture_bmp(640)
            .expect("Capture BMP downscaled should succeed");
        assert!(preview_bmp.len() > 54);
        assert_eq!(&preview_bmp[0..2], b"BM");
        println!(
            "Screen Capture Verified: {}x{} raw ({} bytes) -> 640px preview BMP ({} bytes)",
            raw_w,
            raw_h,
            raw_bgra.len(),
            preview_bmp.len()
        );

        // 6. Save BMP artifact to scratch directory for empirical audit verification
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
        let scratch_dir = std::path::Path::new(&manifest_dir)
            .ancestors()
            .find(|p| p.join("scratch").exists() || p.join("Cargo.lock").exists())
            .map(|p| p.join("scratch"))
            .unwrap_or_else(|| std::path::PathBuf::from("scratch"));

        let _ = std::fs::create_dir_all(&scratch_dir);
        let scratch_file = scratch_dir.join("captured_real_desktop.bmp");
        std::fs::write(&scratch_file, &preview_bmp).expect("Should save captured BMP artifact");
        println!(
            "Saved real desktop preview artifact to: {}",
            scratch_file.display()
        );
        assert!(scratch_file.exists());
        let file_len = std::fs::metadata(&scratch_file).unwrap().len();
        assert_eq!(file_len, preview_bmp.len() as u64);
    }
}
