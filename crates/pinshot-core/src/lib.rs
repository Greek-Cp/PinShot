//! PinShot domain core.
//!
//! This crate holds platform-independent domain logic (capture geometry,
//! selection→pixel mapping, cropping, PNG encoding, and output naming). Per the
//! project constitution it has **no GUI dependency and makes no network
//! requests**, and it is testable headless (`cargo test -p pinshot-core`).
//! OS-specific behaviour lives behind the traits in [`capture`], implemented by
//! adapters in the Tauri shell.

pub mod capture;
pub mod color;
pub mod crop;
pub mod encode;
pub mod geometry;
pub mod naming;
pub mod pin;
pub mod selection;

pub use capture::{CaptureError, CapturedImage, Display, FrozenFrame, ScreenCapturer};
pub use color::{pixel_hex, pixel_rgb};
pub use crop::{crop_region, CropError};
pub use encode::{to_png, EncodeError};
pub use geometry::Rect;
pub use naming::output_filename;
pub use pin::{pin_logical_size, pin_placement};
pub use selection::{displays_for, to_physical};
