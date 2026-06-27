//! Offline QR / barcode detection over a captured image (FR-027/FR-028/FR-029).
//!
//! Uses the pure-Rust `rqrr` decoder fed a greyscale view of the RGBA buffer —
//! no platform engine, **no network**. `Open URL` (handing a decoded URL to the
//! OS browser) is the shell's job; this layer only decodes on-device.

use super::{QrCode, QrResult};
use crate::capture::CapturedImage;
use crate::geometry::Rect;
use rqrr::PreparedImage;

#[inline]
fn luma(r: u8, g: u8, b: u8) -> u8 {
    // Rec. 601 luma; integer math keeps it deterministic.
    ((r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000) as u8
}

/// Decodes every QR code found in `image`, returning their values, screen
/// rectangles, and whether each looks like a URL. Empty when none are found.
pub fn detect(image: &CapturedImage) -> QrResult {
    let w = image.width as usize;
    let h = image.height as usize;
    if w == 0 || h == 0 || image.rgba.len() != w * h * 4 {
        return QrResult::default();
    }

    let rgba = &image.rgba;
    let mut prepared = PreparedImage::prepare_from_greyscale(w, h, |x, y| {
        let i = (y * w + x) * 4;
        luma(rgba[i], rgba[i + 1], rgba[i + 2])
    });

    let mut codes = Vec::new();
    for grid in prepared.detect_grids() {
        if let Ok((_meta, content)) = grid.decode() {
            let rect = bounds_of(&grid.bounds);
            let is_url = looks_like_url(&content);
            codes.push(QrCode {
                value: content,
                rect,
                is_url,
            });
        }
    }
    QrResult { codes }
}

fn bounds_of(corners: &[rqrr::Point; 4]) -> Rect {
    let xs = corners.iter().map(|p| p.x);
    let ys = corners.iter().map(|p| p.y);
    let min_x = xs.clone().min().unwrap_or(0);
    let max_x = xs.max().unwrap_or(0);
    let min_y = ys.clone().min().unwrap_or(0);
    let max_y = ys.max().unwrap_or(0);
    Rect::new(
        min_x,
        min_y,
        (max_x - min_x).max(0) as u32,
        (max_y - min_y).max(0) as u32,
    )
}

fn looks_like_url(value: &str) -> bool {
    let v = value.trim().to_ascii_lowercase();
    v.starts_with("http://") || v.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrcode::{Color, QrCode as Encoder};

    /// Renders `data` to a black-on-white RGBA image with a quiet zone, each QR
    /// module scaled to `scale` pixels — enough for `rqrr` to decode.
    fn render_qr(data: &str, scale: u32) -> CapturedImage {
        let code = Encoder::new(data.as_bytes()).expect("encode qr");
        let modules = code.to_colors();
        let n = (modules.len() as f64).sqrt() as u32; // square grid
        let quiet = 4u32;
        let dim = (n + quiet * 2) * scale;
        let mut rgba = vec![255u8; (dim * dim * 4) as usize];
        for my in 0..n {
            for mx in 0..n {
                if modules[(my * n + mx) as usize] != Color::Dark {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = (quiet + mx) * scale + dx;
                        let py = (quiet + my) * scale + dy;
                        let i = ((py * dim + px) * 4) as usize;
                        rgba[i] = 0;
                        rgba[i + 1] = 0;
                        rgba[i + 2] = 0;
                        rgba[i + 3] = 255;
                    }
                }
            }
        }
        CapturedImage {
            width: dim,
            height: dim,
            rgba,
        }
    }

    #[test]
    fn decodes_a_url_qr_offline() {
        let img = render_qr("https://pinshot.app", 6);
        let result = detect(&img);
        assert_eq!(result.codes.len(), 1);
        assert_eq!(result.codes[0].value, "https://pinshot.app");
        assert!(result.codes[0].is_url);
        assert!(result.codes[0].rect.width > 0);
    }

    #[test]
    fn decodes_non_url_text() {
        let img = render_qr("PINSHOT-OFFLINE", 6);
        let result = detect(&img);
        assert_eq!(result.codes.len(), 1);
        assert_eq!(result.codes[0].value, "PINSHOT-OFFLINE");
        assert!(!result.codes[0].is_url);
    }

    #[test]
    fn empty_image_finds_nothing() {
        let img = CapturedImage {
            width: 16,
            height: 16,
            rgba: vec![255u8; 16 * 16 * 4],
        };
        assert!(detect(&img).codes.is_empty());
    }
}
