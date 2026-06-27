//! Step-number sequencing: assign the next marker number and renumber the
//! sequence after one is removed, so markers always read 1, 2, 3 … in stacking
//! order (FR-033). Pure functions over the document.

use super::{AnnotationDoc, AnnotationKind};

/// The number to give the next Step-Number marker: one more than the count of
/// existing step markers.
pub fn next_index(doc: &AnnotationDoc) -> u32 {
    doc.items
        .iter()
        .filter(|a| a.kind == AnnotationKind::StepNumber)
        .count() as u32
        + 1
}

/// Reassigns every Step-Number marker's `step_index` to 1..=n following the
/// current stacking (z) order, closing any gap left by a deletion.
pub fn renumber(doc: &mut AnnotationDoc) {
    doc.normalize_z();
    let mut n = 1u32;
    for a in doc.items.iter_mut() {
        if a.kind == AnnotationKind::StepNumber {
            a.style.step_index = n;
            n += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{Geometry, Point, Style};
    use crate::capture::CapturedImage;

    fn doc() -> AnnotationDoc {
        AnnotationDoc::new(
            CapturedImage {
                width: 10,
                height: 10,
                rgba: vec![0u8; 10 * 10 * 4],
            },
            1.0,
        )
    }

    fn add_step(d: &mut AnnotationDoc, at: f32) -> u64 {
        let s = Style {
            step_index: next_index(d),
            ..Default::default()
        };
        d.add(
            AnnotationKind::StepNumber,
            Geometry::Anchor(Point::new(at, at)),
            s,
        )
    }

    #[test]
    fn next_index_counts_only_steps() {
        let mut d = doc();
        assert_eq!(next_index(&d), 1);
        add_step(&mut d, 1.0);
        // A non-step annotation does not advance the step counter.
        d.add(
            AnnotationKind::Rect,
            Geometry::Rect(crate::geometry::Rect::new(0, 0, 2, 2)),
            Style::default(),
        );
        assert_eq!(next_index(&d), 2);
    }

    #[test]
    fn renumber_closes_gaps_in_order() {
        let mut d = doc();
        let _s1 = add_step(&mut d, 1.0);
        let s2 = add_step(&mut d, 2.0);
        let _s3 = add_step(&mut d, 3.0);
        assert_eq!(d.get(s2).unwrap().style.step_index, 2);
        d.remove(s2);
        renumber(&mut d);
        let indices: Vec<u32> = d
            .items
            .iter()
            .filter(|a| a.kind == AnnotationKind::StepNumber)
            .map(|a| a.style.step_index)
            .collect();
        assert_eq!(indices, vec![1, 2]);
    }
}
