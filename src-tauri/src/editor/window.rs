//! The editor window: a borderless, always-on-top webview hosting `editor.html`.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// Opens (or replaces) the editor window, sized to fit the capture at its
/// logical scale plus room for the floating toolbar and action bar.
pub fn open(app: &AppHandle, width: u32, height: u32, scale: f64) -> tauri::Result<()> {
    if let Some(existing) = app.get_webview_window("editor") {
        let _ = existing.close();
    }

    let scale = scale.max(0.1);
    let logical_w = (width as f64 / scale).max(200.0);
    let logical_h = (height as f64 / scale).max(150.0);
    // Horizontal padding for the canvas margin; vertical room for the toolbar
    // (top) and the action bar (bottom).
    let win_w = logical_w + 48.0;
    let win_h = logical_h + 140.0;

    let window = WebviewWindowBuilder::new(app, "editor", WebviewUrl::App("editor.html".into()))
        .title("PinShot Editor")
        .inner_size(win_w, win_h)
        .min_inner_size(360.0, 280.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .shadow(true)
        .center()
        .focused(true)
        .build()?;
    let _ = window.set_focus();
    Ok(())
}

/// Closes the editor window if open.
pub fn close(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("editor") {
        let _ = window.close();
    }
}
