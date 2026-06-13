//! Selection overlay windows — one borderless, always-on-top window per display.
//!
//! Each window loads `overlay.html?display=<id>` and covers exactly its
//! display; the overlay frontend pulls its frozen frame via the
//! `get_overlay_frame` command and reports the selection back.

use pinshot_core::Display;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

fn label_for(display_id: u32) -> String {
    format!("overlay-{display_id}")
}

/// Creates and shows one overlay window per display, positioned to cover it.
pub fn show(app: &AppHandle, displays: &[Display]) -> tauri::Result<()> {
    for display in displays {
        let label = label_for(display.id);
        if let Some(existing) = app.get_webview_window(&label) {
            let _ = existing.close();
        }

        // Tauri positions/sizes windows in logical units; convert from the
        // display's physical pixels using its scale factor.
        let scale = display.scale_factor.max(0.1);
        let lx = f64::from(display.origin.0) / scale;
        let ly = f64::from(display.origin.1) / scale;
        let lw = f64::from(display.size.0) / scale;
        let lh = f64::from(display.size.1) / scale;

        let url = WebviewUrl::App(format!("overlay.html?display={}", display.id).into());
        WebviewWindowBuilder::new(app, &label, url)
            .title("PinShot Capture")
            .position(lx, ly)
            .inner_size(lw, lh)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .shadow(false)
            .build()?;
    }
    Ok(())
}

/// Closes every overlay window.
pub fn close(app: &AppHandle) {
    for (label, window) in app.webview_windows() {
        if label.starts_with("overlay-") {
            let _ = window.close();
        }
    }
}
