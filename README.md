# PinShot

[![CI](https://github.com/Greek-Cp/PinShot/actions/workflows/ci.yml/badge.svg)](https://github.com/Greek-Cp/PinShot/actions/workflows/ci.yml)

> Open-source, **100% offline** screenshot & pin tool for macOS & Windows — a
> Snipaste-class app with privacy as the first feature. No accounts, no cloud,
> no telemetry.

> **Status:** pre-alpha. Implemented so far: the CI/CD pipeline
> (spec [`001-github-cicd`](specs/001-github-cicd/)) and the capture pipeline —
> region capture to clipboard or PNG, with a selection magnifier and color
> readout (spec [`002-capture-to-output`](specs/002-capture-to-output/)). Pin,
> OCR, and beautify are on the [roadmap](about.md).

## Capturing

PinShot runs in the tray/menu bar. To capture:

- Press **Cmd/Ctrl + Shift + A** (or pick **Capture** from the tray icon) — a
  selection overlay covers every display.
- **Drag** to select a region. A magnifier, the live width×height, and the pixel
  color under the cursor are shown.
- **Adjust** the selection before committing: drag its **edges or corners** to
  resize, or drag **inside** it to move. The dimensions update live.
- **Enter** copies the selection to the clipboard; **S** saves it as a PNG to
  `Pictures/PinShot/`; **P** pins it; **C** copies the hovered pixel's color;
  **Esc** (or right-click) cancels.

### Pinning

Press **P** (or the **Pin** button) to turn the selection into a floating,
always-on-top window that stays above other apps. Open as many pins as you like.
On a pin: **drag** to move it, **double-click** to close it, **C** to copy its
image, **Esc** to close. Pins last until you close them or quit PinShot.

Everything happens on-device — no network, no account.

## Architecture

PinShot follows a clean, inward-pointing dependency layout (see the
[constitution](.specify/memory/constitution.md)):

```
ui/  (TypeScript webview)  →  src-tauri/  (Tauri shell)  →  crates/pinshot-core/  (domain)
```

- **`crates/pinshot-core`** — platform-independent domain logic (capture
  geometry, image ops, later OCR). No GUI, no network; tested headless.
- **`src-tauri`** — the Tauri 2.x shell: windows, tray, hotkeys, and the
  OS-specific adapters that implement the core's ports.
- **`ui`** — the webview frontend (Vite + TypeScript). Renders state and
  forwards intents; no business logic.

## Build from source

Prerequisites: [Rust](https://rustup.rs) (the pinned toolchain in
`rust-toolchain.toml` is installed automatically), [Node.js 20+](https://nodejs.org),
and the [Tauri system dependencies](https://tauri.app/start/prerequisites/)
for your OS.

```bash
# Frontend
cd ui && npm ci && npm run build && cd ..

# Run the desktop app (debug)
cargo run -p pinshot

# Run the core test suite (headless, no GUI needed)
cargo test --workspace
```

## Installing a release build

Download the latest installer from the [Releases](../../releases) page:
a `.dmg` for macOS (Apple Silicon) or a `.msi`/`.exe` for Windows.

> **Builds are unsigned.** PinShot has no paid code-signing certificate yet, so
> the OS warns you on first launch. This is expected — follow the steps below.

### macOS — getting past Gatekeeper

Because the app isn't signed/notarized, macOS may show one of:

- *"PinShot can't be opened because Apple cannot check it for malicious software."*
- *"'PinShot' is damaged and can't be opened."* (common on Apple Silicon when the
  download is quarantined)

Pick **one** of these:

**Option A — Right-click → Open (simplest)**

1. Open the `.dmg` and drag **PinShot** into **Applications**.
2. In Applications, **right-click** (or Control-click) **PinShot → Open**.
3. On the dialog, click **Open** again. macOS remembers this choice for next time.

**Option B — Remove the quarantine flag (use this for the "damaged" error)**

Run this once in Terminal, then open the app normally:

```sh
xattr -dr com.apple.quarantine /Applications/PinShot.app
```

**Then grant Screen Recording** (required to capture the screen):
System Settings → **Privacy & Security → Screen Recording** → enable **PinShot**,
then quit and reopen it. PinShot lives in the **menu bar** (no Dock icon), so
look for its tray icon — capture with `⌘⇧A` or the tray's **Capture** item.

> If macOS still blocks it, open **System Settings → Privacy & Security**, scroll
> to the Security section, and click **Open Anyway** next to the PinShot notice.

### Windows

On the SmartScreen prompt, click **More info → Run anyway**.

## Privacy

The application core makes **zero network requests**. There is no telemetry, no
analytics, and no account. This is enforced as a binding project principle and
is verifiable from the source.

## Branch protection

`main` requires the single **CI OK** status check to pass before merging
(Settings → Branches → add rule for `main` → require status check `CI OK`).
That one check aggregates build + test on both platforms plus the quality gate,
so nothing merges unless every platform is green.

## License

**GPL-3.0-or-later** — see the [LICENSE](LICENSE) file for the full text.

PinShot is free and open source. You are welcome to use, study, modify, and
fork it. Because GPL-3.0 is a copyleft license, any distributed fork or
derivative work **must also stay open source under the same license** — this
protects PinShot from being turned into a closed-source product.
