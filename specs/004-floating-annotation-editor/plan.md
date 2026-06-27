# Implementation Plan: Floating Annotation Editor & Smart Capture Toolkit

**Branch**: `004-floating-annotation-editor` | **Date**: 2026-06-27 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-floating-annotation-editor/spec.md`

## Summary

Turn PinShot's capture pipeline (002) and floating pin (003) into the full
**"float shot"** experience: after a selection is committed, a **floating,
horizontal, keyboard-first annotation editor** opens over the frozen capture —
a tool toolbar that expands into **contextual properties**, an infinite
**undo/redo history**, a **floating action bar** (Pin / Copy / Save / OCR /
More — **no Share**), **smart tools** (OCR, QR, color picker, spotlight,
magnifier, step numbers, crop), a **background tray / menu-bar app with no main
window**, and a dedicated **Settings window**.

The architectural crux is **Constitution IV**: the **annotation model**,
**flatten/compositing**, **pixel effects** (blur / pixelate / spotlight /
magnifier), **QR decode**, **color math**, **crop**, and **PNG/JPG/WebP encode**
all live as **pure functions in `pinshot-core`**, unit-tested headless; the
TypeScript canvas only renders a live preview and forwards intents; the Tauri
shell owns windows, tray, hotkeys, settings I/O, and the **platform OCR adapter**
behind a trait. This keeps the privacy guarantee auditable in one place, keeps a
future Linux port feasible, and lets the riskiest logic (DPI-correct flatten,
pixel effects) be tested without a GUI.

We reuse 002's frozen-frame + `crop_region` and 003's pin window model unchanged;
the only genuinely new machinery is the annotation document + history, the smart
tools, the app shell (tray/settings), and the export pipeline — delivered in the
prioritized slices US1→US6.

## Technical Context

**Language/Version**: Rust 1.96.0 (pinned), TypeScript (Node 20+).

**Primary Dependencies**:
- Existing/reused: Tauri 2.x (windows, tray, IPC, global-shortcut), `xcap`
  (capture), `arboard` (clipboard image), `image` (RGBA→PNG/JPG/WebP), `base64`,
  `global-hotkey`, `dirs`.
- **New, local-only, license-compatible (MIT/Apache-2.0)**:
  - `rqrr` **or** `bardecoder` — pure-Rust **offline** QR decode (core).
  - `imageproc` / `fast_image_resize` — fast blur / pixelate / resize kernels
    for the effect ops (core) [only if `image`'s built-ins are insufficient].
  - `serde` + `toml` (or reuse `serde_json`) — settings (de)serialization.
  - Fonts: bundle an open-licensed UI font for text annotation rendering, or use
    platform text APIs in the webview for preview and `ab_glyph`/`rusttype` in
    core for flatten — **decision deferred to Phase 0 D5**.
- **OCR** is **not** a crate: macOS **Apple Vision** via FFI/`objc2`; Windows
  **`Windows.Media.Ocr`** via the `windows` crate — both OS-provided, offline.
- **No new network-capable dependency.** Every addition is verified against
  Principle I (no network) and license compatibility before adoption.

**Storage**: One local, human-readable **settings file** (`Settings.toml`) in the
OS config dir (`dirs::config_dir()/PinShot/`); in-memory `EditSession`,
`HistoryStack`, and `PinRegistry`; output PNG/JPG/WebP files in the configured
folder. No database, no remote storage.

**Testing**: `cargo test -p pinshot-core` for the annotation model, flatten,
pixel effects, QR decode, color conversions, crop, encode, naming, and settings
schema — all **headless**. Shell adapters (OCR, clipboard, windows) are covered
by manual `quickstart.md` scenarios on both OSes; OCR has a thin trait so core
logic around it stays testable with a fake engine.

**Target Platform**: macOS and Windows desktop (Tauri); extends the 001 Cargo
workspace + Vite frontend.

**Project Type**: Desktop app (Tauri) — existing workspace.

**Performance Goals** (from spec NFRs): startup < 300 ms (NFR-001), capture
overlay < 100 ms (NFR-002), 60 fps interactions (NFR-003), idle memory < 120 MB
(NFR-004), idle CPU ≈ 0 (NFR-007).

**Constraints**: zero network anywhere in core (FR-046/SC-010); **no Share**
affordance (FR-047/SC-010); DPI-exact flatten/effects/pin across mixed-DPI
(FR-049/SC-002); all heavy logic testable headless (FR-048); macOS/Windows parity
(FR-049); no code signing.

**Scale/Scope**: Single user, local. One `EditSession` at a time; history of tens
to hundreds of annotations; 1–N pins; 1–4 displays.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Constitution v1.1.0**

| Principle | Gate | Status |
|---|---|---|
| I. Privacy-First & Offline-Only | Capture, annotate, flatten, OCR (Vision/Windows OCR), QR (offline `rqrr`), color, crop, encode, clipboard, file, and pin are **all local**. No new network-capable dependency. Update check stays isolated/opt-in/off. Search/Translate are explicit OS hand-offs, never core requests (FR-028). **No Share** anywhere (FR-047). | ✅ PASS |
| II. Performance Is a Feature | Editor opens over the already-frozen 002 frame (no recapture); flatten/encode run in core off the UI thread; smart tools operate on the held RGBA. Budgets in NFR-001…007 are release gates measured in `quickstart.md`. | ✅ PASS |
| III. Cross-Platform Parity | Identical editor/tools/shell on both OSes; mixed-DPI flatten/effect/pin is a mandatory test (SC-002). The only platform-divergent piece — OCR — sits behind one trait with two `#[cfg]` adapters. | ✅ PASS |
| IV. Clean Architecture: Core as a Library | Annotation model, flatten, pixel effects, QR, color, crop, encode, settings schema = pure `pinshot-core`. Shell owns windows/tray/hotkeys/IPC/OCR-adapter/clipboard/files. UI renders + forwards intents only. New feature defines the core API surface first, then shell/UI. | ✅ PASS |
| V. Strict Scope & Simplicity | Scope = the floating editor + smart tools + shell + settings, sliced US1→US6 in roadmap order (annotation → shell → OCR/QR → polish → advanced pin). No accounts, sync, telemetry, screen recording, or online AI. New deps are each the documented vetted choice or the simplest local option; justified below. | ✅ PASS |
| VI. Maintainability & Contributor Experience | Same toolchain/commands as 001–003; new core modules get doc comments and carry the heaviest unit tests (flatten/effects/QR/DPI math are the risk). Each new crate/module has one responsibility; IPC additions documented in `contracts/`. | ✅ PASS |

**No violations.** Complexity Tracking notes the few judgment calls (see
[Complexity Tracking](#complexity-tracking)); each is the *simpler correct*
option, not gratuitous complexity.

## Project Structure

### Documentation (this feature)

```text
specs/004-floating-annotation-editor/
├── spec.md                 # Feature spec (done)
├── plan.md                 # This file
├── data-model.md           # Phase 1: entities, annotation model, state machines
├── contracts/
│   ├── editor-ipc.md       # UI ↔ shell: editor lifecycle, annotations, output
│   ├── smart-tools-ipc.md  # UI ↔ shell: OCR, QR, color, crop
│   └── app-shell-ipc.md    # UI ↔ shell: tray, settings, hotkeys
├── quickstart.md           # Phase 1: validation incl. offline, mixed-DPI, perf
├── checklists/
│   └── requirements.md     # Spec quality checklist
└── tasks.md                # Phase 2 (/speckit.tasks — not produced here)
```

### Source Code (repository root) — additions to the existing workspace

```text
crates/pinshot-core/src/
├── geometry.rs          # (exists) Rect + handle/resize/clamp helpers (003) — reuse
├── capture.rs           # (exists) Display, FrozenFrame, ScreenCapturer — reuse
├── crop.rs              # (exists) crop_region — reuse (also for Crop tool)
├── encode.rs            # (exists) PNG — EXTEND: JPG + WebP + quality/compression
├── color.rs             # (exists) pixel_rgb/hex — EXTEND: rgb↔hsl, format RGB/HSL
├── naming.rs            # (exists) output filename — EXTEND: configurable pattern
├── pin.rs               # (exists) pin size/placement — EXTEND: scale/opacity math
├── selection.rs         # (exists) logical↔physical — reuse
├── annotation/          # NEW: the annotation engine (pure)
│   ├── mod.rs           #   Annotation, AnnotationKind, Style, AnnotationDoc
│   ├── geometry.rs      #   hit-test, bounds, resize/move per kind
│   ├── render.rs        #   flatten: composite annotations onto base RGBA
│   ├── effects.rs       #   blur (gaussian), pixelate, spotlight, magnifier
│   ├── text.rs          #   text layout + rasterize for flatten
│   └── step.rs          #   step-number sequencing/renumber
├── history.rs           # NEW: HistoryStack (undo/redo command log) — pure
├── smart/               # NEW: offline smart tools (pure, OCR behind a trait)
│   ├── mod.rs           #   SmartResult, OcrResult, QrResult, ColorSample
│   ├── qr.rs            #   QR/barcode decode via rqrr (offline)
│   └── ocr.rs           #   OcrEngine trait + types (impls live in the shell)
├── settings.rs          # NEW: Settings schema + serde + defaults + validation
└── lib.rs               # re-exports for all the above

src-tauri/src/
├── lib.rs               # builder: tray + hotkeys + windows + state (extend)
├── capture/             # (exists, 002/003)
│   ├── mod.rs           #   capture flow (reuse) → now can hand off to editor
│   ├── overlay.rs       #   (exists) selection overlay — reuse
│   ├── pin.rs           #   (exists) pin windows — EXTEND: opacity/click-through/resize/annotate
│   ├── output.rs        #   (exists) clipboard + file — EXTEND: JPG/WebP via encode
│   └── xcap_capturer.rs #   (exists) capture adapter — reuse
├── editor/              # NEW: shell side of the editor
│   ├── mod.rs           #   open/close editor window; EditSession state; commands
│   ├── window.rs        #   create the floating editor webview(s)
│   └── export.rs        #   flatten via core → clipboard/file/pin
├── smart/               # NEW: shell side of smart tools
│   ├── mod.rs           #   IPC commands: run_ocr / detect_qr / pick_color / crop
│   └── ocr/             #   OcrEngine impls behind the core trait
│       ├── macos.rs     #     Apple Vision (objc2)  [cfg(target_os="macos")]
│       └── windows.rs   #     Windows.Media.Ocr     [cfg(target_os="windows")]
├── settings/            # NEW: shell side of settings
│   ├── mod.rs           #   load/save Settings.toml; apply (theme/launch/hotkeys)
│   └── hotkeys.rs       #   register/remap global hotkeys + conflict detection
├── tray.rs              # NEW/EXTEND: native tray/menu-bar menu (Capture/Settings/About/Updates/Quit)
└── main.rs              # (exists)

ui/
├── overlay.html         # (exists, 002/003) — reuse
├── pin.html             # (exists, 003) — EXTEND: opacity/click-through/resize/annotate
├── editor.html          # NEW: the floating editor page (toolbar + canvas + action bar)
├── settings.html        # NEW: the Settings window page
└── src/
    ├── overlay.ts       # (exists) — reuse
    ├── pin.ts           # (exists) — EXTEND (US6)
    ├── editor/          # NEW: editor UI (renders state, forwards intents)
    │   ├── editor.ts    #   bootstrap, IPC wiring, keyboard map
    │   ├── toolbar.ts   #   FloatingToolbar + ContextualProperties
    │   ├── canvas.ts    #   Canvas render + live draw + hit-test preview
    │   ├── history.ts   #   HistoryPanel view
    │   ├── actionbar.ts #   Pin/Copy/Save/OCR/More (no Share)
    │   └── smart.ts     #   OCR/QR/Color/Crop result surfaces
    └── settings.ts      # NEW: Settings UI (tabs, hotkey recorder)
```

**Structure Decision**: Extend the existing workspace in place. All pure logic
concentrates under two new `pinshot-core` areas — `annotation/` (the engine) and
`smart/` (OCR trait + offline QR) — plus `history.rs` and `settings.rs`, so the
DPI-correct flatten, pixel effects, QR, and undo logic that drive SC-002/SC-004/
SC-007/SC-008 are unit-tested headless. All side effects (editor/settings/pin
windows, tray, hotkeys, clipboard, files, OCR FFI) live in `src-tauri/src/`
behind the core's ports. The editor and settings are new Vite entries
(`editor.html`, `settings.html`) shipping in the same bundle, mirroring how
`overlay.html` (002) and `pin.html` (003) were added.

## Module Boundaries *(deliverable 18)*

```
┌──────────────────────────────────────────────────────────────────────┐
│ UI (TypeScript webviews)  — render state, forward intents, no logic    │
│   editor/ (toolbar, canvas, history, actionbar, smart) · settings.ts   │
│   · pin.ts · overlay.ts                                                │
└───────────────▲───────────────────────────────────────────┬──────────┘
                │ Tauri IPC (commands/events, camelCase)      │ intents
                │ contracts/{editor,smart-tools,app-shell}-ipc.md
┌───────────────┴───────────────────────────────────────────▼──────────┐
│ Tauri shell (src-tauri)  — windows, tray, hotkeys, IPC, platform I/O   │
│   editor/ · smart/ (incl. ocr/{macos,windows}) · settings/ · tray ·    │
│   capture/ (reused) · pin (reused/extended)                            │
└───────────────▲───────────────────────────────────────────┬──────────┘
                │ calls pure core APIs        implements ports│ (ScreenCapturer,
                │                                              │  OcrEngine)
┌───────────────┴───────────────────────────────────────────▼──────────┐
│ pinshot-core (headless, no GUI, NO NETWORK)                            │
│   annotation/ (model · render/flatten · effects · text · step) ·       │
│   history · smart/ (qr · ocr trait · color) · settings schema ·        │
│   crop · encode (png/jpg/webp) · geometry · selection · pin math       │
└───────────────▲───────────────────────────────────────────┬──────────┘
                │ trait impls (#[cfg])                        │
┌───────────────┴───────────────────────────────────────────▼──────────┐
│ Platform adapters (in shell, behind core traits)                       │
│   xcap capture · Apple Vision OCR (macOS) · Windows.Media.Ocr (Win) ·  │
│   arboard clipboard · Tauri windows                                    │
└────────────────────────────────────────────────────────────────────────┘
```

**The dependency rule holds**: inner layers never import outer ones. `OcrEngine`
is the one platform-divergent port; the UI imports neither core nor adapters —
it speaks only the documented IPC surface.

## Annotation Engine Design *(deliverable 9)*

**Goal**: an editable object model that previews live in the webview canvas but
flattens deterministically (and DPI-correctly) in core for output.

- **Model**: `AnnotationDoc { base: CaptureImage, items: Vec<Annotation> }`.
  Each `Annotation { id, kind, geometry, style, z }` where `kind ∈ {Rect,
  Ellipse, Arrow, Line, Pencil, Highlighter, Text, Blur, Pixelate, Spotlight,
  Magnifier, StepNumber}`. Geometry is in **logical capture coordinates**;
  flatten maps to physical pixels via the capture's scale (reuse `selection`/
  `geometry`).
- **Two-phase rendering** (decision D1):
  1. **Live preview** in `ui/editor/canvas.ts` (HTML canvas / SVG) for 60 fps
     interaction — vector shapes drawn directly; blur/pixelate/spotlight shown
     with a cheap CSS/canvas approximation for *preview only*.
  2. **Authoritative flatten** in `core::annotation::render::flatten(doc) ->
     CapturedImage` on **Copy/Save/Pin**, compositing every item onto a copy of
     the base RGBA. Effects (`effects.rs`) read the **original** base pixels
     (never already-effected output) so re-editing a blur region is lossless
     (Edge Case). This is the pixel-exact result the user gets.
- **Effects** (`effects.rs`, pure): `gaussian_blur(region, strength)`,
  `pixelate(region, block)`, `spotlight(region, dim)`, `magnify(center, radius,
  zoom)` — each takes the base RGBA + params and returns the modified buffer;
  unit-tested on small fixtures.
- **Text** (`text.rs`): layout + rasterize a string with font/size/weight/color/
  background/shadow into the buffer at flatten time; preview uses the webview's
  native text rendering, flatten uses an in-core rasterizer for parity (font
  choice = D5).
- **Step numbers** (`step.rs`): maintain a stable 1..n sequence; deleting a
  marker renumbers the rest (FR-033).
- **Hit-testing & editing** (`geometry.rs`): `hit_test(doc, point) -> Option<id>`
  and per-kind resize/move reuse 003's `Rect` handle math; this is pure so
  selection/restyle logic is testable.

**Why core-owned flatten**: the privacy-critical, DPI-critical, cross-platform
output is computed once, in tested Rust — the webview can render fast and
approximate, but never decides the final pixels (Principle IV + SC-002).

## Capture Engine Design *(deliverable 10)*

Reuse 002 wholesale; this feature only changes the **handoff target**.

- **Trigger → freeze → overlay → select/adjust** is unchanged (002 + 003). At
  trigger, all monitors are frozen once into RGBA `FrozenFrame`s; overlay windows
  (pre-created, hidden at startup) show instantly (< 100 ms, NFR-002).
- **New handoff**: on **Edit commit**, instead of going straight to clipboard/
  file (002) or a pin (003), the shell crops the frozen region via
  `crop_region` (reused, DPI-exact) into a `CaptureImage` and opens the **editor
  window** seeded with that image as the `AnnotationDoc.base`. The express path
  (direct Copy/Save/Pin without editing) remains.
- **Capture modes** (FR-040: Region / Window / Full Screen): Region exists (002);
  Window/Full-Screen feed the same `CaptureImage` into the same editor. Delay
  timer, include-cursor, include-shadow are capture-time options applied before
  freezing (shell/adapter concern; cursor/shadow are platform flags on `xcap`/OS
  capture).
- **No recapture for editing/effects**: every annotation, effect, magnifier, and
  color read operates on the already-held base RGBA — fast and offline.

## Menu Bar / System Tray Design *(deliverable 11)*

- **Native tray** via Tauri `tray-icon` (already a dependency, 002). The icon is
  the **only** persistent UI; the app is an **accessory/agent** process:
  - macOS: `ActivationPolicy::Accessory` (no Dock icon, lives in the menu bar).
  - Windows: no taskbar window; a system-tray icon (hidden main window / `skip
    taskbar`).
- **Menu** (FR-002): `Capture` · `Settings` · `About` · `Check for Updates` ·
  `Quit`. Built as a **native menu** (not a webview) for instant, native feel
  (NFR-005). `Capture` triggers the capture flow; `Settings` opens/foregrounds
  the Settings window; `Check for Updates` is opt-in and only fetches a static
  version file **when clicked** (Principle I); `About` shows a small native/about
  dialog; `Quit` closes pins and exits.
- **Startup** (NFR-001): tray + hotkeys are registered during a minimal init that
  pre-creates the hidden overlay windows (002) but **no** editor/settings window —
  keeping startup < 300 ms and idle memory < 120 MB.

## Settings Window Design *(deliverable 12)*

- **One normal window** (`settings.html`), opened only from the tray (FR-003).
  Tabs (a settings window may use a tab rail — this is not the editor):
  - **General** (FR-039): Launch at Login, Check for Updates (opt-in/off), Theme
    (Light/Dark/System), Language (EN/ID initial).
  - **Capture** (FR-040): Region/Window/Full-Screen, Delay Timer, Include Cursor,
    Include Shadow.
  - **Hotkeys** (FR-041): a row per action with a **HotkeyRecorder** — click →
    "recording" → press chord → captured; conflict detection against PinShot's
    own bindings and (where detectable) OS-reserved ones; warn before save.
  - **Annotation** (FR-042): Stroke Color, Fill Color, Font, Font Size, Arrow
    Size, Highlighter Opacity, Blur Strength, Pixelate Size — these seed the
    editor's per-tool defaults (`ToolProperties`).
  - **Export** (FR-043): Default Format (PNG/JPG/WebP), Filename pattern,
    Clipboard behavior, Compression.
  - **Advanced** (FR-044): Developer Mode, Logs (open log file/folder), Reset
    Settings.
- **Persistence** (FR-045): `core::settings` defines the schema + defaults +
  validation; the shell `settings/mod.rs` (de)serializes `Settings.toml` in the
  config dir, applies side effects (register launch-at-login, re-register
  hotkeys, broadcast theme), and falls back to defaults on missing/corrupt file.
  No network.

## State Machine *(deliverable 14 — summary; full diagrams in [data-model.md](./data-model.md))*

**App lifecycle:**
```
Launched ──init(tray,hotkeys,hidden overlays)──► TrayIdle
TrayIdle ──Capture (hotkey/menu)──► Capturing ──► Selecting (002/003)
Selecting ──Edit commit──► Editing
Selecting ──express Copy/Save/Pin──► Output ──► TrayIdle
TrayIdle ──Settings──► SettingsOpen ──close──► TrayIdle
TrayIdle ──Quit──► (close pins) ──► Exit
```

**Editor session:**
```
EditorOpen ──select tool──► ToolActive(props shown)
ToolActive ──draw──► add Annotation (push history) ──► ToolActive
ToolActive ──select object──► ObjectSelected(restyle/move/resize/delete)
any ──Undo/Redo──► history cursor moves (canvas re-renders)
any ──OCR/QR/Color──► SmartResult shown (non-destructive)
any ──Crop commit──► base reframed (push history)
EditorOpen ──Copy/Save/Pin──► flatten(core) → Output ──► close editor
EditorOpen ──Esc──► discard ──► TrayIdle (clipboard/FS unchanged)
```

## Sequence Diagrams *(deliverable 15)*

**A. Capture → annotate → copy (US1):**
```
User        Shell(tray/hotkey)     Overlay(UI)     core            Editor(UI)
 │ ⌘⇧A ───────►│ freeze all frames                                   
 │             │ show overlay ─────►│ (002)                            
 │ drag/adjust ───────────────────►│ select+adjust (002/003)          
 │ Edit ───────────────────────────│ commit_selection(rect,EDIT)      
 │             │ crop_region(frame,rect)──►│ CaptureImage              
 │             │ open editor window ───────────────────────►│ load base
 │ pick Rect, draw ───────────────────────────────────────►│ add Annotation (preview)
 │ press C ────────────────────────────────────────────────│ copy()
 │             │◄── flatten(doc) ──── core ◄────────────────│ invoke export
 │             │ arboard.set_image(flattened)                          
 │             │ close editor ─────────────────────────────►│          
```

**B. OCR (US4):**
```
User    Editor(UI)        Shell(smart)        OcrEngine(adapter)   core
 │ OCR ──►│ run_ocr(captureId)──►│ ocr.recognize(rgba)──►│ (Vision/WinOCR, offline)
 │        │◄──────── OcrResult{text,regions} ◄───────────│
 │ Copy Text ──►│ (clipboard text)  — no network at any step
```

**C. Pin with later annotation (US6):**
```
User   Editor   Shell(pin)        core            Pin(UI)
 │ P ──►│ create_pin(flatten or live doc)──►│ pin size/placement
 │      │ open pin window ─────────────────────────────►│ render
 │ annotate on pin ─────────────────────────────────────│ add Annotation
 │ copy from pin ──► flatten(pin.doc) ─► clipboard (local)
```

## Phase 0 — Research / Key Decisions

- **D1 — Preview in webview, flatten in core.** Canvas renders fast vector
  previews + cheap effect approximations; `core::annotation::render::flatten`
  produces the authoritative, DPI-exact output on export. Rationale: 60 fps
  interactivity (NFR-003) **and** tested, privacy-/DPI-correct pixels (SC-002,
  Principle IV). Alternative (render everything in core per frame) rejected —
  too slow for live drawing. Alternative (trust webview pixels for output)
  rejected — not headless-testable, DPI-fragile.
- **D2 — Editor is a Tauri webview window over the capture.** One editor window
  per capture session, borderless, seeded via IPC. Reuses the 002/003 window
  pattern; simplest model with native feel.
- **D3 — OCR behind a single `OcrEngine` trait in core.** Two `#[cfg]` adapters
  in the shell (Apple Vision via `objc2`; `Windows.Media.Ocr` via `windows`).
  Keeps Principle III/IV; core logic uses a fake engine in tests.
- **D4 — QR decode in core with `rqrr` (offline).** Pure Rust, MIT/Apache,
  no network. `bardecoder` is the fallback if multi-format barcodes are needed.
- **D5 — Text flatten font.** Bundle one open-licensed font (e.g., Inter/DejaVu)
  and rasterize with `ab_glyph` in core for cross-platform-identical text output;
  preview uses the webview's native text. (Confirm license + size in tasks.)
- **D6 — Effects via `image` first, `imageproc`/`fast_image_resize` only if
  needed.** Start with `image`'s blur/resize; add a kernel crate only if perf on
  large captures misses NFR-003/004. Justify any addition (Principle V).
- **D7 — Settings = `serde` + `toml`, schema in core.** Human-readable, testable
  defaults/validation in `core::settings`; shell does I/O + side effects.
- **D8 — Accessory/agent process for "no main window".** macOS
  `ActivationPolicy::Accessory`; Windows no-taskbar + tray. Native tray menu
  (not webview) for startup speed and native feel (NFR-001/005).
- **D9 — Reuse 002 frozen frame + `crop_region` for capture, editing base, the
  Crop tool, and color reads.** No recapture; everything operates on held RGBA.
- **D10 — Search/Translate are OS hand-offs, default-hidden where networked.**
  Core never calls out; "Open URL"/"Search" use the OS opener by explicit user
  action (FR-028). Online translate is deferred; offline translate is a later
  roadmap item.

## Phase 1 — Data Model & Contracts (pointers)

- **Data model** → [data-model.md](./data-model.md): `EditSession`,
  `AnnotationDoc`, `Annotation`/`AnnotationKind`, `Style`, `ToolProperties`,
  `HistoryStack`/`Command`, `SmartResult` (`OcrResult`/`QrResult`/`ColorSample`),
  `Settings`, `Hotkey`, `ExportProfile`, extended `Pin`; plus the app + editor
  state machines.
- **Contracts**:
  - [contracts/editor-ipc.md](./contracts/editor-ipc.md) — open/close editor,
    add/update/delete annotation, undo/redo, flatten→copy/save/pin.
  - [contracts/smart-tools-ipc.md](./contracts/smart-tools-ipc.md) — run_ocr,
    detect_qr, pick_color, crop.
  - [contracts/app-shell-ipc.md](./contracts/app-shell-ipc.md) — tray actions,
    settings get/set, hotkey record/register + conflict.

## Complexity Tracking

*No constitution violations.* Judgment calls (each the simpler correct option,
not added complexity):

| Decision | Why it's simplest-correct | Rejected alternative |
|---|---|---|
| Preview in webview, **flatten in core** (D1) | Only way to get both 60 fps and tested DPI-exact offline output | All-core render (too slow) / trust-webview pixels (not testable, DPI-fragile) |
| New `pinshot-core` sub-modules `annotation/`, `smart/` | One responsibility each; keeps the heaviest logic headless-tested | Putting flatten/effects/QR in the shell (un-testable, violates IV) |
| One `OcrEngine` trait, two `#[cfg]` adapters (D3) | Isolates the single platform-divergent path | Scattering `#[cfg]` through business logic |
| Adding `rqrr` + `serde`/`toml` (+ maybe a kernel/font crate) | Each is local-only, license-compatible, and the simplest offline option | Hand-rolling QR/blur/serialization |
