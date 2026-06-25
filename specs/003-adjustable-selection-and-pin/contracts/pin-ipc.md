# IPC Contract: Pin & Adjustable Selection

UI (overlay/pin webviews) ↔ Tauri shell. Extends 002's `capture-ipc.md`; only the
additions are listed. All payloads are camelCase. Logical (CSS) coordinates cross
the boundary; physical/DPI math stays in the shell/core.

## Commands (UI → shell)

### `create_pin`
Turn the current selection into a floating pin.
- **Args**: `{ displayId: u32, rect: { x: i32, y: i32, width: u32, height: u32 } }`
  — `rect` is the **adjusted** logical selection on that display.
- **Effect**: `to_physical(rect)` → `crop_region` → store a `PinnedImage` in the
  `PinRegistry`; open a borderless always-on-top pin window sized/placed by
  `pin::pin_logical_size` / `pin::pin_placement`; then close the capture overlay
  and clear the session.
- **Returns**: `{ pinId: u32 }`. **Errors**: `"empty_selection"`, `"no active capture"`, crop/encode errors as strings.

### `get_pin_image`
The pin page pulls its image on load.
- **Args**: `{ pinId: u32 }`
- **Returns**: `{ dataUrl: string, width: u32, height: u32, scaleFactor: f64 }`
  — `dataUrl` reuses `PinnedImage.png` (from `source_png` when available).
- **Errors**: `"unknown pin"`.

### `close_pin`
- **Args**: `{ pinId: u32 }` — close that pin's window and drop it from the registry. Idempotent.

### `raise_pin`
- **Args**: `{ pinId: u32 }` — bring the pin to the front (`set_focus`) on interaction.

### `copy_pin`
- **Args**: `{ pinId: u32 }` — place the pin's original image on the clipboard (reuse `output::copy_image`). **Errors**: `"unknown pin"`, clipboard errors.

### `commit_selection` (unchanged from 002)
Now always carries the adjusted rect; no signature change.

## Events (shell → UI)

None new. Pins are pulled, not pushed; lifecycle is driven by the commands above.

## Notes

- **Window labels**: pin windows use label `pin-<pinId>`; the shell maps label ↔ id for close/raise.
- **Drag**: movement uses the webview's `getCurrentWindow().startDragging()` (no position IPC).
- **Close gesture**: `pin.ts` calls `close_pin` on double-click (per clarification).
- **Z-order**: every pin is `always_on_top`; `raise_pin` orders them among themselves.
