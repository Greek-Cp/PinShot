//! Geometry primitives used by capture selection.
//!
//! Kept dependency-free and fully unit-tested so selection math is verified
//! without launching the GUI.

use serde::{Deserialize, Serialize};

/// An axis-aligned rectangle in integer pixel coordinates.
///
/// `width` and `height` are always non-negative: [`Rect::from_points`]
/// normalises a selection dragged in any direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    /// Creates a rectangle from a top-left origin and a size.
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Builds a normalised rectangle from two opposite corner points.
    ///
    /// The drag start and end can be in any order; the result always has a
    /// top-left origin with non-negative dimensions.
    pub fn from_points(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        let x = x0.min(x1);
        let y = y0.min(y1);
        let width = x0.abs_diff(x1);
        let height = y0.abs_diff(y1);
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Area in pixels.
    pub const fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Whether the rectangle covers zero pixels (a click without a drag).
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Returns a copy moved by `(dx, dy)`. Used when the user drags an existing
    /// selection (or a pin) without changing its size.
    pub const fn translate(&self, dx: i32, dy: i32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    /// Grows the rectangle so it is at least `min_w × min_h`, keeping the
    /// top-left origin fixed. Used after a resize so a selection can never
    /// collapse to a zero/sub-pixel size that has no grabbable handles.
    pub const fn clamp_min(&self, min_w: u32, min_h: u32) -> Rect {
        Rect::new(
            self.x,
            self.y,
            if self.width < min_w {
                min_w
            } else {
                self.width
            },
            if self.height < min_h {
                min_h
            } else {
                self.height
            },
        )
    }

    /// Constrains the rectangle to lie within `bounds`: first shrinks it to fit
    /// (never wider/taller than `bounds`), then shifts it so it sits fully
    /// inside. Used to keep an adjusted selection on its display and to nudge a
    /// pin back on-screen.
    pub fn clamp_to(&self, bounds: Rect) -> Rect {
        let w = self.width.min(bounds.width);
        let h = self.height.min(bounds.height);
        let max_x = bounds.x + (bounds.width - w) as i32;
        let max_y = bounds.y + (bounds.height - h) as i32;
        let x = self.x.clamp(bounds.x, max_x);
        let y = self.y.clamp(bounds.y, max_y);
        Rect::new(x, y, w, h)
    }

    /// The overlapping rectangle of `self` and `other`, or `None` if they do
    /// not overlap. Used to blit each display's frozen pixels into a crop.
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let ax0 = self.x as i64;
        let ay0 = self.y as i64;
        let ax1 = ax0 + self.width as i64;
        let ay1 = ay0 + self.height as i64;
        let bx0 = other.x as i64;
        let by0 = other.y as i64;
        let bx1 = bx0 + other.width as i64;
        let by1 = by0 + other.height as i64;

        let x0 = ax0.max(bx0);
        let y0 = ay0.max(by0);
        let x1 = ax1.min(bx1);
        let y1 = ay1.min(by1);

        if x1 <= x0 || y1 <= y0 {
            None
        } else {
            Some(Rect::new(
                x0 as i32,
                y0 as i32,
                (x1 - x0) as u32,
                (y1 - y0) as u32,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_points_normalises_reversed_drag() {
        // Dragged bottom-right to top-left.
        let rect = Rect::from_points(100, 80, 20, 10);
        assert_eq!(rect, Rect::new(20, 10, 80, 70));
    }

    #[test]
    fn from_points_handles_negative_origin() {
        let rect = Rect::from_points(-5, -5, 5, 5);
        assert_eq!(rect, Rect::new(-5, -5, 10, 10));
    }

    #[test]
    fn area_and_empty() {
        assert_eq!(Rect::new(0, 0, 4, 3).area(), 12);
        assert!(Rect::new(10, 10, 0, 50).is_empty());
        assert!(!Rect::new(10, 10, 1, 1).is_empty());
    }

    #[test]
    fn intersection_overlap() {
        let a = Rect::new(0, 0, 100, 100);
        let b = Rect::new(50, 50, 100, 100);
        assert_eq!(a.intersection(&b), Some(Rect::new(50, 50, 50, 50)));
    }

    #[test]
    fn intersection_none_when_disjoint() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(20, 20, 10, 10);
        assert_eq!(a.intersection(&b), None);
    }

    #[test]
    fn intersection_touching_edges_is_none() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(10, 0, 10, 10);
        assert_eq!(a.intersection(&b), None);
    }

    #[test]
    fn translate_moves_origin_keeps_size() {
        assert_eq!(
            Rect::new(10, 20, 30, 40).translate(-5, 7),
            Rect::new(5, 27, 30, 40)
        );
    }

    #[test]
    fn clamp_min_grows_small_sides_only() {
        assert_eq!(
            Rect::new(3, 4, 1, 50).clamp_min(8, 8),
            Rect::new(3, 4, 8, 50)
        );
        // Already large enough → unchanged.
        assert_eq!(
            Rect::new(0, 0, 20, 20).clamp_min(8, 8),
            Rect::new(0, 0, 20, 20)
        );
    }

    #[test]
    fn clamp_to_shifts_inside_bounds() {
        let bounds = Rect::new(0, 0, 100, 100);
        // Hanging off the right/bottom edge is shifted back in.
        assert_eq!(
            Rect::new(90, 95, 30, 20).clamp_to(bounds),
            Rect::new(70, 80, 30, 20)
        );
        // Negative origin shifted to the top-left corner.
        assert_eq!(
            Rect::new(-10, -10, 10, 10).clamp_to(bounds),
            Rect::new(0, 0, 10, 10)
        );
    }

    #[test]
    fn clamp_to_shrinks_when_larger_than_bounds() {
        let bounds = Rect::new(0, 0, 50, 50);
        assert_eq!(
            Rect::new(-5, -5, 200, 200).clamp_to(bounds),
            Rect::new(0, 0, 50, 50)
        );
    }
}
