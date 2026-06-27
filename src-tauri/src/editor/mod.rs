//! The floating annotation editor — shell side (feature 004, US1/US2).
//!
//! Owns the single in-flight [`EditSession`] (an `AnnotationDoc` + undo
//! `HistoryStack`) and the Tauri commands the editor webview calls. All
//! annotation mutation, flatten, and history logic lives in `pinshot_core`
//! (Constitution IV); this module only locks state, forwards intents to the
//! core, and returns serialisable snapshots. Geometry crosses IPC in **base
//! image pixels** (the webview maps pointer → image pixels), so the core stays
//! the single source of truth for the output pixels.

mod export;
mod window;

use std::sync::Mutex;

use base64::Engine;
use pinshot_core::{
    flatten, to_png, Annotation, AnnotationDoc, AnnotationId, AnnotationKind, CapturedImage,
    Command, Geometry, HistoryStack, Style,
};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::capture::pin::PinRegistry;

/// One in-flight edit: the document, its undo history, the owning display's
/// scale, and a revision the UI uses to detect drift.
pub struct EditSession {
    doc: AnnotationDoc,
    history: HistoryStack,
    scale: f64,
    revision: u64,
}

impl EditSession {
    fn response(&self) -> DocResponse {
        DocResponse {
            revision: self.revision,
            items: self.doc.items.clone(),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
        }
    }
}

/// App-wide editor state (one session at a time).
#[derive(Default)]
pub struct EditState {
    session: Mutex<Option<EditSession>>,
}

/// Registers editor state. Call from Tauri setup.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    app.manage(EditState::default());
    Ok(())
}

/// Seeds a new edit session from a freshly cropped capture and opens the
/// floating editor window (called by `capture::edit_selection`).
pub fn open_session(app: &AppHandle, image: CapturedImage, scale: f64) -> tauri::Result<()> {
    let (w, h) = (image.width, image.height);
    let doc = AnnotationDoc::new(image, scale);
    {
        let state = app.state::<EditState>();
        *state.session.lock().expect("edit session lock") = Some(EditSession {
            doc,
            history: HistoryStack::new(),
            scale,
            revision: 0,
        });
    }
    window::open(app, w, h, scale)
}

/// The base capture image for the editor page (PNG data URL + size/scale).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorImagePayload {
    width: u32,
    height: u32,
    scale_factor: f64,
    data_url: String,
}

/// A snapshot of the document after a mutation (drives the canvas re-render).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocResponse {
    revision: u64,
    items: Vec<Annotation>,
    can_undo: bool,
    can_redo: bool,
}

/// Result of adding an annotation.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddResponse {
    id: AnnotationId,
    revision: u64,
}

fn with_session<T>(
    state: &State<'_, EditState>,
    f: impl FnOnce(&mut EditSession) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = state.session.lock().expect("edit session lock");
    let session = guard.as_mut().ok_or("no active editor")?;
    f(session)
}

/// The editor page pulls its base image on load.
#[tauri::command]
pub fn editor_get_image(state: State<'_, EditState>) -> Result<EditorImagePayload, String> {
    let guard = state.session.lock().expect("edit session lock");
    let session = guard.as_ref().ok_or("no active editor")?;
    let png = to_png(&session.doc.base).map_err(|e| e.to_string())?;
    let data_url = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png)
    );
    Ok(EditorImagePayload {
        width: session.doc.base.width,
        height: session.doc.base.height,
        scale_factor: session.scale,
        data_url,
    })
}

/// Resyncs the UI from the authoritative document.
#[tauri::command]
pub fn editor_get_doc(state: State<'_, EditState>) -> Result<DocResponse, String> {
    let guard = state.session.lock().expect("edit session lock");
    let session = guard.as_ref().ok_or("no active editor")?;
    Ok(session.response())
}

/// Commits a freshly drawn annotation (geometry in base-image pixels).
#[tauri::command]
pub fn editor_add(
    state: State<'_, EditState>,
    kind: AnnotationKind,
    geometry: Geometry,
    style: Style,
) -> Result<AddResponse, String> {
    with_session(&state, |s| {
        let id = s.doc.alloc_id();
        let z = s.doc.next_z();
        let annotation = Annotation {
            id,
            kind,
            geometry,
            style,
            z,
        };
        s.history.push(&mut s.doc, Command::Add(annotation));
        s.revision += 1;
        Ok(AddResponse {
            id,
            revision: s.revision,
        })
    })
}

/// Moves/restyles an existing annotation.
#[tauri::command]
pub fn editor_update(
    state: State<'_, EditState>,
    id: AnnotationId,
    geometry: Option<Geometry>,
    style: Option<Style>,
) -> Result<DocResponse, String> {
    with_session(&state, |s| {
        let current = s.doc.get(id).ok_or("unknown_annotation")?;
        let before = (current.geometry.clone(), current.style.clone());
        let after = (
            geometry.unwrap_or_else(|| before.0.clone()),
            style.unwrap_or_else(|| before.1.clone()),
        );
        s.history
            .push(&mut s.doc, Command::Mutate { id, before, after });
        s.revision += 1;
        Ok(s.response())
    })
}

/// Deletes an annotation (Eraser / Delete).
#[tauri::command]
pub fn editor_delete(state: State<'_, EditState>, id: AnnotationId) -> Result<DocResponse, String> {
    with_session(&state, |s| {
        let annotation = s.doc.get(id).cloned().ok_or("unknown_annotation")?;
        s.history.push(&mut s.doc, Command::Remove(annotation));
        s.revision += 1;
        Ok(s.response())
    })
}

/// Undoes the last edit.
#[tauri::command]
pub fn editor_undo(state: State<'_, EditState>) -> Result<DocResponse, String> {
    with_session(&state, |s| {
        s.history.undo(&mut s.doc);
        s.revision += 1;
        Ok(s.response())
    })
}

/// Redoes the next undone edit.
#[tauri::command]
pub fn editor_redo(state: State<'_, EditState>) -> Result<DocResponse, String> {
    with_session(&state, |s| {
        s.history.redo(&mut s.doc);
        s.revision += 1;
        Ok(s.response())
    })
}

/// Clears all annotations (FR-022).
#[tauri::command]
pub fn editor_clear(state: State<'_, EditState>) -> Result<DocResponse, String> {
    with_session(&state, |s| {
        s.history.clear(&mut s.doc);
        s.revision += 1;
        Ok(s.response())
    })
}

/// Flattens the document and delivers it to Copy / Save / Pin, then closes the
/// editor (FR-023/024/025). The single output path.
#[tauri::command]
pub fn editor_export(
    app: AppHandle,
    state: State<'_, EditState>,
    pins: State<'_, PinRegistry>,
    target: String,
    format: Option<String>,
) -> Result<export::ExportResponse, String> {
    let (image, scale) = {
        let guard = state.session.lock().expect("edit session lock");
        let session = guard.as_ref().ok_or("no active editor")?;
        (
            flatten(&session.doc).map_err(|e| e.to_string())?,
            session.scale,
        )
    };

    let response = export::deliver(&app, &pins, &image, scale, &target, format.as_deref())?;

    window::close(&app);
    *state.session.lock().expect("edit session lock") = None;
    Ok(response)
}

/// Discards the editor with no side effects (Esc / cancel, FR-012).
#[tauri::command]
pub fn editor_close(app: AppHandle, state: State<'_, EditState>) {
    window::close(&app);
    *state.session.lock().expect("edit session lock") = None;
}
