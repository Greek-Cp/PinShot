# Feature Specification: Capture Area to Clipboard & File

**Feature Branch**: `002-capture-to-output`

**Created**: 2026-06-13

**Status**: Draft

**Input**: User description: "Capture a screen region via global hotkey or tray and output it to the clipboard or a PNG file, with selection magnifier, dimensions, and pixel color readout, correct on multi-monitor mixed-DPI setups, fully offline"

## User Scenarios & Testing *(mandatory)*

This feature is the foundational capture pipeline for PinShot: pressing a hotkey
brings up a selection overlay, the user drags a region, and the pixels go to the
clipboard or a file. It is the slice every later feature (pin, annotation, OCR,
beautify) builds on. Pinning the result into a floating window is a separate,
later feature and is explicitly out of scope here.

### User Story 1 - Capture a region to the clipboard (Priority: P1)

As anyone who shares screenshots, I press the capture hotkey (or pick "Capture"
from the tray/menu-bar icon), a selection overlay appears across all my
monitors, I drag a rectangle over the area I want, and on release the captured
image is on my clipboard ready to paste — without the OS screenshot tool, an
account, or any upload.

**Why this priority**: This is the smallest end-to-end slice that delivers
real value (it replaces "screenshot region → clipboard" from the OS tool) and
it establishes the trigger → overlay → select → capture → output pipeline that
US2, US3, and all future features reuse. Without it nothing else can exist.

**Independent Test**: Press the hotkey, drag a region over known content, then
paste into an image editor. The pasted image matches the selected region
exactly. Trigger from the tray menu and confirm the same result. This is
shippable on its own as a minimal capture tool.

**Acceptance Scenarios**:

1. **Given** the app is running in the tray, **When** the user presses the capture hotkey, **Then** a selection overlay covering every connected display appears.
2. **Given** the overlay is shown, **When** the user drags a rectangle and releases, **Then** the pixels inside that rectangle are placed on the system clipboard as an image.
3. **Given** the overlay is shown, **When** the user presses Esc (or right-clicks), **Then** the overlay closes and the clipboard is unchanged.
4. **Given** the app is running, **When** the user selects "Capture" from the tray/menu-bar icon, **Then** the same selection overlay appears.
5. **Given** a capture has just been copied, **When** the user pastes into another application, **Then** the pasted image is identical to the selected region.

---

### User Story 2 - Save a capture to a PNG file (Priority: P2)

As a developer or technical writer, after selecting a region I want it saved as
a PNG file in a predictable folder with an automatic filename, so I can attach
it to a bug report or document without manually choosing a path every time.

**Why this priority**: Saving to disk is the second fundamental output and is
required for documentation workflows, but it depends on the capture pipeline
from US1 and adds value on top of it rather than replacing it.

**Independent Test**: Select a region and choose the save action; confirm a PNG
file appears in the default folder with an auto-generated name and that it opens
to exactly the captured region.

**Acceptance Scenarios**:

1. **Given** a region has been selected, **When** the user invokes the save action, **Then** a PNG file containing the captured region is written to the default folder.
2. **Given** a save, **When** the user opens the resulting file, **Then** it is a valid PNG matching the captured region.
3. **Given** two captures saved in quick succession, **When** both are saved, **Then** their filenames do not collide and neither overwrites the other.
4. **Given** the default folder does not exist or is not writable, **When** the user invokes save, **Then** a clear, actionable error is shown and no data is silently lost.

---

### User Story 3 - Precise selection aids: magnifier, dimensions, color (Priority: P3)

As someone who needs pixel-accurate captures, while I am selecting I see a
magnifier with zoomed pixels under my cursor, the live width×height of my
selection, the cursor's pixel coordinate, and the color of the pixel under the
cursor — and I can copy that color value with a keypress.

**Why this priority**: These aids make selection precise and unlock the
color-picker value, but capture works without them, so they layer on top of
US1/US2.

**Independent Test**: Start a selection, hover over known content, and confirm
the magnifier shows the correct zoomed pixels, the W×H matches the dragged size,
and the displayed color equals the hovered pixel's color; press the copy-color
key and confirm the HEX value on the clipboard matches that pixel.

**Acceptance Scenarios**:

1. **Given** the overlay is shown, **When** the user moves the cursor, **Then** a magnifier displays the zoomed pixels around the cursor and the cursor's pixel coordinate.
2. **Given** the user is dragging a selection, **When** the rectangle changes size, **Then** the displayed width×height updates to the exact pixel dimensions of the selection.
3. **Given** the cursor is over a pixel, **When** the user reads the color readout, **Then** it shows that pixel's color in HEX and RGB.
4. **Given** the color readout is shown, **When** the user presses the copy-color key, **Then** the color value is placed on the clipboard and matches the hovered pixel exactly.

---

### Edge Cases

- **Mixed-DPI multi-monitor**: Displays at different scale factors (e.g., a 2× Retina laptop screen beside a 1× external monitor). Captured pixels and the selection rectangle MUST be correct on every display — no offset, no wrong-scale, no wrong-monitor capture. This is the project's highest historical risk and is a mandatory test case.
- **Selection spanning two monitors**: A rectangle dragged across the seam between displays MUST capture a single correct composite of the covered area.
- **Zero-area selection**: A click with no drag (width or height = 0) MUST NOT write to the clipboard or a file; the overlay handles it gracefully.
- **Double trigger**: Pressing the hotkey while the overlay is already open MUST NOT open a second overlay.
- **Display change mid-selection**: A monitor is unplugged or rearranged while the overlay is open; the flow MUST NOT crash and MUST either adapt or cancel cleanly.
- **macOS Screen Recording permission missing**: The OS withholds capture permission; the app MUST surface a clear, actionable message guiding the user to grant it, never crash or fail silently.
- **Clipboard or filesystem unavailable**: The clipboard is locked or the save folder is read-only; the user MUST get a clear error and the operation MUST fail visibly without partial/corrupt output.
- **Very large selection**: Selecting an entire 4K/5K display MUST complete without excessive memory use or noticeable lag.
- **Cancel after drag start**: Beginning a drag then pressing Esc MUST leave clipboard and filesystem unchanged.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST display a selection overlay that covers every connected display when capture is triggered.
- **FR-002**: The selection overlay MUST become visible within 100ms of the trigger on baseline hardware (consistent with the performance principle).
- **FR-003**: Users MUST be able to select a rectangular region by dragging on the overlay.
- **FR-004**: Users MUST be able to cancel an in-progress capture (e.g., Esc or right-click), leaving the clipboard and filesystem unchanged.
- **FR-005**: On confirmation of a non-empty selection, the system MUST capture exactly the screen pixels within the selected rectangle.
- **FR-006**: The system MUST place the captured image on the system clipboard (US1).
- **FR-007**: The system MUST be able to save the captured image as a PNG file with an auto-generated, non-colliding filename in a configured default folder (US2).
- **FR-008**: Captured pixels and the selection rectangle MUST be correct on multi-monitor setups, including displays with different DPI/scale factors and selections that span displays.
- **FR-009**: A zero-area selection MUST NOT produce a capture, a clipboard write, or a file.
- **FR-010**: The system MUST trigger capture via a default global hotkey that is active whenever the app is running in the tray/menu bar. (Hotkey remapping and conflict detection are out of scope — later feature.)
- **FR-011**: The system MUST provide a tray/menu-bar entry to start a capture and to quit the app.
- **FR-012**: During selection, the system MUST display the current selection dimensions as exact pixel width×height (US3).
- **FR-013**: During selection, the system MUST display a magnifier showing zoomed pixels around the cursor and the cursor's pixel coordinate (US3).
- **FR-014**: During selection, the system MUST display the color of the pixel under the cursor in HEX and RGB and allow copying it to the clipboard with a keypress (US3).
- **FR-015**: The entire capture flow MUST operate fully offline, issuing no network request of any kind (privacy principle).
- **FR-016**: When a required OS screen-capture permission is not granted, the system MUST present a clear, actionable message and MUST NOT crash or fail silently.
- **FR-017**: The capture domain logic (selection-geometry normalisation and region→image extraction) MUST live in the headless core library and be unit-testable without a display or GUI.
- **FR-018**: Triggering capture while an overlay is already active MUST NOT open a second overlay.
- **FR-019**: The capture flow MUST behave equivalently on macOS and Windows (cross-platform parity).

### Key Entities *(include if feature involves data)*

- **Capture Request**: An intent to capture, originating from a trigger (global hotkey or tray menu) at a point in time.
- **Display**: A connected monitor with a position in the virtual desktop, a pixel resolution, and a scale (DPI) factor.
- **Selection Region**: A normalised rectangle in virtual-desktop coordinates describing the area the user chose; always has non-negative dimensions.
- **Captured Image**: The pixels extracted for a selection region, with width, height, and pixel color data.
- **Output Target**: Where a captured image goes — the system clipboard or a PNG file at a destination path.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From the hotkey press, the selection overlay is visible in under 100ms on baseline hardware.
- **SC-002**: A captured region pasted into another application matches the selected pixels exactly — with no DPI scaling or positional error — on single-display, multi-display, and mixed-DPI multi-display setups, verified on both macOS and Windows.
- **SC-003**: Zero network requests occur during the entire capture flow, verifiable with a network monitor.
- **SC-004**: Saving produces a valid PNG that opens to the correct image in the configured folder with no manual path entry, and repeated saves never collide.
- **SC-005**: The on-overlay dimension, magnifier, and color readouts match ground truth — width×height exact to the pixel, and the copied color value equals the hovered pixel's actual color.
- **SC-006**: Canceling a capture leaves the clipboard and filesystem byte-for-byte unchanged.
- **SC-007**: A user can complete "hotkey → select → paste elsewhere" in a single uninterrupted gesture with no intermediate dialogs.

## Assumptions

- **OS permissions**: On macOS the user grants the Screen Recording permission; this feature only needs to detect a missing permission and guide the user (a full onboarding flow is a later concern). On Windows no special capture permission is assumed.
- **Default hotkey**: A sensible per-platform default hotkey is chosen for this feature; user remapping and conflict detection are deferred to v0.2.
- **Default save location & naming**: A sensible default folder (e.g., the user's Pictures area under a PinShot subfolder) and an automatic timestamp-based filename are used; a full settings UI to configure these is v0.2.
- **File format**: PNG is the only save format in this feature; JPG and other formats come later.
- **Color picker scope**: Color readout exists only within the selection overlay here; a standalone color-picker mode is v0.2.
- **Out of scope**: Pinning the capture into a floating window, annotation, OCR, beautify, history, scrolling capture, and screen recording are all separate later features.
- **Still images only**: This feature captures still images; video/scrolling capture is a non-goal.
- **Architecture**: Selection geometry and region→image extraction are implemented in `pinshot-core` behind a platform-capture port; OS-specific capture is provided by the Tauri shell, consistent with the project's clean-architecture principle. The specific capture library is a planning decision, not part of this spec.
