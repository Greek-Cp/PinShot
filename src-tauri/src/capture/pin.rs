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
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

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
        .resizable(false)
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
pub fn raise_window(app: &AppHandle, pin_id: u32) {
    if let Some(window) = app.get_webview_window(&label_for(pin_id)) {
        let _ = window.set_focus();
    }
}
