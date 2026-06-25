# Implementation Plan: Adjustable Selection & Floating Pin

**Branch**: `003-adjustable-selection-and-pin` | **Date**: 2026-06-25 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-adjustable-selection-and-pin/spec.md`

## Summary

Extend feature 002's capture overlay into an interactive one and add PinShot's
namesake **floating pin**. Two slices:

1. **Floating Pin (US1, P1)** — from the selection toolbar (or a shortcut), the
   user turns the current selection into a borderless, always-on-top window that
   shows exactly the captured region and floats above other apps. Pins are
   draggable, independently closable, can be opened many at once, raise on
   interaction, and can copy their image back to the clipboard. Pins are
   session-scoped (no restart persistence in this feature).
2. **Adjustable Selection (US2, P2)** — after the first drag is released the
   overlay shows eight handles (4 corners + 4 edges); dragging a handle resizes,
   dragging inside moves, and the live W×H readout tracks every change. All
   outputs (copy/save/pin) use the final adjusted rectangle.

US3 (remove the native blue text-selection highlight) is a one-line overlay fix
(`user-select: none`) already applied during specification and is covered here
only by FR-019 / a quickstart check.

The technical crux is the same as 002: **DPI-correct geometry on mixed-DPI
multi-monitor** (now also for *pin placement/size*, SC-002) and keeping the
overlay budget under 100ms (SC-001/FR-023). We meet it by keeping all geometry
math — selection resize/move normalisation, min-size clamping, and pin
fit/position math — as pure functions in `pinshot-core`, unit-tested headless,
and by reusing 002's frozen-frame + `crop_region` pipeline unchanged. Each pin is
its own Tauri window (the simplest correct model for independent move/close/z-order).

## Technical Context

**Language/Version**: Rust 1.96.0 (pinned), TypeScript (Node 20+).

**Primary Dependencies**: Tauri 2.x (multi-window create/position, always-on-top,
`startDragging`, IPC); reuse `arboard` (clipboard image), `image`/`base64`
(already present). **No new crates anticipated** — pin windows and drag use
Tauri's built-in window APIs. (If a transparent-window need forces it, the only
candidate is enabling Tauri's existing window transparency, not a new dependency.)

**Storage**: In-memory pin registry in the shell (`HashMap<PinId, PinnedImage>`);
no files, no database. Pin persistence across restarts is out of scope.

**Testing**: `cargo test -p pinshot-core` for the new geometry (resize/move/clamp,
pin fit/position) — headless; manual quickstart for overlay interaction, pin
behaviour, always-on-top, and mixed-DPI placement on both OSes.

**Target Platform**: macOS and Windows desktop (Tauri); extends the 001 workspace.

**Project Type**: Desktop app (Tauri) — existing Cargo workspace + Vite frontend.

**Performance Goals**: Overlay still visible <100ms from trigger (unchanged from
002); a pin window appears promptly after confirm; many static pins keep idle CPU
≈0 and memory within budget (FR-023/SC-008).

**Constraints**: Zero network (FR-020/SC-007); DPI-exact pin size+position across
mixed-DPI displays and across the seam (FR-010/SC-002); geometry testable headless
(FR-021); cross-platform parity incl. the always-on-top contract (FR-022); cancel
during adjust leaves clipboard+FS unchanged (preserve 002).

**Scale/Scope**: Single user, local. 1–N pins (typically a handful); 1–4 displays.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Constitution v1.1.0**

| Principle | Gate | Status |
|---|---|---|
| I. Privacy-First & Offline-Only | Crop, encode, clipboard, and window display are all local; no new network-capable dependency. Pins hold pixels in memory only. | ✅ PASS |
| II. Performance Is a Feature | Overlay path is unchanged (still <100ms); pins reuse the already-decoded frame and the `source_png` bytes (no re-encode). Static pins are inert webviews — idle cost watched in quickstart (SC-008). | ✅ PASS |
| III. Cross-Platform Parity | Pin create/move/close, always-on-top, and resize use Tauri APIs available on both OSes; mixed-DPI pin placement is a mandatory test (FR-010/FR-022/SC-002). The always-on-top-vs-fullscreen contract is documented (Assumptions). | ✅ PASS |
| IV. Clean Architecture: Core as a Library | New geometry (resize/move normalise, min-size clamp, pin fit-to-workarea, logical↔physical placement) lands in `pinshot-core` as pure functions behind no I/O; window/clipboard side effects live in the shell; UI only renders and forwards intents. | ✅ PASS |
| V. Strict Scope & Simplicity | Exactly the roadmap's "pin" step plus selection polish. v1 pin = move/close/front/copy only; annotation, zoom, opacity, click-through, save-from-pin, and persistence are explicitly deferred. No new deps. | ✅ PASS |
| VI. Maintainability & Contributor Experience | Same toolchain/commands; new core fns get doc comments and the heaviest unit tests (the DPI math is the risk); IPC additions documented in `contracts/`. One pin = one window is the obvious, debuggable model. | ✅ PASS |

**No violations — Complexity Tracking is empty.** One judgment call — *one Tauri
window per pin* rather than a single transparent canvas hosting all pins — is the
simpler correct option for independent move/close/z-order and native drag, not
added complexity (see Phase 0 D2).

## Project Structure

### Documentation (this feature)

```text
specs/003-adjustable-selection-and-pin/
├── spec.md              # Feature spec (done)
├── plan.md              # This file
├── contracts/
│   └── pin-ipc.md       # Phase 1: UI ↔ shell commands/events for pin + adjust
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 (/speckit.tasks)
```

Phase 0 (research) and Phase 1 (data-model) are condensed inline below — this
feature reuses 002's capture/crop/overlay foundation, so the only genuinely new
design is the pin window model and the resize interaction, captured here and in
`contracts/pin-ipc.md`.

### Source Code (repository root) — additions to the existing workspace

```text
crates/pinshot-core/src/
├── geometry.rs          # Rect (exists) — add: resize-by-handle, move, clamp_min, clamp_to
├── selection.rs         # (exists) — reuse to_physical for pin region; no change expected
├── pin.rs               # NEW: pure pin geometry — logical size + on-screen placement
└── lib.rs               # re-export the new pin module + geometry helpers

src-tauri/src/
├── capture/
│   ├── mod.rs           # add `create_pin` command + PinRegistry in app state; extend toolbar intents
│   ├── pin.rs           # NEW: shell side — create/move/close/raise pin windows; get_pin_image; copy
│   └── overlay.rs       # (exists) — unchanged (overlay windows)
└── lib.rs               # register pin commands + manage PinRegistry state

ui/
├── overlay.html         # (exists) — unchanged markup
├── pin.html             # NEW: minimal page hosting one pinned image
└── src/
    ├── overlay.ts       # add resize/move handles + a Pin toolbar button/shortcut; (blue-fix done)
    └── pin.ts           # NEW: render pinned image, native-drag to move, Esc/✕ to close, copy
```

**Structure Decision**: Extends 002's modules in place. Pure geometry (resize/move/
clamp + pin fit/placement) concentrates in `pinshot-core` (`geometry.rs`, new
`pin.rs`) so SC-002/SC-003/SC-004 correctness is unit-tested headless. All window
and clipboard side effects live in `src-tauri/src/capture/pin.rs` behind the
existing pattern. The pin UI is a third Vite entry (`pin.html`) shipping in the
same bundle, mirroring how `overlay.html` was added in 002.

## Phase 0 — Research / Key Decisions

- **D1 — Reuse the frozen-frame + crop pipeline.** Pin and adjusted outputs run
  through the same `to_physical` → `crop_region` path as 002; on macOS the
  backdrop/pin source bytes reuse `FrozenFrame.source_png` (no re-encode).
- **D2 — One Tauri window per pin.** Each pin is a borderless, always-on-top,
  non-OS-resizable `WebviewWindow` loading `pin.html?id=<n>`. Rationale: gives
  independent move/close, natural z-order via focus, and native smooth drag via
  Tauri `startDragging()` — far simpler than hit-testing many pins inside one
  transparent fullscreen window. Alternative (single canvas) rejected as more
  complex with worse OS integration.
- **D3 — Drag to move.** Use Tauri's `getCurrentWindow().startDragging()` on
  pointer-down in `pin.ts` for OS-native, jank-free movement; no per-frame
  position IPC.
- **D4 — Pin sizing (clarified).** Always render the captured region at full
  physical size, shown at the owning display's logical scale (logical = physical /
  scale). **No auto-scaling** even when larger than the screen — the pin opens
  with its top-left on-screen and the user drags to reveal the rest. So `pin.rs`
  needs only logical-size + placement math, not fit-to-workarea.
- **D5 — Pin placement.** Open the pin at the selection's on-screen location
  (logical origin derived from physical origin/scale, like `overlay.rs`), nudged
  fully on-screen. Pure `pin::placement` returns logical (x,y,w,h); the shell
  feeds Tauri.
- **D6 — Always-on-top contract.** All pins set `always_on_top(true)`. "On top"
  means above normal app windows; behaviour over another app's native fullscreen
  follows OS policy and is documented, not guaranteed (Assumptions). Raise on
  interaction via `set_focus()`.
- **D9 — Close gesture (clarified).** Double-click anywhere on a pin closes it
  (`close_pin`); no chrome/✕ button. A single pointer-down starts a native drag
  (D3), so the handler distinguishes click-drag from double-click.
- **D7 — Resize handles are UI; normalisation is core.** Handle hit-testing and
  pointer math live in `overlay.ts`; the rectangle is always pushed through pure
  `geometry` helpers (`resize_to`, `translate`, `clamp_min`, `clamp_to(bounds)`)
  so it can never become negative/zero/out-of-bounds — these carry the unit tests.
- **D8 — No new dependencies.** Confirmed: Tauri windows + existing
  `arboard`/`image`/`base64` cover everything.

## Phase 1 — Data Model & Contracts (condensed)

**New/extended entities** (mirrors spec Key Entities):

- `Rect` (core, exists) gains pure helpers: `clamp_min(min_w, min_h)`,
  `clamp_to(bounds: Rect)`, `translate(dx, dy)`, and resize-by-anchor used by the
  eight handles.
- `Handle` (UI enum): `{ N, S, E, W, NE, NW, SE, SW }` → which edges a drag moves.
- `PinnedImage` (shell state): `{ id, png: Vec<u8>, width, height, scale_factor,
  display_origin }` — pixels + the metadata needed to size/place the window and to
  copy the original image.
- `PinRegistry` (shell state): `Mutex<HashMap<PinId, PinnedImage>>` + next-id.

**IPC additions** (full detail in [contracts/pin-ipc.md](./contracts/pin-ipc.md)):

- `create_pin({ displayId, rect }) -> { pinId }` — crop the frozen region, store a
  `PinnedImage`, open the pin window, close the capture overlay.
- `get_pin_image(pinId) -> { dataUrl, width, height, scaleFactor }` — the pin page
  pulls its image (reusing `source_png` where available).
- `close_pin(pinId)` / `raise_pin(pinId)` — window lifecycle.
- `copy_pin(pinId)` — place the pin's original image on the clipboard.
- Overlay `commit_selection` is unchanged but now receives the *adjusted* rect.

## Complexity Tracking

*No constitution violations — table intentionally empty.*
