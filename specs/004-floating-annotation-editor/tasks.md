# Tasks: Floating Annotation Editor & Smart Capture Toolkit

**Input**: Design documents from `/specs/004-floating-annotation-editor/`

**Prerequisites**: spec.md ✅, plan.md ✅, data-model.md ✅, contracts/ ✅ (editor-ipc, smart-tools-ipc, app-shell-ipc), quickstart.md ✅, checklists/requirements.md ✅

**Clarifications applied** (Session 2026-06-27): the floating editor **always
opens** on commit, instant in-editor C/S/P = express (Q1); **OCR removed**,
deferred to roadmap v0.3 (Q2); **QR detection kept** (offline, in-core, Q3); a
pin **keeps its editable annotation document**, flattened only on copy/save (Q4);
**crop is non-destructive** — reframes + keeps + clips-on-export, undoable (Q5).

**Tests**: Unit tests cover the new/extended `pinshot-core` logic only — the
annotation model, flatten/compositing, pixel effects, history undo/redo, QR
decode, color conversions, encode (PNG/JPG/WebP), and the settings schema. That
pure logic carries SC-002/004/007/008 (Constitution IV / FR-048). Editor/tray/
settings/pin GUI behaviour, mixed-DPI placement, perf budgets, and offline checks
are validated manually via quickstart.md (a real desktop session is required).

## Implementation status (2026-06-27)

**Done & unit-tested headless (`cargo test -p pinshot-core`, 80 tests green;
fmt + clippy `-D warnings` clean; `cargo build --workspace` green)** — the entire
pure `pinshot-core` foundation:

- T001 core deps (`rqrr`, `serde`, `toml`, image `jpeg`); T004–T009 annotation
  model, edit geometry/hit-test, **flatten** compositing, **history** undo/redo,
  encode (PNG + JPG; **WebP deferred**, see T008), lib re-exports.
- Pulled-forward pure-core pieces: T011 text rasteriser (embedded 5×7 font),
  T021 blur/pixelate, T038 spotlight/magnify + step renumber, T039 HSL/colour,
  T034 **offline QR decode** (`rqrr`, real round-trip test), T027 settings schema.

**Remaining (need the Tauri shell + a desktop session to build/validate)**: the
editor/tray/settings/pin **shell IPC and GUI** — T002–T003, T010, T012–T020,
T022–T026, T028–T033, T035–T037, T040–T045 — and polish T046–T049. These wire
the tested core to windows/commands/webviews and are validated via quickstart.md.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1 = floating editor + output, US2 = contextual props + history,
  US3 = tray + settings, US4 = QR detection, US5 = visual smart tools,
  US6 = advanced pin

---

## Phase 1: Setup (Dependencies & Build Wiring)

- [x] T001 [P] Add `rqrr` (offline QR), `serde` (+`derive`), and `toml` to `crates/pinshot-core/Cargo.toml`; keep `image` (already present) for JPG/WebP encode
- [ ] T002 [P] Add two Vite entries `ui/editor.html` and `ui/settings.html` and register them as inputs in `ui/vite.config.ts` (inputs: `index.html`, `overlay.html`, `pin.html`, `editor.html`, `settings.html`)
- [ ] T003 [P] In `src-tauri/capabilities/default.json`, add capabilities for the editor window (`editor` label) and the settings window (`settings` label), including `core:window:allow-start-dragging` for the editor toolbar drag

**Checkpoint**: Workspace resolves and builds with new deps (`cargo build --workspace`, `npm --prefix ui run build` emits the new HTML entries).

---

## Phase 2: Foundational (pure core annotation engine + scaffolding — blocks all stories)

**Purpose**: The display-independent annotation model, flatten, history, and the
editor scaffolding every user story builds on; all unit-tested headless.

- [x] T004 [P] Create `crates/pinshot-core/src/annotation/mod.rs`: `Annotation { id, kind, geometry, style, z }`, `AnnotationKind` (Rect/Ellipse/Arrow/Line/Pencil/Highlighter/Text/Blur/Pixelate/Spotlight/Magnifier/StepNumber), `Geometry` (Rect/Segment/Path/Anchor/Loupe), `Style` (stroke/fill/opacity/radius/arrow_head/dashed/text/effect/step), and `AnnotationDoc { base: CaptureImage, items }`; unit tests for construction + z-order normalisation
- [x] T005 [P] Create `crates/pinshot-core/src/annotation/geometry.rs`: `hit_test(doc, point) -> Option<id>`, `bounds(annotation) -> Rect`, and per-kind `resize`/`translate` reusing 003's `Rect` handle math; unit tests for hit-testing overlap order and resize/move normalisation
- [x] T006 [P] Create `crates/pinshot-core/src/annotation/render.rs`: `flatten(&AnnotationDoc) -> Result<CapturedImage, RenderError>` compositing vector kinds (rect/ellipse/arrow/line/pencil/highlighter) onto a copy of the base RGBA at the display scale; effect/text/step kinds are dispatched to their modules (stubs OK until T0xx); DPI-correct; unit tests on small fixtures (SC-002)
- [x] T007 [P] Create `crates/pinshot-core/src/history.rs`: `HistoryStack { commands, cursor }` and `Command` (Add/Remove/Mutate/Reorder/Crop/Renumber) with `apply`/`invert`; `undo`/`redo`/`clear`; a new command while `cursor < len` truncates the stale redo tail; unit tests incl. redo-branch truncation (SC-004)
- [x] T008 [P] Extend `crates/pinshot-core/src/encode.rs`: add `to_jpg(&CapturedImage, quality)` and `to_webp(&CapturedImage, quality)` alongside `to_png`; unit tests round-trip each format and verify dimensions _(PNG+JPG implemented & tested; WebP deferred — returns `EncodeError::Unsupported` pending a dedicated encoder dependency, plan D6)_
- [x] T009 Re-export `annotation`, `history`, and the new `encode` items from `crates/pinshot-core/src/lib.rs`
- [ ] T010 Create the shell editor module skeleton: `src-tauri/src/editor/mod.rs` (the `EditSession { doc, history, tool, props, selected }` state + module decls) and `src-tauri/src/editor/window.rs` (create the borderless floating editor webview); declare `mod editor;` and `manage` the editor state in `src-tauri/src/lib.rs`

**Checkpoint**: `cargo test -p pinshot-core` passes; shell compiles with the editor skeleton.

---

## Phase 3: User Story 1 — Annotate a capture and copy/save/pin it (Priority: P1) 🎯 MVP

**Goal**: After a selection commit the floating editor **always** opens; draw
core annotations (rect/ellipse/arrow/line/pencil/highlighter/text) on a
horizontal floating toolbar, then Copy/Save/Pin the flattened result.

**Independent Test**: Capture → editor opens with a horizontal toolbar + action
bar (no sidebar) → draw a rectangle + arrow + text → press C and paste; the image
matches the on-screen flatten exactly; Save writes a file; Pin floats the result;
Esc cancels with no side effects (quickstart §US1).

- [x] T011 [P] [US1] Create `crates/pinshot-core/src/annotation/text.rs`: `rasterize(&TextStyle, &mut buffer, at)` drawing text (font/size/weight/color/background/shadow) into the RGBA at flatten time; wire it into `render::flatten`; unit test renders a known glyph box (font choice per plan D5)
- [ ] T012 [US1] In `src-tauri/src/editor/mod.rs`: on **commit** (capture handoff) `crop_region` the frozen selection (reuse 002) into a `CaptureImage`, seed an `EditSession`, and **always** open the editor window (Q1); change `src-tauri/src/capture/mod.rs` `commit_selection` to route to the editor instead of direct output
- [ ] T013 [US1] Implement the core editor IPC commands in `src-tauri/src/editor/mod.rs` per `contracts/editor-ipc.md`: `set_tool`, `add_annotation`, `update_annotation`, `delete_annotation`, `get_doc`, and `close_editor`; register them in `src-tauri/src/lib.rs`
- [ ] T014 [US1] Implement `src-tauri/src/editor/export.rs`: `export({target, format})` → `annotation::render::flatten` → clipboard (reuse 002 `output::copy_image`) / file (reuse `encode`+naming) / pin (reuse 003 `create_pin`, **passing the editable doc** per Q4); close the editor on success; wire the `export` command
- [ ] T015 [US1] Emit `editor://load` to the editor window on open (image data URL/custom-protocol, `width/height/scaleFactor`, seeded `defaults`, `theme`) per `contracts/editor-ipc.md`
- [ ] T016 [US1] Create `ui/editor.html` + `ui/src/editor/editor.ts`: bootstrap, receive `editor://load`, wire IPC, and install the keyboard map (tool single-keys; C/S/P actions; Esc → `close_editor`)
- [ ] T017 [US1] Create `ui/src/editor/toolbar.ts`: the **horizontal floating toolbar** (Select/Rect/Ellipse/Arrow/Line/Pencil/Highlighter/Text/…), draggable, keyboard-navigable — **no left/right sidebar** (FR-009)
- [ ] T018 [US1] Create `ui/src/editor/canvas.ts`: render the base image at logical size; live-draw rect/ellipse/arrow/line/pencil/highlighter/text; **Shift** constrains, **scroll** changes thickness (FR-014); select/move existing objects (hit-test preview); push intents via `add_annotation`/`update_annotation`
- [ ] T019 [US1] Create `ui/src/editor/actionbar.ts`: the floating action bar **Pin · Copy · Save · More** (FR-011) — **no Share** (FR-047) — calling `export({target})`
- [ ] T020 [US1] Verify: `cargo build --workspace` + `cargo test -p pinshot-core` green; manual quickstart §US1 (editor opens, draw, C/S/P, Esc no-op, no sidebar, no Share)

**Checkpoint**: MVP — capture → floating annotation editor → copy/save/pin works offline on both platforms.

---

## Phase 4: User Story 2 — Contextual tool properties, full toolset & history (Priority: P2)

**Goal**: Selecting a tool expands the toolbar to **only** that tool's
properties; add blur/pixelate/eraser; restyle existing objects live; infinite
undo/redo with a history panel.

**Independent Test**: Select Rectangle → only its props show; restyle a drawn
shape live; add 5 annotations → history lists them; Undo ~50/Redo exact;
Clear History returns the unannotated capture (quickstart §US2).

- [x] T021 [P] [US2] Create `crates/pinshot-core/src/annotation/effects.rs`: `gaussian_blur(base, region, strength)` and `pixelate(base, region, block)` reading the **original** base pixels (lossless re-edit, Edge Case); wire into `render::flatten`; unit tests on fixtures
- [ ] T022 [US2] Add the contextual-properties + history IPC to `src-tauri/src/editor/mod.rs` per `contracts/editor-ipc.md`: `set_tool_props`, `reorder_annotation`, `undo`, `redo`, `clear_history` (driving `history.rs`); register in `src-tauri/src/lib.rs`
- [ ] T023 [US2] In `ui/src/editor/toolbar.ts`: add the **ContextualProperties** panel that shows ONLY the active tool's controls (Rect: stroke/thickness/fill/opacity/radius; Arrow: color/thickness/arrowhead/dashed; Text: font/size/weight/bg/shadow; Blur: strength; Pixelate: block; Highlighter: opacity) and collapses on tool change (FR-017/FR-018)
- [ ] T024 [US2] In `ui/src/editor/canvas.ts`: wire Blur/Pixelate/Eraser tools; restyling a selected object calls `update_annotation` live and sets the new value as the default for the next object of that kind (FR-019)
- [ ] T025 [US2] Create `ui/src/editor/history.ts`: the HistoryPanel listing annotations in creation order with type icons, plus Undo/Redo and **Clear History** (FR-020/FR-022)
- [ ] T026 [US2] Verify: `cargo test -p pinshot-core` green; manual quickstart §US2 (only-active-tool props, live restyle, history list, ~50 undo/redo, clear)

**Checkpoint**: The editor is precise and repeatable; full toolset + history shippable.

---

## Phase 5: User Story 3 — Background app, menu bar / tray & Settings (Priority: P2)

**Goal**: Launch with **no window** (tray/menu-bar only); the menu opens
Capture/Settings/About/Check-Updates/Quit; Settings is the only normal window,
with recordable hotkeys + conflict detection and persisted local settings.

**Independent Test**: Launch → no window/Dock-taskbar entry, only a tray icon →
menu items present → Settings opens a window → record a new Capture hotkey
(conflict flagged) → toggle theme; settings persist across restart (quickstart §US3).

- [x] T027 [P] [US3] Create `crates/pinshot-core/src/settings.rs`: the `Settings` schema (general/capture/hotkeys/annotation/export/advanced) + `defaults`, `validate`, `from_toml`, `to_toml`, and `ExportProfile`/`Hotkey`/`Theme`/`CaptureMode` types; unit tests for defaults + round-trip + corrupt-file fallback; re-export from `lib.rs`
- [ ] T028 [US3] Create/extend `src-tauri/src/tray.rs`: native tray/menu-bar menu (**Capture · Settings · About · Check for Updates · Quit**) and set the process to accessory/agent so **no main window** appears (macOS `ActivationPolicy::Accessory`; Windows tray + skip-taskbar) (FR-001/FR-002)
- [ ] T029 [US3] Create `src-tauri/src/settings/mod.rs`: load/save `Settings.toml` in `dirs::config_dir()/PinShot/` via `core::settings`, apply side effects (theme broadcast `editor://theme`, launch-at-login, reseed editor `ToolProperties`); fall back to defaults on missing/corrupt file (FR-045)
- [ ] T030 [US3] Create `src-tauri/src/settings/hotkeys.rs`: register/remap global hotkeys via `global-hotkey` and detect conflicts (PinShot-own + detectable OS-reserved) (FR-041)
- [ ] T031 [US3] Implement the app-shell IPC in `src-tauri/src/settings/mod.rs` per `contracts/app-shell-ipc.md`: `get_settings`, `set_settings`, `reset_settings`, `record_hotkey`, `open_logs`, and the isolated opt-in `check_updates`; register in `src-tauri/src/lib.rs`
- [ ] T032 [US3] Create `ui/settings.html` + `ui/src/settings.ts`: tabbed Settings (General/Capture/Hotkeys/Annotation/Export/Advanced) with a **HotkeyRecorder** (record → conflict warn → save) (FR-039–FR-044)
- [ ] T033 [US3] Verify: `cargo test -p pinshot-core` green; manual quickstart §US3 (no window on launch, menu, settings window, record hotkey + conflict, theme persists, offline load)

**Checkpoint**: PinShot feels like a native background utility; all defaults/hotkeys are user-configurable.

---

## Phase 6: User Story 4 — Smart code capture: QR detection (Priority: P3)

**Goal**: Detect QR/barcodes **offline** in the capture and offer Copy URL /
Open URL (explicit OS-browser hand-off); plus the color picker plumbing.

**Independent Test**: Capture a QR → Copy URL / Open URL with the correct decoded
value; multiple codes → choice; none → graceful; zero network observed (quickstart §US4).

- [x] T034 [P] [US4] Create `crates/pinshot-core/src/smart/mod.rs` (`SmartResult`, `QrResult`, `ColorSample`) and `crates/pinshot-core/src/smart/qr.rs`: `detect(&CaptureImage) -> QrResult` via `rqrr` (offline); unit tests decoding a fixture QR (0/1/many codes); re-export from `lib.rs`
- [ ] T035 [US4] Create `src-tauri/src/smart/mod.rs`: IPC `detect_qr`, `copy_text`, and `open_external({url, reason:"qr"})` per `contracts/smart-tools-ipc.md` — `open_external` is the only outbound, explicit, OS-browser hand-off (FR-029); register in `src-tauri/src/lib.rs`
- [ ] T036 [US4] Create `ui/src/editor/smart.ts` (QR chip): show detected value(s) with **Copy URL** (`copy_text`) and, for URLs, **Open URL** (`open_external`); surface from the editor on open/edit
- [ ] T037 [US4] Verify: `cargo test -p pinshot-core` green; manual quickstart §US4 (QR copy/open, multi/none, **zero network** via monitor)

**Checkpoint**: Offline QR detection works end-to-end; no OCR, no network.

---

## Phase 7: User Story 5 — Visual smart tools: spotlight, magnifier, step numbers, color, crop (Priority: P3)

**Goal**: Step numbers (auto-increment), spotlight (dim outside), magnifier
(circular loupe), color picker (HEX/RGB/HSL), and non-destructive crop
(Free/1:1/16:9/4:3).

**Independent Test**: Place 3 step numbers (auto-increment, renumber on delete);
spotlight dims outside; magnifier zooms; color shows HEX/RGB/HSL copyable; crop to
16:9 keeps annotations correctly placed; each is undoable (quickstart §US5).

- [x] T038 [P] [US5] Extend `crates/pinshot-core/src/annotation/effects.rs` with `spotlight(base, region, dim)` and `magnify(base, center, radius, zoom)`, and create `crates/pinshot-core/src/annotation/step.rs` (`next_index`, `renumber`); wire into `render::flatten`; unit tests
- [x] T039 [P] [US5] Extend `crates/pinshot-core/src/color.rs` with `rgb_to_hsl`, `hsl_to_rgb`, `format_rgb`, `format_hsl` (alongside existing `pixel_rgb`/`pixel_hex`); unit tests for known conversions; re-export from `lib.rs`
- [ ] T040 [US5] Add `pick_color` and `crop`/`crop_base` IPC: `pick_color` in `src-tauri/src/smart/mod.rs` (color via core), `crop_base` in `src-tauri/src/editor/mod.rs` — **non-destructive** reframe that keeps all items and clips-on-export, pushing a reversible `Crop` command (Q5); register in `src-tauri/src/lib.rs`
- [ ] T041 [US5] In `ui/src/editor/` (canvas/toolbar/smart): wire the Spotlight, Magnifier, Step Number, Color Picker (HEX/RGB/HSL copy) tools, and the Crop bar (Free/1:1/16:9/4:3)
- [ ] T042 [US5] Verify: `cargo test -p pinshot-core` green; manual quickstart §US5 (step renumber, spotlight, magnifier, color copy, crop keeps annotations, all undoable)

**Checkpoint**: The documentation-grade smart tools are complete.

---

## Phase 8: User Story 6 — Advanced floating pin (Priority: P4)

**Goal**: Pin opacity, click-through (with guaranteed escape), resize/zoom, and
annotate-after-pin — building on the editable-doc pin (Q4) and feature 003.

**Independent Test**: Pin → lower opacity (still on top) → click-through (clicks
pass; escape regains control) → resize crisp on any DPI → annotate the pin and
copy/save includes it (quickstart §US6).

- [ ] T043 [US6] Extend `src-tauri/src/capture/pin.rs`: per-pin `set_opacity`, `set_click_through` (with an always-available disable path, e.g. a hotkey/tray toggle), and `set_scale`/resize; the pin already carries the editable `AnnotationDoc` (Q4); register the commands in `src-tauri/src/lib.rs`
- [ ] T044 [US6] Extend `ui/src/pin.ts`: opacity slider, click-through toggle (+ escape affordance), resize handles, and annotate-on-pin (reuse the editor canvas/flatten); copy/save from the pin flattens its doc (FR-035–FR-038)
- [ ] T045 [US6] Verify: build + manual quickstart §US6 (opacity, click-through escape, resize crisp, annotate-then-copy includes annotations) — incl. **mixed-DPI** (Constitution III)

**Checkpoint**: The namesake pin is a full reference surface; all six stories functional.

---

## Phase 9: Polish & Cross-Cutting

- [ ] T046 [P] Instrument and verify the performance budgets (NFR-001…007 / SC-009): startup <300ms, capture overlay <100ms, 60fps interactions, idle memory <120MB, idle CPU ≈0 — log timestamps on the trigger→show and flatten→output paths
- [ ] T047 [P] Update `README.md`: document the floating editor + tools, the tray menu, the Settings window, QR detection, advanced pins, and the **offline / no-Share** guarantee
- [ ] T048 Full gate: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `npm --prefix ui run lint && npm --prefix ui run format:check && npm --prefix ui run build` all green
- [ ] T049 Run quickstart.md end-to-end on **both** macOS and Windows (core tests automated; editor/tray/settings/pin GUI + mixed-DPI + **offline** monitor checks manual) and record SC-001…SC-011 status

---

## Dependencies & Execution Order

- **Phase 1 → Phase 2 → user stories**: deps/build wiring, then the pure annotation engine + editor scaffolding.
- **US1 (Phase 3)** is the MVP — depends on Phase 2 (annotation model, flatten, encode) and reuses 002 `crop_region` + 003 `create_pin`.
- **US2 (Phase 4)** depends on US1 (the editor + canvas) and Phase 2 history; adds effects + contextual props + history panel.
- **US3 (Phase 5)** is largely independent (tray + settings shell) but its hotkeys drive the global capture and its `annotation` defaults seed the editor — best after US1/US2; depends on Phase 2 only for `CaptureImage`.
- **US4 (Phase 6)** depends on Phase 2 + the editor (US1) to surface QR results.
- **US5 (Phase 7)** depends on US1/US2 (canvas + effects) and US4's smart IPC module.
- **US6 (Phase 8)** depends on US1 (editor canvas/flatten + create_pin) and 003's pin window.
- **Polish (Phase 9)** after the stories.
- **Parallel**: T001/T002/T003; T004/T005/T006/T007/T008; T011; T021; T027; T034; T038/T039; T046/T047 are independent files.

## Implementation Strategy

MVP first: Phases 1–3 (US1 — the floating editor + copy/save/pin) → validate →
US2 (contextual props + history) → US3 (tray + settings) → US4 (QR) → US5 (visual
tools) → US6 (advanced pin) → polish. The `pinshot-core` annotation/flatten/
effects/history/QR/color/settings logic is fully unit-tested at each step
(Constitution IV); editor/tray/settings/pin behaviour, mixed-DPI, perf, and
offline are validated by running the app per quickstart.md. Commit after each
story; land via PR to branch-protected `main`.
