//! The annotation engine's domain model (feature 004).
//!
//! An [`AnnotationDoc`] is a [`CapturedImage`] base plus an ordered, editable
//! stack of [`Annotation`] objects. Everything here is pure data + geometry in
//! the base image's **pixel** coordinate space (the shell maps logical/CSS
//! coordinates to image pixels at the IPC boundary, like 002's `selection`).
//! Rendering to pixels lives in [`render`](crate::annotation::render); per-kind
//! pixel effects in [`effects`](crate::annotation::effects). Keeping the model
//! free of any GUI/encoding dependency lets the whole engine be unit-tested
//! headless (Constitution IV / FR-048).

pub mod effects;
pub mod geometry;
pub mod render;
pub mod step;
pub mod text;

use crate::capture::CapturedImage;
use crate::geometry::Rect;

/// Stable identifier for an annotation within an editing session.
pub type AnnotationId = u64;

/// A point in base-image pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// An RGBA colour, 8 bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Rgba(r, g, b, 255)
    }
}

/// The shape backing an annotation, in base-image pixel coordinates.
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    /// Axis-aligned box: Rectangle, Ellipse, Blur, Pixelate, Spotlight, Crop region.
    Rect(Rect),
    /// Two endpoints: Arrow, Line.
    Segment { a: Point, b: Point },
    /// Freehand polyline: Pencil, Highlighter.
    Path(Vec<Point>),
    /// A single anchor: Text, StepNumber.
    Anchor(Point),
    /// A circular loupe: Magnifier.
    Loupe { center: Point, radius: f32 },
}

/// Arrowhead rendering for [`AnnotationKind::Arrow`]/[`AnnotationKind::Line`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowHead {
    None,
    Open,
    #[default]
    Filled,
}

/// Text-specific style for [`AnnotationKind::Text`].
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub content: String,
    /// Cap height in pixels (the embedded font is scaled to this).
    pub size: u32,
    pub color: Rgba,
    /// Optional filled background behind the text (padding included).
    pub background: Option<Rgba>,
    /// Draw a 1px drop shadow under the glyphs.
    pub shadow: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            content: String::new(),
            size: 16,
            color: Rgba::rgb(0, 0, 0),
            background: None,
            shadow: false,
        }
    }
}

/// The full (superset) visual style of an annotation. Only the fields relevant
/// to a [`Annotation::kind`] are honoured by [`render`](crate::annotation::render).
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub stroke: Rgba,
    pub stroke_width: u32,
    pub fill: Option<Rgba>,
    /// Object opacity, 0.0–1.0 (multiplies every drawn pixel's alpha).
    pub opacity: f32,
    pub corner_radius: u32,
    pub arrow_head: ArrowHead,
    pub dashed: bool,
    pub text: TextStyle,
    /// Gaussian-ish blur radius for [`AnnotationKind::Blur`].
    pub blur_strength: u32,
    /// Mosaic block size for [`AnnotationKind::Pixelate`].
    pub pixelate_block: u32,
    /// Darkening of the area outside a [`AnnotationKind::Spotlight`], 0.0–1.0.
    pub spotlight_dim: f32,
    /// Zoom factor (>1.0) for [`AnnotationKind::Magnifier`].
    pub magnifier_zoom: f32,
    /// 1-based marker number for [`AnnotationKind::StepNumber`].
    pub step_index: u32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            stroke: Rgba::rgb(0xEF, 0x44, 0x44), // PinShot red
            stroke_width: 4,
            fill: None,
            opacity: 1.0,
            corner_radius: 0,
            arrow_head: ArrowHead::Filled,
            dashed: false,
            text: TextStyle::default(),
            blur_strength: 8,
            pixelate_block: 12,
            spotlight_dim: 0.6,
            magnifier_zoom: 2.0,
            step_index: 1,
        }
    }
}

/// The kind of an annotation object. Drives which [`Geometry`] and [`Style`]
/// fields apply and how [`render`](crate::annotation::render) draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationKind {
    Rect,
    Ellipse,
    Arrow,
    Line,
    Pencil,
    Highlighter,
    Text,
    Blur,
    Pixelate,
    Spotlight,
    Magnifier,
    StepNumber,
}

/// One editable object on the canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub id: AnnotationId,
    pub kind: AnnotationKind,
    pub geometry: Geometry,
    pub style: Style,
    /// Stacking order; higher draws on top. Equals the item's index after
    /// [`AnnotationDoc::normalize_z`].
    pub z: u32,
}

/// A capture plus its editable annotation stack — the unit the editor mutates
/// and that [`render::flatten`](crate::annotation::render::flatten) composites.
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationDoc {
    /// Immutable base pixels (never mutated by drawing; effects read from it).
    pub base: CapturedImage,
    /// Owning display's scale factor (logical = physical / scale). Carried for
    /// the shell; flatten works directly in base pixels.
    pub scale: f64,
    /// Z-ordered low→high.
    pub items: Vec<Annotation>,
    next_id: AnnotationId,
}

impl AnnotationDoc {
    /// Creates an empty document over `base`.
    pub fn new(base: CapturedImage, scale: f64) -> Self {
        Self {
            base,
            scale,
            items: Vec::new(),
            next_id: 1,
        }
    }

    /// Adds an annotation on top of the stack, assigning a fresh id and the
    /// next z-index. Returns the new id.
    pub fn add(&mut self, kind: AnnotationKind, geometry: Geometry, style: Style) -> AnnotationId {
        let id = self.next_id;
        self.next_id += 1;
        let z = self.items.len() as u32;
        self.items.push(Annotation {
            id,
            kind,
            geometry,
            style,
            z,
        });
        id
    }

    /// Returns a reference to the annotation with `id`, if present.
    pub fn get(&self, id: AnnotationId) -> Option<&Annotation> {
        self.items.iter().find(|a| a.id == id)
    }

    /// Returns a mutable reference to the annotation with `id`, if present.
    pub fn get_mut(&mut self, id: AnnotationId) -> Option<&mut Annotation> {
        self.items.iter_mut().find(|a| a.id == id)
    }

    /// Removes the annotation with `id`, returning it (and renormalising z).
    pub fn remove(&mut self, id: AnnotationId) -> Option<Annotation> {
        let pos = self.items.iter().position(|a| a.id == id)?;
        let removed = self.items.remove(pos);
        self.normalize_z();
        Some(removed)
    }

    /// Reassigns `z` to equal each item's index after sorting by current z, so
    /// the stack stays a gap-free 0..n ordering (FR-016).
    pub fn normalize_z(&mut self) {
        self.items.sort_by_key(|a| a.z);
        for (i, a) in self.items.iter_mut().enumerate() {
            a.z = i as u32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> AnnotationDoc {
        let base = CapturedImage {
            width: 4,
            height: 4,
            rgba: vec![0u8; 4 * 4 * 4],
        };
        AnnotationDoc::new(base, 1.0)
    }

    #[test]
    fn add_assigns_incrementing_ids_and_z() {
        let mut d = doc();
        let a = d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(0, 0, 2, 2)),
            Style::default(),
        );
        let b = d.add(
            AnnotationKind::Line,
            Geometry::Segment {
                a: Point::new(0.0, 0.0),
                b: Point::new(3.0, 3.0),
            },
            Style::default(),
        );
        assert_ne!(a, b);
        assert_eq!(d.get(a).unwrap().z, 0);
        assert_eq!(d.get(b).unwrap().z, 1);
    }

    #[test]
    fn remove_renormalises_z() {
        let mut d = doc();
        let a = d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(0, 0, 1, 1)),
            Style::default(),
        );
        let b = d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(1, 1, 1, 1)),
            Style::default(),
        );
        let c = d.add(
            AnnotationKind::Rect,
            Geometry::Rect(Rect::new(2, 2, 1, 1)),
            Style::default(),
        );
        assert!(d.remove(b).is_some());
        // a stays z=0, c collapses from z=2 to z=1.
        assert_eq!(d.get(a).unwrap().z, 0);
        assert_eq!(d.get(c).unwrap().z, 1);
        assert_eq!(d.items.len(), 2);
        // Removing a missing id is a no-op.
        assert!(d.remove(999).is_none());
    }

    #[test]
    fn default_style_is_pinshot_red_opaque() {
        let s = Style::default();
        assert_eq!(s.stroke, Rgba::rgb(0xEF, 0x44, 0x44));
        assert_eq!(s.opacity, 1.0);
        assert_eq!(s.arrow_head, ArrowHead::Filled);
    }
}
