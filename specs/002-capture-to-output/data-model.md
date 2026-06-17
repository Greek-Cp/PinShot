# Phase 1 Data Model: Capture Area to Clipboard & File

**Feature**: 002-capture-to-output | **Date**: 2026-06-13

These are in-memory domain types (no persistence beyond the output PNG). Types
that carry geometry/pixel logic live in `pinshot-core`; the shell owns the I/O.

## Entities

### Display
A connected monitor.
- `id: u32` — stable identifier for the session.
- `origin: (i32, i32)` — top-left in **physical** virtual-desktop pixels.
- `size: (u32, u32)` — physical pixel resolution.
- `scale_factor: f64` — DPI scale (e.g. 1.0, 1.5, 2.0).
- Validation: `scale_factor > 0`; `size` non-zero.

### FrozenFrame
One display's pixels captured at trigger time.
- `display_id: u32` — owning [Display].
- `width, height: u32` — physical pixels (== Display.size).
- `rgba: Vec<u8>` — `width*height*4` bytes.
- Validation: `rgba.len() == width*height*4`.
- Lifetime: exists only between trigger and overlay close; dropped on cancel/finish.

### CaptureSession
The in-flight capture.
- `displays: Vec<Display>`
- `frames: Vec<FrozenFrame>` — one per display.
- `state: OverlayState` (below).
- Invariant: one frame per display; all dropped when state returns to `Idle`.

### SelectionRegion
The user's chosen rectangle, normalised, in **physical** virtual-desktop pixels.
- Backed by the existing `pinshot_core::Rect` (non-negative width/height).
- Derived from overlay logical coords via `selection::to_physical(...)`.
- Validation: `!is_empty()` required before any output (FR-009).

### CapturedImage
The cropped result.
- `width, height: u32`
- `rgba: Vec<u8>` — `width*height*4`.
- Produced by `crop::crop_region(frames, displays, region)`.

### OutputTarget
Where a CapturedImage goes.
- `Clipboard` — via `arboard`.
- `File { dir: PathBuf, name: String }` — PNG via `encode` + filesystem.

## Overlay State Machine

```
Idle ──trigger(hotkey|tray)──► Capturing
Capturing ──frames ready, overlay shown──► Selecting
Selecting ──drag──► Selecting            (live W×H, magnifier, color update)
Selecting ──confirm (non-empty)──► Output
Selecting ──Esc / right-click / zero-area confirm──► Idle   (no clipboard/FS change)
Output ──clipboard set / file written──► Idle
Output ──error (clipboard/FS/permission)──► Idle  (surface actionable message)
Capturing ──permission denied / capture error──► Idle (surface actionable message)
```

- A `trigger` received in any non-`Idle` state is ignored (FR-018: no second overlay).
- Returning to `Idle` always drops `frames` and hides overlay windows.

## Core API surface (new/extended in `pinshot-core`)

- `geometry::Rect` (exists) + helpers for offsetting into a display's frame.
- `capture::{Display, FrozenFrame, ScreenCapturer}` — `ScreenCapturer` extended to
  `capture_all() -> Result<(Vec<Display>, Vec<FrozenFrame>), CaptureError>`.
- `selection::to_physical(logical_rect, display) -> Rect` and
  `selection::displays_for(region, &[Display]) -> Vec<u32>`.
- `crop::crop_region(&[FrozenFrame], &[Display], Rect) -> Result<CapturedImage, CropError>`.
- `encode::to_png(&CapturedImage) -> Result<Vec<u8>, EncodeError>`.
- `naming::output_filename(now, existing) -> String`.

All of the above are pure (no I/O) and unit-tested headless (FR-017). The shell
provides the `ScreenCapturer` implementation (`xcap`) and performs clipboard/file
writes.
