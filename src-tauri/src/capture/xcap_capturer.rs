//! `ScreenCapturer` implementation over the `xcap` crate (platform adapter).
//!
//! Implements the core's capture port; the rest of the app depends only on the
//! trait, not on xcap (Constitution IV).

use pinshot_core::{CaptureError, Display, FrozenFrame, ScreenCapturer};
use xcap::Monitor;

/// Captures real monitors via `xcap`.
pub struct XcapCapturer;

fn backend<E: std::fmt::Display>(e: E) -> CaptureError {
    CaptureError::Backend(e.to_string())
}

impl ScreenCapturer for XcapCapturer {
    fn capture_all(&self) -> Result<(Vec<Display>, Vec<FrozenFrame>), CaptureError> {
        let monitors = Monitor::all().map_err(backend)?;
        if monitors.is_empty() {
            return Err(CaptureError::NoDisplays);
        }

        let mut displays = Vec::with_capacity(monitors.len());
        let mut frames = Vec::with_capacity(monitors.len());

        for monitor in monitors {
            let id = monitor.id().map_err(backend)?;
            let x = monitor.x().map_err(backend)?;
            let y = monitor.y().map_err(backend)?;
            let scale = monitor.scale_factor().map_err(backend)? as f64;

            // The captured image is authoritative for physical size, so the
            // frame always matches Display.size (crop relies on this).
            let image = monitor.capture_image().map_err(backend)?;
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
            });
        }

        Ok((displays, frames))
    }
}
