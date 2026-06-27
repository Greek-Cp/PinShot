//! Output adapters: write a captured image to the clipboard or a PNG file.
//!
//! Side-effecting glue around the pure `pinshot_core` encode/naming logic.

use std::borrow::Cow;
use std::path::PathBuf;

use chrono::{Datelike, Local, Timelike};
use pinshot_core::{output_filename, to_png, CapturedImage};

/// Failure while delivering a capture to an output target.
#[derive(Debug)]
pub enum OutputError {
    ClipboardUnavailable(String),
    NoPicturesDir,
    WriteFailed(String),
    Encode(String),
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputError::ClipboardUnavailable(e) => write!(f, "clipboard unavailable: {e}"),
            OutputError::NoPicturesDir => write!(f, "could not locate the Pictures folder"),
            OutputError::WriteFailed(e) => write!(f, "could not write file: {e}"),
            OutputError::Encode(e) => write!(f, "could not encode PNG: {e}"),
        }
    }
}

/// Copies the image to the system clipboard as raster image data.
pub fn copy_image(image: &CapturedImage) -> Result<(), OutputError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| OutputError::ClipboardUnavailable(e.to_string()))?;
    clipboard
        .set_image(arboard::ImageData {
            width: image.width as usize,
            height: image.height as usize,
            bytes: Cow::Borrowed(&image.rgba),
        })
        .map_err(|e| OutputError::ClipboardUnavailable(e.to_string()))
}

/// Copies text (used for the color readout) to the clipboard.
pub fn copy_text(text: &str) -> Result<(), OutputError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| OutputError::ClipboardUnavailable(e.to_string()))?;
    clipboard
        .set_text(text.to_owned())
        .map_err(|e| OutputError::ClipboardUnavailable(e.to_string()))
}

/// Saves the image as a timestamped, collision-free PNG inside `dir` (created if
/// needed). Returns the path written.
fn save_png_in(image: &CapturedImage, dir: PathBuf) -> Result<PathBuf, OutputError> {
    std::fs::create_dir_all(&dir).map_err(|e| OutputError::WriteFailed(e.to_string()))?;

    let existing: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| OutputError::WriteFailed(e.to_string()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    let now = Local::now();
    let name = output_filename(
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        &existing,
    );

    let bytes = to_png(image).map_err(|e| OutputError::Encode(e.to_string()))?;
    let path = dir.join(name);
    std::fs::write(&path, bytes).map_err(|e| OutputError::WriteFailed(e.to_string()))?;
    Ok(path)
}

/// Saves the image as a PNG in `Pictures/PinShot/` (the capture "Save" target).
pub fn save_png(image: &CapturedImage) -> Result<PathBuf, OutputError> {
    let dir = dirs::picture_dir()
        .ok_or(OutputError::NoPicturesDir)?
        .join("PinShot");
    save_png_in(image, dir)
}

/// Saves the image as a PNG in `Documents/PinShots/` (the pin "Save" target).
pub fn save_to_documents(image: &CapturedImage) -> Result<PathBuf, OutputError> {
    let dir = dirs::document_dir()
        .ok_or(OutputError::NoPicturesDir)?
        .join("PinShots");
    save_png_in(image, dir)
}
