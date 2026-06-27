//! Offline "smart" tools over a captured image: QR detection and the color
//! sample produced by the colour picker. Pure and on-device (FR-046); the shell
//! exposes these over IPC and performs any explicit OS-browser hand-off.
//!
//! **OCR is intentionally not here** — text extraction and Search/Translate are
//! deferred to the roadmap (Clarification Q2). This module ships QR + colour.

pub mod qr;

pub use crate::color::ColorSample;
pub use qr::detect;

use crate::geometry::Rect;

/// One decoded code from [`qr::detect`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrCode {
    pub value: String,
    /// Bounding rectangle of the code in image pixels.
    pub rect: Rect,
    /// Whether `value` is an `http(s)` URL (drives the **Open URL** affordance).
    pub is_url: bool,
}

/// The result of QR/barcode detection over a capture.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QrResult {
    pub codes: Vec<QrCode>,
}
