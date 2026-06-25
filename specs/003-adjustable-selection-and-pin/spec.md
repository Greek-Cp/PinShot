# Feature Specification: Adjustable Selection & Floating Pin

**Feature Branch**: `003-adjustable-selection-and-pin`

**Created**: 2026-06-25

**Status**: Draft

**Input**: User description: "After choosing an area the user can grab the selection's edges/handles to resize and reposition it before committing, and can pin the captured image as a floating always-on-top window that stays above other apps; also remove the native text-selection (blue) highlight that appears while dragging the overlay."

## User Scenarios & Testing *(mandatory)*

This feature turns PinShot's one-shot capture (feature 002) into an interactive
capture: the user can fine-tune the selection rectangle after the first drag,
and—most importantly—**pin** the result as a floating window that hovers above
every other app. Pinning is PinShot's namesake and headline differentiator; it
was explicitly deferred out of 002 and lands here. It builds directly on the
existing trigger → overlay → select → output pipeline and reuses the same
geometry/crop core (Constitution IV). Annotation, OCR, beautify, and persisting
pins across app restarts remain out of scope.

### User Story 1 - Pin a capture as a floating window (Priority: P1)

As someone comparing two things on screen (a reference and my work, a design and
its build, a chat and a form), after I select a region I click **Pin** and the
captured image becomes a small borderless window that floats on top of every
other application. I can drag it anywhere, keep it visible while I switch apps,
open several at once, and close it when I'm done — all fully offline.

**Why this priority**: Pinning is the product's defining feature and the reason
it is called "PinShot". It delivers the largest unique value and exercises the
floating-window, always-on-top, and multi-monitor positioning machinery every
later pin-related feature (annotation on a pin, pin opacity, pin export) will
reuse. It is independently shippable as "capture → pin".

**Independent Test**: Capture a region, click Pin, then switch to another app and
confirm the pinned image stays visible on top; drag it to a new position; open a
second pin; close one and confirm the other remains. Verify the pinned pixels
match the captured region exactly.

**Acceptance Scenarios**:

1. **Given** a non-empty selection on the overlay, **When** the user invokes Pin, **Then** a borderless floating window showing exactly the captured region appears and the capture overlay closes.
2. **Given** a pinned window exists, **When** the user focuses or opens another application, **Then** the pinned window remains visible and stays above that application.
3. **Given** a pinned window, **When** the user drags it, **Then** it moves to follow the cursor and stays where released.
4. **Given** a pinned window, **When** the user invokes its close action (e.g., Esc while focused, a close affordance, or the assigned shortcut), **Then** that pin disappears and any other pins are unaffected.
5. **Given** one pin is open, **When** the user captures and pins a second region, **Then** both pins are visible and independently movable and closable.
6. **Given** a pinned window, **When** the user copies from it (clipboard action), **Then** the original captured image is placed on the clipboard, identical to the pinned pixels.

---

### User Story 2 - Adjust the selection before committing (Priority: P2)

As someone who rarely drags the perfect rectangle on the first try, after I
release the initial drag I can grab any edge or corner handle to resize the
selection, or drag inside it to move the whole rectangle, watching the live
dimensions update — and only then do I copy, save, or pin. I no longer have to
cancel and redraw from scratch.

**Why this priority**: It removes the most common friction in the existing
capture flow and makes every output (copy/save/pin) more precise, but capture
still works without it, so it layers on top of US1 and feature 002.

**Independent Test**: Drag a rough rectangle, release, then drag a corner handle
to enlarge it and an edge handle to shrink another side, then drag inside to
reposition; confirm the dimension readout tracks each change and that the
committed output matches the final adjusted rectangle, not the original drag.

**Acceptance Scenarios**:

1. **Given** a selection has been drawn and released, **When** the user hovers an edge or corner, **Then** a resize handle/affordance and an appropriate resize cursor are shown.
2. **Given** a selection with handles, **When** the user drags a corner handle, **Then** the two adjacent edges move together and the opposite corner stays anchored.
3. **Given** a selection with handles, **When** the user drags an edge handle, **Then** only that edge moves and the dimension readout updates to the exact new pixel size.
4. **Given** a selection, **When** the user presses and drags inside the rectangle (not on a handle), **Then** the whole rectangle moves without changing size.
5. **Given** an adjusted selection, **When** the user commits (copy/save/pin), **Then** the output contains exactly the adjusted rectangle's pixels.
6. **Given** a resize drag that would invert an edge past the opposite one, **When** the user crosses over, **Then** the rectangle normalises (no negative size) and the readout stays correct.

---

### User Story 3 - Clean selection visuals (Priority: P3)

As a user dragging the capture overlay, I see only PinShot's own selection
rectangle and dimmed surround — never the operating system's blue
text-selection highlight smeared across the frozen frame and on-screen text.

**Why this priority**: It is a visual-correctness defect in the current overlay
(the native selection highlight leaks through during a drag). It is small and
independent, but the overlay is usable without the fix, so it ranks below the
two functional additions.

**Independent Test**: Drag a selection across an area of the frozen frame that
contains text; confirm no blue native-selection highlight appears anywhere — only
PinShot's rectangle, handles, and dimmed backdrop are visible.

**Acceptance Scenarios**:

1. **Given** the capture overlay is shown, **When** the user drags to select, **Then** no native text/element selection highlight appears over the frame image or any HUD text.
2. **Given** a completed selection, **When** the user adjusts it (US2), **Then** the overlay still shows no native-selection highlight.

---

### Edge Cases

- **Pin larger than the display**: Pinning a near-fullscreen selection MUST keep the image at its full physical size (no down-scaling); the pin opens with its top-left on screen and the user drags it to reveal off-screen parts. It MUST remain movable and never trap the user with no visible grab area.
- **Pin on multi-monitor / mixed-DPI**: A pin created on a 2× display then dragged to a 1× display MUST render at the correct physical size and stay crisp; placement and size MUST be correct across the seam (the project's highest-risk area, mandatory test).
- **Display removed under a pin**: If the monitor a pin lives on is unplugged, the pin MUST move onto a remaining display rather than vanish off-screen, and MUST NOT crash.
- **Many pins**: Opening many pins MUST not leak memory or degrade idle CPU once they are static (Performance principle).
- **Always-on-top vs. native fullscreen**: Behaviour over another app in macOS native fullscreen / Windows exclusive-fullscreen MUST be defined and consistent (see Assumptions); it MUST NOT crash or trap input.
- **Zero/!tiny selection then Pin**: A zero-area selection MUST NOT create a pin (consistent with 002's zero-area rule); a very small selection MUST still produce a usable, draggable pin.
- **Resize to zero / inverted drag**: Shrinking an edge to or past zero MUST clamp to a sane minimum and never produce a negative-size or zero-area committable selection.
- **Handle vs. move vs. new-selection ambiguity**: Pressing on a handle resizes, inside moves, and outside the rectangle starts a brand-new selection — these MUST be unambiguous and discoverable.
- **Pin stacking order**: With several overlapping pins, the most recently interacted pin MUST come to the front; all MUST remain above normal application windows.
- **Cancel during adjust**: Pressing Esc while adjusting (before commit) MUST cancel the capture with the clipboard and filesystem unchanged (002 behaviour preserved).

## Requirements *(mandatory)*

### Functional Requirements

#### Floating Pin (US1)

- **FR-001**: Users MUST be able to pin a non-empty selection, producing a floating window that displays exactly the captured region's pixels.
- **FR-002**: A pinned window MUST stay above normal application windows (always-on-top) while the user works in other applications.
- **FR-003**: A pinned window MUST be movable by direct drag and stay where the user releases it.
- **FR-004**: Users MUST be able to close an individual pin without affecting other pins.
- **FR-005**: Multiple pins MUST be able to exist simultaneously, each independently movable, focusable, and closable.
- **FR-006**: The most recently interacted-with pin MUST render in front of other overlapping pins.
- **FR-007**: Users MUST be able to copy a pin's image back to the clipboard, byte-identical to the captured region.
- **FR-008**: A pinned window MUST be borderless/chromeless (no OS title bar) so it reads as a floating image, while still exposing discoverable move and close affordances.
- **FR-009**: Pin creation MUST be reachable from the selection toolbar and via a keyboard shortcut on the overlay, alongside the existing Copy and Save actions.
- **FR-010**: A pinned window's image MUST be correct in size and position on multi-monitor and mixed-DPI setups, including after being dragged between displays of different scale factors.
- **FR-011**: Pins are session-scoped: they MUST persist while the app runs and need not survive an app restart in this feature (persistence is a later concern).

#### Adjustable Selection (US2)

- **FR-012**: After the initial drag is released, the system MUST present resize handles on the selection's corners and edges.
- **FR-013**: Users MUST be able to resize the selection by dragging a corner handle (moving both adjacent edges) or an edge handle (moving one edge), with the opposite side(s) anchored.
- **FR-014**: Users MUST be able to move the whole selection by dragging inside it without changing its size.
- **FR-015**: The live dimension readout MUST update to the exact pixel width×height throughout any resize or move.
- **FR-016**: Resizing or moving MUST keep the selection a normalised, non-negative rectangle even when a drag inverts an edge past the opposite one, and MUST clamp to a minimum usable size rather than collapse to zero.
- **FR-017**: Committing any output (copy, save, or pin) after adjustment MUST use the final adjusted rectangle, not the original drag.
- **FR-018**: Pressing outside the existing selection MUST start a fresh selection (discarding the previous rectangle), preserving the current draw-from-scratch behaviour.

#### Selection Visuals (US3)

- **FR-019**: The capture overlay MUST NOT display the operating system's native text/element selection highlight during or after a drag; only PinShot's own rectangle, handles, dimmed backdrop, and HUD are shown.

#### Cross-cutting

- **FR-020**: The entire pin and selection-adjust flow MUST operate fully offline, issuing no network request (Privacy principle).
- **FR-021**: Selection-geometry operations (resize/move normalisation, clamping, and pin size/position math) MUST live in the headless core library and be unit-testable without a display (Clean Architecture principle).
- **FR-022**: The pin and adjustable-selection flows MUST behave equivalently on macOS and Windows (Cross-Platform Parity); the always-on-top contract MUST be defined identically where the OS allows.
- **FR-023**: Creating a pin from a capture MUST not regress the capture-overlay visibility budget (overlay still visible under 100ms from trigger); a pin window itself SHOULD appear promptly after the user confirms.

### Key Entities *(include if feature involves data)*

- **Selection Region**: A normalised rectangle in logical overlay coordinates with non-negative size and a minimum size; produced by the initial drag and mutated by resize/move before commit (extends the 002 entity).
- **Resize Handle**: One of the eight grab points (4 corners + 4 edges) on a selection; each maps to which edge(s) a drag moves and which stay anchored.
- **Pinned Image**: A captured image displayed in a floating window — carries the pixel data, its physical size, the display/scale it was captured at, and its current on-screen position and z-order.
- **Pin Window**: The floating, borderless, always-on-top host for a Pinned Image; one per pin, with move/close affordances and a stacking order relative to other pins.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After confirming Pin, a floating window showing the exact captured region appears, and it remains visible above a different, focused application without the user taking any extra step.
- **SC-002**: A user can drag a pin to any position on any connected display and it renders at the correct physical size with no DPI scaling or positional error, verified on single-display, multi-display, and mixed-DPI setups on both macOS and Windows.
- **SC-003**: Starting from a released selection, a user can enlarge, shrink, and reposition it to a target rectangle and commit, with the output matching the final rectangle exactly — completed without cancelling and redrawing.
- **SC-004**: The dimension readout equals the committed output's true pixel width×height during and after every resize/move.
- **SC-005**: No native blue selection highlight is visible at any point during a drag or adjust, confirmed visually over text-bearing content.
- **SC-006**: Multiple pins can be open at once; closing one leaves the others intact, and the most recently touched pin is frontmost.
- **SC-007**: Zero network requests occur during pinning and selection adjustment, verifiable with a network monitor.
- **SC-008**: With several static pins open, idle CPU stays effectively zero and memory stays within the always-running-app budget (Performance principle).

## Assumptions

- **Scope of pin actions (v1)**: A pin supports move (drag), close (double-click the pin), front-on-interact, and copy-to-clipboard. Annotating on a pin, changing opacity, resizing/zooming the pinned image, click-through, and saving from the pin are deferred to later features.
- **Pin persistence**: Pins live only for the current app session; restoring pins after an app restart is out of scope here and may become a later "pin history" feature (subject to the local-storage / privacy rules).
- **Always-on-top contract**: "On top" means above normal application windows. Behaviour relative to another app's native/exclusive fullscreen follows what each OS permits for a floating utility window and is documented rather than guaranteed identical; it must never crash or steal input.
- **Pin sizing**: A pin always shows the captured region at its full physical size (rendered at the owning display's logical scale); it is never auto-scaled, even when larger than the screen (the user drags to reveal more). A configurable default zoom is a later concern.
- **Adjust interaction model**: Eight handles (4 corners + 4 edges); inside-drag moves; outside-press starts a new selection; Esc cancels; the existing magnifier/color/dimension HUD from 002 remains available during selection.
- **Minimum selection size**: A small but non-zero minimum (a few logical pixels) is enforced so handles remain grabbable and outputs stay valid; the exact value is a planning decision.
- **Keyboard shortcut for Pin**: A sensible default overlay shortcut is added for Pin alongside Copy (↵) and Save (S); remapping is out of scope (consistent with 002's hotkey assumption).
- **Architecture**: Resize/move normalisation, clamping, and pin size/position math are added to `pinshot-core` behind the existing ports; floating-window creation, always-on-top, and drag are provided by the Tauri shell; the overlay/pin UI lives in the TypeScript layer and forwards intents over the existing thin IPC surface (Constitution IV). The specific windowing/clipboard mechanisms are planning decisions, not part of this spec.
- **Builds on 002**: The trigger → overlay → select → output pipeline, the frozen-frame model, and the crop/encode core from feature 002 are reused unchanged except where this spec extends them.
