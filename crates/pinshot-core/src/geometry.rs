//! Geometry primitives used by capture selection.
//!
//! Kept dependency-free and fully unit-tested so selection math is verified
//! without launching the GUI.

/// An axis-aligned rectangle in integer pixel coordinates.
///
/// `width` and `height` are always non-negative: [`Rect::from_points`]
/// normalises a selection dragged in any direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}
