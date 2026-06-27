//! Flattening an [`AnnotationDoc`](super::AnnotationDoc) to pixels.
//!
//! [`flatten`] composites every annotation, in z-order, onto a copy of the base
//! image and returns the authoritative [`CapturedImage`] that Copy/Save/Pin use
//! (spec FR-023/024/025). All drawing is pure and DPI-correct (it works in the
//! base's physical pixels), so the privacy- and DPI-critical output is computed
//! once, in tested Rust, never in the webview (Constitution IV / SC-002).

// Pixel-plotting helpers naturally take (buf, width, height, x, y, …); the
// argument count is inherent to graphics primitives, not accidental complexity.
#![allow(clippy::too_many_arguments)]

use super::effects;
use super::geometry::bounds;
use super::text;
use super::{Annotation, AnnotationDoc, AnnotationKind, ArrowHead, Geometry, Point, Rgba};
use crate::capture::CapturedImage;
use crate::geometry::Rect;

/// Error returned when a document cannot be flattened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// The base pixel buffer length did not match `width * height * 4`.
    InvalidBuffer,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::InvalidBuffer => write!(f, "base pixel buffer does not match dimensions"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Composites all annotations onto the base and returns the flattened image.
pub fn flatten(doc: &AnnotationDoc) -> Result<CapturedImage, RenderError> {
    let (w, h) = (doc.base.width, doc.base.height);
    if doc.base.rgba.len() != (w as usize * h as usize * 4) {
        return Err(RenderError::InvalidBuffer);
    }
    let base = &doc.base.rgba;
    let mut out = base.clone();

    let mut items: Vec<&Annotation> = doc.items.iter().collect();
    items.sort_by_key(|a| a.z);

    for a in items {
        draw(base, &mut out, w, h, a);
    }

    Ok(CapturedImage {
        width: w,
        height: h,
        rgba: out,
    })
}

fn draw(base: &[u8], out: &mut [u8], w: u32, h: u32, a: &Annotation) {
    let s = &a.style;
    let op = s.opacity.clamp(0.0, 1.0);
    match a.kind {
        AnnotationKind::Rect => {
            let r = rect_of(&a.geometry);
            if let Some(fill) = s.fill {
                fill_rect(out, w, h, r, fill, op);
            }
            stroke_rect(out, w, h, r, s.stroke_width.max(1), s.stroke, op);
        }
        AnnotationKind::Ellipse => {
            let r = rect_of(&a.geometry);
            if let Some(fill) = s.fill {
                fill_ellipse(out, w, h, r, fill, op);
            }
            stroke_ellipse(out, w, h, r, s.stroke_width.max(1), s.stroke, op);
        }
        AnnotationKind::Line | AnnotationKind::Arrow => {
            if let Geometry::Segment { a: p0, b: p1 } = &a.geometry {
                draw_line(out, w, h, *p0, *p1, s.stroke_width.max(1), s.stroke, op);
                if a.kind == AnnotationKind::Arrow && s.arrow_head != ArrowHead::None {
                    draw_arrowhead(out, w, h, *p0, *p1, s.stroke_width.max(1), s.stroke, op);
                }
            }
        }
        AnnotationKind::Pencil => {
            if let Geometry::Path(points) = &a.geometry {
                draw_path(out, w, h, points, s.stroke_width.max(1), s.stroke, op);
            }
        }
        AnnotationKind::Highlighter => {
            if let Geometry::Path(points) = &a.geometry {
                // Translucent marker: a wide, low-opacity stroke.
                let marker_op = if op >= 1.0 { 0.4 } else { op };
                draw_path(
                    out,
                    w,
                    h,
                    points,
                    s.stroke_width.max(8),
                    s.stroke,
                    marker_op,
                );
            }
        }
        AnnotationKind::Text => {
            if let Geometry::Anchor(at) = &a.geometry {
                text::draw_text(out, w, h, *at, &s.text, op);
            }
        }
        AnnotationKind::Blur => {
            effects::gaussian_blur(base, out, w, h, rect_of(&a.geometry), s.blur_strength);
        }
        AnnotationKind::Pixelate => {
            effects::pixelate(base, out, w, h, rect_of(&a.geometry), s.pixelate_block);
        }
        AnnotationKind::Spotlight => {
            effects::spotlight(out, w, h, rect_of(&a.geometry), s.spotlight_dim);
        }
        AnnotationKind::Magnifier => {
            if let Geometry::Loupe { center, radius } = &a.geometry {
                effects::magnify(base, out, w, h, *center, *radius, s.magnifier_zoom);
                // A ring around the loupe for definition.
                stroke_ellipse(
                    out,
                    w,
                    h,
                    Rect::new(
                        (center.x - radius) as i32,
                        (center.y - radius) as i32,
                        (radius * 2.0) as u32,
                        (radius * 2.0) as u32,
                    ),
                    2,
                    Rgba::rgb(255, 255, 255),
                    op,
                );
            }
        }
        AnnotationKind::StepNumber => {
            if let Geometry::Anchor(at) = &a.geometry {
                draw_step(out, w, h, *at, s.step_index, s.stroke, op);
            }
        }
    }
    // Silence unused warning for shapes that ignore bounds; keep the helper used.
    let _ = bounds(&a.geometry);
}

// --- pixel helpers ---

#[inline]
fn idx(width: u32, x: u32, y: u32) -> usize {
    (y as usize * width as usize + x as usize) * 4
}

/// Alpha-blends `c` (using its alpha × `opacity`) over the pixel at `(x, y)`.
/// Shared with [`text`](super::text). Out-of-range coordinates are ignored.
pub(crate) fn blend_at(buf: &mut [u8], width: u32, x: u32, y: u32, c: Rgba, opacity: f32) {
    let a = (c.3 as f32 / 255.0) * opacity.clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    let i = idx(width, x, y);
    if i + 3 >= buf.len() {
        return;
    }
    let inv = 1.0 - a;
    buf[i] = (c.0 as f32 * a + buf[i] as f32 * inv).round() as u8;
    buf[i + 1] = (c.1 as f32 * a + buf[i + 1] as f32 * inv).round() as u8;
    buf[i + 2] = (c.2 as f32 * a + buf[i + 2] as f32 * inv).round() as u8;
    buf[i + 3] = (255.0 * a + buf[i + 3] as f32 * inv).round() as u8;
}

fn rect_of(g: &Geometry) -> Rect {
    match g {
        Geometry::Rect(r) => *r,
        other => bounds(other),
    }
}

fn fill_rect(buf: &mut [u8], w: u32, h: u32, r: Rect, c: Rgba, op: f32) {
    let x0 = r.x.max(0);
    let y0 = r.y.max(0);
    let x1 = ((r.x + r.width as i32).min(w as i32)).max(0);
    let y1 = ((r.y + r.height as i32).min(h as i32)).max(0);
    for y in y0..y1 {
        for x in x0..x1 {
            blend_at(buf, w, x as u32, y as u32, c, op);
        }
    }
}

fn stroke_rect(buf: &mut [u8], w: u32, h: u32, r: Rect, thick: u32, c: Rgba, op: f32) {
    let t = thick as i32;
    // Top, bottom, left, right bands.
    fill_rect(
        buf,
        w,
        h,
        Rect::new(r.x, r.y, r.width, thick.min(r.height)),
        c,
        op,
    );
    fill_rect(
        buf,
        w,
        h,
        Rect::new(r.x, r.y + r.height as i32 - t, r.width, thick.min(r.height)),
        c,
        op,
    );
    fill_rect(
        buf,
        w,
        h,
        Rect::new(r.x, r.y, thick.min(r.width), r.height),
        c,
        op,
    );
    fill_rect(
        buf,
        w,
        h,
        Rect::new(r.x + r.width as i32 - t, r.y, thick.min(r.width), r.height),
        c,
        op,
    );
}

fn ellipse_norm(r: Rect, x: i32, y: i32) -> f32 {
    let rx = (r.width as f32 / 2.0).max(0.5);
    let ry = (r.height as f32 / 2.0).max(0.5);
    let cx = r.x as f32 + rx;
    let cy = r.y as f32 + ry;
    let nx = (x as f32 + 0.5 - cx) / rx;
    let ny = (y as f32 + 0.5 - cy) / ry;
    nx * nx + ny * ny
}

fn fill_ellipse(buf: &mut [u8], w: u32, h: u32, r: Rect, c: Rgba, op: f32) {
    for y in r.y.max(0)..(r.y + r.height as i32).min(h as i32) {
        for x in r.x.max(0)..(r.x + r.width as i32).min(w as i32) {
            if ellipse_norm(r, x, y) <= 1.0 {
                blend_at(buf, w, x as u32, y as u32, c, op);
            }
        }
    }
}

fn stroke_ellipse(buf: &mut [u8], w: u32, h: u32, r: Rect, thick: u32, c: Rgba, op: f32) {
    // Inner boundary scaled down by the stroke thickness.
    let inner = (1.0 - (thick as f32 * 2.0) / (r.width.min(r.height).max(1) as f32)).max(0.0);
    let inner2 = inner * inner;
    for y in r.y.max(0)..(r.y + r.height as i32).min(h as i32) {
        for x in r.x.max(0)..(r.x + r.width as i32).min(w as i32) {
            let n = ellipse_norm(r, x, y);
            if n <= 1.0 && n >= inner2 {
                blend_at(buf, w, x as u32, y as u32, c, op);
            }
        }
    }
}

fn stamp(buf: &mut [u8], w: u32, h: u32, cx: i32, cy: i32, thick: u32, c: Rgba, op: f32) {
    let half = (thick as i32) / 2;
    for y in cy - half..=cy + half {
        for x in cx - half..=cx + half {
            if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                continue;
            }
            blend_at(buf, w, x as u32, y as u32, c, op);
        }
    }
}

fn draw_line(buf: &mut [u8], w: u32, h: u32, a: Point, b: Point, thick: u32, c: Rgba, op: f32) {
    // Bresenham over the integer grid, stamping a `thick` square per step.
    let (mut x0, mut y0) = (a.x.round() as i32, a.y.round() as i32);
    let (x1, y1) = (b.x.round() as i32, b.y.round() as i32);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        stamp(buf, w, h, x0, y0, thick, c, op);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn draw_path(buf: &mut [u8], w: u32, h: u32, points: &[Point], thick: u32, c: Rgba, op: f32) {
    if points.len() == 1 {
        stamp(
            buf,
            w,
            h,
            points[0].x as i32,
            points[0].y as i32,
            thick,
            c,
            op,
        );
    }
    for pair in points.windows(2) {
        draw_line(buf, w, h, pair[0], pair[1], thick, c, op);
    }
}

fn draw_arrowhead(
    buf: &mut [u8],
    w: u32,
    h: u32,
    from: Point,
    tip: Point,
    thick: u32,
    c: Rgba,
    op: f32,
) {
    let dx = tip.x - from.x;
    let dy = tip.y - from.y;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let (ux, uy) = (dx / len, dy / len);
    let size = (thick as f32 * 4.0).max(8.0);
    // Two barbs at ±30° from the shaft.
    for angle in [2.61_f32, -2.61_f32] {
        let (sa, ca) = angle.sin_cos();
        let bx = tip.x + (ux * ca - uy * sa) * size;
        let by = tip.y + (ux * sa + uy * ca) * size;
        draw_line(buf, w, h, tip, Point::new(bx, by), thick, c, op);
    }
}

fn draw_step(buf: &mut [u8], w: u32, h: u32, at: Point, index: u32, c: Rgba, op: f32) {
    let radius = 11.0_f32;
    let r = Rect::new(
        (at.x - radius) as i32,
        (at.y - radius) as i32,
        (radius * 2.0) as u32,
        (radius * 2.0) as u32,
    );
    fill_ellipse(buf, w, h, r, c, op);
    // White number centred in the disc.
    let label = index.to_string();
    let (tw, th) = text::measure(&label, 12);
    let style = super::TextStyle {
        content: label,
        size: 12,
        color: Rgba::rgb(255, 255, 255),
        background: None,
        shadow: false,
    };
    text::draw_text(
        buf,
        w,
        h,
        Point::new(at.x - tw as f32 / 2.0, at.y - th as f32 / 2.0),
        &style,
        op,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{AnnotationDoc, Style};

    fn doc(w: u32, h: u32) -> AnnotationDoc {
        AnnotationDoc::new(
            CapturedImage {
                width: w,
                height: h,
                rgba: vec![0u8; (w * h * 4) as usize],
            },
            1.0,
        )
    }

    fn px(img: &CapturedImage, x: u32, y: u32) -> [u8; 4] {
        let i = idx(img.width, x, y);
        [
            img.rgba[i],
            img.rgba[i + 1],
            img.rgba[i + 2],
            img.rgba[i + 3],
        ]
    }

    #[test]
    fn rejects_bad_base_buffer() {
        let mut d = doc(2, 2);
        d.base.rgba.truncate(3);
        assert_eq!(flatten(&d).unwrap_err(), RenderError::InvalidBuffer);
    }

    #[test]
    fn empty_doc_returns_base_unchanged() {
        let d = doc(8, 8);
        let out = flatten(&d).unwrap();
        assert_eq!(out.rgba, d.base.rgba);
        assert_eq!((out.width, out.height), (8, 8));
    }

    #[test]
    fn rectangle_stroke_draws_on_the_border_not_centre() {
        let mut d = doc(20, 20);
        let s = Style {
            stroke: Rgba::rgb(255, 0, 0),
            stroke_width: 2,
            fill: None,
            ..Default::default()
        };
        d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(4, 4, 10, 10)),
            s,
        );
        let out = flatten(&d).unwrap();
        // Border pixel is red…
        assert_eq!(px(&out, 4, 4), [255, 0, 0, 255]);
        // …centre is untouched (transparent black base).
        assert_eq!(px(&out, 9, 9), [0, 0, 0, 0]);
    }

    #[test]
    fn fill_paints_interior() {
        let mut d = doc(20, 20);
        let s = Style {
            fill: Some(Rgba::rgb(0, 0, 255)),
            ..Default::default()
        };
        d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(2, 2, 12, 12)),
            s,
        );
        let out = flatten(&d).unwrap();
        assert_eq!(px(&out, 8, 8), [0, 0, 255, 255]);
    }

    #[test]
    fn flatten_is_deterministic() {
        let mut d = doc(16, 16);
        d.add(
            AnnotationKind::Arrow,
            Geometry::Segment {
                a: Point::new(1.0, 1.0),
                b: Point::new(14.0, 12.0),
            },
            Style::default(),
        );
        d.add(
            AnnotationKind::Ellipse,
            Geometry::Rect(Rect::new(2, 2, 10, 8)),
            Style::default(),
        );
        assert_eq!(flatten(&d).unwrap(), flatten(&d).unwrap());
    }

    #[test]
    fn blur_kind_changes_pixels_via_effects() {
        // White-on-left base so blur near the seam produces grey.
        let mut d = doc(8, 4);
        for y in 0..4 {
            for x in 0..4 {
                let i = idx(8, x, y);
                d.base.rgba[i] = 255;
                d.base.rgba[i + 1] = 255;
                d.base.rgba[i + 2] = 255;
                d.base.rgba[i + 3] = 255;
            }
        }
        let s = Style {
            blur_strength: 2,
            ..Default::default()
        };
        d.add(
            AnnotationKind::Blur,
            Geometry::Rect(Rect::new(0, 0, 8, 4)),
            s,
        );
        let out = flatten(&d).unwrap();
        let p = px(&out, 4, 1);
        assert!(
            p[0] > 0 && p[0] < 255,
            "blur should grey the seam, got {p:?}"
        );
    }

    #[test]
    fn z_order_later_draws_on_top() {
        let mut d = doc(10, 10);
        let red = Style {
            fill: Some(Rgba::rgb(255, 0, 0)),
            ..Default::default()
        };
        let blue = Style {
            fill: Some(Rgba::rgb(0, 0, 255)),
            ..Default::default()
        };
        d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(0, 0, 10, 10)),
            red,
        );
        d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(0, 0, 10, 10)),
            blue,
        );
        let out = flatten(&d).unwrap();
        assert_eq!(px(&out, 5, 5), [0, 0, 255, 255]); // blue (added last) wins
    }
}
