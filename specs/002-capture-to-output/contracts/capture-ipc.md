# Contract: Capture Overlay IPC (UI ↔ Shell)

**Feature**: 002-capture-to-output | **Date**: 2026-06-13

The selection overlay (`ui/overlay.ts`) and the Tauri shell communicate over a
small, explicit IPC surface. Commands are Tauri `invoke` calls (UI → shell);
events are emitted shell → UI. This is the only coupling between the layers; the
UI holds no capture/geometry logic (Constitution IV).

## Events (shell → overlay)

### `capture://frame`
Emitted to each per-display overlay window right before it is shown.
```jsonc
{
  "displayId": 1,
  "width": 3840,           // physical px
  "height": 2160,
  "scaleFactor": 2.0,
  "origin": [0, 0],        // physical virtual-desktop origin
  "frameDataUrl": "data:image/png;base64,..." // frozen frame for this display
}
```
The overlay renders `frameDataUrl` full-screen and uses the metadata for
coordinate reporting. (Large frames may instead be served via a custom protocol
URL; the field name/shape stays the same — implementation note for tasks.)

## Commands (overlay → shell)

### `commit_selection`
Confirm a non-empty selection and choose an output.
```jsonc
// request
{ "rect": { "x": 100, "y": 80, "width": 640, "height": 360 }, // overlay logical px
  "displayId": 1,
  "output": "clipboard" | "file" }
// response
{ "ok": true, "output": "clipboard" }
// or
{ "ok": true, "output": "file", "path": "/Users/x/Pictures/PinShot/PinShot_2026-06-13_10-22-31.png" }
// or
{ "ok": false, "error": "permission_denied" | "empty_selection" | "clipboard_unavailable" | "write_failed", "message": "..." }
```
The shell maps `rect`+`displayId` to physical pixels via `pinshot-core::selection`,
crops via `crop`, then writes to the chosen target. `empty_selection` enforces
FR-009; errors carry an actionable `message` (FR-016).

### `copy_color`
Copy the hovered pixel color to the clipboard (US3).
```jsonc
// request  { "hex": "#1E90FF" }
// response { "ok": true }
```

### `cancel_capture`
Dismiss the overlay with no side effects (FR-004/SC-006).
```jsonc
// request  {}
// response { "ok": true }
```

## Notes

- Triggering (hotkey/tray) is **not** part of this contract — it originates in the
  shell, which freezes the screen, emits `capture://frame`, and shows the overlay.
- All payloads are local; nothing crosses the process/network boundary beyond
  Tauri IPC (FR-015).
- Coordinate authority: the overlay reports **logical** rect + `displayId`; the
  **shell/core** owns the logical→physical mapping. The UI never computes physical
  pixels.
