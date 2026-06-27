//! Delivering a flattened edit to Copy / Save / Pin (FR-023/024/025).
//!
//! Reuses 002's clipboard/file output and 003's pin window. The flattened pixels
//! come from `pinshot_core::flatten`; this module only performs the side effect.

use pinshot_core::CapturedImage;
use serde::Serialize;
use tauri::AppHandle;

use crate::capture::output;
use crate::capture::pin::{open_window, PinRegistry, PinnedImage};

/// Result of an export action.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResponse {
    target: String,
    path: Option<String>,
    pin_id: Option<u32>,
}

/// Sends `image` to the requested target. `format` is reserved for the Export
/// settings (PNG/JPG/WebP); file save is PNG for now (US3 wires the rest).
pub fn deliver(
    app: &AppHandle,
    pins: &PinRegistry,
    image: &CapturedImage,
    scale: f64,
    target: &str,
    _format: Option<&str>,
) -> Result<ExportResponse, String> {
    match target {
        "clipboard" => {
            output::copy_image(image).map_err(|e| e.to_string())?;
            Ok(ExportResponse {
                target: "clipboard".to_string(),
                path: None,
                pin_id: None,
            })
        }
        "file" => {
            let path = output::save_png(image).map_err(|e| e.to_string())?;
            Ok(ExportResponse {
                target: "file".to_string(),
                path: Some(path.to_string_lossy().into_owned()),
                pin_id: None,
            })
        }
        "pin" => {
            let scale = scale.max(0.1);
            let logical_w = (image.width as f64 / scale).max(1.0);
            let logical_h = (image.height as f64 / scale).max(1.0);
            let pin_id = pins.register(PinnedImage {
                image: image.clone(),
                scale_factor: scale,
            });
            // Open near the top-left; the user drags it where they want.
            open_window(app, pin_id, (80.0, 80.0, logical_w, logical_h))
                .map_err(|e| e.to_string())?;
            Ok(ExportResponse {
                target: "pin".to_string(),
                path: None,
                pin_id: Some(pin_id),
            })
        }
        other => Err(format!("unknown export target: {other}")),
    }
}
