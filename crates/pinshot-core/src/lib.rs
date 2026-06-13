//! PinShot domain core.
//!
//! This crate holds platform-independent domain logic (capture geometry, and
//! later image operations and OCR). Per the project constitution it has **no
//! GUI dependency and makes no network requests**, and it is testable headless
//! (`cargo test -p pinshot-core`). OS-specific behaviour lives behind the
//! traits in [`capture`], implemented by adapters in the Tauri shell.

pub mod capture;
pub mod geometry;

pub use capture::ScreenCapturer;
pub use geometry::Rect;
