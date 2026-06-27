# Feature Specification: Floating Annotation Editor & Smart Capture Toolkit

**Feature Branch**: `004-floating-annotation-editor`

**Created**: 2026-06-27

**Status**: Draft

**Input**: Design mockup (`image.png`, "PinShot — Capture. Annotate. Pin.") plus
product brief: turn PinShot's one-shot capture into a fast, floating,
keyboard-first annotation experience — a horizontal floating toolbar with
contextual tool properties, a floating action bar (Pin / Copy / Save), an
infinite undo/redo history, smart tools (QR detection, color picker,
spotlight, magnifier, step numbers, crop), a background tray / menu-bar app with
no main window, and a dedicated Settings window. **No share / cloud / upload /
shareable-URL feature** — everything stays local.

---

## Product Vision *(deliverable 1)*

PinShot is **not** an image editor. It is a lightweight, privacy-first
screenshot utility whose entire reason to exist is **speed**:

```
Hotkey → Capture → Annotate → Copy / Save / Pin   (a few seconds, start to finish)
```

This feature delivers the **"float shot"** experience: after a capture, a
**floating, contextual, keyboard-first** editor appears directly over the work —
never a Photoshop-style window with permanent sidebars, never a separate
document-management app. The editor is a horizontal toolbar plus a small action
bar that expands only to show what the current tool needs, then gets out of the
way.

It must **feel native** on macOS and Windows: instant, smooth (60 fps),
animated, and silent in the background until summoned.

**This feature is explicitly NOT**: a layered photo editor, a design tool
(Figma/Canva), a paint program (MS Paint/Photoshop), or anything that uploads,
syncs, or shares over a network. Local-first, offline-only, no accounts. See
[Non-Goals](#non-goals).

---

## User Personas *(deliverable 2)*

| # | Persona | Context | Primary jobs-to-be-done | What "fast" means to them |
|---|---|---|---|---|
| P1 | **Dana, the developer** | Files bug reports, documents APIs, shares error messages | Capture an error → arrow + box the cause → copy into the issue; redact a token before pasting | Hotkey-to-pasted-annotated-image in < 10 s, hands on keyboard |
| P2 | **Tomas, the technical writer** | Writes tutorials & docs | Add numbered steps, spotlight a UI region, blur a token, save a crisp PNG | Numbered callouts and redaction without leaving the keyboard |
| P3 | **Mira, the support agent** | Explains fixes to customers all day | Box the button to click, pin a reference next to the customer's screen, copy | Reuse the same few tools instantly, every ticket |
| P4 | **Sven, the privacy-conscious office worker** | Shares internal dashboards | Redact sensitive data (blur/pixelate), confirm nothing leaves the machine | Trustworthy redaction; zero network, verifiably |
| P5 | **Aiko, the researcher** | Captures figures, diagrams & references | Detect a QR/URL in a figure, pick exact brand colors, magnify a detail | One-tap offline QR/URL + color, zero network |

All personas share the same expectation: **the tool disappears between uses**
(tray/menu bar only) and **reappears instantly** on a hotkey.

---

## Clarifications

### Session 2026-06-27

- Q: After a region is selected, does the editor always open or is there an express (no-editor) output path? → A: The editor **always** opens; instant in-editor Copy/Save/Pin shortcuts serve as the express path (no separate express mode).
- Q: Should OCR (text extraction) be part of this feature? → A: **No** — OCR is removed and deferred to the roadmap (v0.3); Search/Translate are dropped with it.
- Q: Should QR code detection stay? → A: **Yes** — QR detection remains an offline, in-core smart feature (Copy URL / Open URL).
- Q: When pinning from the editor, does the pin store a flat image or the editable annotations? → A: The pin **keeps the editable annotation document**; pixels are flattened only on copy/save (so annotate-after-pin is native).
- Q: What happens to annotations when cropping after annotating? → A: Crop **reframes and keeps** all annotations; those outside the new frame are clipped on export (not deleted), reversible via undo.

---

## User Scenarios & Testing *(mandatory)*

This feature builds directly on the existing pipeline — **002** (trigger →
overlay → select → crop → clipboard/file) and **003** (adjustable selection +
floating pin). After a region is selected and committed, a **floating annotation
editor always opens** over the frozen capture (Clarification Q1).
The same headless `pinshot-core` crop/encode core is reused; this spec adds the
annotation model, flatten/compositing, smart tools, the app shell (tray +
Settings), and export options.

User stories are ordered by priority. Each is an independently shippable slice
that leaves PinShot more capable without depending on a later story.

### User Story 1 - Annotate a capture and copy/save/pin it (Priority: P1)

As Dana, after I select a region the **floating annotation toolbar appears right
over the capture**. I draw a red rectangle around the broken metric and an arrow
pointing at it, hit **C** to copy, and paste the annotated image into my issue —
without ever touching a sidebar, a menu, or the mouse beyond drawing. If I'd
rather keep it on screen, I hit **P** to pin it.

**Why this priority**: This is the defining "float shot" experience and the
product's core loop (Capture → Annotate → Copy/Save/Pin). It delivers the
largest unique value, exercises the floating-editor, annotation-object, flatten,
and output machinery every later story reuses, and is independently shippable as
"capture → annotate → output".

**Independent Test**: Capture a region; confirm a floating horizontal toolbar
(not a sidebar) appears over the image. Draw a rectangle and an arrow, add text;
press Copy and verify the clipboard image contains the flattened annotations
exactly as drawn; press Save and verify the file matches; press Pin and verify a
floating pin shows the annotated result. Confirm Esc cancels with clipboard and
filesystem unchanged.

**Acceptance Scenarios**:

1. **Given** a committed selection, **When** the user enters edit, **Then** a floating horizontal toolbar and a floating action bar (Pin / Copy / Save / More) appear over the captured image, with no left/right sidebar and no separate editor window chrome.
2. **Given** the Rectangle tool is active, **When** the user drags on the image, **Then** a rectangle annotation is created live and remains an editable object (selectable, movable, restyle-able, deletable).
3. **Given** the Arrow tool, **When** the user drags while holding Shift, **Then** the arrow snaps to straight angles; **When** the user scrolls, **Then** the stroke thickness changes live.
4. **Given** one or more annotations, **When** the user presses Copy (C), **Then** the captured image with all annotations flattened on top is placed on the clipboard, pixel-identical to what is shown.
5. **Given** annotations, **When** the user presses Save (S), **Then** a file containing the flattened result is written to the configured folder in the configured format.
6. **Given** annotations, **When** the user presses Pin (P), **Then** a floating, always-on-top pin of the flattened result appears (reusing feature 003), and the editor closes.
7. **Given** the editor is open, **When** the user presses Esc, **Then** the editor closes and the clipboard and filesystem are unchanged.

---

### User Story 2 - Contextual tool properties, full toolset & history (Priority: P2)

As Tomas, when I pick a tool the **toolbar expands to show only that tool's
properties** — for Rectangle: stroke color, thickness, fill, opacity, corner
radius; for Text: font, size, weight, background, shadow. I switch tools entirely
from the keyboard, restyle a shape after drawing it, and use an **infinite
undo/redo** with a visible **history** of every annotation (Rectangle → Arrow →
Text → Highlighter → Blur).

**Why this priority**: It turns the editor from "draw and ship" into a precise,
repeatable tool, and adds the full annotation toolset and the history/undo
machinery. It layers on top of US1 (which already works without per-tool
property panels) and is independently demonstrable.

**Independent Test**: Select Rectangle; confirm the toolbar shows exactly its
properties (stroke/thickness/fill/opacity/radius) and nothing else. Draw a shape,
change its stroke color and opacity after the fact, and confirm it updates live.
Add five different annotations; confirm they appear in the history panel in order;
undo three times and redo twice and confirm the canvas and history match; clear
history and confirm an empty canvas.

**Acceptance Scenarios**:

1. **Given** any tool is selected, **When** it becomes active, **Then** the toolbar expands to show only that tool's properties and collapses them when another tool is chosen.
2. **Given** a selected existing annotation, **When** the user changes a property (e.g., stroke color), **Then** that object updates live and the new value becomes the default for the next object of that type.
3. **Given** the full toolset, **When** the user presses a tool's shortcut, **Then** that tool activates (Select, Rectangle, Ellipse, Arrow, Line, Pencil, Highlighter, Text, Blur, Pixelate, Eraser, plus the smart tools from US4/US5).
4. **Given** a sequence of annotations, **When** the user opens the history panel, **Then** each annotation appears as an entry in creation order.
5. **Given** any editing state, **When** the user presses Undo / Redo repeatedly, **Then** annotations are removed/restored in exact reverse/forward order with no limit within the session.
6. **Given** a non-empty history, **When** the user clears history, **Then** all annotations are removed and the canvas shows the unannotated capture.

---

### User Story 3 - Background app, menu bar / tray & Settings (Priority: P2)

As Mira, PinShot **never opens a window when it launches** — it lives in the
macOS **menu bar** (top-right) / Windows **system tray**. Clicking its icon shows
a small menu: **Capture / Settings / About / Check Updates / Quit**. Only when I
choose **Settings** does a normal window open, where I customize hotkeys, capture
behavior, annotation defaults, export format, theme, and language.

**Why this priority**: It is the native, "disappears between uses" shell that
makes the whole product feel like CleanShot X / Raycast rather than a desktop
app, and it is where every shortcut and default in US1/US2/US4/US5 is configured.
It is independently shippable (a tray app + Settings window) and is required for
the keyboard-first promise (recordable hotkeys).

**Independent Test**: Launch the app; confirm no window and no dock/taskbar app
window appears, only a menu-bar/tray icon. Click the icon; confirm the menu
(Capture / Settings / About / Check Updates / Quit). Choose Capture and confirm
it starts a capture; choose Settings and confirm a normal window opens. In
Settings, record a new hotkey for Capture Region, trigger it, and confirm it
works; toggle theme and confirm the editor honors it.

**Acceptance Scenarios**:

1. **Given** the app starts (login or manual), **When** it finishes launching, **Then** no main/editor window and no dock/taskbar window is shown — only a menu-bar (macOS) / system-tray (Windows) icon.
2. **Given** the tray/menu-bar icon, **When** the user clicks it, **Then** a lightweight menu appears with Capture, Settings, About, Check for Updates, and Quit.
3. **Given** the menu, **When** the user selects Settings, **Then** a normal application window opens (the only normal window the app ever shows).
4. **Given** the Hotkeys settings, **When** the user records a new shortcut for an action, **Then** the new shortcut is captured, conflicts are detected and flagged, and the action responds to the new shortcut afterward.
5. **Given** the General settings, **When** the user changes theme / language / launch-at-login, **Then** the change persists across restarts and is reflected in the UI.
6. **Given** Settings changes, **When** the app restarts, **Then** settings load from a local human-readable file with no network access.

---

### User Story 4 - Smart code capture: QR detection (Priority: P3)

As Aiko, if a capture contains a **QR code** PinShot detects it **automatically
and entirely offline** and offers **Copy URL** (and, for URLs, **Open URL**) — no
internet round-trip to read the code, and the screenshot itself never leaves the
machine.

**Why this priority**: QR detection is a high-delight, low-cost smart feature that
reuses the captured-image pipeline and is fully offline. It sits above the core
editor (US1/US2) and shell (US3). (Offline **OCR** text extraction is a separate,
later roadmap item — v0.3 — and is intentionally **not** part of this feature.)

**Independent Test**: Capture an area containing a QR code; confirm PinShot offers
Copy URL and (for a URL) Open URL with the correctly decoded value. Capture an
area with no code; confirm a graceful "no code found". Verify with a network
monitor that detection issues **no** network request and that Open URL only
launches the OS browser on an explicit click.

**Acceptance Scenarios**:

1. **Given** a captured image containing a QR code or supported barcode, **When** edit runs, **Then** PinShot detects it **offline** and offers **Copy URL** / **Open URL** using the decoded value.
2. **Given** a decoded URL, **When** the user invokes **Open URL**, **Then** PinShot hands the URL to the OS default browser by explicit user action only (never silently), and the screenshot itself is never transmitted.
3. **Given** a captured image with **multiple** codes, **When** detection runs, **Then** PinShot offers a choice among the decoded values.
4. **Given** no decodable code is present, **When** detection runs, **Then** PinShot reports "no code found" gracefully without error and without any network call.

---

### User Story 5 - Visual smart tools: spotlight, magnifier, step numbers, color, crop (Priority: P3)

As Tomas making documentation, I use **Step Numbers** that auto-increment
(1, 2, 3 …) with editable colors, **Spotlight** to darken everything except a
region, a **Magnifier** loupe to zoom a detail, the **Color Picker** to read a
pixel's HEX/RGB/HSL, and **Crop/Resize** (Free / 1:1 / 16:9 / 4:3) to re-frame
after capture.

**Why this priority**: These are the "smart features" that make documentation and
design-review screenshots shine. They are each small, independent tools layered
on the US1/US2 annotation model and are roadmap-polish (v0.5), so they rank below
QR detection.

**Independent Test**: Use each tool in turn on a capture — place three step
numbers and confirm they auto-increment and recolor; apply Spotlight and confirm
everything outside the region dims; place a Magnifier and adjust its zoom; pick a
color and confirm HEX/RGB/HSL with copy; crop to 16:9 and confirm the output
matches the new frame. Confirm each is undoable.

**Acceptance Scenarios**:

1. **Given** the Step Number tool, **When** the user clicks repeatedly, **Then** markers appear numbered 1, 2, 3 … with a chosen color, and renumber consistently if one is deleted.
2. **Given** the Spotlight tool, **When** the user marks a region, **Then** everything outside that region is darkened in the output.
3. **Given** the Magnifier tool, **When** the user places a loupe and adjusts zoom, **Then** a circular magnified view of the underlying pixels is rendered into the image.
4. **Given** the Color Picker, **When** the user points at a pixel, **Then** its color is shown as HEX/RGB/HSL and any representation can be copied.
5. **Given** the Crop tool, **When** the user re-crops with a Free/1:1/16:9/4:3 constraint and commits, **Then** the output contains exactly the new frame.
6. **Given** any of these tools, **When** the user undoes, **Then** the effect is removed like any other annotation.

---

### User Story 6 - Advanced floating pin (Priority: P4)

As Mira, after pinning a reference I can adjust its **opacity**, toggle
**click-through** so it floats as a ghost over my work, **resize** it, and even
**add annotations after pinning** — extending the basic pin from feature 003.

**Why this priority**: It deepens the namesake pin into a power-user reference
surface, but the basic pin (003) already delivers the core value, so these
enhancements rank last.

**Independent Test**: Pin a capture; lower its opacity and confirm it becomes
translucent; enable click-through and confirm clicks pass to the app beneath;
resize the pin and confirm crisp scaling; draw an annotation on the pin and
confirm it persists on that pin.

**Acceptance Scenarios**:

1. **Given** a pin, **When** the user adjusts opacity, **Then** the pin becomes more/less translucent and stays always-on-top.
2. **Given** a pin, **When** the user enables click-through, **Then** mouse events pass to the window beneath, and there remains a discoverable way to disable click-through.
3. **Given** a pin, **When** the user resizes/zooms it, **Then** the image scales and stays crisp at the new size on any display/DPI.
4. **Given** a pin, **When** the user annotates it, **Then** the annotation is drawn on that pin and is included if the pin is subsequently copied or saved.

---

### Edge Cases

- **Empty/zero-area selection → Edit**: A zero-area selection MUST NOT open an editor (consistent with 002's zero-area rule); a tiny selection MUST still open a usable editor.
- **Capture spanning mixed-DPI displays**: The editor MUST show and flatten the capture at correct physical size; outputs MUST be DPI-exact across the display seam (highest-risk area, mandatory test).
- **Very large capture**: The editor MUST remain responsive (60 fps interactions) and stay within the idle/active memory budget; flatten/encode MUST not block the UI thread.
- **Undo across tool switches & smart tools**: Undo/redo MUST treat blur, spotlight, magnifier, crop, and step numbers as ordinary history entries; redo after a new edit MUST discard the orphaned redo branch predictably.
- **Crop after annotating**: Re-cropping MUST reframe the base and keep all annotations correctly positioned relative to the new frame; annotations falling outside the new frame MUST be clipped in the exported output but retained in the document, so crop is non-destructive and reversible via undo.
- **Blur/pixelate of moving region**: Editing a blur region's bounds MUST re-derive the obscured pixels from the original capture, never from already-blurred output (no irreversible double-blur during editing).
- **QR detection edge cases**: Detection MUST handle 0, 1, or many codes — reporting "no code found" gracefully and offering a choice when more than one is found; decoding MUST never hang or call the network.
- **Hotkey conflict**: Recording a shortcut already used by PinShot or reserved by the OS MUST be detected and the user warned before saving.
- **Theme = system + OS theme change**: Switching OS appearance while the editor is open MUST update the UI without a restart.
- **Click-through pin with no escape**: A click-through pin MUST always retain a non-click-through way to regain control (e.g., a hotkey or the tray menu) so the user is never trapped.
- **Settings file missing/corrupt**: MUST fall back to documented defaults and rewrite a valid file, never crash.
- **Quit with unsaved annotations / open pins**: Behavior MUST be defined (e.g., quitting closes pins; an open editor with unsaved work warns or is explicitly discardable).

---

## Requirements *(mandatory)*

### Functional Requirements

#### App shell, tray / menu bar & entry (US3)

- **FR-001**: On launch (manual or at login) the app MUST start silently with **no main/editor window** and **no dock/taskbar application window** — it MUST present only a macOS menu-bar item / Windows system-tray icon as its primary entry point.
- **FR-002**: Clicking the tray/menu-bar icon MUST open a lightweight menu containing **Capture**, **Settings**, **About**, **Check for Updates**, and **Quit**.
- **FR-003**: Selecting **Settings** MUST open a normal application window; this MUST be the only normal (non-overlay, non-pin) window the app ever displays.
- **FR-004**: Global capture hotkeys MUST trigger a capture regardless of which application is focused, with no PinShot window required.
- **FR-005**: The app MUST offer a **Launch at Login** option (default OFF) that survives restarts.

#### Capture → editor handoff (US1)

- **FR-006**: After a non-empty selection is committed to **Edit**, the system MUST open a floating annotation editor hosting the captured region's pixels.
- **FR-007**: The editor MUST display the capture at its correct physical size and DPI on the display where it was captured, reusing the 002 frozen-frame / crop pipeline.
- **FR-008**: A zero-area selection MUST NOT open an editor; the existing 002 cancel semantics MUST be preserved.

#### Floating editor & toolbar (US1)

- **FR-009**: The editor MUST present a single **floating, horizontal toolbar** for tools and a **floating action bar** for outputs — it MUST NOT use a left sidebar, a right sidebar, or a Photoshop-style multi-panel layout.
- **FR-010**: The floating toolbar and action bar MUST be repositionable (draggable) and remain visually associated with the capture.
- **FR-011**: The action bar MUST offer **Pin**, **Copy**, **Save**, and **More** — and MUST NOT offer any Share / upload / cloud / shareable-URL action.
- **FR-012**: The editor MUST be keyboard-first: every tool and every action MUST have a keyboard shortcut, and **Esc** MUST cancel/close the editor with no side effects.

#### Annotation tools & objects (US1 core, US2 full)

- **FR-013**: The editor MUST provide these annotation tools: **Select/Move, Rectangle, Ellipse, Arrow, Line, Pencil (freehand), Highlighter, Text, Blur, Pixelate, Eraser**, plus the smart tools defined in US4/US5 (**Color Picker, Spotlight, Magnifier, Step Number, Crop**).
- **FR-014**: Holding **Shift** MUST constrain lines/arrows to fixed angles and shapes to 1:1 (square/circle); scrolling MUST adjust the active stroke thickness.
- **FR-015**: Each annotation MUST be an **editable object**: after creation it can be selected, moved, resized, restyled, reordered, and deleted; it MUST NOT be irreversibly baked into the image until output.
- **FR-016**: Annotations MUST render in a deterministic **z-order** (later objects above earlier ones), reflected in the history.

#### Contextual tool properties (US2)

- **FR-017**: Selecting a tool MUST expand the toolbar to show **only** that tool's properties, and collapse them when another tool is selected.
- **FR-018**: Each tool MUST expose its relevant properties, at minimum: **Rectangle/Ellipse** — stroke color, thickness, fill on/off, fill color, opacity, corner radius (rectangle); **Arrow/Line** — color, thickness, arrowhead style, dashed; **Pencil/Highlighter** — color, thickness, opacity; **Text** — font, size, weight, color, background, shadow, opacity; **Blur** — strength; **Pixelate** — block size; **Step Number** — color (and starting index); **Spotlight** — dim strength; **Magnifier** — zoom level; **Highlighter** — opacity.
- **FR-019**: Changing a property MUST update any selected object live **and** set the default for subsequently created objects of that type.

#### History & undo / redo (US2)

- **FR-020**: The editor MUST maintain an ordered **annotation history** (stack) and present it in a history panel listing each annotation by type in creation order.
- **FR-021**: The editor MUST support **unlimited undo and redo** within an editing session, restoring exact state in order; a new edit after undo MUST discard the stale redo branch.
- **FR-022**: The editor MUST provide a **Clear History** action that removes all annotations and returns to the unannotated capture.

#### Outputs / action bar (US1)

- **FR-023**: **Copy** MUST flatten all annotations onto the capture and place the result on the clipboard, pixel-identical to what is shown.
- **FR-024**: **Save** MUST flatten and write a file in the configured format and location with the configured filename pattern.
- **FR-025**: **Pin** MUST create a floating, always-on-top pin that carries the **editable annotation document** (pixels flattened only on copy/save), reusing feature 003 — so annotate-after-pin (US6) is the same model, not a separate layer.
- **FR-026**: Output actions MUST be reachable both from the action bar and via keyboard shortcuts (Pin/Copy/Save).

#### Smart tools — QR detection (US4)

- **FR-027**: **QR / barcode detection** MUST run **fully offline** in `pinshot-core` on the captured image, handling zero, one, or many codes.
- **FR-028**: When one or more codes are found, PinShot MUST offer **Copy** of each decoded value (e.g., **Copy URL**) — a fully offline clipboard action.
- **FR-029**: For codes whose value is a URL, PinShot MUST offer **Open URL** as an **explicit, user-initiated** hand-off to the OS default browser (never silent, never sending the image itself); PinShot's core MUST NOT perform the network request.

> Offline **OCR** (text extraction) and any **Search/Translate** on extracted text are **out of scope** for this feature and deferred to the roadmap (see [Future Roadmap](#future-roadmap-deliverable-20)).

#### Smart tools — visual (US5)

- **FR-030**: The **Color Picker** MUST read a pixel's color and present it as **HEX, RGB, and HSL**, each individually copyable.
- **FR-031**: The **Spotlight** tool MUST darken everything outside a chosen region in the output, with adjustable dim strength.
- **FR-032**: The **Magnifier** tool MUST render a circular loupe of the underlying pixels at an adjustable zoom into the image.
- **FR-033**: The **Step Number** tool MUST place auto-incrementing numbered markers (1, 2, 3 …) with an editable color, renumbering consistently when a marker is added or removed.
- **FR-034**: The **Crop/Resize** tool MUST allow re-cropping the capture after the fact with **Free, 1:1, 16:9, and 4:3** constraints; committing MUST reframe the base so the output contains exactly the new frame while keeping existing annotations — any falling outside the new frame are clipped on export, not deleted, and the crop is reversible via undo.

#### Advanced pin (US6)

- **FR-035**: A pin MUST support **opacity** adjustment while remaining always-on-top.
- **FR-036**: A pin MUST support a **click-through** mode that passes mouse events to the window beneath, with an always-available way to disable it (so the user is never trapped).
- **FR-037**: A pin MUST support **resize/zoom** of its image, staying crisp across displays/DPI.
- **FR-038**: A pin MUST support **annotation after pinning**, and such annotations MUST be included when the pin is copied or saved.

#### Settings (US3)

- **FR-039**: **General** settings MUST include Launch at Login, Check for Updates (opt-in, default OFF), Theme (Light / Dark / System), and Language.
- **FR-040**: **Capture** settings MUST include capture modes (Region, Window, Full Screen), a Delay Timer, Include Cursor, and Include Shadow.
- **FR-041**: **Hotkeys** settings MUST let the user customize **every** shortcut, record a new shortcut by pressing keys, and MUST detect and warn on conflicts (with PinShot's own bindings and, where detectable, OS-reserved ones).
- **FR-042**: **Annotation** settings MUST expose defaults for Stroke Color, Fill Color, Font, Font Size, Arrow Size, Highlighter Opacity, Blur Strength, and Pixelate Size.
- **FR-043**: **Export** settings MUST include Default Format (PNG / JPG / WebP), Default Filename pattern, Clipboard behavior, and Compression level.
- **FR-044**: **Advanced** settings MUST include a Developer Mode toggle, access to Logs, and a Reset Settings action.
- **FR-045**: All settings MUST persist as a **local, human-readable** file (TOML/JSON) and load with no network access; a missing/corrupt file MUST fall back to documented defaults.

#### Cross-cutting (constitution-derived)

- **FR-046**: The entire feature MUST operate **fully offline** — no component of the core may issue a network request. The update check MUST remain isolated, opt-in, and default OFF, fetching only a static version file when explicitly invoked.
- **FR-047**: PinShot MUST NOT provide any **Share / cloud / upload / shareable-URL** capability anywhere; the only outputs are clipboard, local file, pin, and OS-level drag-and-drop. (The "Share" item in the source mockup is intentionally excluded.)
- **FR-048**: The annotation model, flatten/compositing, blur/pixelate/spotlight/magnifier pixel operations, QR decode, color math, crop, and PNG/JPG/WebP encoding MUST live in the headless `pinshot-core` crate and be **unit-testable without a display**; platform-specific clipboard/window/capture behavior MUST sit behind traits.
- **FR-049**: All behavior MUST be **equivalent on macOS and Windows** (Cross-Platform Parity), including mixed-DPI multi-monitor correctness for the editor, smart tools, and pins.
- **FR-050**: The feature MUST meet the performance budgets in [Non-Functional Requirements](#non-functional-requirements-deliverable-4).

### Non-Functional Requirements *(deliverable 4)*

- **NFR-001 (Startup)**: Cold app startup to "ready in tray, hotkeys armed" MUST be **< 300 ms** on baseline hardware.
- **NFR-002 (Capture latency)**: Hotkey → capture overlay visible MUST be **< 100 ms** (cold), unchanged from the constitution gate; selection-commit → editor visible SHOULD be near-instant.
- **NFR-003 (Animation)**: Toolbar, contextual-panel, and tool interactions MUST animate at **60 fps**; no interaction may block the UI thread.
- **NFR-004 (Memory)**: Idle memory footprint MUST stay **< 120 MB**; editing a typical capture and holding a few pins MUST stay within an always-running-app budget.
- **NFR-005 (Native feel)**: The UI MUST feel native on each OS — system theme awareness, platform-correct shortcut glyphs (⌘ vs Ctrl), smooth animations, and no flashing/placeholder windows.
- **NFR-006 (Offline-verifiable)**: Zero network requests MUST be observable with a network monitor across the entire capture → annotate → smart-tool → output flow.
- **NFR-007 (Idle cost)**: With the app idle in the tray, CPU usage MUST be effectively zero.

### Key Entities *(deliverable 16 summary — full detail in [data-model.md](./data-model.md))*

- **EditSession**: One in-flight edit of a capture — owns the source image, the annotation document, the history, the active tool, and per-tool default properties.
- **CaptureImage**: The cropped source pixels from 002 (RGBA + physical size + display/scale); the immutable base layer.
- **Annotation**: An editable vector/effect object placed on the capture — has a type (Rectangle, Ellipse, Arrow, Line, Pencil, Highlighter, Text, Blur, Pixelate, Spotlight, Magnifier, StepNumber), geometry, style, and z-index.
- **ToolProperties**: The current style values for each tool (stroke/fill/opacity/font/etc.), shown in the contextual panel and used as defaults for new objects.
- **HistoryStack**: The ordered command/annotation history enabling unlimited undo/redo and the history panel.
- **SmartResult**: Output of a smart tool — `QrResult` (decoded values + regions) and `ColorSample` (HEX/RGB/HSL).
- **ExportProfile**: Format (PNG/JPG/WebP), filename pattern, compression, and clipboard behavior used by Save/Copy.
- **Settings**: The persisted, human-readable configuration (General, Capture, Hotkeys, Annotation, Export, Advanced).
- **Hotkey**: A user-customizable binding from a key chord to an action, with conflict state.
- **Pin** *(extends 003)*: A floating image window — now also carrying opacity, click-through, scale/zoom, and an optional own annotation document.

---

## UX Flow *(deliverable 5)*

**Primary loop (Capture → Annotate → Output):**

```
[idle: tray/menu-bar only]
        │  Capture hotkey (e.g. ⌘⇧A) — or tray ▸ Capture
        ▼
[capture overlay]  freeze screen → drag/adjust selection (002/003)
        │  release / confirm  ──►  editor ALWAYS opens (instant C/S/P = express)
        ▼
[floating editor]  toolbar + action bar appear over the capture
        │  pick tool (keyboard) → toolbar shows that tool's properties
        │  draw / restyle / undo-redo  (history panel tracks the stack)
        ▼
[output]  Copy (C) │ Save (S) │ Pin (P) │ More
        ▼
[back to idle]  editor closes; pins (if any) keep floating
```

**Smart-tool sub-flows:**

```
QR:   detected on edit (offline)  →  Copy URL │ Open URL*
Color:tool ▸ Color Picker  →  hover pixel  →  HEX/RGB/HSL  →  copy
Crop: tool ▸ Crop  →  Free│1:1│16:9│4:3  →  reframe (keeps annotations)
```
\* Open URL is an explicit, user-initiated hand-off to the OS browser (FR-029).

**Settings flow:** tray ▸ Settings → normal window → tabs (General / Capture /
Hotkeys / Annotation / Export / Advanced) → change persists to local file → tray
app and next editor honor it.

---

## Information Architecture *(deliverable 6)*

```
PinShot (background agent, no main window)
├── Menu bar / System tray            ← primary entry point
│   ├── Capture            (starts capture)
│   ├── Settings           (opens the only normal window)
│   ├── About
│   ├── Check for Updates  (opt-in, default off)
│   └── Quit
├── Capture overlay        (002/003)  ← transient, per display
│   └── Selection + adjust + magnifier/color HUD
├── Floating editor        (this feature) ← transient, over the capture
│   ├── Floating toolbar           (horizontal: tools)
│   │   └── Contextual properties  (expands per active tool)
│   ├── Canvas                     (capture + annotation objects)
│   ├── History panel              (annotation stack, clear)
│   └── Floating action bar        (Pin · Copy · Save · More)
│       └── Smart results          (QR URL · Color values)
├── Pin windows            (003 + US6)  ← persistent until closed
│   └── opacity · click-through · resize · annotate
└── Settings window        (US3)        ← on demand
    ├── General · Capture · Hotkeys · Annotation · Export · Advanced
```

The app has exactly **one** normal window (Settings); everything else is a
transient overlay or a borderless floating surface — reinforcing the
"contextual, floating, disappears-between-uses" philosophy.

---

## Screen Specifications *(deliverable 7)*

### S1 — Menu bar / System tray menu
- **Trigger**: click the icon. **Surface**: native popover/menu, not a webview window.
- **Items**: Capture · Settings · About · Check for Updates · Quit (FR-002).
- **States**: idle icon; (future) subtle activity indicator during capture. No counts/badges that imply network.

### S2 — Floating editor (the "float shot" screen)
- **Layout**: the capture image centered; a **horizontal floating toolbar** detached above/below it; a **floating action bar** below; an optional **history panel** and **contextual properties** panel that appear contextually — **never** docked sidebars (FR-009).
- **Toolbar (tools, left→right)**: Select · Rectangle · Ellipse · Arrow · Line · Pencil · Highlighter · Text · Blur · Pixelate · Spotlight · Magnifier · Color Picker · Step Number · Crop · Eraser · Undo · Redo · More.
- **Contextual properties**: replaces the tool row's extension with the active tool's controls only (FR-017/FR-018); e.g., Rectangle → Stroke (swatches + custom) · Thickness (1/2/4/8) · Fill (toggle + color) · Opacity (slider) · Radius.
- **Action bar**: Pin (P) · Copy (C) · Save (S) · More (FR-011). No Share.
- **History panel**: ordered list of annotations with type icons + Clear History (FR-020/FR-022).
- **Affordances/Hints**: footer micro-hints ("Drag to move · Hold Shift for straight lines · Scroll to change thickness · Esc to cancel"); selection handles on the active object; live dimension/coordinate readout reused from 002.
- **Theme**: honors Light/Dark/System (NFR-005).

### S3 — Smart result surfaces
- **QR chip**: detected value(s) with **Copy URL** and (for URLs) **Open URL** (FR-027–FR-029).
- **Color readout**: swatch + HEX/RGB/HSL rows, each copyable (FR-030).
- **Crop bar**: Free / 1:1 / 16:9 / 4:3 toggles + commit (FR-034).

### S4 — Pin window
- Borderless, always-on-top image (003); adds opacity slider, click-through toggle, resize handles, close (US6/FR-035–FR-038). Discoverable but chrome-light.

### S5 — Settings window (the only normal window)
- Left tab rail (this is a settings window, not the editor — tabs are acceptable here): **General · Capture · Hotkeys · Annotation · Export · Advanced**.
- Each tab renders the controls in FR-039–FR-044; a hotkey row enters "recording" state on click (FR-041).

---

## Component Specifications *(deliverable 8 — product level; engineering detail in [plan.md](./plan.md))*

| Component | Responsibility | Key states / props | Notes |
|---|---|---|---|
| `FloatingToolbar` | Host tool buttons; reflect active tool | active tool, hover, disabled | Horizontal only; draggable; keyboard-navigable |
| `ContextualProperties` | Show only the active tool's controls | tool, current values | Expands/collapses on tool change (FR-017) |
| `Canvas` | Render capture + annotation objects; hit-test; live draw | objects[], selection, zoom | 60 fps; non-blocking (NFR-003) |
| `AnnotationObject` | One editable shape/effect | type, geometry, style, z | Editable until output (FR-015) |
| `HistoryPanel` | List + navigate the stack | entries[], cursor | Clear History (FR-022) |
| `ActionBar` | Pin/Copy/Save/More | enabled per state | No Share (FR-047) |
| `SmartResultPanel` | Present QR/Color results | result kind + payload | Copy actions; offline (FR-027–FR-030) |
| `PinWindow` | Floating image surface | opacity, clickThrough, scale | Extends 003 (FR-035–FR-038) |
| `TrayMenu` | Native entry point | menu items | Native, not webview (FR-002) |
| `SettingsWindow` | Edit & persist settings | tab, dirty | Only normal window (FR-003) |
| `HotkeyRecorder` | Capture a new chord | recording, conflict | Conflict detection (FR-041) |

---

## Keyboard Shortcut Specification *(deliverable 13)*

All shortcuts are **customizable** (FR-041). Defaults below use ⌘ on macOS / Ctrl
on Windows. Tool single-keys are active while the editor is focused; capture
hotkeys are global.

**Global (system-wide):**

| Action | macOS | Windows |
|---|---|---|
| Capture Region | ⌘⇧A | Ctrl+Shift+A |
| Capture Window | ⌘⇧W | Ctrl+Shift+W |
| Capture Full Screen | ⌘⇧S | Ctrl+Shift+S |
| Pin (clipboard → pin) | ⌘P | Ctrl+P |
| Show/Hide all pins | ⌘⇧H | Ctrl+Shift+H |

**Editor — tools (single key):**

| Key | Tool | Key | Tool |
|---|---|---|---|
| V | Select/Move | T | Text |
| R | Rectangle | H | Highlighter |
| O | Ellipse/Oval | B | Blur |
| A | Arrow | X | Pixelate |
| L | Line | M | Magnifier |
| P | Pencil | N | Step Number |
| K | Color Picker | C* | Crop (in-editor) |
| E | Eraser | G | Spotlight |

\* Crop uses a non-conflicting binding in-editor since **C = Copy** in the action bar.

**Editor — actions:**

| Action | macOS | Windows |
|---|---|---|
| Copy | C | C |
| Save | S | S |
| Pin | P | P |
| Undo | ⌘Z | Ctrl+Z |
| Redo | ⌘⇧Z | Ctrl+Shift+Z |
| Cancel/Close editor | Esc | Esc |

**Modifiers while drawing:** Shift = constrain (straight line / square / circle);
Scroll = stroke thickness; Alt/Option = draw from center (where applicable).

> Default tool keys are a planning recommendation; the binding scheme and any
> conflict resolution are finalized in `plan.md`. The hard requirement is that
> **all** of them are remappable with conflict detection (FR-041).

---

## Success Criteria *(mandatory)* — Acceptance Criteria *(deliverable 19)*

### Measurable Outcomes

- **SC-001**: From a committed selection, a floating **horizontal** toolbar and action bar appear over the capture with no docked sidebar, and a first-time user can draw a rectangle + arrow and copy the annotated image in **under 10 seconds**, keyboard-only.
- **SC-002**: The clipboard/file/pin output is **pixel-identical** to the on-screen flattened result, verified on single-display, multi-display, and mixed-DPI setups on **both** macOS and Windows.
- **SC-003**: Selecting any tool shows **only** that tool's properties; changing a property updates the selected object live and the default for the next object — verified for every tool.
- **SC-004**: Undo/redo restores exact state for **≥ 50** consecutive operations with no limit hit and no visual divergence; Clear History returns the unannotated capture.
- **SC-005**: On launch, **no** window and **no** dock/taskbar app entry appears — only a tray/menu-bar icon; Settings is the only normal window that ever opens.
- **SC-006**: A user can record a new hotkey for any action, conflicts are flagged, and the action responds to the new binding after save and after restart.
- **SC-007**: QR detection yields the correct decoded URL/value with **zero** network requests observed; **Open URL** only launches the OS browser on an explicit click, and never transmits the screenshot.
- **SC-008**: Each visual smart tool (Spotlight, Magnifier, Step Number, Color Picker, Crop) produces the specified effect in the output and is fully undoable.
- **SC-009**: Startup **< 300 ms**, capture overlay **< 100 ms**, interactions sustain **60 fps**, idle memory **< 120 MB**, idle CPU ≈ 0 — measured on baseline hardware.
- **SC-010**: **Zero** network requests occur across capture → annotate → smart tools → output, verifiable with a network monitor; there is **no** Share/upload/cloud affordance anywhere in the UI.
- **SC-011**: A pin can be made translucent, click-through (with a guaranteed escape), resized crisply, and annotated, with annotations included on copy/save.

## Assumptions

- **Builds on 002 + 003**: The trigger → overlay → select → adjust → crop/encode →
  pin pipeline and the frozen-frame model are reused unchanged except where this
  spec extends them. The editor is a new transient surface between selection and
  output; it **always** opens on commit (Q1), and the instant in-editor
  Copy/Save/Pin shortcuts are the express path (no editor-skipping mode).
- **Edit as default landing (resolved)**: After a selection commit the floating
  editor **always** opens; there is no separate "express" mode — the in-editor
  Copy/Save/Pin shortcuts act instantly, so selecting and then immediately
  pressing C/S/P is the fast path.
- **Annotation model lives in core**: Annotation objects, hit-testing geometry,
  flatten/compositing, and blur/pixelate/spotlight/magnifier pixel ops are pure
  `pinshot-core` logic; the TypeScript canvas renders the live preview and
  forwards intents (Constitution IV). The exact rendering split (core rasterize
  vs. canvas preview then core flatten-on-export) is finalized in `plan.md`.
- **OCR is out of scope (resolved)**: Text extraction (OCR) and any Search/
  Translate on extracted text are **removed from this feature** and deferred to
  the roadmap (v0.3). No OCR action, engine, or hotkey ships here.
- **QR decode is offline & in-core (resolved)**: A pure-Rust offline decoder runs
  in `pinshot-core`; **Copy URL** is a local clipboard action and **Open URL** is
  an explicit, user-initiated hand-off to the OS browser (FR-029).
- **Pin keeps editable annotations (resolved)**: A pin created from the editor
  carries its annotation document and is flattened only when copied or saved, so
  annotating a pin afterward (US6) uses the same model rather than a new layer.
- **Export formats**: PNG (default), JPG, WebP via the `image` crate; compression
  applies to JPG/WebP. Filename pattern extends 002's timestamp naming.
- **Settings storage**: A single local TOML (or JSON) file in the OS config dir;
  human-readable, backup-friendly; no schema migration concerns at this stage.
- **Hotkey scheme**: Defaults in the table above are recommendations; remapping
  and conflict detection are the hard requirement. Tool single-keys apply only
  while the editor is focused, so they don't collide with global hotkeys.
- **Theme & language**: Light/Dark/System theming and at least EN + ID strings;
  full localization framework is a later roadmap item.
- **Scope delivery**: This spec captures the **complete** Floating Editor design
  vision for coherence, but it ships in the prioritized, independently-shippable
  slices (US1→US6) and respects the constitution's roadmap order (annotation →
  shell/settings → QR/visual tools → advanced pin). Later slices MUST NOT
  destabilize earlier ones.
- **No signing**: Builds remain unsigned (constitution); nothing here requires
  signing or certificates.

## Future Roadmap *(deliverable 20)*

| Horizon | Candidate enhancements (not in this spec) |
|---|---|
| **Next** | Scrolling/long capture; capture Window/Full-Screen modes wired to the editor; drag-and-drop the annotated image into other apps |
| **OCR** | Offline OCR (macOS Vision / Windows OCR): extract → Copy Text, plus optional Search/Translate hand-offs; Tesseract for more languages; searchable OCR history — **deferred from this feature** to v0.3 |
| **Beautify** | Background/padding/shadow/rounded-corner "beautify" export presets + social aspect ratios (about.md pillar) |
| **Pins** | Pin persistence across restarts (local, privacy-respecting); pin groups/workspaces; rotate/flip; double-click actions |
| **Smart** | Auto-redact (detect emails / tokens / numbers → one-click blur); barcode types beyond QR |
| **Platform** | Linux (X11 first); full localization framework; per-display color profiles |
| **Power** | Plugin/tool extensibility; templated annotation presets |

These are **explicitly deferred** and listed only to show direction; none relax
the offline / no-share / no-account constitution.

## Non-Goals

- ❌ **Share / cloud upload / shareable URLs** of any kind (the mockup's "Share"
  button is intentionally removed) — outputs are clipboard, local file, pin, and
  OS drag-and-drop only.
- ❌ Accounts, login, or sync.
- ❌ Telemetry or analytics in any form.
- ❌ A layered photo/design editor, permanent sidebars, or a Photoshop/Figma/Canva
  layout.
- ❌ Any online-API AI feature; QR and color are all offline (OCR is deferred to a later version, not made online).
- ❌ Screen recording (per constitution, before v1.0).
