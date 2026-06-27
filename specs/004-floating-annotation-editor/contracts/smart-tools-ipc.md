# IPC Contract: Smart Tools (UI ↔ Shell)

**Feature**: 004-floating-annotation-editor | **Date**: 2026-06-27

QR detection, color picking, and crop result surfaces. The editor webview
(`ui/src/editor/smart.ts`) calls these; the shell (`src-tauri/src/smart/`) runs
the offline `pinshot-core` logic and returns plain data. All payloads
**camelCase**; everything is **offline** (Principle I).

> **OCR is out of scope for this feature** (deferred to roadmap v0.3). There is
> deliberately **no** `run_ocr` / text-extraction / Search / Translate command in
> this surface.

Extends `editor-ipc.md`.

## Commands (editor → shell)

### `detect_qr`
Decode QR/barcodes offline (FR-027/FR-028).
- **Args**: `{ sessionId }`
- **Effect**: `smart::qr::detect(base)` in core (`rqrr`, no network).
- **Returns**:
  ```jsonc
  { "ok": true, "codes": [
      { "value": "https://pinshot.app", "isUrl": true, "rect": { "x":40,"y":60,"width":120,"height":120 } }
  ] }
  // 0 codes → { "ok": true, "codes": [] }   // "no code found" (graceful)
  ```
- **UI**: for each code show **Copy** (`copy_text`); for `isUrl` codes also show
  **Open URL** (`open_external`).

### `copy_text`
Place a decoded value (e.g. a QR URL) or a formatted color string on the
clipboard — the always-available, fully-offline action.
- **Args**: `{ text }` → `{ ok: true }`.

### `open_external` *(explicit, user-initiated hand-off — FR-029)*
For QR **Open URL**: hand a URL to the **OS default browser**. PinShot's core
performs **no** network request; the OS opens the link by the user's explicit
action, and the screenshot itself is never transmitted.
- **Args**: `{ url: string, reason: "qr" }`
- **Returns**: `{ ok: true }`. **Errors**: `"invalid_url"`, `"open_failed"`.

### `pick_color`
Read a pixel's color and format it (FR-030).
- **Args**: `{ sessionId, x: u32, y: u32 }` — logical coords; shell maps to physical.
- **Effect**: `color::pixel_rgb` + `rgb_to_hsl` + formatters in core.
- **Returns**:
  ```jsonc
  { "ok": true, "hex": "#4F46E5", "rgb": [79,70,229], "hsl": [243,76,59] }
  ```
- **UI**: each of HEX / RGB / HSL is independently copyable via `copy_text`.

### `crop`
(Companion of `crop_base` in `editor-ipc.md` for the Crop result bar.)
- **Args**: `{ sessionId, rect, aspect: "free"|"1:1"|"16:9"|"4:3" }`
- **Effect**: non-destructive reframe — keeps all annotations, clips (not deletes)
  those outside the frame on export (Q5).
- **Returns**: `{ ok: true, revision, width, height }` — see `editor-ipc.md`.

## Notes

- **No network in core** (FR-046): QR (`rqrr`) and color are fully on-device.
  Only `open_external` leaves the app — via the **OS browser**, by an **explicit
  user action**, never silently, and never sending the screenshot (only a
  user-chosen URL) (FR-029).
- **No Share** (FR-047): there is deliberately no "share image", "upload", or
  "create link" command anywhere in this surface.
- **No OCR** (clarified): no text recognition, Search, or Translate command ships
  here; those are roadmap items.
- **Testability** (FR-048): QR/color/format logic is pure core and unit-tested
  headless, no platform engine required.
- **Non-destructive**: QR/color results never mutate the `AnnotationDoc` (they are
  side panels); turning a QR finding into an annotation goes through
  `add_annotation` like any other object.
