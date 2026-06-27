# Feature Specification: Floating Annotation Editor & Smart Capture Toolkit

**Feature Branch**: `004-floating-annotation-editor`

**Created**: 2026-06-27

**Status**: Draft

**Input**: Design mockup (`image.png`, "PinShot — Capture. Annotate. Pin.") plus
product brief: turn PinShot's one-shot capture into a fast, floating,
keyboard-first annotation experience — a horizontal floating toolbar with
contextual tool properties, a floating action bar (Pin / Copy / Save / OCR), an
infinite undo/redo history, smart tools (OCR, QR detection, color picker,
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
| P1 | **Dana, the developer** | Files bug reports, documents APIs, shares error messages | Capture an error → arrow + box the cause → copy into the issue; OCR a stack trace from a video call | Hotkey-to-pasted-annotated-image in < 10 s, hands on keyboard |
| P2 | **Tomas, the technical writer** | Writes tutorials & docs | Add numbered steps, spotlight a UI region, blur a token, save a crisp PNG | Numbered callouts and redaction without leaving the keyboard |
| P3 | **Mira, the support agent** | Explains fixes to customers all day | Box the button to click, pin a reference next to the customer's screen, copy | Reuse the same few tools instantly, every ticket |
| P4 | **Sven, the privacy-conscious office worker** | Shares internal dashboards | Redact sensitive data (blur/pixelate), confirm nothing leaves the machine | Trustworthy redaction; zero network, verifiably |
| P5 | **Aiko, the multilingual researcher** | Pulls text from images/PDFs | OCR a foreign-language figure, copy the text, open a referenced QR/URL | Reliable offline OCR + one-tap QR/URL |

All personas share the same expectation: **the tool disappears between uses**
(tray/menu bar only) and **reappears instantly** on a hotkey.

---

## User Scenarios & Testing *(mandatory)*

This feature builds directly on the existing pipeline — **002** (trigger →
overlay → select → crop → clipboard/file) and **003** (adjustable selection +
floating pin). After a region is selected and the user chooses **Edit** (the new
default landing), a **floating annotation editor** opens over the frozen capture.
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

1. **Given** a committed selection, **When** the user enters edit, **Then** a floating horizontal toolbar and a floating action bar (Pin / Copy / Save / OCR / More) appear over the captured image, with no left/right sidebar and no separate editor window chrome.
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

### User Story 4 - Smart text & code capture: OCR and QR (Priority: P3)

As Aiko, after capturing a figure I press **OCR** and PinShot extracts the text
**entirely offline**, offering **Copy Text** (and optional, clearly user-initiated
Search/Translate). If the capture contains a **QR code**, PinShot detects it
automatically and offers **Open URL / Copy URL** — no internet round-trip to read
it.

**Why this priority**: OCR is PinShot's public-launch differentiator (roadmap
v0.3) and QR detection is a high-delight, low-cost companion. Both are
independently valuable and reuse the captured-image pipeline, but they sit above
the core editor (US1/US2) and shell (US3).

**Independent Test**: Capture an area containing printed text; press OCR and
confirm the extracted text is available to copy and matches the source. Capture
an area containing a QR code; confirm PinShot offers Open URL / Copy URL with the
decoded value. Verify with a network monitor that neither action issues a network
request (the optional Open URL / Search hands off to the OS browser by explicit
user action).

**Acceptance Scenarios**:

1. **Given** a captured image containing legible text, **When** the user invokes OCR, **Then** the recognized text is presented with a Copy Text action and matches the source within OCR tolerance — produced fully offline.
2. **Given** recognized OCR text, **When** the user invokes the optional Search or Translate action, **Then** PinShot hands the text to an external app/OS by explicit user action only (never silently), and this is clearly labeled as leaving the offline boundary.
3. **Given** a captured image containing a QR code or supported barcode, **When** edit/OCR runs, **Then** PinShot detects it offline and offers Open URL and Copy URL using the decoded value.
4. **Given** no text/QR is present, **When** the user invokes OCR, **Then** PinShot reports "no text found" gracefully without error.

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
OCR/QR.

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
- **Crop after annotating**: Re-cropping MUST keep existing annotations correctly positioned relative to the new frame (or clearly define that annotations outside the crop are removed/clipped).
- **Blur/pixelate of moving region**: Editing a blur region's bounds MUST re-derive the obscured pixels from the original capture, never from already-blurred output (no irreversible double-blur during editing).
- **OCR with no text / unsupported language**: MUST report "no text found" or "language unavailable" gracefully; MUST never hang or call the network.
- **QR with multiple codes**: MUST handle 0, 1, or many codes — offering a choice when more than one is found.
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
- **FR-011**: The action bar MUST offer **Pin**, **Copy**, **Save**, **OCR**, and **More** — and MUST NOT offer any Share / upload / cloud / shareable-URL action.
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
- **FR-025**: **Pin** MUST create a floating, always-on-top pin of the flattened (or live-editable, per US6) result, reusing feature 003.
- **FR-026**: Output actions MUST be reachable both from the action bar and via keyboard shortcuts (Pin/Copy/Save and OCR).

#### Smart tools — OCR & QR (US4)

- **FR-027**: **OCR** MUST extract text from the captured image **fully offline** using the platform engine (macOS Vision / Windows OCR), and offer **Copy Text**.
- **FR-028**: PinShot MAY offer **Search** and **Translate** on OCR text, but only as **explicit, user-initiated** actions that hand text to an external app/OS; these MUST be clearly labeled as leaving the offline boundary and MUST be OFF/absent by default where they would require network. PinShot's core MUST NOT itself perform any network request for them.
- **FR-029**: **QR / barcode detection** MUST run offline on the captured image; when one or more codes are found, PinShot MUST offer **Open URL** and **Copy URL** (or copy raw value) for each.

#### Smart tools — visual (US5)

- **FR-030**: The **Color Picker** MUST read a pixel's color and present it as **HEX, RGB, and HSL**, each individually copyable.
- **FR-031**: The **Spotlight** tool MUST darken everything outside a chosen region in the output, with adjustable dim strength.
- **FR-032**: The **Magnifier** tool MUST render a circular loupe of the underlying pixels at an adjustable zoom into the image.
- **FR-033**: The **Step Number** tool MUST place auto-incrementing numbered markers (1, 2, 3 …) with an editable color, renumbering consistently when a marker is added or removed.
- **FR-034**: The **Crop/Resize** tool MUST allow re-cropping the capture after the fact with **Free, 1:1, 16:9, and 4:3** constraints; the committed output MUST contain exactly the new frame.

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
- **FR-048**: The annotation model, flatten/compositing, blur/pixelate/spotlight/magnifier pixel operations, QR decode, color math, crop, and PNG/JPG/WebP encoding MUST live in the headless `pinshot-core` crate and be **unit-testable without a display**; platform-specific OCR/clipboard/window behavior MUST sit behind traits.
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
- **SmartResult**: Output of a smart tool — `OcrResult` (text + regions), `QrResult` (decoded values), `ColorSample` (HEX/RGB/HSL).
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
        │  release / confirm  ──►  Edit (default)        ──►  or direct Copy/Save/Pin (express)
        ▼
[floating editor]  toolbar + action bar appear over the capture
        │  pick tool (keyboard) → toolbar shows that tool's properties
        │  draw / restyle / undo-redo  (history panel tracks the stack)
        ▼
[output]  Copy (C) │ Save (S) │ Pin (P) │ OCR │ More
        ▼
[back to idle]  editor closes; pins (if any) keep floating
```

**Smart-tool sub-flows:**

```
OCR:  action bar ▸ OCR  →  offline text extracted  →  Copy Text │ (Search/Translate*)
QR:   detected on edit/OCR  →  Open URL │ Copy URL
Color:tool ▸ Color Picker  →  hover pixel  →  HEX/RGB/HSL  →  copy
Crop: tool ▸ Crop  →  Free│1:1│16:9│4:3  →  commit re-frames the output
```
\* Search/Translate are explicit, user-initiated hand-offs to the OS/browser (FR-028).

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
│   └── Floating action bar        (Pin · Copy · Save · OCR · More)
│       └── Smart results          (OCR text · QR URL · Color values)
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
- **Action bar**: Pin (P) · Copy (C) · Save (S) · OCR · More (FR-011). No Share.
- **History panel**: ordered list of annotations with type icons + Clear History (FR-020/FR-022).
- **Affordances/Hints**: footer micro-hints ("Drag to move · Hold Shift for straight lines · Scroll to change thickness · Esc to cancel"); selection handles on the active object; live dimension/coordinate readout reused from 002.
- **Theme**: honors Light/Dark/System (NFR-005).

### S3 — Smart result surfaces
- **OCR panel**: extracted text in a scrollable, selectable field + Copy Text (+ optional Search/Translate, clearly marked) (FR-027/FR-028).
- **QR chip**: detected value(s) with Open URL / Copy URL (FR-029).
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
| `ActionBar` | Pin/Copy/Save/OCR/More | enabled per state | No Share (FR-047) |
| `SmartResultPanel` | Present OCR/QR/Color results | result kind + payload | Copy actions; offline (FR-027–FR-030) |
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
| OCR (capture text) | ⌘⇧O | Ctrl+Shift+O |
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
| OCR | ⌘⇧O | Ctrl+Shift+O |
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
- **SC-007**: OCR extracts copyable text matching the source within OCR tolerance, and QR detection yields the correct URL — both with **zero** network requests observed.
- **SC-008**: Each visual smart tool (Spotlight, Magnifier, Step Number, Color Picker, Crop) produces the specified effect in the output and is fully undoable.
- **SC-009**: Startup **< 300 ms**, capture overlay **< 100 ms**, interactions sustain **60 fps**, idle memory **< 120 MB**, idle CPU ≈ 0 — measured on baseline hardware.
- **SC-010**: **Zero** network requests occur across capture → annotate → smart tools → output, verifiable with a network monitor; there is **no** Share/upload/cloud affordance anywhere in the UI.
- **SC-011**: A pin can be made translucent, click-through (with a guaranteed escape), resized crisply, and annotated, with annotations included on copy/save.

## Assumptions

- **Builds on 002 + 003**: The trigger → overlay → select → adjust → crop/encode →
  pin pipeline and the frozen-frame model are reused unchanged except where this
  spec extends them. The editor is a new transient surface between selection and
  output; an "express" path (commit straight to Copy/Save/Pin without editing)
  remains available.
- **Edit as default landing**: After a selection commit, the floating editor is
  the default next step; whether Copy/Save/Pin without editing requires a
  modifier or a separate action is a planning decision.
- **Annotation model lives in core**: Annotation objects, hit-testing geometry,
  flatten/compositing, and blur/pixelate/spotlight/magnifier pixel ops are pure
  `pinshot-core` logic; the TypeScript canvas renders the live preview and
  forwards intents (Constitution IV). The exact rendering split (core rasterize
  vs. canvas preview then core flatten-on-export) is finalized in `plan.md`.
- **OCR engines**: macOS uses Apple Vision; Windows uses `Windows.Media.Ocr`;
  both offline. Tesseract is an optional later fallback for additional languages
  (roadmap), not required here. Initial languages: English + Indonesian.
- **QR decode is offline & in-core**: A pure-Rust offline decoder runs in
  `pinshot-core`; "Open URL" hands off to the OS browser by explicit user action.
- **Search/Translate are out-of-scope-for-offline**: Any web Search or online
  Translate is an explicit, clearly-labeled hand-off to the OS/browser, never a
  silent core request; an offline translate engine, if added, is a later concern
  (FR-028). They may be hidden by default.
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
  shell/settings → OCR/QR → polish tools → advanced pin). Later slices MUST NOT
  destabilize earlier ones.
- **No signing**: Builds remain unsigned (constitution); nothing here requires
  signing or certificates.

## Future Roadmap *(deliverable 20)*

| Horizon | Candidate enhancements (not in this spec) |
|---|---|
| **Next** | Scrolling/long capture; capture Window/Full-Screen modes wired to the editor; drag-and-drop the annotated image into other apps |
| **OCR+** | Tesseract fallback for CJK & more languages; OCR region selection; searchable history by OCR text |
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
- ❌ Any online-API AI feature; OCR/QR/color are all offline.
- ❌ Screen recording (per constitution, before v1.0).
