//! Capture ports.
//!
//! The core defines *what* a screen capturer must do; the Tauri shell provides
//! the OS-specific *how* (via `xcap` or platform APIs) behind this trait. This
//! keeps platform code out of the domain (Constitution IV) and lets capture
//! logic be tested against an in-memory fake.

use crate::geometry::Rect;

/// A captured image: raw RGBA8 pixels plus dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedImage {
    pub width: u32,
    pub height: u32,
    /// RGBA8, row-major, `width * height * 4` bytes.
    pub rgba: Vec<u8>,
}

/// Error returned when a capture cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    /// The requested region was empty or fell outside available displays.
    InvalidRegion,
    /// The platform backend failed; carries a human-readable reason.
    Backend(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::InvalidRegion => write!(f, "invalid capture region"),
            CaptureError::Backend(reason) => write!(f, "capture backend error: {reason}"),
        }
    }
}

impl std::error::Error for CaptureError {}

/// Captures pixels from the screen. Implemented per-platform in the shell.
pub trait ScreenCapturer {
    /// Captures the given screen region.
    fn capture_region(&self, region: Rect) -> Result<CapturedImage, CaptureError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// In-memory capturer that lets the domain be tested without a display.
    struct FakeCapturer;

    impl ScreenCapturer for FakeCapturer {
        fn capture_region(&self, region: Rect) -> Result<CapturedImage, CaptureError> {
            if region.is_empty() {
                return Err(CaptureError::InvalidRegion);
            }
            let len = region.area() as usize * 4;
            Ok(CapturedImage {
                width: region.width,
                height: region.height,
                rgba: vec![0; len],
            })
        }
    }

    #[test]
    fn fake_capturer_returns_sized_buffer() {
        let img = FakeCapturer
            .capture_region(Rect::new(0, 0, 2, 2))
            .expect("non-empty region captures");
        assert_eq!(img.width, 2);
        assert_eq!(img.rgba.len(), 2 * 2 * 4);
    }

    #[test]
    fn empty_region_is_rejected() {
        let err = FakeCapturer
            .capture_region(Rect::new(0, 0, 0, 0))
            .unwrap_err();
        assert_eq!(err, CaptureError::InvalidRegion);
    }
}
