# Quickstart: Validating Capture to Clipboard & File

**Feature**: 002-capture-to-output | **Date**: 2026-06-13

## Headless core tests (run anywhere, no display)

```bash
cargo test -p pinshot-core
```
Covers selection-geometry mapping (logical→physical across scale factors), crop
(including cross-display composite), PNG encoding round-trip, and filename
collision suffixing — the logic behind SC-002/SC-004/SC-005.

## Run the app

```bash
cd ui && npm ci && npm run build && cd ..
cargo run -p pinshot
```
The app starts in the tray/menu bar (no main window needed for capture).

## US1 — capture to clipboard

1. Press the capture hotkey (`Cmd+Shift+A` on macOS, `Ctrl+Shift+A` on Windows)
   → the selection overlay appears across all displays.
2. Drag a rectangle over known content and release.
3. Paste into an image editor → **the pasted image matches the selected region
   exactly** (SC-002).
4. Repeat triggering from the tray menu's **Capture** item.
5. Press the hotkey, then **Esc** → overlay closes, clipboard unchanged (SC-006).

**<100ms check (SC-001)**: trigger repeatedly; the overlay must feel instant.
Instrument the trigger→show path with a timestamp log if measuring precisely.

## US2 — save to PNG

1. Trigger, select, choose the **save** action.
2. Confirm a PNG appears under `Pictures/PinShot/` with a timestamp name and
   opens to the correct image (SC-004).
3. Save two captures within the same second → filenames differ (no overwrite).
4. Make the folder read-only and save → a clear error appears, nothing is lost.

## US3 — magnifier, dimensions, color

1. During selection, move the cursor → magnifier shows correct zoomed pixels and
   the cursor coordinate.
2. Drag → the live **W×H** equals the selection's pixel size (SC-005).
3. Read the color readout (HEX/RGB) over a known color; press the copy-color key
   → clipboard holds that exact color value.

## Mixed-DPI multi-monitor (mandatory — SC-002)

On a setup with a HiDPI (2×) display next to a 1× display:
1. Capture a region fully on the **2×** display → pasted image is pixel-exact.
2. Capture a region fully on the **1×** display → pixel-exact.
3. Capture a region **spanning the seam** → single correct composite, no offset
   or scale error.

Run this on **both** macOS and Windows.

## Offline check (SC-003)

With a network monitor (e.g. Little Snitch / `lsof -i` / Resource Monitor)
running, perform a full capture→clipboard and capture→file cycle → **zero
network connections** from the app.

## Permission check (macOS)

Revoke PinShot's Screen Recording permission in System Settings, trigger a
capture → the app shows a clear, actionable message pointing to the setting,
and does not crash (FR-016).
