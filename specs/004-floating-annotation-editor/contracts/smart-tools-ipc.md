# IPC Contract: Smart Tools (UI ↔ Shell)

**Feature**: 004-floating-annotation-editor | **Date**: 2026-06-27

OCR, QR detection, color picking, and crop result surfaces. The editor webview
(`ui/src/editor/smart.ts`) calls these; the shell (`src-tauri/src/smart/`) runs
the offline `pinshot-core` logic and the platform `OcrEngine` adapter, then
returns plain data. All payloads **camelCase**; everything is **offline**
(Principle I). Extends `editor-ipc.md`.

## Commands (editor → shell)

### `run_ocr`
Extract text from the current capture, fully offline (FR-027).
- **Args**: `{ sessionId, langs?: ["en","id", …] }` — defaults to Settings language + English.
- **Effect**: shell calls the `OcrEngine` adapter (Apple Vision on macOS /
  `Windows.Media.Ocr` on Windows) on the held base RGBA.
- **Returns**:
  ```jsonc
  { "ok": true,
    "text": "Invoice #INV-2024-001\nDate: 25 May 2024\nTotal: $1,250.00",
    "lines": [ { "text": "Invoice #INV-2024-001", "rect": { "x":0,"y":0,"width":220,"height":18 } } ],
    "language": "en" }
  // or { "ok": true, "text": "", "lines": [], "language": null }   // "no text found" (graceful)
  // or { "ok": false, "error": "ocr_unavailable"|"language_unavailable", "message": "…" }
  ```

### `copy_text`
Place OCR text on the clipboard (the always-available, fully-offline action).
- **Args**: `{ text }` → `{ ok: true }`.

### `open_external` *(explicit, user-initiated hand-off — FR-028)*
For optional **Search** / **Translate** / QR **Open URL**: hand a URL/query to the
**OS default browser**. PinShot's core performs **no** network request; the OS
opens the link by the user's explicit action.
- **Args**: `{ url: string, reason: "qr"|"search"|"translate" }`
- **Returns**: `{ ok: true }`. **Errors**: `"invalid_url"`, `"open_failed"`.
- **Note**: Online **Translate** is not implemented in-app here; `translate`
  simply opens an external translator if the user enables it. Default UI hides
  networked actions unless the user turns them on (Assumptions / FR-028).

### `detect_qr`
Decode QR/barcodes offline (FR-029).
- **Args**: `{ sessionId }`
- **Effect**: `smart::qr::detect(base)` in core (`rqrr`, no network).
- **Returns**:
  ```jsonc
  { "ok": true, "codes": [
      { "value": "https://pinshot.app", "isUrl": true, "rect": { "x":40,"y":60,"width":120,"height":120 } }
  ] }
  // 0 codes → { "ok": true, "codes": [] }
  ```
- **UI**: for `isUrl` codes show **Open URL** (`open_external`) + **Copy URL**
  (`copy_text`); for non-URL show **Copy** only.

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
(Alias/companion of `crop_base` in `editor-ipc.md` for the Crop result bar.)
- **Args**: `{ sessionId, rect, aspect: "free"|"1:1"|"16:9"|"4:3" }`
- **Returns**: `{ ok: true, revision, width, height }` — see `editor-ipc.md`.

## Notes

- **No network in core** (FR-046): OCR (OS engine), QR (`rqrr`), and color are all
  on-device. Only `open_external` leaves the app — via the **OS browser**, by an
  **explicit user action**, never silently, and never sending the screenshot
  itself (only a user-chosen URL/text) (FR-028).
- **No Share** (FR-047): there is deliberately no "share image", "upload", or
  "create link" command anywhere in this surface.
- **Testability** (FR-048): QR/color/format logic is pure core (unit-tested); OCR
  sits behind the `OcrEngine` trait so OCR-consuming flows test against a fake.
- **Non-destructive**: smart results never mutate the `AnnotationDoc` (they are
  side panels); turning an OCR/QR finding into an annotation goes through
  `add_annotation` like any other object.
