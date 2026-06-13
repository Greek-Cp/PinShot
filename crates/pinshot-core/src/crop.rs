//! Extracting the selected pixels from the frozen frames.
//!
//! Given a selection rectangle in physical virtual-desktop pixels and the per
//! display frozen frames, [`crop_region`] produces a single image — blitting
//! each display's contribution into place, so a selection that spans two
//! monitors composites correctly. Pixels not covered by any display stay
//! transparent.

use crate::capture::{CapturedImage, Display, FrozenFrame};
use crate::geometry::Rect;

/// Error returned when a crop cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CropError {
    /// The selection had zero area (FR-009).
    EmptyRegion,
    /// A frame's pixel dimensions did not match its display's size.
    FrameMismatch { display_id: u32 },
}

impl std::fmt::Display for CropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CropError::EmptyRegion => write!(f, "selection region is empty"),
            CropError::FrameMismatch { display_id } => {
                write!(f, "frame size does not match display {display_id}")
            }
        }
    }
}

impl std::error::Error for CropError {}

/// Crops `region` (physical virtual-desktop pixels) out of the frozen frames.
///
/// Each display whose bounds overlap `region` contributes its overlapping
/// pixels; areas outside every display remain transparent (RGBA 0,0,0,0).
pub fn crop_region(
    frames: &[FrozenFrame],
    displays: &[Display],
    region: Rect,
) -> Result<CapturedImage, CropError> {
    if region.is_empty() {
        return Err(CropError::EmptyRegion);
    }

    let out_w = region.width as usize;
    let out_h = region.height as usize;
    let mut rgba = vec![0u8; out_w * out_h * 4];

    for display in displays {
        let Some(frame) = frames.iter().find(|f| f.display_id == display.id) else {
            continue;
        };
        if frame.width != display.size.0 || frame.height != display.size.1 {
            return Err(CropError::FrameMismatch {
                display_id: display.id,
            });
        }

        let Some(inter) = region.intersection(&display.bounds()) else {
            continue;
        };

        let frame_w = frame.width as usize;
        let run = inter.width as usize;
        for row in 0..inter.height as i32 {
            let vy = inter.y + row;
            let fy = (vy - display.origin.1) as usize;
            let oy = (vy - region.y) as usize;
            let fx = (inter.x - display.origin.0) as usize;
            let ox = (inter.x - region.x) as usize;

            let src = (fy * frame_w + fx) * 4;
            let dst = (oy * out_w + ox) * 4;
            rgba[dst..dst + run * 4].copy_from_slice(&frame.rgba[src..src + run * 4]);
        }
    }

    Ok(CapturedImage {
        width: region.width,
        height: region.height,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a frame whose every pixel encodes (x, y, tag, 255), so a crop's
    /// provenance is checkable pixel by pixel.
    fn tagged_frame(display_id: u32, w: u32, h: u32, tag: u8) -> FrozenFrame {
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                rgba.extend_from_slice(&[x as u8, y as u8, tag, 255]);
            }
        }
        FrozenFrame {
            display_id,
            width: w,
            height: h,
            rgba,
        }
    }

    fn px(img: &CapturedImage, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * img.width + x) * 4) as usize;
        [
            img.rgba[i],
            img.rgba[i + 1],
            img.rgba[i + 2],
            img.rgba[i + 3],
        ]
    }

    #[test]
    fn empty_region_is_rejected() {
        let err = crop_region(&[], &[], Rect::new(0, 0, 0, 5)).unwrap_err();
        assert_eq!(err, CropError::EmptyRegion);
    }

    #[test]
    fn single_display_crop_picks_correct_pixels() {
        let display = Display {
            id: 1,
            origin: (0, 0),
            size: (4, 4),
            scale_factor: 1.0,
        };
        let frame = tagged_frame(1, 4, 4, 7);
        let img = crop_region(&[frame], &[display], Rect::new(1, 2, 2, 1)).unwrap();
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 1);
        // Output (0,0) maps to frame (1,2); output (1,0) to frame (2,2).
        assert_eq!(px(&img, 0, 0), [1, 2, 7, 255]);
        assert_eq!(px(&img, 1, 0), [2, 2, 7, 255]);
    }

    #[test]
    fn spanning_two_displays_composites() {
        let left = Display {
            id: 1,
            origin: (0, 0),
            size: (4, 2),
            scale_factor: 1.0,
        };
        let right = Display {
            id: 2,
            origin: (4, 0),
            size: (2, 2),
            scale_factor: 1.0,
        };
        let frames = vec![tagged_frame(1, 4, 2, 11), tagged_frame(2, 2, 2, 22)];
        // Region x=2..6 (width 4), y=0..2: cols 2,3 from left; cols 0,1 from right.
        let img = crop_region(&frames, &[left, right], Rect::new(2, 0, 4, 2)).unwrap();
        assert_eq!(px(&img, 0, 0), [2, 0, 11, 255]); // left frame x=2
        assert_eq!(px(&img, 1, 0), [3, 0, 11, 255]); // left frame x=3
        assert_eq!(px(&img, 2, 0), [0, 0, 22, 255]); // right frame x=0
        assert_eq!(px(&img, 3, 1), [1, 1, 22, 255]); // right frame x=1,y=1
    }

    #[test]
    fn area_outside_displays_is_transparent() {
        let display = Display {
            id: 1,
            origin: (0, 0),
            size: (2, 2),
            scale_factor: 1.0,
        };
        let frame = tagged_frame(1, 2, 2, 5);
        // Region extends one pixel past the display on the right.
        let img = crop_region(&[frame], &[display], Rect::new(0, 0, 3, 2)).unwrap();
        assert_eq!(px(&img, 0, 0), [0, 0, 5, 255]);
        assert_eq!(px(&img, 2, 0), [0, 0, 0, 0]); // uncovered → transparent
    }

    #[test]
    fn frame_size_mismatch_errors() {
        let display = Display {
            id: 1,
            origin: (0, 0),
            size: (4, 4),
            scale_factor: 1.0,
        };
        let wrong = tagged_frame(1, 2, 2, 1);
        let err = crop_region(&[wrong], &[display], Rect::new(0, 0, 2, 2)).unwrap_err();
        assert_eq!(err, CropError::FrameMismatch { display_id: 1 });
    }
}
