# Tasks: Adjustable Selection & Floating Pin

**Input**: Design documents from `/specs/003-adjustable-selection-and-pin/`

**Prerequisites**: spec.md ✅, plan.md ✅ (research + data-model condensed inline), contracts/pin-ipc.md ✅

**Clarifications applied**: close a pin by **double-click**; v1 pin is **static**
(move/close/copy only); a pin **never auto-scales** (full physical size, drag to
reveal); build order this pass is **US2 (resize) → US1 (pin)**.

**Tests**: Unit tests cover the new `pinshot-core` geometry only (resize/move/
clamp + pin size/placement) — that pure logic carries SC-002/003/004. Overlay
interaction, pin behaviour, always-on-top, and mixed-DPI placement are validated
manually (a real desktop session is required).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US2 = adjustable selection, US1 = floating pin, US3 = clean visuals

## Phase 1: Setup

- [x] T001 [P] Add a third Vite entry `ui/pin.html` and register it as an input in `ui/vite.config.ts` (inputs: `index.html`, `overlay.html`, `pin.html`)
- [x] T002 Confirm no new crates needed (Tauri windows + existing `arboard`/`image`/`base64`); add the pin-window capability (`pin-*` windows + `core:window:allow-start-dragging`) in `src-tauri/capabilities/default.json`

**Checkpoint**: `npm run build` emits `pin.html`; `cargo build --workspace` green.

---

## Phase 2: Foundational (pure core geometry — blocks both stories)

**Purpose**: The display-independent math both stories depend on, unit-tested headless.

- [x] T003 [P] Extend `crates/pinshot-core/src/geometry.rs`: add `Rect::translate(dx,dy)`, `Rect::clamp_min(min_w,min_h)`, `Rect::clamp_to(bounds: Rect)`, and `Rect::resize(edges: EdgeSet, dx, dy)` (or equivalent anchor-based resize) used by the eight handles; unit tests for resize per edge/corner, inversion normalising to non-negative, min-size clamp, and bounds clamp
- [x] T004 [P] Create `crates/pinshot-core/src/pin.rs`: pure `pin_logical_size(physical: (u32,u32), scale: f64) -> (f64,f64)` and `pin_placement(region_physical: Rect, display_origin: (i32,i32), scale: f64, work_area: Rect) -> (f64,f64,f64,f64)` returning logical x/y/w/h with the top-left nudged on-screen (no down-scaling); unit tests across scale 1.0/2.0 and an oversized region
- [x] T005 Re-export the new geometry helpers and `pin` module from `crates/pinshot-core/src/lib.rs`

**Checkpoint**: `cargo test -p pinshot-core` passes; shell still compiles.

---

## Phase 3: User Story 2 — Adjustable selection (Priority: P2) 🎯 first this pass

**Goal**: After the initial drag, grab handles to resize or drag inside to move; outputs use the adjusted rect.

**Independent Test**: Drag a rough rect, release, resize via a corner and an edge, move it by inside-drag; the W×H readout tracks each change and the committed output matches the final rect (quickstart §US2).

- [x] T006 [US2] In `ui/src/overlay.ts`: after `mouseup` with a real selection, render eight handles (4 corners + 4 edges) on `selectionEl`, each with the correct resize cursor; keep the dim/backdrop/HUD from 002
- [x] T007 [US2] In `ui/src/overlay.ts`: hit-test pointer-down — on a handle → resize mode (move the bound edge(s), anchor the opposite); inside the rect → move mode; outside → start a fresh selection (preserve 002 behaviour)
- [x] T008 [US2] In `ui/src/overlay.ts`: drive resize/move through the core helpers' logic (mirror `clamp_min`/normalise client-side), live-update the W×H badge, and keep the selection a valid non-negative rect; commit (copy/save/pin) uses the adjusted rect
- [ ] T009 [US2] Verify: `cargo test -p pinshot-core` green; build + manual quickstart §US2 (enlarge, shrink, move, inverted-drag normalises, commit matches final rect), and confirm FR-019 (no blue highlight) still holds

**Checkpoint**: Selection is fully adjustable before any output; US2 shippable on its own.

---

## Phase 4: User Story 1 — Floating pin (Priority: P1)

**Goal**: Turn a selection into a borderless, always-on-top, draggable window; multi-pin; double-click to close; copy from pin.

**Independent Test**: Capture → Pin → switch apps (pin stays on top) → drag it → open a second pin → double-click one to close (other remains) → copy from a pin and paste (quickstart §US1).

- [x] T010 [US1] Create `src-tauri/src/capture/pin.rs`: `PinRegistry` (`Mutex<HashMap<PinId, PinnedImage>>` + next-id) and functions to create a pin window (borderless, `always_on_top`, no taskbar, sized via `pin::pin_logical_size`, positioned via `pin::pin_placement`), `close_pin`, `raise_pin`
- [x] T011 [US1] In `src-tauri/src/capture/mod.rs` + `lib.rs`: `manage` the `PinRegistry`; add the `create_pin({displayId, rect})` command — reuse `to_physical` + `crop_region`, store a `PinnedImage` (PNG via `source_png` when present, else `to_png`), open the pin window, then `overlay::close` + clear the session; register `get_pin_image`, `close_pin`, `raise_pin`, `copy_pin` commands
- [x] T012 [US1] Create `ui/pin.html` + `ui/src/pin.ts`: read `?id=`, pull image via `get_pin_image`, render it full-size (`image-rendering` crisp, `user-select:none`); pointer-down → `getCurrentWindow().startDragging()` and `raise_pin`; **double-click → `close_pin`**; right-nothing; offline only
- [x] T013 [US1] Add a **Pin** action to the overlay toolbar and a shortcut key (e.g. `P`) in `ui/src/overlay.ts`, calling `create_pin` with the (adjusted) rect alongside Copy/Save; wire `copy_pin` (e.g. `C`/button) in `pin.ts`
- [ ] T014 [US1] Verify: build + manual quickstart §US1 (pin floats above another app, drag-move, multi-pin, double-click close leaves others, copy-from-pin matches), and **mixed-DPI placement** (pin made on a 2× display, dragged to a 1× display, renders at correct physical size) — mandatory per Constitution III

**Checkpoint**: Capture → Pin works end-to-end; both user stories functional.

---

## Phase 5: Polish & Cross-Cutting

- [x] T015 [P] Handle the "display removed under a pin" edge case in `src-tauri/src/capture/pin.rs`: if a pin would be fully off-screen, reposition it onto a remaining display (FR + edge case), never crash
- [x] T016 [P] Update `README.md`: document Pin (create + double-click close + copy) and the adjustable-selection handles
- [x] T017 Full gate: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `npm --prefix ui run lint && npm --prefix ui run build` all green
- [ ] T018 Run quickstart end-to-end (core tests automated; GUI + mixed-DPI + offline manual) and record SC-001…SC-008 status

---

## Dependencies & Execution Order

- **Phase 1 → Phase 2 → stories**: setup + pure geometry first.
- **US2 (Phase 3)** first this pass — depends only on Phase 2 geometry + 002's overlay.
- **US1 (Phase 4)** next — depends on Phase 2 (`pin` math) and reuses 002's `to_physical`/`crop_region`; benefits from US2 (pins the adjusted rect) but does not require it.
- **Polish (Phase 5)** after both stories.
- **Parallel**: T003/T004; T015/T016 are independent files.

## Implementation Strategy

Resize first (smallest, self-contained, immediate friction win), validate, then
the pin (the headline feature). The `pinshot-core` geometry is fully unit-tested
at each step; GUI/pin behaviour is validated by running the app per the quickstart
scenarios. Commit after each story.
