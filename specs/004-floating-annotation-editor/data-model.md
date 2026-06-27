# Phase 1 Data Model: Floating Annotation Editor & Smart Capture Toolkit

**Feature**: 004-floating-annotation-editor | **Date**: 2026-06-27

These are mostly in-memory domain types. The only persisted entity is
[Settings](#settings) (a local `Settings.toml`); outputs are PNG/JPG/WebP files.
Types that carry geometry / pixel / encoding / schema logic live in
`pinshot-core` (headless, unit-tested); the shell owns windows, tray, hotkeys,
clipboard, files, and the OCR adapter. Coordinates that cross IPC are **logical**
(CSS) px; physical/DPI math stays in core/shell (reusing 002/003).

## Entity Map

```
EditSession 1───1 AnnotationDoc 1───* Annotation ──has──► Style
     │                  │
     │                  └──base 1───1 CaptureImage (from 002)
     │
     ├──1───1 HistoryStack 1───* Command
     ├──1───1 ToolProperties (per-kind defaults)
     └──produces──► SmartResult { OcrResult | QrResult | ColorSample }

Settings 1───* Hotkey         Settings 1───1 ExportProfile
PinRegistry 1───* Pin (extends 003) ──may own──► AnnotationDoc
```

---

## Core entities (in `pinshot-core`)

### CaptureImage *(reused from 002)*
Immutable base layer for an edit.
- `width, height: u32` — physical pixels.
- `rgba: Vec<u8>` — `width*height*4`.
- `scale_factor: f64` — owning display scale (logical = physical / scale).
- `source_png: Option<Vec<u8>>` — original encoded bytes when available (003).
- Validation: `rgba.len() == width*height*4`; `scale_factor > 0`.

### AnnotationDoc
The editable overlay on top of a `CaptureImage`.
- `base: CaptureImage` — never mutated by drawing (effects read from it).
- `items: Vec<Annotation>` — z-ordered low→high.
- Invariant: `items` ids are unique; z equals index after normalization.
- Produces output only via `render::flatten(&AnnotationDoc) -> CapturedImage`.

### Annotation
One editable object.
- `id: u64` — stable within the session.
- `kind: AnnotationKind` — discriminant (below).
- `geometry: Geometry` — in **logical capture coords**:
  - `Rect { rect: Rect }` (Rectangle, Ellipse, Blur, Pixelate, Spotlight, Crop region)
  - `Segment { a: Point, b: Point }` (Arrow, Line)
  - `Path { points: Vec<Point> }` (Pencil, Highlighter)
  - `Anchor { at: Point }` (Text, StepNumber)
  - `Loupe { center: Point, radius: f32 }` (Magnifier)
- `style: Style` — visual params (below).
- `z: u32` — stacking order (FR-016).
- Validation: geometry normalized & clamped (reuse 003 `Rect::clamp_*`); paths
  non-empty.

### AnnotationKind
```
Rect · Ellipse · Arrow · Line · Pencil · Highlighter · Text
· Blur · Pixelate · Spotlight · Magnifier · StepNumber
```
(`Select`, `ColorPicker`, `Crop`, `Eraser` are **tools/gestures**, not stored
object kinds — Crop reframes `base`; Eraser deletes objects; Select/ColorPicker
don't create objects.)

### Style
Superset of per-kind visual properties (only the relevant subset applies per
kind; mirrors the contextual properties in spec FR-018).
- `stroke_color: Rgba`, `stroke_width: f32`, `dashed: bool`
- `fill_enabled: bool`, `fill_color: Rgba`
- `opacity: f32` (0–1)
- `corner_radius: f32` (Rectangle)
- `arrow_head: ArrowHead { None|Open|Filled }` (Arrow/Line)
- `text: TextStyle { content, font, size, weight, color, background: Option<Rgba>, shadow: bool }`
- `effect: EffectParams { blur_strength: f32, pixelate_block: u32, spotlight_dim: f32, magnifier_zoom: f32 }`
- `step: StepStyle { index: u32, color: Rgba }`
- Validation: `0 ≤ opacity ≤ 1`; `stroke_width > 0`; `pixelate_block ≥ 1`;
  `magnifier_zoom > 1`.

### ToolProperties
Current default `Style` per `AnnotationKind` + active tool — seeded from
[Settings](#settings) Annotation defaults; updated when the user changes a
property (FR-019). New objects of a kind are created with that kind's defaults.

### HistoryStack
Unlimited in-session undo/redo (FR-021) + the history panel source (FR-020).
- `commands: Vec<Command>` — applied log.
- `cursor: usize` — index of the last *applied* command.
- `undo()` → move cursor back & invert; `redo()` → re-apply.
- A new command issued while `cursor < len` **truncates** the stale redo tail
  (Edge Case: orphaned redo branch).
- `clear()` → empties to the unannotated base (FR-022).

### Command
A reversible edit (the unit of undo/redo).
- Variants: `AddAnnotation(Annotation)`, `RemoveAnnotation(id, snapshot)`,
  `MutateAnnotation(id, before: Style|Geometry, after)`, `Reorder(id, from, to)`,
  `Crop(before_base_ref, after_rect)`, `Renumber(before, after)`.
- Each variant defines `apply` and `invert` on the `AnnotationDoc`.

### SmartResult
Non-destructive output of a smart tool (US4/US5); not part of the doc.
- `OcrResult { text: String, lines: Vec<OcrLine{ text, rect }> , language: Option<String> }`
- `QrResult { codes: Vec<QrCode{ value: String, rect: Rect, is_url: bool }> }`
- `ColorSample { rgb: (u8,u8,u8), hex: String, hsl: (u16,u8,u8) }`
- All produced offline; `OcrResult` from the shell `OcrEngine` adapter, the rest
  pure in core.

### OcrEngine *(port/trait in core; impls in shell)*
```
trait OcrEngine { fn recognize(&self, img: &CaptureImage, langs: &[Lang])
                    -> Result<OcrResult, OcrError>; }
```
- macOS adapter: Apple Vision (`objc2`); Windows adapter: `Windows.Media.Ocr`.
- A `FakeOcrEngine` enables headless tests of OCR-consuming logic.

### Settings *(persisted — the only stored entity)*
Local `Settings.toml`; schema + defaults + validation in `core::settings`.
- `general: { launch_at_login: bool=false, check_updates: bool=false,
   theme: Theme{Light|Dark|System}=System, language: Lang=En }`
- `capture: { mode: CaptureMode{Region|Window|FullScreen}=Region,
   delay_secs: u8=0, include_cursor: bool=false, include_shadow: bool=true }`
- `hotkeys: Vec<Hotkey>` — see below.
- `annotation: { stroke_color, fill_color, font, font_size, arrow_size,
   highlighter_opacity, blur_strength, pixelate_size }` (seeds `ToolProperties`).
- `export: ExportProfile` — see below.
- `advanced: { developer_mode: bool=false }`
- Invariant: a missing/corrupt file → in-memory defaults + rewrite valid file
  (FR-045); load never touches the network (Principle I).

### Hotkey
- `action: ActionId` (e.g., `CaptureRegion`, `CaptureWindow`, `CaptureFullScreen`,
  `Pin`, `Ocr`, `ToggleAllPins`, plus editor tool/action ids).
- `chord: KeyChord { mods, key }`.
- `scope: Scope{ Global | Editor }`.
- `conflict: Option<ConflictKind>` — set by the recorder when the chord clashes
  with another PinShot binding or a detectable OS-reserved one (FR-041).

### ExportProfile
Used by Copy/Save (FR-043).
- `format: ImageFormat{ Png | Jpg | Webp }=Png`
- `filename_pattern: String` (extends 002 timestamp naming, e.g.
  `"PinShot_{date}_{time}"`).
- `compression: u8` (quality for Jpg/Webp; ignored for Png).
- `clipboard: ClipboardBehavior{ Image | ImageAndFile | FileOnly }=Image`.

### Pin *(extends 003)*
Floating image window state (shell registry).
- `id, png, width, height, scale_factor, display_origin` (003) **plus**
  `opacity: f32=1.0`, `click_through: bool=false`, `scale: f32=1.0`,
  `doc: Option<AnnotationDoc>` (annotate-after-pin, US6/FR-038).
- Copy/Save from a pin flattens `doc` (if any) over its image.

---

## Shell-owned state

- `EditSession` (one at a time): `{ doc: AnnotationDoc, history: HistoryStack,
  tool: ToolId, props: ToolProperties, selected: Option<u64> }` — the live edit.
- `PinRegistry`: `Mutex<HashMap<PinId, Pin>>` + next-id (003, extended).
- `SettingsStore`: loaded `Settings` + path + dirty flag; applies side effects
  (launch-at-login, hotkey registration, theme broadcast).
- `HotkeyManager`: registered global chords ↔ `ActionId` (re-registered on
  settings change).

---

## State Machines *(deliverable 14)*

### App lifecycle
```
            init(tray, hotkeys, hidden overlay windows; NO editor/settings window)
Launched ─────────────────────────────────────────────────────────► TrayIdle
TrayIdle  ──Capture(hotkey|menu)──► Capturing ──frames ready──► Selecting   (002/003)
Selecting ──Edit commit (non-empty)──► Editing
Selecting ──express Copy/Save/Pin──► Output ──► TrayIdle
Selecting ──Esc / zero-area──► TrayIdle            (clipboard/FS unchanged)
TrayIdle  ──Settings──► SettingsOpen ──save/close──► TrayIdle
TrayIdle  ──Check Updates (explicit)──► fetch static version file ──► TrayIdle
Editing   ──Copy/Save/Pin──► Output ──► TrayIdle (+ Pin stays floating)
Editing   ──Esc──► TrayIdle                        (discard; nothing written)
TrayIdle  ──Quit──► close all pins ──► Exit
```
- A capture trigger received while not `TrayIdle` is ignored (no second overlay/
  editor), preserving 002's single-overlay rule.

### Editor session
```
EditorOpen(base loaded)
  ├─ select tool ─────► ToolActive(props panel shows this tool only)        (FR-017)
  │     ├─ draw ──────► AddAnnotation → history.push → ToolActive            (FR-015,020)
  │     └─ Shift/scroll ─► constrain / thickness (live)                      (FR-014)
  ├─ select object ───► ObjectSelected
  │     ├─ restyle ───► MutateAnnotation(style)  (live + sets default)       (FR-019)
  │     ├─ move/resize ► MutateAnnotation(geometry)                          (FR-015)
  │     └─ delete/Eraser ► RemoveAnnotation                                   (FR-013)
  ├─ Undo/Redo ───────► history cursor ∓1; canvas re-render                  (FR-021)
  ├─ Clear History ───► history.clear(); base only                          (FR-022)
  ├─ OCR/QR/Color ────► SmartResult shown (non-destructive)                 (FR-027-030)
  ├─ Crop commit ─────► reframe base; reposition/clip items; history.push   (FR-034)
  ├─ Copy/Save/Pin ───► flatten(core) → Output; close editor                (FR-023-025)
  └─ Esc ─────────────► discard; TrayIdle (clipboard/FS unchanged)          (FR-012)
```

### History cursor (undo/redo)
```
   apply C1   apply C2   apply C3
[]────────►[C1]────────►[C1 C2]────────►[C1 C2 C3]   cursor=2 (0-based last applied)
                                   undo │
                              [C1 C2]·C3  cursor=1   (C3 invertible via redo)
                          new C4 │ (cursor<len ⇒ truncate C3)
                       [C1 C2 C4]         cursor=2
```

---

## Core API surface (new/extended in `pinshot-core`)

Pure, no I/O, unit-tested headless (FR-048):

- `annotation::{Annotation, AnnotationKind, Geometry, Style, AnnotationDoc}`.
- `annotation::geometry::{hit_test, bounds, resize, translate}`.
- `annotation::render::flatten(&AnnotationDoc) -> Result<CapturedImage, RenderError>`.
- `annotation::effects::{gaussian_blur, pixelate, spotlight, magnify}` (region + base → buffer).
- `annotation::text::rasterize(&TextStyle, &mut buffer, at)`.
- `annotation::step::{next_index, renumber}`.
- `history::{HistoryStack, Command}` with `apply`/`invert`.
- `smart::qr::detect(&CaptureImage) -> QrResult` (offline, `rqrr`).
- `smart::ocr::{OcrEngine, OcrResult, OcrError, Lang}` (trait + types; impls in shell).
- `color::{rgb_to_hsl, hsl_to_rgb, format_rgb, format_hsl}` (extends existing `pixel_hex`/`pixel_rgb`).
- `encode::{to_png, to_jpg, to_webp}` (extends existing `to_png`).
- `naming::output_filename(pattern, now, existing)` (extends existing).
- `settings::{Settings, Theme, Lang, CaptureMode, ExportProfile, Hotkey, defaults, validate, from_toml, to_toml}`.

The shell provides `ScreenCapturer` (xcap, 002) and `OcrEngine` (Vision/Windows
OCR) implementations and performs all clipboard/file/window/tray side effects.
