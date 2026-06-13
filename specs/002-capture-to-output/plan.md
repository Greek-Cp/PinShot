# Implementation Plan: Capture Area to Clipboard & File

**Branch**: `002-capture-to-output` | **Date**: 2026-06-13 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-capture-to-output/spec.md`

## Summary

Build the foundational capture pipeline: a global hotkey (or tray entry) freezes
the current screen, shows a borderless selection overlay across every display,
and on confirmation crops the frozen pixels and sends them to the clipboard
(US1) or a PNG file (US2); during selection an overlay HUD shows a magnifier,
live width×height, and the pixel color under the cursor with copy (US3).

The technical crux is **DPI-correct multi-monitor capture** (SC-002) and the
**<100ms overlay** (SC-001). Both are met by the same core decision: at trigger
time we **capture all monitors once into frozen RGBA buffers**, then show a
**pre-created, hidden-at-startup** overlay window per monitor that displays its
frozen buffer. Selection happens against frozen pixels, so capture is instant,
the magnifier/color readout are free (we already hold the pixels), and we never
capture our own overlay. All cropping, PNG encoding, and coordinate→pixel
mapping live in the headless `pinshot-core` crate and are unit-tested without a
display.

## Technical Context

**Language/Version**: Rust 1.96.0 (pinned), TypeScript (Node 20)

**Primary Dependencies**: Tauri 2.x (windows, tray, IPC); `xcap` (multi-monitor screen capture); `global-hotkey` (global capture hotkey); `arboard` (clipboard image write); `image` (RGBA→PNG encode); `dirs` (default Pictures folder). All are OS/local-only — no network dependency, all MIT/Apache-2.0 (GPL-3.0 compatible).

**Storage**: Local PNG files in a default folder (Pictures/PinShot); no database. Settings persistence (folder/hotkey config UI) is out of scope (v0.2).

**Testing**: `cargo test -p pinshot-core` for geometry/crop/encode/mapping (headless); manual quickstart scenarios for the GUI overlay and DPI/multi-monitor correctness on both OSes.

**Target Platform**: macOS and Windows desktop (Tauri).

**Project Type**: Desktop app (Tauri) — existing workspace from feature 001.

**Performance Goals**: Overlay visible <100ms after hotkey (SC-001); whole-display capture without noticeable lag or excess memory.

**Constraints**: Zero network in the capture flow (FR-015/SC-003); DPI-exact crop on mixed-DPI multi-monitor (FR-008/SC-002); cancel leaves clipboard+FS unchanged (SC-006); capture logic testable headless (FR-017).

**Scale/Scope**: Single user, local. One overlay window per connected display; typically 1–4 displays.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.* — **Constitution v1.1.0**

| Principle | Gate | Status |
|---|---|---|
| I. Privacy-First & Offline-Only | Capture, crop, encode, clipboard, and file save are all local. No new dependency makes a network call (xcap/global-hotkey/arboard/image/dirs are OS/local only). | ✅ PASS |
| II. Performance Is a Feature | Overlay windows are pre-created hidden at startup and merely shown+populated on trigger; screen is frozen once. Design targets <100ms (SC-001) and is measured in quickstart. | ✅ PASS |
| III. Cross-Platform Parity | Identical flow on macOS + Windows; mixed-DPI multi-monitor is a mandatory test (FR-008/FR-019/SC-002). CI already builds/tests both. | ✅ PASS |
| IV. Clean Architecture: Core as a Library | Geometry, coordinate→pixel mapping, crop (incl. cross-display composite), and PNG encode live in `pinshot-core` as pure functions behind ports; `xcap`/clipboard/file/window I/O live in the shell implementing those ports. Core tested headless. | ✅ PASS |
| V. Strict Scope & Simplicity | Capture→output only. No pin, annotation, OCR, beautify, history, settings UI, or hotkey remapping. New deps are each the documented vetted choice (about.md). | ✅ PASS |
| VI. Maintainability & Contributor Experience | Same toolchain/commands as 001; new core APIs get doc comments; selection/DPI math is the riskiest part and is the most heavily unit-tested. IPC contract documented in `contracts/`. | ✅ PASS |

**No violations — Complexity Tracking is empty.** (One judgment call — one overlay window *per display* rather than a single spanning window — is justified in [research.md](./research.md) D3 as the simpler correct option for mixed-DPI, not added complexity.)

## Project Structure

### Documentation (this feature)

```text
specs/002-capture-to-output/
├── spec.md
├── plan.md              # This file
├── research.md          # Phase 0: capture/overlay/clipboard decisions
├── data-model.md        # Phase 1: entities + overlay state machine
├── contracts/
│   └── capture-ipc.md   # Phase 1: UI ↔ shell command/event contract
├── quickstart.md        # Phase 1: validation incl. mixed-DPI + offline
├── checklists/
│   └── requirements.md  # Spec quality checklist (passed)
└── tasks.md             # Phase 2 (/speckit.tasks — not produced here)
```

### Source Code (repository root) — additions to the existing workspace

```text
crates/pinshot-core/src/
├── geometry.rs          # Rect (exists) — extend with virtual-desktop mapping helpers
├── capture.rs           # ScreenCapturer port (exists) — extend: Display, multi-monitor frame set
├── selection.rs         # NEW: map overlay/logical coords → physical pixels; pick display(s)
├── crop.rs              # NEW: crop a Rect from frozen frames, incl. cross-display composite
├── encode.rs            # NEW: RGBA → PNG bytes (pure, uses `image`)
├── naming.rs            # NEW: timestamp-based, non-colliding output filename
└── lib.rs               # re-exports

src-tauri/src/
├── lib.rs               # builder: tray + hotkey + overlay registration (extend)
├── capture/
│   ├── mod.rs           # capture flow orchestration (trigger → freeze → show overlay)
│   ├── xcap_capturer.rs # ScreenCapturer impl over xcap (platform adapter)
│   ├── overlay.rs       # pre-create per-display overlay windows; show/hide/populate
│   ├── output.rs        # clipboard (arboard) + PNG file save adapters
│   ├── hotkey.rs        # global-hotkey registration → trigger
│   └── tray.rs          # Tauri tray: Capture / Quit
└── main.rs              # (exists)

ui/
├── index.html           # main window (exists)
├── overlay.html         # NEW: selection overlay page
├── src/main.ts          # main window (exists)
└── src/overlay.ts       # NEW: render frozen frame, drag selection, magnifier, color HUD
```

**Structure Decision**: Extends the existing Cargo workspace + Tauri shell + Vite
frontend from 001. Pure capture logic concentrates in new `pinshot-core` modules
(`selection`, `crop`, `encode`, `naming`) so the DPI/geometry correctness that
drives SC-002 is unit-tested headless; all platform side effects (screen grab,
windows, clipboard, files, hotkey, tray) live in `src-tauri/src/capture/` behind
the core's ports. The overlay is a second Vite entry (`overlay.html`) so it ships
in the same bundle as the main window.

## Phase Outputs

- **Phase 0** → [research.md](./research.md): 10 decisions (capture lib, freeze-then-overlay, per-display windows, hotkey, tray, clipboard image, PNG/folder, magnifier source, permission handling, DPI coordinate model).
- **Phase 1** → [data-model.md](./data-model.md) (entities + overlay state machine), [contracts/capture-ipc.md](./contracts/capture-ipc.md) (UI↔shell commands/events), [quickstart.md](./quickstart.md) (validation incl. mixed-DPI, offline, cancel).

## Complexity Tracking

*No constitution violations — table intentionally empty.*
