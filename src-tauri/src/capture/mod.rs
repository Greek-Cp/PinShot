//! Capture flow orchestration and the Tauri commands the overlay calls.
//!
//! Trigger (hotkey/tray) → freeze all displays → show per-display overlays.
//! The overlay pulls its frame via `get_overlay_frame`, then confirms with
//! `commit_selection` (clipboard or PNG file) or `cancel_capture`. All
//! geometry/crop/encode is delegated to `pinshot_core`; this module only wires
//! platform side effects (capture, windows, clipboard, files).

mod output;
mod overlay;
mod pin;
// Pixel capture is platform-split: macOS uses the system `screencapture` tool
// (xcap's CoreGraphics path is broken on macOS 15+), everything else uses xcap.
#[cfg(target_os = "macos")]
mod macos_capturer;
#[cfg(not(target_os = "macos"))]
mod xcap_capturer;

use std::sync::Mutex;

use base64::Engine;
use pinshot_core::{
    crop_region, pin_placement, to_physical, to_png, CaptureError, CapturedImage, Display,
    FrozenFrame, Rect, ScreenCapturer,
};

use pin::{PinRegistry, PinnedImage};
use serde::{Deserialize, Serialize};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg(target_os = "macos")]
use macos_capturer::MacScreencaptureCapturer as PlatformCapturer;
#[cfg(not(target_os = "macos"))]
use xcap_capturer::XcapCapturer as PlatformCapturer;

/// Default capture hotkey (area select). Remapping is a later feature.
const CAPTURE_HOTKEY: &str = "CmdOrCtrl+Shift+A";

/// The frozen state of one in-flight capture.
struct CaptureSession {
    displays: Vec<Display>,
    frames: Vec<FrozenFrame>,
}

/// App-wide capture state: the platform capturer and the current session.
pub struct CaptureState {
    capturer: PlatformCapturer,
    session: Mutex<Option<CaptureSession>>,
}

impl CaptureState {
    pub fn new() -> Self {
        Self {
            capturer: PlatformCapturer,
            session: Mutex::new(None),
        }
    }
}

impl Default for CaptureState {
    fn default() -> Self {
        Self::new()
    }
}

/// Registers tray, global hotkey, and shared state. Call from the Tauri setup.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    app.manage(CaptureState::new());
    app.manage(PinRegistry::default());

    let capture_item = MenuItemBuilder::with_id("capture", "Capture").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&capture_item, &quit_item])
        .build()?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "capture" => start_capture(app),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    if let Err(e) = app
        .global_shortcut()
        .on_shortcut(CAPTURE_HOTKEY, |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                start_capture(app);
            }
        })
    {
        eprintln!("PinShot: failed to register capture hotkey: {e}");
    }

    Ok(())
}

/// Begins a capture: freeze all displays and show the overlays. Ignored if a
/// capture is already in progress (FR-018).
pub fn start_capture(app: &AppHandle) {
    let state = app.state::<CaptureState>();
    if state.session.lock().expect("session lock").is_some() {
        return; // a capture is already active
    }

    match state.capturer.capture_all() {
        Ok((displays, frames)) => {
            // macOS returns an (essentially) black frame, not an error, until
            // Screen Recording is authorized AND the app has been relaunched.
            // A very common false-positive: an ad-hoc-signed build whose grant
            // went stale after a rebuild — the toggle still shows enabled but
            // the capture is black. Showing the always-on-top overlay then
            // would just cover the System Settings pane the user needs. Guide
            // them instead (FR-016).
            if frames.iter().all(|f| f.is_blank()) {
                notify_error(
                    app,
                    "PinShot can't see your screen. Open System Settings → Privacy & Security → Screen Recording. If PinShot already appears there, remove it with the “−” button, then quit and reopen PinShot and allow it again — after a rebuild the existing permission goes stale even though it still looks enabled. macOS only applies the change after a relaunch.",
                );
                return;
            }
            *state.session.lock().expect("session lock") = Some(CaptureSession {
                displays: displays.clone(),
                frames,
            });
            if let Err(e) = overlay::show(app, &displays) {
                *state.session.lock().expect("session lock") = None;
                notify_error(app, &format!("Could not open the capture overlay: {e}"));
            }
        }
        Err(CaptureError::PermissionDenied) => notify_error(
            app,
            "PinShot needs Screen Recording permission. Enable it in System Settings → Privacy & Security → Screen Recording, then try again.",
        ),
        Err(e) => notify_error(app, &format!("Capture failed: {e}")),
    }
}

/// Surfaces an error to the user. For now logs and emits an event the main
/// window can show; a richer dialog is a later refinement.
fn notify_error(app: &AppHandle, message: &str) {
    eprintln!("PinShot: {message}");
    let _ = app.emit("capture://error", message.to_string());
}

/// One display's frozen frame, delivered to its overlay window.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FramePayload {
    width: u32,
    height: u32,
    scale_factor: f64,
    origin_x: i32,
    origin_y: i32,
    /// PNG of the frozen frame as a `data:` URL for the overlay backdrop.
    data_url: String,
}

/// A logical selection rectangle reported by the overlay.
#[derive(Deserialize)]
pub struct RectArg {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

/// Result of a committed selection.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitResponse {
    output: String,
    path: Option<String>,
}

/// Returns the frozen frame (as a PNG data URL) and metadata for one display's
/// overlay window.
#[tauri::command]
pub fn get_overlay_frame(
    state: State<'_, CaptureState>,
    display_id: u32,
) -> Result<FramePayload, String> {
    let guard = state.session.lock().expect("session lock");
    let session = guard.as_ref().ok_or("no active capture")?;
    let display = session
        .displays
        .iter()
        .find(|d| d.id == display_id)
        .ok_or("unknown display")?;
    let frame = session
        .frames
        .iter()
        .find(|f| f.display_id == display_id)
        .ok_or("no frame for display")?;

    // Reuse the backend's own PNG (macOS captures straight to PNG) so we never
    // re-encode the full-screen frame just to draw the backdrop — that encode,
    // plus cloning the multi-megabyte RGBA buffer, was the overlay's main lag.
    // Backends without a source PNG (xcap) encode once here as a fallback.
    let png = match &frame.source_png {
        Some(bytes) => bytes.clone(),
        None => {
            let image = CapturedImage {
                width: frame.width,
                height: frame.height,
                rgba: frame.rgba.clone(),
            };
            to_png(&image).map_err(|e| e.to_string())?
        }
    };
    let data_url = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png)
    );

    Ok(FramePayload {
        width: frame.width,
        height: frame.height,
        scale_factor: display.scale_factor,
        origin_x: display.origin.0,
        origin_y: display.origin.1,
        data_url,
    })
}

/// Confirms a non-empty selection and writes it to the chosen output.
#[tauri::command]
pub fn commit_selection(
    app: AppHandle,
    state: State<'_, CaptureState>,
    display_id: u32,
    rect: RectArg,
    output: String,
) -> Result<CommitResponse, String> {
    let image = {
        let guard = state.session.lock().expect("session lock");
        let session = guard.as_ref().ok_or("no active capture")?;
        let display = session
            .displays
            .iter()
            .find(|d| d.id == display_id)
            .ok_or("unknown display")?;

        let logical = Rect::new(rect.x, rect.y, rect.width, rect.height);
        if logical.is_empty() {
            return Err("empty_selection".to_string());
        }
        let region = to_physical(logical, display);
        crop_region(&session.frames, &session.displays, region).map_err(|e| e.to_string())?
    };

    let response = match output.as_str() {
        "clipboard" => {
            output::copy_image(&image).map_err(|e| e.to_string())?;
            CommitResponse {
                output: "clipboard".to_string(),
                path: None,
            }
        }
        "file" => {
            let path = output::save_png(&image).map_err(|e| e.to_string())?;
            CommitResponse {
                output: "file".to_string(),
                path: Some(path.to_string_lossy().into_owned()),
            }
        }
        other => return Err(format!("unknown output target: {other}")),
    };

    overlay::close(&app);
    *state.session.lock().expect("session lock") = None;
    Ok(response)
}

/// Copies a color string (HEX) to the clipboard (US3).
#[tauri::command]
pub fn copy_color(hex: String) -> Result<(), String> {
    output::copy_text(&hex).map_err(|e| e.to_string())
}

/// Dismisses the overlay with no side effects (FR-004/SC-006).
#[tauri::command]
pub fn cancel_capture(app: AppHandle, state: State<'_, CaptureState>) {
    overlay::close(&app);
    *state.session.lock().expect("session lock") = None;
}

/// One pinned image, delivered to its pin window.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PinImagePayload {
    width: u32,
    height: u32,
    scale_factor: f64,
    data_url: String,
}

/// Result of pinning a selection.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePinResponse {
    pin_id: u32,
}

/// Pins the (adjusted) selection as a floating window (US1). Crops the frozen
/// region, stores it, opens a borderless always-on-top window at the capture's
/// on-screen location, then closes the overlay — mirroring `commit_selection`.
#[tauri::command]
pub fn create_pin(
    app: AppHandle,
    state: State<'_, CaptureState>,
    pins: State<'_, PinRegistry>,
    display_id: u32,
    rect: RectArg,
) -> Result<CreatePinResponse, String> {
    let (image, region, origin, size, scale) = {
        let guard = state.session.lock().expect("session lock");
        let session = guard.as_ref().ok_or("no active capture")?;
        let display = session
            .displays
            .iter()
            .find(|d| d.id == display_id)
            .ok_or("unknown display")?;

        let logical = Rect::new(rect.x, rect.y, rect.width, rect.height);
        if logical.is_empty() {
            return Err("empty_selection".to_string());
        }
        let region = to_physical(logical, display);
        let image =
            crop_region(&session.frames, &session.displays, region).map_err(|e| e.to_string())?;
        (
            image,
            region,
            display.origin,
            display.size,
            display.scale_factor,
        )
    };

    // Full physical size at the display's logical scale; nudged on-screen using
    // the display's logical bounds as the work area (good enough for v1).
    let work = Rect::new(
        (origin.0 as f64 / scale).round() as i32,
        (origin.1 as f64 / scale).round() as i32,
        (size.0 as f64 / scale).round() as u32,
        (size.1 as f64 / scale).round() as u32,
    );
    let placement = pin_placement(region, origin, scale, work);

    let pin_id = pins.register(PinnedImage {
        image,
        scale_factor: scale,
    });
    pin::open_window(&app, pin_id, placement).map_err(|e| e.to_string())?;

    overlay::close(&app);
    *state.session.lock().expect("session lock") = None;
    Ok(CreatePinResponse { pin_id })
}

/// Returns a pin's image (as a PNG data URL) and size for its window.
#[tauri::command]
pub fn get_pin_image(pins: State<'_, PinRegistry>, pin_id: u32) -> Result<PinImagePayload, String> {
    let (image, scale_factor) = pins.snapshot(pin_id).ok_or("unknown pin")?;
    let png = to_png(&image).map_err(|e| e.to_string())?;
    let data_url = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png)
    );
    Ok(PinImagePayload {
        width: image.width,
        height: image.height,
        scale_factor,
        data_url,
    })
}

/// Closes a single pin (its window and registry entry). Idempotent.
#[tauri::command]
pub fn close_pin(app: AppHandle, pins: State<'_, PinRegistry>, pin_id: u32) {
    pins.remove(pin_id);
    pin::close_window(&app, pin_id);
}

/// Brings a pin to the front when the user interacts with it.
#[tauri::command]
pub fn raise_pin(app: AppHandle, pin_id: u32) {
    pin::raise_window(&app, pin_id);
}

/// Copies a pin's image back to the clipboard (FR-007).
#[tauri::command]
pub fn copy_pin(pins: State<'_, PinRegistry>, pin_id: u32) -> Result<(), String> {
    let (image, _scale) = pins.snapshot(pin_id).ok_or("unknown pin")?;
    output::copy_image(&image).map_err(|e| e.to_string())
}
