//! The saved-image preview toast: a small borderless, always-on-top window at
//! the screen's bottom-right that shows a thumbnail of a just-saved pin and
//! offers Show in Finder / Copy Path / Delete on double-click.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// Opens (or replaces) the preview toast for `path`, anchored bottom-right.
pub fn open(app: &AppHandle, path: &str) -> tauri::Result<()> {
    if let Some(existing) = app.get_webview_window("preview") {
        let _ = existing.close();
    }

    let w = 280.0_f64;
    let h = 240.0_f64;
    let (x, y) = bottom_right(app, w, h);

    let url = WebviewUrl::App(format!("preview.html?path={}", encode_query(path)).into());
    WebviewWindowBuilder::new(app, "preview", url)
        .title("PinShot Saved")
        .inner_size(w, h)
        .position(x, y)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .transparent(true)
        .shadow(false)
        .focused(false)
        .build()?;
    Ok(())
}

/// Closes the preview toast if open (e.g. after Delete).
pub fn close(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("preview") {
        let _ = window.close();
    }
}

/// Bottom-right corner of the primary monitor, in logical coords, for a `w`×`h`
/// window with a small margin.
fn bottom_right(app: &AppHandle, w: f64, h: f64) -> (f64, f64) {
    let margin = 24.0;
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let sf = monitor.scale_factor();
        let pos = monitor.position();
        let size = monitor.size();
        let mx = pos.x as f64 / sf;
        let my = pos.y as f64 / sf;
        let mw = size.width as f64 / sf;
        let mh = size.height as f64 / sf;
        (mx + mw - w - margin, my + mh - h - margin)
    } else {
        (1000.0, 600.0)
    }
}

/// Percent-encode a path so it survives as a URL query value.
fn encode_query(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
