//! Encoding a captured image to PNG bytes.
//!
//! Pure and testable: takes RGBA pixels, returns PNG file bytes. The shell
//! writes those bytes to a file (US2). Uses the `image` crate's PNG encoder.

use crate::capture::CapturedImage;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};

/// Error returned when a [`CapturedImage`] cannot be encoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// The pixel buffer length did not match `width * height * 4`.
    InvalidBuffer,
    /// The encoder failed.
    Encode(String),
    /// The requested format is not supported in this build (e.g. WebP, whose
    /// encoder is a deferred dependency decision — see `to_webp`).
    Unsupported(&'static str),
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::InvalidBuffer => write!(f, "pixel buffer size does not match dimensions"),
            EncodeError::Encode(reason) => write!(f, "encode failed: {reason}"),
            EncodeError::Unsupported(fmt) => write!(f, "format not supported in this build: {fmt}"),
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

/// Encodes a captured image as JPEG bytes at `quality` (1–100). JPEG has no
/// alpha channel, so the alpha is dropped (captures are opaque).
pub fn to_jpg(image: &CapturedImage, quality: u8) -> Result<Vec<u8>, EncodeError> {
    let expected = image.width as usize * image.height as usize * 4;
    if image.rgba.len() != expected {
        return Err(EncodeError::InvalidBuffer);
    }
    // RGBA → RGB.
    let mut rgb = Vec::with_capacity(image.width as usize * image.height as usize * 3);
    for px in image.rgba.chunks_exact(4) {
        rgb.extend_from_slice(&px[0..3]);
    }
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, quality.clamp(1, 100))
        .encode(&rgb, image.width, image.height, ExtendedColorType::Rgb8)
        .map_err(|e| EncodeError::Encode(e.to_string()))?;
    Ok(out)
}

/// Encodes a captured image as WebP bytes.
///
/// **Deferred**: WebP encoding requires a dedicated encoder dependency (the
/// `image` crate ships only a WebP *decoder*; a lossless encoder means adding
/// `image-webp`/`webp`). To avoid destabilising the cross-platform build that
/// decision is deferred (plan D6), and this returns [`EncodeError::Unsupported`]
/// so callers fall back to PNG/JPG until the dependency lands.
pub fn to_webp(image: &CapturedImage, _quality: u8) -> Result<Vec<u8>, EncodeError> {
    let expected = image.width as usize * image.height as usize * 4;
    if image.rgba.len() != expected {
        return Err(EncodeError::InvalidBuffer);
    }
    Err(EncodeError::Unsupported("webp"))
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

    #[test]
    fn encodes_jpeg_that_decodes_to_same_dimensions() {
        let img = CapturedImage {
            width: 8,
            height: 4,
            rgba: vec![120; 8 * 4 * 4],
        };
        let bytes = to_jpg(&img, 90).expect("encodes jpeg");
        // JPEG SOI marker.
        assert_eq!(&bytes[0..2], &[0xFF, 0xD8]);
        let decoded = image::load_from_memory(&bytes).expect("decodes");
        assert_eq!((decoded.width(), decoded.height()), (8, 4));
    }

    #[test]
    fn jpeg_rejects_bad_buffer() {
        let img = CapturedImage {
            width: 2,
            height: 2,
            rgba: vec![0; 3],
        };
        assert_eq!(to_jpg(&img, 80).unwrap_err(), EncodeError::InvalidBuffer);
    }

    #[test]
    fn webp_is_deferred_unsupported() {
        let img = CapturedImage {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
        };
        assert_eq!(
            to_webp(&img, 80).unwrap_err(),
            EncodeError::Unsupported("webp")
        );
    }
}
