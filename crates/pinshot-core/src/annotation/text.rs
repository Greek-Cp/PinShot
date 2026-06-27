//! Text rasterisation using a small embedded 5×7 bitmap font.
//!
//! Rendering text into the flattened image in pure Rust — with no font file or
//! native text API — keeps the output **identical on macOS and Windows**
//! (Constitution III) and **unit-testable headless** (Constitution IV). The
//! font covers digits, A–Z (lowercase is mapped to uppercase), space, and the
//! punctuation common in callouts; unknown glyphs render as blank. This is
//! deliberately minimal (plan D5 deferred a richer font); it is enough for
//! Text annotations and Step-Number markers.

// Pixel-plotting helpers take (buf, width, height, x, y, …) by nature.
#![allow(clippy::too_many_arguments)]

use super::render::blend_at;
use super::{Point, Rgba, TextStyle};

/// Columns per glyph cell (plus one spacing column when advancing).
const COLS: u32 = 5;
/// Rows per glyph cell.
const ROWS: u32 = 7;

/// Returns the 5-column bitmap for `c` (each byte's bit *r* = row *r*, top→
/// bottom). Lowercase maps to uppercase; unknown chars are blank.
fn glyph(c: char) -> [u8; 5] {
    let c = c.to_ascii_uppercase();
    match c {
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00],
        '!' => [0x00, 0x00, 0x5F, 0x00, 0x00],
        '#' => [0x14, 0x7F, 0x14, 0x7F, 0x14],
        '%' => [0x23, 0x13, 0x08, 0x64, 0x62],
        '(' => [0x00, 0x1C, 0x22, 0x41, 0x00],
        ')' => [0x00, 0x41, 0x22, 0x1C, 0x00],
        '+' => [0x08, 0x08, 0x3E, 0x08, 0x08],
        ',' => [0x00, 0x50, 0x30, 0x00, 0x00],
        '-' => [0x08, 0x08, 0x08, 0x08, 0x08],
        '.' => [0x00, 0x00, 0x40, 0x40, 0x00],
        '/' => [0x20, 0x10, 0x08, 0x04, 0x02],
        '0' => [0x3E, 0x51, 0x49, 0x45, 0x3E],
        '1' => [0x00, 0x42, 0x7F, 0x40, 0x00],
        '2' => [0x42, 0x61, 0x51, 0x49, 0x46],
        '3' => [0x21, 0x41, 0x45, 0x4B, 0x31],
        '4' => [0x18, 0x14, 0x12, 0x7F, 0x10],
        '5' => [0x27, 0x45, 0x45, 0x45, 0x39],
        '6' => [0x3C, 0x4A, 0x49, 0x49, 0x30],
        '7' => [0x01, 0x71, 0x09, 0x05, 0x03],
        '8' => [0x36, 0x49, 0x49, 0x49, 0x36],
        '9' => [0x06, 0x49, 0x49, 0x29, 0x1E],
        ':' => [0x00, 0x36, 0x36, 0x00, 0x00],
        '?' => [0x02, 0x01, 0x51, 0x09, 0x06],
        'A' => [0x7E, 0x11, 0x11, 0x11, 0x7E],
        'B' => [0x7F, 0x49, 0x49, 0x49, 0x36],
        'C' => [0x3E, 0x41, 0x41, 0x41, 0x22],
        'D' => [0x7F, 0x41, 0x41, 0x22, 0x1C],
        'E' => [0x7F, 0x49, 0x49, 0x49, 0x41],
        'F' => [0x7F, 0x09, 0x09, 0x09, 0x01],
        'G' => [0x3E, 0x41, 0x49, 0x49, 0x7A],
        'H' => [0x7F, 0x08, 0x08, 0x08, 0x7F],
        'I' => [0x00, 0x41, 0x7F, 0x41, 0x00],
        'J' => [0x20, 0x40, 0x41, 0x3F, 0x01],
        'K' => [0x7F, 0x08, 0x14, 0x22, 0x41],
        'L' => [0x7F, 0x40, 0x40, 0x40, 0x40],
        'M' => [0x7F, 0x02, 0x0C, 0x02, 0x7F],
        'N' => [0x7F, 0x04, 0x08, 0x10, 0x7F],
        'O' => [0x3E, 0x41, 0x41, 0x41, 0x3E],
        'P' => [0x7F, 0x09, 0x09, 0x09, 0x06],
        'Q' => [0x3E, 0x41, 0x51, 0x21, 0x5E],
        'R' => [0x7F, 0x09, 0x19, 0x29, 0x46],
        'S' => [0x46, 0x49, 0x49, 0x49, 0x31],
        'T' => [0x01, 0x01, 0x7F, 0x01, 0x01],
        'U' => [0x3F, 0x40, 0x40, 0x40, 0x3F],
        'V' => [0x1F, 0x20, 0x40, 0x20, 0x1F],
        'W' => [0x3F, 0x40, 0x38, 0x40, 0x3F],
        'X' => [0x63, 0x14, 0x08, 0x14, 0x63],
        'Y' => [0x07, 0x08, 0x70, 0x08, 0x07],
        'Z' => [0x61, 0x51, 0x49, 0x45, 0x43],
        _ => [0x00, 0x00, 0x00, 0x00, 0x00],
    }
}

/// Integer pixels-per-font-cell-row for a requested cap height `size`.
fn scale_for(size: u32) -> u32 {
    (size as f32 / ROWS as f32).round().max(1.0) as u32
}

/// Returns the rendered `(width, height)` in pixels for `text` at `size`.
pub fn measure(text: &str, size: u32) -> (u32, u32) {
    let s = scale_for(size);
    let advance = (COLS + 1) * s;
    let w = text.chars().count() as u32 * advance;
    (w, ROWS * s)
}

/// Draws `style.content` with its top-left at `at` into the RGBA `buf`,
/// honouring optional background, drop shadow, and `opacity`.
pub fn draw_text(
    buf: &mut [u8],
    width: u32,
    height: u32,
    at: Point,
    style: &TextStyle,
    opacity: f32,
) {
    let s = scale_for(style.size);
    let (tw, th) = measure(&style.content, style.size);
    let ox = at.x.round() as i32;
    let oy = at.y.round() as i32;

    if let Some(bg) = style.background {
        let pad = s as i32 * 2;
        fill_rect(
            buf,
            width,
            height,
            ox - pad,
            oy - pad,
            tw as i32 + pad * 2,
            th as i32 + pad * 2,
            bg,
            opacity,
        );
    }

    let advance = ((COLS + 1) * s) as i32;
    for (ci, ch) in style.content.chars().enumerate() {
        let gx = ox + ci as i32 * advance;
        let cols = glyph(ch);
        for (col, bits) in cols.iter().enumerate() {
            for row in 0..ROWS {
                if bits & (1 << row) == 0 {
                    continue;
                }
                let px = gx + col as i32 * s as i32;
                let py = oy + row as i32 * s as i32;
                if style.shadow {
                    stamp(
                        buf,
                        width,
                        height,
                        px + 1,
                        py + 1,
                        s,
                        Rgba(0, 0, 0, 160),
                        opacity,
                    );
                }
                stamp(buf, width, height, px, py, s, style.color, opacity);
            }
        }
    }
}

/// Fills an `s`×`s` block of `color` at `(x, y)`.
fn stamp(
    buf: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    s: u32,
    color: Rgba,
    opacity: f32,
) {
    fill_rect(buf, width, height, x, y, s as i32, s as i32, color, opacity);
}

fn fill_rect(
    buf: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Rgba,
    opacity: f32,
) {
    for yy in y..y + h {
        for xx in x..x + w {
            if xx < 0 || yy < 0 || xx >= width as i32 || yy >= height as i32 {
                continue;
            }
            blend_at(buf, width, xx as u32, yy as u32, color, opacity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank(w: u32, h: u32) -> Vec<u8> {
        vec![0u8; (w * h * 4) as usize]
    }

    #[test]
    fn measure_scales_with_size() {
        // "12" at size 7 → scale 1 → 2 chars * (5+1) = 12 wide, 7 tall.
        assert_eq!(measure("12", 7), (12, 7));
        // size 14 → scale 2 → double.
        assert_eq!(measure("12", 14), (24, 14));
    }

    #[test]
    fn draws_visible_pixels_for_a_digit() {
        let (w, h) = (16u32, 16u32);
        let mut buf = blank(w, h);
        let style = TextStyle {
            content: "1".to_string(),
            size: 7,
            color: Rgba::rgb(255, 255, 255),
            background: None,
            shadow: false,
        };
        draw_text(&mut buf, w, h, Point::new(2.0, 2.0), &style, 1.0);
        // Some white pixel was written inside the glyph box.
        let any = buf.chunks_exact(4).any(|p| p[0] == 255 && p[3] > 0);
        assert!(any, "expected the digit to draw at least one opaque pixel");
    }

    #[test]
    fn blank_glyph_for_unknown_char_draws_nothing() {
        let (w, h) = (16u32, 16u32);
        let mut buf = blank(w, h);
        let style = TextStyle {
            content: "~".to_string(),
            size: 7,
            color: Rgba::rgb(255, 255, 255),
            background: None,
            shadow: false,
        };
        draw_text(&mut buf, w, h, Point::new(2.0, 2.0), &style, 1.0);
        assert!(
            buf.iter().all(|&b| b == 0),
            "unknown glyph should draw nothing"
        );
    }
}
