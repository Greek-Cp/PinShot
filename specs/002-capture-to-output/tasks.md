# Tasks: Capture Area to Clipboard & File

**Input**: Design documents from `/specs/002-capture-to-output/`

**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅, quickstart.md ✅

**Tests**: Unit tests are included for the `pinshot-core` modules only — the
project constitution makes the core testable-headless (FR-017) and the feature's
correctness (SC-002/004/005) lives in that pure logic. GUI/integration behavior
is validated manually via quickstart.md (a real desktop session is required).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1 = capture→clipboard, US2 = save to PNG, US3 = selection aids

## Phase 1: Setup (Dependencies & Build Wiring)

- [x] T001 [P] Add `image` to `crates/pinshot-core/Cargo.toml` dependencies
- [x] T002 [P] Add `xcap`, `global-hotkey`, `arboard`, `dirs`, `chrono` to `src-tauri/Cargo.toml` dependencies
- [x] T003 [P] Add a second Vite entry `ui/overlay.html` and configure multi-page build in `ui/vite.config.ts` (inputs: `index.html`, `overlay.html`)

**Checkpoint**: Workspace resolves and builds with new deps (`cargo build --workspace`, `npm run build`).

---

## Phase 2: Foundational (Core capture logic — blocks all stories)

**Purpose**: The pure, headless geometry/capture types every user story needs.

- [x] T004 [P] Extend `crates/pinshot-core/src/capture.rs`: add `Display { id, origin, size, scale_factor }` and `FrozenFrame { display_id, width, height, rgba }`; extend `ScreenCapturer` with `capture_all() -> Result<(Vec<Display>, Vec<FrozenFrame>), CaptureError>`; add unit tests with a fake capturer
- [x] T005 [P] Create `crates/pinshot-core/src/selection.rs`: `to_physical(logical: Rect, display: &Display) -> Rect` and `displays_for(region: Rect, displays: &[Display]) -> Vec<u32>`; unit tests across scale factors 1.0/1.5/2.0 and spanning displays
- [x] T006 [P] Create `crates/pinshot-core/src/crop.rs`: `crop_region(frames, displays, region) -> Result<CapturedImage, CropError>` including cross-display composite; unit tests (single-display crop, spanning crop, out-of-bounds, empty region)
- [x] T007 Wire `selection`, `crop`, and the new `capture` items into `crates/pinshot-core/src/lib.rs` re-exports
- [x] T008 Create `src-tauri/src/capture/mod.rs` with the `CaptureSession`/`OverlayState` types and module declarations; declare `mod capture;` in `src-tauri/src/lib.rs`

**Checkpoint**: `cargo test -p pinshot-core` passes; shell compiles with the capture module skeleton.

---

## Phase 3: User Story 1 — Capture region to clipboard (Priority: P1) 🎯 MVP

**Goal**: Hotkey/tray → multi-monitor selection overlay → selected pixels on the clipboard.

**Independent Test**: Press hotkey, drag a region, paste into an editor → matches exactly (quickstart §US1).

- [x] T009 [US1] Implement `src-tauri/src/capture/xcap_capturer.rs`: `ScreenCapturer` over `xcap` producing `Display` (origin/size/scale) and `FrozenFrame` (RGBA) for every monitor
- [x] T010 [US1] Implement `src-tauri/src/capture/overlay.rs`: pre-create one hidden, borderless, always-on-top fullscreen overlay window per display at startup; `show_with_frames` (emit `capture://frame`) and `hide_all`
- [x] T011 [US1] Implement `src-tauri/src/capture/hotkey.rs`: register default hotkey (`CmdOrCtrl+Shift+A`) via `global-hotkey`; on press, start capture; ignore if a session is already active (FR-018)
- [x] T012 [US1] Implement `src-tauri/src/capture/tray.rs`: Tauri tray icon with **Capture** and **Quit** items
- [x] T013 [US1] Implement clipboard output in `src-tauri/src/capture/output.rs`: `copy_image(&CapturedImage)` via `arboard`
- [x] T014 [US1] Orchestrate in `src-tauri/src/capture/mod.rs`: trigger → `capture_all` → store `CaptureSession` → `show_with_frames`; Tauri commands `commit_selection` (map via `selection`, `crop`, then clipboard) and `cancel_capture`; register hotkey+tray+overlays and commands in `src-tauri/src/lib.rs`
- [x] T015 [US1] Create `ui/overlay.html` + `ui/src/overlay.ts`: render the frozen frame fullscreen, drag a selection rectangle, dim outside selection, Esc/right-click → `cancel_capture`, release on non-empty → `commit_selection({output:"clipboard"})`
- [ ] T016 [US1] Validate: `cargo build --workspace` + `cargo test -p pinshot-core` green; manual quickstart §US1 (hotkey→select→paste, tray trigger, Esc no-op)

**Checkpoint**: MVP — minimal offline region-capture-to-clipboard tool works on both platforms.

---

## Phase 4: User Story 2 — Save a capture to PNG (Priority: P2)

**Goal**: Save the selected region as a PNG to a default folder with an auto filename.

**Independent Test**: Select → save → a valid PNG appears in `Pictures/PinShot/` matching the region (quickstart §US2).

- [x] T017 [P] [US2] Create `crates/pinshot-core/src/encode.rs`: `to_png(&CapturedImage) -> Result<Vec<u8>, EncodeError>` using `image`; unit test encodes then decodes to verify dimensions/pixels
- [x] T018 [P] [US2] Create `crates/pinshot-core/src/naming.rs`: `output_filename(y,mo,d,h,mi,s, existing: &[String]) -> String` → `PinShot_YYYY-MM-DD_HH-MM-SS[_N].png`; unit tests for format + same-second collision suffix; re-export in `lib.rs`
- [x] T019 [US2] Add file output to `src-tauri/src/capture/output.rs`: resolve `dirs::picture_dir()/PinShot` (create if missing), format timestamp via `chrono`, call `naming` + `encode`, write file; map missing/read-only dir to an actionable error
- [x] T020 [US2] Extend `commit_selection` in `mod.rs` for `output:"file"` (return saved path) and add a save trigger (key/action) in `ui/src/overlay.ts`
- [ ] T021 [US2] Validate: `cargo test -p pinshot-core` green; manual quickstart §US2 (save, open PNG, double-save no collision, read-only error)

**Checkpoint**: Capture can be saved to disk as well as copied.

---

## Phase 5: User Story 3 — Magnifier, dimensions, color (Priority: P3)

**Goal**: On-overlay precision aids reading the frozen frame.

**Independent Test**: During selection, magnifier/W×H/color match ground truth; copy-color works (quickstart §US3).

- [x] T022 [P] [US3] Create `crates/pinshot-core/src/color.rs`: `pixel_hex(rgba: &[u8], width, x, y) -> String` and `pixel_rgb(...)`; unit tests; re-export in `lib.rs`
- [x] T023 [US3] In `ui/src/overlay.ts`: render a magnifier canvas (zoomed crop of the frozen frame around the cursor) with the cursor coordinate, and a live width×height readout bound to the drag
- [x] T024 [US3] In `ui/src/overlay.ts` + `mod.rs`: show the cursor pixel color (HEX/RGB) and add a copy-color key wired to a `copy_color` command (`arboard` text) in `output.rs`
- [ ] T025 [US3] Validate: `cargo test -p pinshot-core` green; manual quickstart §US3 (magnifier pixels, exact W×H, color copy)

**Checkpoint**: All three user stories functional.

---

## Phase 6: Polish & Cross-Cutting

- [x] T026 [P] Permission handling in `src-tauri/src/capture/mod.rs`: detect a failed/empty capture (e.g. macOS Screen Recording not granted) and surface a clear, actionable message instead of crashing (FR-016)
- [x] T027 [P] Document the default capture hotkey and tray usage in `README.md`
- [x] T028 Full gate: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `npm --prefix ui run lint && npm --prefix ui run build` all green
- [ ] T029 Run quickstart.md end-to-end (core tests automated; GUI + mixed-DPI + offline checks manual) and record SC-001…SC-007 status

---

## Dependencies & Execution Order

- **Phase 1 → Phase 2 → user stories**: deps and core types come first.
- **US1 (Phase 3)** is the MVP — do first; depends on Phase 2 (`capture`, `selection`, `crop`).
- **US2 (Phase 4)** depends on Phase 2 + the `commit_selection`/overlay from US1 (adds `encode`/`naming`/file output).
- **US3 (Phase 5)** depends on the overlay from US1 (adds magnifier/color in the same overlay).
- **Polish (Phase 6)** after the stories.
- **Parallel**: T001/T002/T003; T004/T005/T006; T017/T018; T022 are independent files.

## Implementation Strategy

MVP first: Phases 1–3 (US1) → validate → US2 → US3 → polish. The `pinshot-core`
modules are fully unit-tested at each step; GUI behavior is validated by running
the app per quickstart.md. Commit after each story; land via PR to `main`.
