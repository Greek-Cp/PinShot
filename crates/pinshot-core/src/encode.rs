//! Encoding a captured image to PNG bytes.
//!
//! Pure and testable: takes RGBA pixels, returns PNG file bytes. The shell
//! writes those bytes to a file (US2). Uses the `image` crate's PNG encoder.

use crate::capture::CapturedImage;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};

/// Error returned when a [`CapturedImage`] cannot be encoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// The pixel buffer length did not match `width * height * 4`.
    InvalidBuffer,
    /// The PNG encoder failed.
    Encode(String),
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::InvalidBuffer => write!(f, "pixel buffer size does not match dimensions"),
            EncodeError::Encode(reason) => write!(f, "PNG encode failed: {reason}"),
        }
    }
}

impl std::error::Error for EncodeError {}

/// Encodes a captured image as PNG file bytes.
pub fn to_png(image: &CapturedImage) -> Result<Vec<u8>, EncodeError> {
    let expected = image.width as usize * image.height as usize * 4;
    if image.rgba.len() != expected {
        return Err(EncodeError::InvalidBuffer);
    }
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(
            &image.rgba,
            image.width,
            image.height,
            ExtendedColorType::Rgba8,
        )
        .map_err(|e| EncodeError::Encode(e.to_string()))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_mismatched_buffer() {
        let img = CapturedImage {
            width: 2,
            height: 2,
            rgba: vec![0; 3], // too short
        };
        assert_eq!(to_png(&img).unwrap_err(), EncodeError::InvalidBuffer);
    }

    #[test]
    fn encodes_valid_png_that_decodes_back() {
        let img = CapturedImage {
            width: 2,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 255, 0, 255],
        };
        let bytes = to_png(&img).expect("encodes");
        // PNG magic number.
        assert_eq!(&bytes[0..4], &[0x89, b'P', b'N', b'G']);
        let decoded = image::load_from_memory(&bytes).expect("decodes");
        assert_eq!(decoded.width(), 2);
        assert_eq!(decoded.height(), 1);
        let rgba = decoded.to_rgba8();
        assert_eq!(rgba.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(rgba.get_pixel(1, 0).0, [0, 255, 0, 255]);
    }
}
