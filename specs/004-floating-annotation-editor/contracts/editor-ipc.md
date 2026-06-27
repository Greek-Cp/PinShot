# IPC Contract: Floating Editor (UI ↔ Shell)

**Feature**: 004-floating-annotation-editor | **Date**: 2026-06-27

The editor webview (`ui/src/editor/*`) and the Tauri shell (`src-tauri/src/editor/`)
communicate over a small, explicit IPC surface. Commands are Tauri `invoke` calls
(UI → shell); events are emitted shell → UI. This is the **only** coupling between
the layers — the UI holds no annotation/flatten/geometry logic; it renders a live
preview and forwards intents, and the shell calls `pinshot-core` for the
authoritative result (Constitution IV). All payloads are **camelCase** and carry
**logical** (CSS) coordinates; physical/DPI math stays in core/shell. Extends
002's `capture-ipc.md` and 003's `pin-ipc.md`; only additions are listed.

## Events (shell → editor)

### `editor://load`
Emitted to the editor window right after it is created, seeding the base image.
```jsonc
{
  "sessionId": 7,
  "captureId": 7,
  "width": 1280,          // physical px of the captured region
  "height": 720,
  "scaleFactor": 2.0,     // logical = physical / scaleFactor
  "imageDataUrl": "data:image/png;base64,...",  // or a custom-protocol URL for large frames
  "defaults": { /* ToolProperties seeded from Settings.annotation */ },
  "theme": "system" | "light" | "dark"
}
```
The editor renders `imageDataUrl` at logical size and uses `defaults` to
initialize each tool's properties.

### `editor://theme`
Broadcast when the OS/user theme changes (Settings or system) so the editor
restyles without a restart.
```jsonc
{ "theme": "light" | "dark" }
```

## Commands (editor → shell)

> The shell keeps the authoritative `EditSession` (doc + history + tool + props).
> The UI mirrors it for preview and sends intents. Each mutating command returns
> the new `revision` (monotonic) so the UI can detect drift; on mismatch the UI
> calls `get_doc` to resync.

### `set_tool`
Activate a tool; the contextual properties panel reflects this (FR-017).
- **Args**: `{ sessionId, tool: "select"|"rect"|"ellipse"|"arrow"|"line"|"pencil"|"highlighter"|"text"|"blur"|"pixelate"|"spotlight"|"magnifier"|"colorPicker"|"stepNumber"|"crop"|"eraser" }`
- **Returns**: `{ ok: true, props: ToolProperties }`

### `set_tool_props`
Change the active tool's default style (and the selected object, if any) — live
(FR-018/FR-019).
- **Args**: `{ sessionId, tool, props: Partial<Style> }`
- **Returns**: `{ ok: true, revision }`

### `add_annotation`
Commit a freshly drawn object (after live preview).
- **Args**: `{ sessionId, kind, geometry, style }` — logical coords.
- **Effect**: shell builds an `Annotation`, pushes `AddAnnotation` to history.
- **Returns**: `{ ok: true, id, revision }`

### `update_annotation`
Move / resize / restyle an existing object.
- **Args**: `{ sessionId, id, geometry?: Geometry, style?: Partial<Style> }`
- **Effect**: pushes `MutateAnnotation`.
- **Returns**: `{ ok: true, revision }`. **Errors**: `"unknown_annotation"`.

### `delete_annotation`
Remove an object (Eraser / Delete).
- **Args**: `{ sessionId, id }` — pushes `RemoveAnnotation` (renumbers steps if needed).
- **Returns**: `{ ok: true, revision }`

### `reorder_annotation`
Change z-order (FR-016).
- **Args**: `{ sessionId, id, to: u32 }` → `{ ok: true, revision }`

### `undo` / `redo`
Unlimited within the session (FR-021).
- **Args**: `{ sessionId }`
- **Returns**: `{ ok: true, revision, canUndo: bool, canRedo: bool }`

### `clear_history`
Remove all annotations; return to the unannotated base (FR-022).
- **Args**: `{ sessionId }` → `{ ok: true, revision }`

### `get_doc`
Resync the UI from the authoritative doc (after a revision mismatch).
- **Args**: `{ sessionId }`
- **Returns**: `{ ok: true, revision, items: Annotation[], canUndo, canRedo }`

### `crop_base`
Re-frame the capture (Crop tool, FR-034).
- **Args**: `{ sessionId, rect, aspect: "free"|"1:1"|"16:9"|"4:3" }`
- **Effect**: reframe `base` (reuse `crop_region`), reposition/clip items, push `Crop`.
- **Returns**: `{ ok: true, revision, width, height }`

### `export`
Flatten in core and emit the result (FR-023/024/025). The single output path for
Copy / Save / Pin.
- **Args**: `{ sessionId, target: "clipboard"|"file"|"pin", format?: "png"|"jpg"|"webp" }`
- **Effect**: `annotation::render::flatten(doc)` → `encode` → target:
  - `clipboard` → `arboard` image (reuse 002 `output::copy_image`).
  - `file` → write via `ExportProfile` (format/pattern/compression).
  - `pin` → create a pin (reuse 003 `create_pin`) from the flattened image; if
    US6 live-annotate is enabled, attach the doc instead of pre-flattening.
- **Returns**:
  ```jsonc
  { "ok": true, "target": "file", "path": "/…/PinShot_2026-06-27_17-40-02.png" }
  // or { "ok": true, "target": "clipboard" }
  // or { "ok": true, "target": "pin", "pinId": 3 }
  // or { "ok": false, "error": "empty_doc"|"encode_failed"|"write_failed"|"clipboard_unavailable", "message": "…" }
  ```
  On success for `clipboard`/`file`/`pin`, the shell closes the editor window
  (pins keep floating).

### `close_editor`
Esc / cancel — discard the session with no side effects (FR-012).
- **Args**: `{ sessionId }` → `{ ok: true }` (clipboard & filesystem unchanged).

## Notes

- **Coordinate authority**: the editor reports **logical** geometry; the shell/
  core own logical→physical mapping for flatten/effects (SC-002). The UI never
  computes physical pixels.
- **Preview vs. truth**: blur/pixelate/spotlight/magnifier shown in the canvas are
  cheap previews; the bytes from `export` are the authoritative core flatten (D1).
- **Offline**: every command is local Tauri IPC; nothing crosses the network
  (FR-046). There is **no** share/upload command in this surface (FR-047).
- **Revision/resync** keeps the UI thin and the shell authoritative without
  streaming the whole doc on every keystroke.
