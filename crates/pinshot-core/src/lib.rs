//! PinShot domain core.
//!
//! This crate holds platform-independent domain logic (capture geometry,
//! selection→pixel mapping, cropping, PNG encoding, and output naming). Per the
//! project constitution it has **no GUI dependency and makes no network
//! requests**, and it is testable headless (`cargo test -p pinshot-core`).
//! OS-specific behaviour lives behind the traits in [`capture`], implemented by
//! adapters in the Tauri shell.

pub mod annotation;
pub mod capture;
pub mod color;
pub mod crop;
pub mod encode;
pub mod geometry;
pub mod history;
pub mod naming;
pub mod pin;
pub mod selection;
pub mod settings;
pub mod smart;

pub use annotation::render::{flatten, RenderError};
pub use annotation::{
    Annotation, AnnotationDoc, AnnotationId, AnnotationKind, Geometry, Point, Rgba, Style,
};
pub use capture::{CaptureError, CapturedImage, Display, FrozenFrame, ScreenCapturer};
pub use color::{hsl_to_rgb, pixel_hex, pixel_rgb, rgb_to_hsl, ColorSample};
pub use crop::{crop_region, CropError};
pub use encode::{to_jpg, to_png, to_webp, EncodeError};
pub use geometry::Rect;
pub use history::{Command, HistoryStack};
pub use naming::output_filename;
pub use pin::{pin_logical_size, pin_placement};
pub use selection::{displays_for, to_physical};
pub use settings::Settings;
pub use smart::{detect as detect_qr, QrCode, QrResult};
