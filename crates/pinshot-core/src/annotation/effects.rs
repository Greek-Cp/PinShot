//! Pixel effects used by redaction and emphasis annotations: Gaussian-ish blur,
//! pixelate (mosaic), spotlight (dim outside), and a magnifier loupe.
//!
//! Blur and pixelate read from the **original** base buffer (never from
//! already-effected output) so re-editing a region is lossless (spec Edge
//! Case). Spotlight and the magnifier operate on the working buffer. All are
//! pure functions over RGBA byte buffers and unit-tested headless.

use super::Point;
use crate::geometry::Rect;

#[inline]
fn idx(width: u32, x: u32, y: u32) -> usize {
    (y as usize * width as usize + x as usize) * 4
}

/// Clamps `region` to the `[0,width)×[0,height)` image, returning inclusive-
/// exclusive `(x0, y0, x1, y1)` in pixels (empty if fully off-image).
fn clamp_region(width: u32, height: u32, region: Rect) -> (u32, u32, u32, u32) {
    let x0 = region.x.max(0) as u32;
    let y0 = region.y.max(0) as u32;
    let x1 = ((region.x + region.width as i32).max(0) as u32).min(width);
    let y1 = ((region.y + region.height as i32).max(0) as u32).min(height);
    (x0.min(width), y0.min(height), x1, y1)
}

/// Box-blur approximation of a Gaussian over `region`, sampling `base` and
/// writing into `out`. `radius` 0 is a no-op. Larger radius = softer.
pub fn gaussian_blur(
    base: &[u8],
    out: &mut [u8],
    width: u32,
    height: u32,
    region: Rect,
    radius: u32,
) {
    if radius == 0 {
        return;
    }
    let (x0, y0, x1, y1) = clamp_region(width, height, region);
    let r = radius as i32;
    for y in y0..y1 {
        for x in x0..x1 {
            let (mut sr, mut sg, mut sb, mut sa, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
            for dy in -r..=r {
                for dx in -r..=r {
                    let sx = x as i32 + dx;
                    let sy = y as i32 + dy;
                    if sx < 0 || sy < 0 || sx >= width as i32 || sy >= height as i32 {
                        continue;
                    }
                    let i = idx(width, sx as u32, sy as u32);
                    sr += base[i] as u32;
                    sg += base[i + 1] as u32;
                    sb += base[i + 2] as u32;
                    sa += base[i + 3] as u32;
                    n += 1;
                }
            }
            let o = idx(width, x, y);
            out[o] = (sr / n) as u8;
            out[o + 1] = (sg / n) as u8;
            out[o + 2] = (sb / n) as u8;
            out[o + 3] = (sa / n) as u8;
        }
    }
}

/// Mosaic: replaces each `block`×`block` cell in `region` with that cell's
/// average colour from `base`. `block` < 2 is a no-op.
pub fn pixelate(base: &[u8], out: &mut [u8], width: u32, height: u32, region: Rect, block: u32) {
    if block < 2 {
        return;
    }
    let (x0, y0, x1, y1) = clamp_region(width, height, region);
    let mut by = y0;
    while by < y1 {
        let mut bx = x0;
        while bx < x1 {
            let cx1 = (bx + block).min(x1);
            let cy1 = (by + block).min(y1);
            let (mut sr, mut sg, mut sb, mut sa, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
            for y in by..cy1 {
                for x in bx..cx1 {
                    let i = idx(width, x, y);
                    sr += base[i] as u32;
                    sg += base[i + 1] as u32;
                    sb += base[i + 2] as u32;
                    sa += base[i + 3] as u32;
                    n += 1;
                }
            }
            let n = n.max(1);
            let (ar, ag, ab, aa) = (
                (sr / n) as u8,
                (sg / n) as u8,
                (sb / n) as u8,
                (sa / n) as u8,
            );
            for y in by..cy1 {
                for x in bx..cx1 {
                    let o = idx(width, x, y);
                    out[o] = ar;
                    out[o + 1] = ag;
                    out[o + 2] = ab;
                    out[o + 3] = aa;
                }
            }
            bx += block;
        }
        by += block;
    }
}

/// Darkens everything **outside** `region` by `dim` (0.0 = no change, 1.0 =
/// black) on the working buffer `out`, leaving the spotlighted area untouched.
pub fn spotlight(out: &mut [u8], width: u32, height: u32, region: Rect, dim: f32) {
    let dim = dim.clamp(0.0, 1.0);
    if dim == 0.0 {
        return;
    }
    let keep = 1.0 - dim;
    let (x0, y0, x1, y1) = clamp_region(width, height, region);
    for y in 0..height {
        for x in 0..width {
            let inside = x >= x0 && x < x1 && y >= y0 && y < y1;
            if inside {
                continue;
            }
            let o = idx(width, x, y);
            out[o] = (out[o] as f32 * keep) as u8;
            out[o + 1] = (out[o + 1] as f32 * keep) as u8;
            out[o + 2] = (out[o + 2] as f32 * keep) as u8;
        }
    }
}

/// Renders a circular magnified view of `base` centred at `center`, sampling at
/// `zoom` (>1) and writing into the disc on `out`.
pub fn magnify(
    base: &[u8],
    out: &mut [u8],
    width: u32,
    height: u32,
    center: Point,
    radius: f32,
    zoom: f32,
) {
    if radius <= 0.0 || zoom <= 0.0 {
        return;
    }
    let r2 = radius * radius;
    let x0 = (center.x - radius).floor().max(0.0) as u32;
    let y0 = (center.y - radius).floor().max(0.0) as u32;
    let x1 = ((center.x + radius).ceil() as i64).clamp(0, width as i64) as u32;
    let y1 = ((center.y + radius).ceil() as i64).clamp(0, height as i64) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - center.x;
            let dy = y as f32 + 0.5 - center.y;
            if dx * dx + dy * dy > r2 {
                continue;
            }
            // Sample the base at the centre + the offset shrunk by zoom.
            let sx = (center.x + dx / zoom).round();
            let sy = (center.y + dy / zoom).round();
            if sx < 0.0 || sy < 0.0 || sx >= width as f32 || sy >= height as f32 {
                continue;
            }
            let si = idx(width, sx as u32, sy as u32);
            let o = idx(width, x, y);
            out[o] = base[si];
            out[o + 1] = base[si + 1];
            out[o + 2] = base[si + 2];
            out[o + 3] = base[si + 3];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A 4×4 image: left half white, right half black, opaque.
    fn split_image() -> (Vec<u8>, u32, u32) {
        let (w, h) = (4u32, 4u32);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = idx(w, x, y);
                let v = if x < 2 { 255 } else { 0 };
                buf[i] = v;
                buf[i + 1] = v;
                buf[i + 2] = v;
                buf[i + 3] = 255;
            }
        }
        (buf, w, h)
    }

    #[test]
    fn blur_averages_across_an_edge() {
        let (base, w, h) = split_image();
        let mut out = base.clone();
        gaussian_blur(&base, &mut out, w, h, Rect::new(0, 0, 4, 4), 1);
        // A pixel straddling the white/black seam must become a mid grey, not
        // pure white/black.
        let i = idx(w, 1, 1);
        assert!(
            out[i] > 0 && out[i] < 255,
            "blurred edge should be grey, got {}",
            out[i]
        );
    }

    #[test]
    fn pixelate_makes_a_block_uniform() {
        let (base, w, h) = split_image();
        let mut out = base.clone();
        // One 4×4 block over the whole split image → average of 50% white = ~128.
        pixelate(&base, &mut out, w, h, Rect::new(0, 0, 4, 4), 4);
        let first = out[idx(w, 0, 0)];
        for y in 0..h {
            for x in 0..w {
                assert_eq!(out[idx(w, x, y)], first, "block not uniform at {x},{y}");
            }
        }
        assert_eq!(first, 127); // (255*8 + 0*8)/16 rounded down
    }

    #[test]
    fn spotlight_darkens_outside_only() {
        let (base, w, h) = split_image();
        let mut out = base.clone();
        spotlight(&mut out, w, h, Rect::new(0, 0, 2, 2), 0.5);
        // Inside the spotlight (0,0) is unchanged white.
        assert_eq!(out[idx(w, 0, 0)], 255);
        // Outside-but-white (0,3) is halved.
        assert_eq!(out[idx(w, 0, 3)], 127);
    }

    #[test]
    fn magnify_writes_inside_the_disc() {
        let (base, w, h) = split_image();
        let mut out = base.clone();
        // Sanity: function runs and only touches pixels within the radius.
        magnify(&base, &mut out, w, h, Point::new(2.0, 2.0), 2.0, 2.0);
        // Corner well outside the disc is untouched.
        assert_eq!(out[idx(w, 0, 0)], base[idx(w, 0, 0)]);
    }
}
