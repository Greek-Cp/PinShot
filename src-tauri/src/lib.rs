//! PinShot Tauri shell entry point.
//!
//! The shell owns the OS integration (windows, tray, hotkeys, and the
//! platform capture adapters). Domain logic lives in `pinshot-core`; this
//! layer wires it to Tauri commands. Kept thin on purpose (Constitution IV).

use pinshot_core::Rect;

/// Returns the normalised selection rectangle for a drag, as the frontend
/// would request it. A first, trivial command that proves the core ↔ shell
/// ↔ webview wiring works end to end.
#[tauri::command]
fn selection_rect(x0: i32, y0: i32, x1: i32, y1: i32) -> SelectionRect {
    Rect::from_points(x0, y0, x1, y1).into()
}

/// Serialisable view of [`Rect`] sent across the Tauri IPC boundary.
#[derive(serde::Serialize)]
struct SelectionRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl From<Rect> for SelectionRect {
    fn from(r: Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

/// Builds and runs the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![selection_rect])
        .run(tauri::generate_context!())
        .expect("error while running PinShot");
}
