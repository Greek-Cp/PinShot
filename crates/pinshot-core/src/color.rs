//! Reading a pixel's color from an RGBA buffer (US3).
//!
//! Used by the color readout. Pure so the formatting is verified without a GUI.

/// Returns the `(r, g, b)` of the pixel at `(x, y)` in a row-major RGBA buffer
/// `width` pixels wide, or `None` if the index is out of range.
pub fn pixel_rgb(rgba: &[u8], width: u32, x: u32, y: u32) -> Option<(u8, u8, u8)> {
    let i = (y as usize * width as usize + x as usize) * 4;
    let pixel = rgba.get(i..i + 3)?;
    Some((pixel[0], pixel[1], pixel[2]))
}

/// Returns the pixel color as an uppercase `#RRGGBB` string, or `None` if out
/// of range.
pub fn pixel_hex(rgba: &[u8], width: u32, x: u32, y: u32) -> Option<String> {
    let (r, g, b) = pixel_rgb(rgba, width, x, y)?;
    Some(format!("#{r:02X}{g:02X}{b:02X}"))
}

/// A sampled pixel colour in the three representations the picker copies
/// (FR-030): `#RRGGBB`, `rgb(r, g, b)`, and `hsl(h, s%, l%)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorSample {
    pub rgb: (u8, u8, u8),
    pub hex: String,
    /// Hue in degrees (0–360), saturation and lightness in percent (0–100).
    pub hsl: (u16, u8, u8),
}

impl ColorSample {
    /// Builds a sample (hex + HSL) from an RGB triple.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            rgb: (r, g, b),
            hex: format!("#{r:02X}{g:02X}{b:02X}"),
            hsl: rgb_to_hsl(r, g, b),
        }
    }
}

/// Converts RGB to HSL: hue in degrees (0–360), saturation/lightness in
/// percent (0–100), each rounded to the nearest integer.
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;
    let l = (max + min) / 2.0;

    let (mut h, s) = if delta == 0.0 {
        (0.0, 0.0)
    } else {
        let s = delta / (1.0 - (2.0 * l - 1.0).abs());
        let h = if max == rf {
            ((gf - bf) / delta).rem_euclid(6.0)
        } else if max == gf {
            (bf - rf) / delta + 2.0
        } else {
            (rf - gf) / delta + 4.0
        };
        (h * 60.0, s)
    };
    if h < 0.0 {
        h += 360.0;
    }
    (
        h.round() as u16 % 360,
        (s * 100.0).round() as u8,
        (l * 100.0).round() as u8,
    )
}

/// Converts HSL (hue degrees, saturation/lightness percent) back to RGB.
pub fn hsl_to_rgb(h: u16, s: u8, l: u8) -> (u8, u8, u8) {
    let h = (h % 360) as f64;
    let s = (s.min(100) as f64) / 100.0;
    let l = (l.min(100) as f64) / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    )
}

/// `rgb(r, g, b)` string for copying.
pub fn format_rgb(r: u8, g: u8, b: u8) -> String {
    format!("rgb({r}, {g}, {b})")
}

/// `hsl(h, s%, l%)` string for copying.
pub fn format_hsl(h: u16, s: u8, l: u8) -> String {
    format!("hsl({h}, {s}%, {l}%)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf() -> Vec<u8> {
        // 2×1: pixel0 = pure red, pixel1 = (30,144,255) dodger blue.
        vec![255, 0, 0, 255, 30, 144, 255, 255]
    }

    #[test]
    fn reads_rgb() {
        assert_eq!(pixel_rgb(&buf(), 2, 0, 0), Some((255, 0, 0)));
        assert_eq!(pixel_rgb(&buf(), 2, 1, 0), Some((30, 144, 255)));
    }

    #[test]
    fn formats_hex_uppercase() {
        assert_eq!(pixel_hex(&buf(), 2, 1, 0).as_deref(), Some("#1E90FF"));
    }

    #[test]
    fn out_of_range_is_none() {
        assert_eq!(pixel_rgb(&buf(), 2, 5, 5), None);
    }

    #[test]
    fn rgb_to_hsl_known_values() {
        assert_eq!(rgb_to_hsl(255, 0, 0), (0, 100, 50)); // pure red
        assert_eq!(rgb_to_hsl(0, 0, 0), (0, 0, 0)); // black
        assert_eq!(rgb_to_hsl(255, 255, 255), (0, 0, 100)); // white
                                                            // Indigo #4F46E5 ≈ hsl(243, 76%, 59%).
        let (h, s, l) = rgb_to_hsl(0x4F, 0x46, 0xE5);
        assert!((242..=244).contains(&h), "hue {h}");
        assert!((74..=78).contains(&s), "sat {s}");
        assert!((58..=60).contains(&l), "light {l}");
    }

    #[test]
    fn hsl_round_trips_back_to_rgb() {
        for rgb in [
            (255, 0, 0),
            (0, 128, 64),
            (12, 200, 240),
            (0x4F, 0x46, 0xE5),
        ] {
            let (h, s, l) = rgb_to_hsl(rgb.0, rgb.1, rgb.2);
            let back = hsl_to_rgb(h, s, l);
            // Allow ±5 per channel: storing S/L as integer percent quantises
            // high-saturation colours by up to a few 8-bit levels.
            assert!(
                (back.0 as i32 - rgb.0 as i32).abs() <= 5,
                "{rgb:?} -> {back:?}"
            );
            assert!(
                (back.1 as i32 - rgb.1 as i32).abs() <= 5,
                "{rgb:?} -> {back:?}"
            );
            assert!(
                (back.2 as i32 - rgb.2 as i32).abs() <= 5,
                "{rgb:?} -> {back:?}"
            );
        }
    }

    #[test]
    fn color_sample_formats() {
        let s = ColorSample::from_rgb(0x4F, 0x46, 0xE5);
        assert_eq!(s.hex, "#4F46E5");
        assert_eq!(format_rgb(s.rgb.0, s.rgb.1, s.rgb.2), "rgb(79, 70, 229)");
        assert_eq!(
            format_hsl(s.hsl.0, s.hsl.1, s.hsl.2),
            format!("hsl({}, {}%, {}%)", s.hsl.0, s.hsl.1, s.hsl.2)
        );
    }
}
