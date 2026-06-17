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
}
