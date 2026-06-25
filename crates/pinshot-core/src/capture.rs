//! Capture ports and the data the capture flow operates on.
//!
//! The core defines *what* a screen capturer must do and the geometry of the
//! result; the Tauri shell provides the OS-specific *how* (via `xcap`) behind
//! the [`ScreenCapturer`] trait. This keeps platform code out of the domain
//! (Constitution IV) and lets the selection/crop logic be tested against an
//! in-memory fake without a display.

use crate::geometry::Rect;

/// A connected monitor, described in **physical** virtual-desktop pixels.
///
/// `origin` is the display's top-left within the virtual desktop and `size` is
/// its physical resolution, so the display occupies the rectangle
/// `[origin, origin + size)`. `scale_factor` is the DPI scale (1.0, 1.5, 2.0…)
/// used to convert the overlay's logical coordinates to physical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Display {
    pub id: u32,
    pub origin: (i32, i32),
    pub size: (u32, u32),
    pub scale_factor: f64,
}

impl Display {
    /// The display's rectangle in physical virtual-desktop pixels.
    pub fn bounds(&self) -> Rect {
        Rect::new(self.origin.0, self.origin.1, self.size.0, self.size.1)
    }
}

/// One display's pixels captured at trigger time ("frozen").
///
/// `width`/`height` equal the owning [`Display`]'s physical size and `rgba`
/// holds `width * height * 4` bytes, row-major.
///
/// `source_png` optionally carries the original encoded PNG the capture backend
/// produced (the macOS backend captures straight to PNG). The overlay backdrop
/// can serve these bytes verbatim instead of re-encoding the full-screen `rgba`
/// — re-encoding an ~8 MP frame with zlib is the single most expensive step in
/// showing the overlay, so reusing the bytes we already have removes the lag.
/// Backends that only produce raw pixels (xcap) leave this `None`, and the
/// shell falls back to encoding on demand.
#[derive(Debug, Clone, PartialEq)]
pub struct FrozenFrame {
    pub display_id: u32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub source_png: Option<Vec<u8>>,
}

impl FrozenFrame {
    /// True if the frame is, for all practical purposes, a black "no
    /// permission" capture (at least 99.5% of pixels are pure black, ignoring
    /// alpha).
    ///
    /// macOS hands an app an essentially-black frame instead of an error when
    /// Screen Recording permission has not (yet) taken effect — including the
    /// window between granting it and relaunching, and the very common case of
    /// an ad-hoc-signed build whose TCC grant went stale after a rebuild (the
    /// toggle still shows enabled but the cdhash no longer matches). Such a
    /// frame is not always *perfectly* black — a cursor or a stray menu-bar
    /// pixel can leave a handful of non-black pixels — so an exact all-black
    /// test misses it and the user gets a useless black overlay. A high black
    /// ratio catches the denied capture while staying clear of legitimately
    /// dark screens (dark wallpaper / fullscreen dark apps still carry
    /// anti-aliased text and chrome, far more than 0.5% non-black).
    pub fn is_blank(&self) -> bool {
        let total = self.rgba.len() / 4;
        if total == 0 {
            return true;
        }
        let non_black = self
            .rgba
            .chunks_exact(4)
            .filter(|px| px[0] != 0 || px[1] != 0 || px[2] != 0)
            .count();
        // Blank when <0.5% of pixels carry any colour.
        non_black * 200 < total
    }
}

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
    /// The OS withheld a required screen-capture permission (e.g. macOS Screen
    /// Recording). The shell surfaces an actionable message (FR-016).
    PermissionDenied,
    /// No displays were reported by the platform.
    NoDisplays,
    /// The platform backend failed; carries a human-readable reason.
    Backend(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::PermissionDenied => write!(f, "screen-capture permission denied"),
            CaptureError::NoDisplays => write!(f, "no displays available"),
            CaptureError::Backend(reason) => write!(f, "capture backend error: {reason}"),
        }
    }
}

impl std::error::Error for CaptureError {}

/// Captures every display's pixels at once. Implemented per-platform in the
/// shell (over `xcap`); the returned frames are frozen for selection.
pub trait ScreenCapturer {
    /// Captures all connected displays, returning their metadata and frozen
    /// pixel buffers (one [`FrozenFrame`] per [`Display`]).
    fn capture_all(&self) -> Result<(Vec<Display>, Vec<FrozenFrame>), CaptureError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// In-memory capturer with two displays so the domain can be exercised
    /// without a real screen: a 2× HiDPI display beside a 1× display.
    struct FakeCapturer;

    impl ScreenCapturer for FakeCapturer {
        fn capture_all(&self) -> Result<(Vec<Display>, Vec<FrozenFrame>), CaptureError> {
            let displays = vec![
                Display {
                    id: 1,
                    origin: (0, 0),
                    size: (200, 100),
                    scale_factor: 2.0,
                },
                Display {
                    id: 2,
                    origin: (200, 0),
                    size: (100, 100),
                    scale_factor: 1.0,
                },
            ];
            let frames = displays
                .iter()
                .map(|d| FrozenFrame {
                    display_id: d.id,
                    width: d.size.0,
                    height: d.size.1,
                    source_png: None,
                    rgba: vec![0u8; (d.size.0 * d.size.1 * 4) as usize],
                })
                .collect();
            Ok((displays, frames))
        }
    }

    #[test]
    fn capture_all_returns_one_frame_per_display() {
        let (displays, frames) = FakeCapturer.capture_all().expect("fake captures");
        assert_eq!(displays.len(), 2);
        assert_eq!(frames.len(), 2);
        for (d, f) in displays.iter().zip(&frames) {
            assert_eq!(d.id, f.display_id);
            assert_eq!(f.rgba.len(), (f.width * f.height * 4) as usize);
        }
    }

    #[test]
    fn display_bounds_uses_physical_pixels() {
        let d = Display {
            id: 1,
            origin: (200, 50),
            size: (100, 80),
            scale_factor: 1.0,
        };
        assert_eq!(d.bounds(), Rect::new(200, 50, 100, 80));
    }

    fn frame(rgba: Vec<u8>) -> FrozenFrame {
        FrozenFrame {
            display_id: 1,
            width: 2,
            height: 2,
            rgba,
            source_png: None,
        }
    }

    #[test]
    fn all_black_frame_is_blank() {
        // Opaque black (alpha 255) still counts as blank — alpha is ignored.
        assert!(frame(vec![0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]).is_blank());
        // Fully zeroed (alpha 0) too.
        assert!(frame(vec![0u8; 16]).is_blank());
    }

    #[test]
    fn frame_with_any_colour_is_not_blank() {
        let mut rgba = vec![0u8; 16];
        rgba[8] = 1; // one barely-non-black pixel is enough in a 4px frame
        assert!(!frame(rgba).is_blank());
    }

    /// A denied macOS capture is essentially black but not always perfectly so
    /// (a cursor or stray menu-bar pixel survives); it must still read as blank
    /// so the shell shows the permission guidance instead of a black overlay.
    #[test]
    fn predominantly_black_frame_is_blank() {
        let pixels = 1000;
        let mut rgba = vec![0u8; pixels * 4];
        rgba[0] = 255; // a single non-black pixel out of 1000 (0.1%)
        let f = FrozenFrame {
            display_id: 1,
            width: pixels as u32,
            height: 1,
            rgba,
            source_png: None,
        };
        assert!(f.is_blank());
    }

    /// A legitimately dark screen still carries well over 0.5% colour (chrome,
    /// anti-aliased text) and must not be mistaken for a denied capture.
    #[test]
    fn dark_but_real_frame_is_not_blank() {
        let pixels = 1000;
        let mut rgba = vec![0u8; pixels * 4];
        for i in 0..50 {
            rgba[i * 4] = 30; // 5% of pixels carry colour
        }
        let f = FrozenFrame {
            display_id: 1,
            width: pixels as u32,
            height: 1,
            rgba,
            source_png: None,
        };
        assert!(!f.is_blank());
    }
}
