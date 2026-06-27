//! Pure geometry for editing annotations: bounding boxes, hit-testing, and
//! moving/resizing objects. Used by the editor's select/move/resize gestures
//! (the shell mirrors these for live preview, then commits through here).

use super::{Annotation, AnnotationDoc, AnnotationId, Geometry, Point};
use crate::geometry::Rect;

/// The axis-aligned bounding box of a geometry, in base-image pixels. A small
/// padding keeps thin shapes (lines, anchors) grabbable.
pub fn bounds(geometry: &Geometry) -> Rect {
    match geometry {
        Geometry::Rect(r) => *r,
        Geometry::Segment { a, b } => rect_from_f32(a.x, a.y, b.x, b.y),
        Geometry::Path(points) => points_bounds(points),
        Geometry::Anchor(p) => Rect::new(p.x as i32, p.y as i32, 1, 1),
        Geometry::Loupe { center, radius } => Rect::new(
            (center.x - radius) as i32,
            (center.y - radius) as i32,
            (radius * 2.0) as u32,
            (radius * 2.0) as u32,
        ),
    }
}

/// Returns the id of the **topmost** annotation whose bounds (padded by its
/// stroke width) contain `point`, or `None`. Iterates high z → low so the
/// visually-frontmost object wins (matches what the user clicks).
pub fn hit_test(doc: &AnnotationDoc, point: Point) -> Option<AnnotationId> {
    let mut ordered: Vec<&Annotation> = doc.items.iter().collect();
    ordered.sort_by_key(|a| std::cmp::Reverse(a.z));
    for a in ordered {
        let pad = a.style.stroke_width.max(1) as i32;
        if inflate(bounds(&a.geometry), pad).contains(point) {
            return Some(a.id);
        }
    }
    None
}

/// Moves an annotation's geometry by `(dx, dy)` base pixels.
pub fn translate(geometry: &Geometry, dx: f32, dy: f32) -> Geometry {
    match geometry {
        Geometry::Rect(r) => Geometry::Rect(r.translate(dx as i32, dy as i32)),
        Geometry::Segment { a, b } => Geometry::Segment {
            a: Point::new(a.x + dx, a.y + dy),
            b: Point::new(b.x + dx, b.y + dy),
        },
        Geometry::Path(points) => Geometry::Path(
            points
                .iter()
                .map(|p| Point::new(p.x + dx, p.y + dy))
                .collect(),
        ),
        Geometry::Anchor(p) => Geometry::Anchor(Point::new(p.x + dx, p.y + dy)),
        Geometry::Loupe { center, radius } => Geometry::Loupe {
            center: Point::new(center.x + dx, center.y + dy),
            radius: *radius,
        },
    }
}

/// Resizes a [`Geometry::Rect`] to a new normalised rectangle (clamped to a
/// 1px minimum). Non-rect geometries are returned unchanged — the editor
/// resizes those by dragging their endpoints/handles directly.
pub fn resize(geometry: &Geometry, new_rect: Rect) -> Geometry {
    match geometry {
        Geometry::Rect(_) => Geometry::Rect(new_rect.clamp_min(1, 1)),
        other => other.clone(),
    }
}

// --- helpers ---

trait Contains {
    fn contains(&self, p: Point) -> bool;
}
impl Contains for Rect {
    fn contains(&self, p: Point) -> bool {
        let x = p.x;
        let y = p.y;
        x >= self.x as f32
            && y >= self.y as f32
            && x <= (self.x + self.width as i32) as f32
            && y <= (self.y + self.height as i32) as f32
    }
}

fn inflate(r: Rect, by: i32) -> Rect {
    Rect::new(
        r.x - by,
        r.y - by,
        r.width + (by.max(0) as u32) * 2,
        r.height + (by.max(0) as u32) * 2,
    )
}

fn rect_from_f32(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_points(x0 as i32, y0 as i32, x1 as i32, y1 as i32)
}

fn points_bounds(points: &[Point]) -> Rect {
    if points.is_empty() {
        return Rect::new(0, 0, 0, 0);
    }
    let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for p in points {
        minx = minx.min(p.x);
        miny = miny.min(p.y);
        maxx = maxx.max(p.x);
        maxy = maxy.max(p.y);
    }
    Rect::new(
        minx as i32,
        miny as i32,
        (maxx - minx).max(0.0) as u32,
        (maxy - miny).max(0.0) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{AnnotationKind, Style};
    use crate::capture::CapturedImage;

    fn doc() -> AnnotationDoc {
        AnnotationDoc::new(
            CapturedImage {
                width: 100,
                height: 100,
                rgba: vec![0u8; 100 * 100 * 4],
            },
            1.0,
        )
    }

    #[test]
    fn bounds_of_each_geometry() {
        assert_eq!(
            bounds(&Geometry::Rect(Rect::new(5, 6, 10, 20))),
            Rect::new(5, 6, 10, 20)
        );
        assert_eq!(
            bounds(&Geometry::Segment {
                a: Point::new(10.0, 2.0),
                b: Point::new(2.0, 12.0)
            }),
            Rect::new(2, 2, 8, 10)
        );
        assert_eq!(
            bounds(&Geometry::Loupe {
                center: Point::new(50.0, 50.0),
                radius: 10.0
            }),
            Rect::new(40, 40, 20, 20)
        );
    }

    #[test]
    fn hit_test_returns_topmost() {
        let mut d = doc();
        let lower = d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(0, 0, 40, 40)),
            Style::default(),
        );
        let upper = d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(10, 10, 40, 40)),
            Style::default(),
        );
        // Point inside both → the later-added (higher z) wins.
        assert_eq!(hit_test(&d, Point::new(20.0, 20.0)), Some(upper));
        // Point only in the lower one.
        assert_eq!(hit_test(&d, Point::new(2.0, 2.0)), Some(lower));
        // Point outside both.
        assert_eq!(hit_test(&d, Point::new(90.0, 90.0)), None);
    }

    #[test]
    fn translate_moves_all_geometry_kinds() {
        assert_eq!(
            translate(&Geometry::Rect(Rect::new(0, 0, 4, 4)), 5.0, 6.0),
            Geometry::Rect(Rect::new(5, 6, 4, 4))
        );
        match translate(&Geometry::Anchor(Point::new(1.0, 1.0)), 2.0, 3.0) {
            Geometry::Anchor(p) => assert_eq!((p.x, p.y), (3.0, 4.0)),
            _ => panic!("kind changed"),
        }
    }

    #[test]
    fn resize_rect_clamps_minimum() {
        assert_eq!(
            resize(
                &Geometry::Rect(Rect::new(0, 0, 99, 99)),
                Rect::new(2, 2, 0, 0)
            ),
            Geometry::Rect(Rect::new(2, 2, 1, 1))
        );
    }
}
