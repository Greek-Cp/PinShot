//! Unlimited in-session undo/redo for the annotation editor (FR-020/FR-021).
//!
//! A [`HistoryStack`] is an applied log of reversible [`Command`]s plus a
//! cursor. Issuing a new command after undoing truncates the stale redo tail
//! (spec Edge Case / SC-004). Pure: every operation mutates an
//! [`AnnotationDoc`] passed in, so the whole undo model is unit-tested headless.

use crate::annotation::{Annotation, AnnotationDoc, AnnotationId, Geometry, Style};

/// A reversible edit — the unit of undo/redo.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Add a new annotation (stored whole so redo restores the same id/z).
    Add(Annotation),
    /// Remove an existing annotation (stored whole so undo restores it).
    Remove(Annotation),
    /// Restyle/move an annotation from `before` to `after`.
    Mutate {
        id: AnnotationId,
        before: (Geometry, Style),
        after: (Geometry, Style),
    },
}

impl Command {
    fn apply(&self, doc: &mut AnnotationDoc) {
        match self {
            Command::Add(a) => push(doc, a.clone()),
            Command::Remove(a) => remove(doc, a.id),
            Command::Mutate { id, after, .. } => set(doc, *id, after),
        }
    }

    fn invert(&self, doc: &mut AnnotationDoc) {
        match self {
            Command::Add(a) => remove(doc, a.id),
            Command::Remove(a) => push(doc, a.clone()),
            Command::Mutate { id, before, .. } => set(doc, *id, before),
        }
    }
}

fn push(doc: &mut AnnotationDoc, a: Annotation) {
    doc.items.push(a);
    doc.normalize_z();
}

fn remove(doc: &mut AnnotationDoc, id: AnnotationId) {
    doc.items.retain(|x| x.id != id);
    doc.normalize_z();
}

fn set(doc: &mut AnnotationDoc, id: AnnotationId, value: &(Geometry, Style)) {
    if let Some(a) = doc.get_mut(id) {
        a.geometry = value.0.clone();
        a.style = value.1.clone();
    }
}

/// An applied log of commands with a cursor (`cursor` = number applied).
#[derive(Debug, Default)]
pub struct HistoryStack {
    commands: Vec<Command>,
    cursor: usize,
}

impl HistoryStack {
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `cmd` to `doc`, recording it. Any undone (redoable) tail is
    /// discarded first — issuing a new edit after undo abandons that branch.
    pub fn push(&mut self, doc: &mut AnnotationDoc, cmd: Command) {
        self.commands.truncate(self.cursor);
        cmd.apply(doc);
        self.commands.push(cmd);
        self.cursor += 1;
    }

    /// Undoes the most recently applied command, if any. Returns whether it did.
    pub fn undo(&mut self, doc: &mut AnnotationDoc) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        self.commands[self.cursor].clone().invert(doc);
        true
    }

    /// Re-applies the next undone command, if any. Returns whether it did.
    pub fn redo(&mut self, doc: &mut AnnotationDoc) -> bool {
        if self.cursor >= self.commands.len() {
            return false;
        }
        self.commands[self.cursor].clone().apply(doc);
        self.cursor += 1;
        true
    }

    /// Removes every annotation from `doc` and empties the history (FR-022).
    pub fn clear(&mut self, doc: &mut AnnotationDoc) {
        doc.items.clear();
        self.commands.clear();
        self.cursor = 0;
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor < self.commands.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotation::{AnnotationKind, Geometry, Point, Style};
    use crate::capture::CapturedImage;
    use crate::geometry::Rect;

    fn doc() -> AnnotationDoc {
        AnnotationDoc::new(
            CapturedImage {
                width: 8,
                height: 8,
                rgba: vec![0u8; 8 * 8 * 4],
            },
            1.0,
        )
    }

    fn rect_annotation(id: AnnotationId, z: u32) -> Annotation {
        Annotation {
            id,
            kind: AnnotationKind::Rect,
            geometry: Geometry::Rect(Rect::new(0, 0, 2, 2)),
            style: Style::default(),
            z,
        }
    }

    #[test]
    fn undo_then_redo_restores_state() {
        let mut d = doc();
        let mut h = HistoryStack::new();
        h.push(&mut d, Command::Add(rect_annotation(1, 0)));
        h.push(&mut d, Command::Add(rect_annotation(2, 1)));
        assert_eq!(d.items.len(), 2);

        assert!(h.undo(&mut d));
        assert_eq!(d.items.len(), 1);
        assert!(h.undo(&mut d));
        assert_eq!(d.items.len(), 0);
        assert!(!h.undo(&mut d)); // nothing left to undo

        assert!(h.redo(&mut d));
        assert!(h.redo(&mut d));
        assert_eq!(d.items.len(), 2);
        assert!(!h.redo(&mut d));
    }

    #[test]
    fn many_operations_have_no_limit() {
        let mut d = doc();
        let mut h = HistoryStack::new();
        for i in 1..=60u64 {
            h.push(&mut d, Command::Add(rect_annotation(i, (i - 1) as u32)));
        }
        for _ in 0..60 {
            assert!(h.undo(&mut d));
        }
        assert_eq!(d.items.len(), 0);
        for _ in 0..60 {
            assert!(h.redo(&mut d));
        }
        assert_eq!(d.items.len(), 60);
    }

    #[test]
    fn new_command_after_undo_truncates_redo_branch() {
        let mut d = doc();
        let mut h = HistoryStack::new();
        h.push(&mut d, Command::Add(rect_annotation(1, 0)));
        h.push(&mut d, Command::Add(rect_annotation(2, 1)));
        h.undo(&mut d); // drop #2 (now redoable)
        assert!(h.can_redo());
        h.push(&mut d, Command::Add(rect_annotation(3, 1))); // new edit
        assert!(!h.can_redo(), "stale redo branch must be discarded");
        assert_eq!(d.items.len(), 2);
        assert!(d.get(3).is_some() && d.get(2).is_none());
    }

    #[test]
    fn mutate_round_trips() {
        let mut d = doc();
        let mut h = HistoryStack::new();
        h.push(&mut d, Command::Add(rect_annotation(1, 0)));
        let before = (
            d.get(1).unwrap().geometry.clone(),
            d.get(1).unwrap().style.clone(),
        );
        let after_geo = Geometry::Anchor(Point::new(5.0, 5.0));
        h.push(
            &mut d,
            Command::Mutate {
                id: 1,
                before: before.clone(),
                after: (after_geo.clone(), before.1.clone()),
            },
        );
        assert_eq!(d.get(1).unwrap().geometry, after_geo);
        h.undo(&mut d);
        assert_eq!(d.get(1).unwrap().geometry, before.0);
    }

    #[test]
    fn clear_empties_history_and_doc() {
        let mut d = doc();
        let mut h = HistoryStack::new();
        h.push(&mut d, Command::Add(rect_annotation(1, 0)));
        h.clear(&mut d);
        assert_eq!(d.items.len(), 0);
        assert!(!h.can_undo() && !h.can_redo());
    }
}
