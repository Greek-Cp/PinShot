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
}
