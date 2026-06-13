//! Mapping the overlay's logical selection to physical screen pixels.
//!
//! This is the single place DPI scaling happens. The overlay reports a
//! selection in its window's **logical** (CSS) coordinates, where `(0,0)` is
//! the top-left of the display the overlay covers. We convert that to a
//! rectangle in **physical** virtual-desktop pixels so [`crate::crop`] can
//! extract the exact pixels from the frozen frames. Keeping this here (tested)
//! rather than in the webview is what makes mixed-DPI capture correct (SC-002).

use crate::capture::Display;
use crate::geometry::Rect;

/// Converts a logical selection rectangle on `display`'s overlay into physical
/// virtual-desktop pixels: scale by the display's DPI factor, then offset by
/// the display's origin.
pub fn to_physical(logical: Rect, display: &Display) -> Rect {
    let scale = display.scale_factor;
    let x = display.origin.0 + (f64::from(logical.x) * scale).round() as i32;
    let y = display.origin.1 + (f64::from(logical.y) * scale).round() as i32;
    let width = (f64::from(logical.width) * scale).round() as u32;
    let height = (f64::from(logical.height) * scale).round() as u32;
    Rect::new(x, y, width, height)
}

/// The ids of every display whose bounds intersect `region` (physical
/// virtual-desktop pixels). A selection spanning two monitors returns both.
pub fn displays_for(region: Rect, displays: &[Display]) -> Vec<u32> {
    displays
        .iter()
        .filter(|d| region.intersection(&d.bounds()).is_some())
        .map(|d| d.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hidpi() -> Display {
        Display {
            id: 1,
            origin: (0, 0),
            size: (400, 200),
            scale_factor: 2.0,
        }
    }

    fn external_1x() -> Display {
        Display {
            id: 2,
            origin: (400, 0),
            size: (200, 200),
            scale_factor: 1.0,
        }
    }

    #[test]
    fn scales_logical_to_physical_on_hidpi() {
        // Logical 10,5 size 100x50 on a 2× display → physical doubles.
        let phys = to_physical(Rect::new(10, 5, 100, 50), &hidpi());
        assert_eq!(phys, Rect::new(20, 10, 200, 100));
    }

    #[test]
    fn offsets_by_display_origin_on_1x() {
        let phys = to_physical(Rect::new(10, 10, 50, 50), &external_1x());
        assert_eq!(phys, Rect::new(410, 10, 50, 50));
    }

    #[test]
    fn displays_for_single_display() {
        let region = Rect::new(20, 10, 100, 50);
        assert_eq!(displays_for(region, &[hidpi(), external_1x()]), vec![1]);
    }

    #[test]
    fn displays_for_spanning_region() {
        // Region straddles the seam at x=400.
        let region = Rect::new(380, 10, 60, 50);
        assert_eq!(displays_for(region, &[hidpi(), external_1x()]), vec![1, 2]);
    }
}
