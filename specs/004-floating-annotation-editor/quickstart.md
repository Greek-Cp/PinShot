# Quickstart: Validating the Floating Annotation Editor & Smart Tools

**Feature**: 004-floating-annotation-editor | **Date**: 2026-06-27

Maps to the spec's Success Criteria (SC-001…SC-011). Run the headless core tests
anywhere; run the manual scenarios on **both** macOS and Windows.

## Headless core tests (no display)

```bash
cargo test -p pinshot-core
```
Covers (FR-048): the annotation model + hit-test/resize geometry, **flatten**
compositing (DPI-correct, SC-002), pixel **effects** (blur/pixelate/spotlight/
magnifier on fixtures), **history** undo/redo (incl. redo-branch truncation,
SC-004), **QR** decode (offline), **color** conversions (HEX/RGB/HSL, SC-008),
**encode** (PNG/JPG/WebP round-trips), **settings** schema defaults/validation,
and **OCR-consuming logic via a fake `OcrEngine`**.

## Full workspace gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace && cargo test --workspace
cd ui && npm ci && npm run lint && npm run format:check && npm run build
```

## Run the app

```bash
cd ui && npm ci && npm run build && cd ..
cargo run -p pinshot
```
The app starts **in the tray / menu bar with no window** (SC-005).

## US3 — background app, tray menu & Settings (do this first)

1. Launch → confirm **no** window and **no** Dock/taskbar app entry appears,
   only a menu-bar (macOS) / system-tray (Windows) icon (SC-005, FR-001).
2. Click the icon → menu shows **Capture · Settings · About · Check for Updates ·
   Quit** (FR-002).
3. Choose **Settings** → a normal window opens (the only one — FR-003).
4. Hotkeys tab → **record** a new chord for *Capture Region*; try a chord already
   bound → conflict is flagged before save (FR-041). Save, then trigger the new
   chord → capture starts (SC-006). Restart the app → the binding persists.
5. General tab → toggle **Theme** → the editor/Settings restyle live; toggle
   **Launch at Login** and confirm it persists.

## US1 — annotate → copy / save / pin

1. Press *Capture Region*, select a region, choose **Edit** → a **floating
   horizontal toolbar** + **action bar (Pin · Copy · Save · OCR · More)** appear
   over the capture, **no sidebar** (SC-001, FR-009/FR-011).
2. Draw a **Rectangle** and an **Arrow** (hold **Shift** → straight; **scroll** →
   thickness) and add **Text** (FR-013/FR-014).
3. Press **C** → paste into an image app → the image **matches the on-screen
   flattened annotations exactly** (SC-002, FR-023).
4. Press **S** → a file appears in the configured folder/format (FR-024).
5. Press **P** → an always-on-top **pin** of the annotated result appears; the
   editor closes (FR-025).
6. Re-open, draw something, press **Esc** → editor closes, clipboard & files
   **unchanged** (FR-012).

**Confirm no Share** anywhere in the action bar or menus (SC-010, FR-047).

## US2 — contextual properties, full toolset & history

1. Select **Rectangle** → toolbar shows **only** Stroke / Thickness / Fill /
   Opacity / Radius (FR-017). Select **Text** → it shows font/size/weight/
   background/shadow instead (SC-003).
2. Draw a shape, then change its **stroke color** and **opacity** → the object
   updates live and the next shape inherits the new defaults (FR-019).
3. Add 5 different annotations → the **History panel** lists them in order
   (FR-020).
4. Press **Undo** ~50 times then **Redo** → exact reverse/forward with no limit
   (SC-004, FR-021). Make a new edit mid-undo → the stale redo branch is dropped.
5. **Clear History** → unannotated capture returns (FR-022).

## US4 — OCR & QR (offline)

1. Capture printed text → **OCR** → extracted text is copyable and matches the
   source within tolerance (SC-007, FR-027). Empty area → "no text found",
   no error (Edge Case).
2. Capture a **QR code** → **Open URL / Copy URL** offer the decoded value
   (FR-029). Multiple codes → a choice is offered.
3. **Search/Translate** (if enabled) only open the OS browser by explicit click
   and are clearly labeled as leaving offline (FR-028) — and never send the image.

## US5 — visual smart tools

1. **Step Number**: click 3× → markers **1, 2, 3** in the chosen color; delete #2
   → remaining renumber (FR-033).
2. **Spotlight**: mark a region → everything outside dims in the output (FR-031).
3. **Magnifier**: place a loupe, adjust zoom → circular magnified pixels render
   (FR-032).
4. **Color Picker**: hover a known color → **HEX/RGB/HSL** shown; copy each
   (SC-008, FR-030).
5. **Crop**: re-crop with **16:9** → output equals the new frame; existing
   annotations stay correctly placed (FR-034). Each tool is **undoable** (SC-008).

## US6 — advanced pin

1. Pin a capture → adjust **opacity** (translucent, still on top, FR-035).
2. Enable **click-through** → clicks pass beneath; confirm a hotkey/tray escape
   still regains control (FR-036, Edge Case "no trap").
3. **Resize/zoom** the pin → crisp on any display/DPI (FR-037).
4. **Annotate** the pin, then **copy/save** it → annotations are included
   (SC-011, FR-038).

## Mixed-DPI multi-monitor (mandatory — SC-002)

On a HiDPI (2×) display next to a 1× display, on **both** macOS and Windows:
1. Capture+annotate+flatten fully on the **2×** display → pixel-exact output.
2. Same fully on the **1×** display → pixel-exact.
3. Capture **spanning the seam** → one correct composite; effects/flatten/pin all
   correct, no offset/scale error.

## Offline check (mandatory — SC-010 / FR-046)

With a network monitor (Little Snitch / `lsof -i` / Resource Monitor) running,
perform a full **capture → annotate → OCR → QR → color → copy/save/pin** cycle →
**zero** network connections from the app. The **only** permitted outbound call is
the **explicit** *Check for Updates*, which must be silent unless clicked.

## Performance budgets (SC-009)

- Cold start to "tray ready, hotkeys armed" **< 300 ms** (NFR-001).
- Capture overlay visible **< 100 ms** from hotkey (NFR-002).
- Toolbar/contextual-panel/drawing interactions sustain **60 fps** (NFR-003).
- Idle memory **< 120 MB**; idle CPU ≈ 0 with the app in the tray (NFR-004/007).
Instrument the trigger→show and flatten→output paths with timestamp logs to
measure precisely.

## Parity

Re-run **US1–US6**, mixed-DPI, offline, and perf on **both** macOS and Windows;
any single-platform-only behavior is a failure (FR-049, Constitution III).
