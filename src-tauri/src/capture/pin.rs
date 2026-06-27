//! Floating pins — the shell side of feature 003.
//!
//! Each pin is its own borderless, always-on-top webview window showing a
//! captured region at full physical size. This module owns the in-memory
//! registry of pinned images and the window lifecycle (create, close, raise);
//! the pure size/placement math lives in [`pinshot_core::pin`]. Pins are
//! session-scoped — they vanish when the app quits (no persistence in v1).

use std::collections::HashMap;
use std::sync::Mutex;

use pinshot_core::CapturedImage;
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

/// One pinned capture: the cropped pixels plus the scale of the display it came
/// from (so the pin renders at the correct logical size).
pub struct PinnedImage {
    pub image: CapturedImage,
    pub scale_factor: f64,
}

struct State {
    next_id: u32,
    pins: HashMap<u32, PinnedImage>,
}

/// App-wide registry of live pins, keyed by a monotonically increasing id that
/// also names the pin's window (`pin-<id>`).
pub struct PinRegistry {
    inner: Mutex<State>,
}

impl Default for PinRegistry {
    fn default() -> Self {
        Self {
            inner: Mutex::new(State {
                next_id: 1,
                pins: HashMap::new(),
            }),
        }
    }
}

impl PinRegistry {
    /// Stores a pinned image and returns its new id.
    pub fn register(&self, pin: PinnedImage) -> u32 {
        let mut state = self.inner.lock().expect("pin registry lock");
        let id = state.next_id;
        state.next_id += 1;
        state.pins.insert(id, pin);
        id
    }

    /// Returns a clone of the pin's image and scale, if it still exists.
    pub fn snapshot(&self, id: u32) -> Option<(CapturedImage, f64)> {
        let state = self.inner.lock().expect("pin registry lock");
        state
            .pins
            .get(&id)
            .map(|p| (p.image.clone(), p.scale_factor))
    }

    /// Drops a pin from the registry (idempotent).
    pub fn remove(&self, id: u32) {
        self.inner
            .lock()
            .expect("pin registry lock")
            .pins
            .remove(&id);
    }

    /// Returns all active pin ids.
    pub fn all_ids(&self) -> Vec<u32> {
        let state = self.inner.lock().expect("pin registry lock");
        state.pins.keys().copied().collect()
    }
}

fn label_for(pin_id: u32) -> String {
    format!("pin-{pin_id}")
}

/// Creates the floating window for a pin at the given logical rectangle.
pub fn open_window(
    app: &AppHandle,
    pin_id: u32,
    logical: (f64, f64, f64, f64),
) -> tauri::Result<()> {
    let (x, y, w, h) = logical;
    let label = label_for(pin_id);
    let url = WebviewUrl::App(format!("pin.html?id={pin_id}").into());
    let window = WebviewWindowBuilder::new(app, &label, url)
        .title("PinShot Pin")
        .position(x, y)
        .inner_size(w.max(1.0), h.max(1.0))
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        // Resizable so the webview can zoom the pin via `setSize` (Snipaste-style
        // scroll/pinch). No decorations means there are no user-draggable borders.
        .resizable(true)
        .shadow(true)
        .focused(true)
        .build()?;
    let _ = window.set_focus();
    Ok(())
}

/// Closes a pin's window if it is open.
pub fn close_window(app: &AppHandle, pin_id: u32) {
    if let Some(window) = app.get_webview_window(&label_for(pin_id)) {
        let _ = window.close();
    }
}

/// Brings a pin's window to the front (raise on interaction).
///
/// Before raising, checks if the pin is on a visible monitor. If the pin's
/// center falls outside all available monitors (e.g. because the display it
/// was on was unplugged), it is repositioned onto the primary monitor so the
/// user always has a grabbable area (FR edge case: display removed under a pin).
pub fn raise_window(app: &AppHandle, pin_id: u32) {
    if let Some(window) = app.get_webview_window(&label_for(pin_id)) {
        reposition_if_offscreen(&window);
        let _ = window.set_focus();
    }
}

/// Checks whether a pin window's center is inside any connected monitor.
/// If not, moves the pin to the primary monitor's work area so it remains
/// visible and grabbable.
fn reposition_if_offscreen(window: &tauri::WebviewWindow) {
    let pos = match window.outer_position() {
        Ok(p) => p,
        Err(_) => return,
    };
    let size = match window.outer_size() {
        Ok(s) => s,
        Err(_) => return,
    };

    // Center of the pin window in physical coordinates.
    let cx = pos.x + (size.width as i32 / 2);
    let cy = pos.y + (size.height as i32 / 2);

    let monitors = match window.available_monitors() {
        Ok(m) => m,
        Err(_) => return,
    };

    // Check if the center falls within any connected monitor.
    let on_screen = monitors.iter().any(|m| {
        let mp = m.position();
        let ms = m.size();
        cx >= mp.x && cx < mp.x + ms.width as i32 && cy >= mp.y && cy < mp.y + ms.height as i32
    });

    if on_screen {
        return;
    }

    // Pin is off-screen — move it to the primary monitor.
    let primary = match window.primary_monitor() {
        Ok(Some(m)) => m,
        _ => return,
    };
    let pp = primary.position();

    // Place at the top-left of the primary monitor with a small margin.
    let new_x = pp.x + 20;
    let new_y = pp.y + 20;
    let _ = window.set_position(PhysicalPosition::new(new_x, new_y));
}

/// Iterates all live pin windows and repositions any that are off-screen.
/// Call this when a monitor change is detected (e.g. display disconnected).
pub fn reposition_all_offscreen(app: &AppHandle, registry: &PinRegistry) {
    for pin_id in registry.all_ids() {
        let label = label_for(pin_id);
        if let Some(window) = app.get_webview_window(&label) {
            reposition_if_offscreen(&window);
        }
    }
}
