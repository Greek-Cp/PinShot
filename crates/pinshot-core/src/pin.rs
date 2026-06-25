//! Pure geometry for floating pins.
//!
//! A pin shows a captured region at its **full physical size** (never
//! down-scaled), rendered at the owning display's logical scale. These helpers
//! turn a physical capture region + its display into the logical
//! position/size a windowing layer needs, with the top-left nudged on-screen so
//! the user always has a grab area — even when the pin is larger than the
//! display (they drag to reveal the rest). All side effects (window creation,
//! drag, z-order) live in the Tauri shell; this module stays headless and tested.

use crate::geometry::Rect;

/// Logical (CSS-pixel) size of a pin showing a `physical`-pixel region on a
/// display with the given `scale` factor. Physical / scale, so a 400×300 region
/// on a 2× display is a 200×150 logical window holding a crisp 400×300 image.
pub fn pin_logical_size(physical: (u32, u32), scale: f64) -> (f64, f64) {
    let scale = if scale > 0.0 { scale } else { 1.0 };
    (physical.0 as f64 / scale, physical.1 as f64 / scale)
}

/// Logical placement `(x, y, w, h)` for a pin.
///
/// `region_physical` is the captured region in physical virtual-desktop pixels;
/// `display_origin` and `scale` describe the display it was captured on;
/// `work_area` is that display's usable region in **logical** coordinates (e.g.
/// excluding the menu bar/taskbar). The pin opens at the region's on-screen
/// location and is nudged so its top-left stays within `work_area`. It is **not**
/// shrunk to fit: an oversized pin keeps its full size and simply extends past
/// the edge, draggable to reveal more.
pub fn pin_placement(
    region_physical: Rect,
    display_origin: (i32, i32),
    scale: f64,
    work_area: Rect,
) -> (f64, f64, f64, f64) {
    let scale = if scale > 0.0 { scale } else { 1.0 };
    let (w, h) = pin_logical_size((region_physical.width, region_physical.height), scale);

    // Region origin within the display → logical desktop coordinates.
    let local_x = (region_physical.x - display_origin.0) as f64 / scale;
    let local_y = (region_physical.y - display_origin.1) as f64 / scale;
    let x = display_origin.0 as f64 / scale + local_x;
    let y = display_origin.1 as f64 / scale + local_y;

    // Keep the top-left grabbable: clamp into the work area, but never below its
    // origin even when the pin is wider/taller than the work area.
    let max_x = (work_area.x as f64 + work_area.width as f64 - w).max(work_area.x as f64);
    let max_y = (work_area.y as f64 + work_area.height as f64 - h).max(work_area.y as f64);
    let x = x.clamp(work_area.x as f64, max_x);
    let y = y.clamp(work_area.y as f64, max_y);

    (x, y, w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_size_divides_by_scale() {
        assert_eq!(pin_logical_size((400, 300), 2.0), (200.0, 150.0));
        assert_eq!(pin_logical_size((400, 300), 1.0), (400.0, 300.0));
        // Guard against a zero/garbage scale.
        assert_eq!(pin_logical_size((400, 300), 0.0), (400.0, 300.0));
    }

    #[test]
    fn placement_1x_keeps_region_origin() {
        // Region at (300,200) on a 1× display whose origin is (0,0).
        let (x, y, w, h) = pin_placement(
            Rect::new(300, 200, 400, 300),
            (0, 0),
            1.0,
            Rect::new(0, 0, 1920, 1080),
        );
        assert_eq!((x, y, w, h), (300.0, 200.0, 400.0, 300.0));
    }

    #[test]
    fn placement_2x_converts_to_logical() {
        // 800×600 physical region at physical (200,100) on a 2× display.
        let (x, y, w, h) = pin_placement(
            Rect::new(200, 100, 800, 600),
            (0, 0),
            2.0,
            Rect::new(0, 0, 1440, 900),
        );
        assert_eq!((x, y, w, h), (100.0, 50.0, 400.0, 300.0));
    }

    #[test]
    fn placement_oversized_pins_to_work_area_origin_not_shrunk() {
        // Region larger than the work area: size is preserved (full size),
        // top-left clamped to the work-area origin so it stays grabbable.
        let (x, y, w, h) = pin_placement(
            Rect::new(0, 0, 3000, 2000),
            (0, 0),
            1.0,
            Rect::new(0, 25, 1920, 1055),
        );
        assert_eq!((w, h), (3000.0, 2000.0));
        assert_eq!((x, y), (0.0, 25.0));
    }

    #[test]
    fn placement_offscreen_region_nudged_inside() {
        // Region hanging off the right edge is shifted left so it fits.
        let (x, _y, w, _h) = pin_placement(
            Rect::new(1800, 100, 400, 300),
            (0, 0),
            1.0,
            Rect::new(0, 0, 1920, 1080),
        );
        assert_eq!(w, 400.0);
        assert_eq!(x, 1520.0); // 1920 - 400
    }
}
