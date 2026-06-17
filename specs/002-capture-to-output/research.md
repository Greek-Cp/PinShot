# Phase 0 Research: Capture Area to Clipboard & File

**Feature**: 002-capture-to-output | **Date**: 2026-06-13

Each decision resolves an unknown from the plan's Technical Context or an edge
case from the spec. No `NEEDS CLARIFICATION` remains.

## D1 — Screen capture library: `xcap`

**Decision**: Use `xcap` for enumerating monitors and grabbing their pixels (RGBA).

**Rationale**: It is the vetted choice in `about.md` — one cross-platform API for
macOS + Windows with multi-monitor support and per-monitor scale/position info,
no network, MIT/Apache. It returns per-monitor RGBA buffers we can freeze.

**Alternatives rejected**: Raw platform APIs (ScreenCaptureKit / Windows.Graphics.Capture) — more code and per-OS maintenance for no v0.1 benefit; the older `screenshots` crate — superseded by `xcap`.

## D2 — Freeze-then-overlay (capture first, then select)

**Decision**: On trigger, immediately capture **all** monitors into frozen RGBA
buffers, then display the overlay showing those frozen frames. Selection,
magnifier, and color readout all read the frozen buffers; the final crop is
taken from them.

**Rationale**: (a) Capture is instant from the user's perspective — no re-grab on
confirm. (b) The magnifier and pixel-color readout (US3) need the actual pixels,
which we already hold. (c) We never accidentally capture our own overlay. This is
the established Snipaste/Shottr model.

**Alternatives rejected**: Overlay-first then capture-on-confirm — must hide the
overlay before grabbing (flicker/timing races) and re-capture; can't power the
magnifier/color readout without an extra grab.

## D3 — One overlay window per display (not a single spanning window)

**Decision**: Pre-create one borderless, always-on-top, full-screen overlay
window **per connected display**, each rendering that display's frozen frame at
that display's scale factor. Selection is tracked in a shared virtual-desktop
pixel coordinate space; a selection spanning displays composites from each
display's frozen buffer.

**Rationale**: A single window spanning monitors of different DPI is the classic
source of mixed-DPI bugs (the OS scales one region wrong). Per-display windows
let each render natively at its own scale, which is the simplest way to get
SC-002 right. Mapping logic lives in `pinshot-core::selection` and is unit-tested.

**Alternatives rejected**: One giant window across the virtual desktop — simpler
window management but fragile under mixed DPI (the exact risk the spec flags).

## D4 — Performance: pre-created hidden overlay windows

**Decision**: Create the overlay window(s) hidden at app startup; on trigger,
push the frozen frame and `show()` them. Do not create windows on the hotkey path.

**Rationale**: Tauri/OS window creation can cost >100ms; showing an existing
window is fast. Combined with a single freeze grab, this is how we hit SC-001
(<100ms). Measured in quickstart.

**Alternatives rejected**: Create overlay windows on demand per capture — risks
blowing the 100ms budget and adds first-capture latency.

## D5 — Global hotkey: `global-hotkey`

**Decision**: Register a default capture hotkey with the `global-hotkey` crate.
Default: `CmdOrCtrl+Shift+A` ("A" = area).

**Rationale**: Vetted in `about.md`; cross-platform; works while the app sits in
the tray. The default avoids known OS conflicts (macOS reserves Cmd+Shift+3/4/5;
Windows reserves PrintScreen variants). Remapping + conflict detection are
explicitly v0.2, so a single sensible default is enough now.

**Alternatives rejected**: Tauri's `global-shortcut` plugin — viable, but
`global-hotkey` is the documented choice and keeps the option open to move hotkey
handling fully into the core/shell without a plugin. Either is acceptable; pinned
to the documented one for consistency.

## D6 — Tray: Tauri 2 built-in tray

**Decision**: Use Tauri 2's built-in `TrayIconBuilder` for the menu-bar/tray icon
with "Capture" and "Quit" items.

**Rationale**: We are already in Tauri; its built-in tray wraps the same
`tray-icon` mechanism named in `about.md` with less glue and one less direct
dependency. Keeps the shell simple (Constitution V/VI).

**Alternatives rejected**: The standalone `tray-icon` crate directly — redundant
with Tauri's integrated tray.

## D7 — Clipboard image write: `arboard`

**Decision**: Write the captured image to the clipboard with `arboard`
(`set_image` with RGBA).

**Rationale**: `arboard` supports **image** clipboard data on both macOS and
Windows; the Tauri clipboard plugin is text-oriented and does not cover raster
images well. No network; MIT/Apache.

**Alternatives rejected**: `tauri-plugin-clipboard-manager` — text/HTML focused,
image support is not first-class; raw platform clipboard APIs — more per-OS code.

## D8 — File output: `image` (PNG) + `dirs` (folder)

**Decision**: Encode RGBA→PNG with the `image` crate (in `pinshot-core::encode`,
pure/testable). Default folder = `dirs::picture_dir()/PinShot`; filename =
`PinShot_YYYY-MM-DD_HH-MM-SS[_N].png` (suffix `_N` only on same-second collision).

**Rationale**: `image` is the vetted encoder (about.md), pure and unit-testable;
`dirs` resolves the OS Pictures folder cross-platform. Timestamp naming needs no
settings UI (deferred to v0.2) and the collision suffix satisfies FR-007/SC-004.

**Alternatives rejected**: `png` crate directly — fine but `image` is already the
chosen image-ops dependency for later features (crop/resize/blur), so reuse it.

## D9 — Magnifier & color source: the frozen buffer

**Decision**: The magnifier renders a zoomed crop of the frozen frame around the
cursor; the color readout reads the pixel at the cursor from the frozen frame.
Both are computed in the overlay webview from the frame already sent to it; the
exact pixel color is also verifiable in the core from the same buffer.

**Rationale**: Free consequence of D2 — no extra capture, exact pixels (SC-005).

**Alternatives rejected**: Live re-sampling the screen under the cursor — extra
latency and would sample the overlay itself.

## D10 — DPI / coordinate model

**Decision**: The core works in **physical pixels**. A `Display` carries its
virtual-desktop origin and `scale_factor`. The overlay reports selection in its
window's logical (CSS) coordinates plus which display; `pinshot-core::selection`
maps `logical × scale (+ display origin)` → physical-pixel `Rect` in the frozen
buffer, then `crop` extracts it (compositing across displays when spanning).

**Rationale**: Centralising the scale math in one tested core module is the
single most important thing for SC-002 (no off-by-DPI). Keeps every platform
quirk out of the overlay JS.

**Alternatives rejected**: Doing scale math in the webview — scatters the
riskiest logic into the least-testable layer; rejected per Constitution IV/VI.

## Dependency / license summary

| Crate | Purpose | Network? | License |
|---|---|---|---|
| `xcap` | multi-monitor capture | none | Apache-2.0 |
| `global-hotkey` | global capture hotkey | none | MIT/Apache-2.0 |
| `arboard` | clipboard image write | none | MIT/Apache-2.0 |
| `image` | RGBA→PNG encode | none | MIT/Apache-2.0 |
| `dirs` | default Pictures folder | none | MIT/Apache-2.0 |

All compatible with GPL-3.0 and consistent with the offline/no-network principle.
