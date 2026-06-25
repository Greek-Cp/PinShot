//! macOS screen capturer built on the system `screencapture(1)` tool.
//!
//! **Why not `xcap` for pixels on macOS.** `xcap` (0.9.6, latest at time of
//! writing) grabs pixels via the CoreGraphics `CGWindowListCreateImage` API.
//! Apple deprecated that API in macOS 14 and it returns a blank/black image on
//! macOS 15+ *even when Screen Recording permission is granted* — the modern
//! replacement is ScreenCaptureKit. Rather than hand-roll an unsafe
//! ScreenCaptureKit binding, we shell out to Apple's own `/usr/sbin/screencapture`,
//! which uses the supported path and returns real pixels once the permission is
//! granted. `xcap` is still used for display *enumeration* (origin, scale): that
//! relies on `CGDisplayBounds`, which is unaffected by the deprecation.
//!
//! The capture is per-display (`-D<n>`, where `-D1` is the main display), giving
//! each frame at the display's native physical resolution — exactly what
//! [`crate::capture`]'s crop step expects.

use std::process::Command;

use pinshot_core::{CaptureError, Display, FrozenFrame, ScreenCapturer};
use xcap::Monitor;

/// Absolute path so capture works regardless of the (minimal) PATH a
/// GUI-launched `.app` inherits from launchd.
const SCREENCAPTURE: &str = "/usr/sbin/screencapture";

/// Captures real monitors via the system `screencapture` tool.
pub struct MacScreencaptureCapturer;

fn backend<E: std::fmt::Display>(e: E) -> CaptureError {
    CaptureError::Backend(e.to_string())
}

impl ScreenCapturer for MacScreencaptureCapturer {
    fn capture_all(&self) -> Result<(Vec<Display>, Vec<FrozenFrame>), CaptureError> {
        let monitors = Monitor::all().map_err(backend)?;
        if monitors.is_empty() {
            return Err(CaptureError::NoDisplays);
        }

        let tmp_dir = std::env::temp_dir();
        let pid = std::process::id();

        let mut displays = Vec::with_capacity(monitors.len());
        let mut frames = Vec::with_capacity(monitors.len());

        for (index, monitor) in monitors.iter().enumerate() {
            let id = monitor.id().map_err(backend)?;
            let x = monitor.x().map_err(backend)?;
            let y = monitor.y().map_err(backend)?;
            let scale = monitor.scale_factor().map_err(backend)? as f64;

            // Capture this display to a unique temp PNG. `-x` silences the
            // shutter sound; `-r` strips DPI metadata; `-D<n>` selects the
            // display (1-based, main first).
            let path = tmp_dir.join(format!("pinshot-capture-{pid}-{index}.png"));
            let status = Command::new(SCREENCAPTURE)
                .arg("-x")
                .arg("-r")
                .arg(format!("-D{}", index + 1))
                .arg(&path)
                .status()
                .map_err(backend)?;
            if !status.success() {
                return Err(CaptureError::Backend(format!(
                    "screencapture exited with {status}"
                )));
            }

            let bytes = std::fs::read(&path).map_err(backend)?;
            let _ = std::fs::remove_file(&path);

            // The decoded PNG is authoritative for physical size (the crop step
            // requires frame size == Display.size).
            let image = image::load_from_memory(&bytes).map_err(backend)?.to_rgba8();
            let width = image.width();
            let height = image.height();

            displays.push(Display {
                id,
                origin: (x, y),
                size: (width, height),
                scale_factor: scale,
            });
            frames.push(FrozenFrame {
                display_id: id,
                width,
                height,
                rgba: image.into_raw(),
                // Hand the overlay screencapture's own PNG so it never has to
                // re-encode the full-screen frame just to draw the backdrop.
                source_png: Some(bytes),
            });
        }

        Ok((displays, frames))
    }
}
