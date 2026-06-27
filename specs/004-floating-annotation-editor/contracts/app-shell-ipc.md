# IPC Contract: App Shell — Tray, Settings & Hotkeys (UI ↔ Shell)

**Feature**: 004-floating-annotation-editor | **Date**: 2026-06-27

The background app shell: the native tray/menu-bar menu, the Settings window
(`ui/settings.html` / `ui/src/settings.ts` ↔ `src-tauri/src/settings/`,
`src-tauri/src/tray.rs`), and global hotkey management. Settings persist to a
local `Settings.toml` via `core::settings` (schema/defaults/validation). All
payloads **camelCase**; **no network** (Principle I).

## Tray menu (native — not a webview)

Built natively in the shell (`tray.rs`) for instant, native feel (NFR-005). Items
trigger shell actions directly (no IPC round-trip through a webview):

| Item | Action |
|---|---|
| **Capture** | Start the capture flow (same as the global Capture hotkey). |
| **Settings** | Open/foreground the Settings window (the only normal window — FR-003). |
| **About** | Show a small native About dialog (name, version, GPL, offline statement). |
| **Check for Updates** | **Opt-in, explicit**: fetch only a static version file **when clicked**; default-off elsewhere (Principle I / FR-046). |
| **Quit** | Close all pins and exit. |

> On launch the shell sets the process to accessory/agent (macOS
> `ActivationPolicy::Accessory`; Windows tray, no taskbar window) so **no** main
> window appears (FR-001).

## Events (shell → settings UI)

### `settings://load`
Emitted when the Settings window opens, with the current persisted values.
```jsonc
{ "general": { "launchAtLogin": false, "checkUpdates": false, "theme": "system", "language": "en" },
  "capture": { "mode": "region", "delaySecs": 0, "includeCursor": false, "includeShadow": true },
  "hotkeys": [ { "action": "captureRegion", "chord": "Cmd+Shift+A", "scope": "global", "conflict": null }, … ],
  "annotation": { "strokeColor": "#EF4444", "fillColor": "#00000000", "font": "Inter",
                  "fontSize": 16, "arrowSize": 4, "highlighterOpacity": 0.4,
                  "blurStrength": 8, "pixelateSize": 12 },
  "export": { "format": "png", "filenamePattern": "PinShot_{date}_{time}", "compression": 90, "clipboard": "image" },
  "advanced": { "developerMode": false } }
```

## Commands (settings UI → shell)

### `get_settings`
- **Args**: `{}` → returns the same shape as `settings://load`.

### `set_settings`
Persist a (partial) settings change; the shell validates via `core::settings`,
writes `Settings.toml`, and applies side effects.
- **Args**: `{ patch: PartialSettings }`
- **Effect**: `core::settings::validate` → write file → apply:
  - `general.launchAtLogin` → register/unregister OS login item.
  - `general.theme` → broadcast `editor://theme` + `settings` restyle.
  - `general.language` → swap UI strings (EN/ID initially).
  - `hotkeys` → re-register global chords (see below).
  - `annotation` → reseed editor `ToolProperties` defaults.
  - `export` → used by the next Copy/Save.
- **Returns**: `{ ok: true, settings: Settings }` or
  `{ ok: false, error: "invalid_settings"|"write_failed", message, fieldErrors? }`.

### `reset_settings`
Advanced ▸ Reset (FR-044).
- **Args**: `{}` → restore documented defaults, rewrite the file, re-apply.
- **Returns**: `{ ok: true, settings: Settings }`.

### `record_hotkey`
Enter "recording" for one action and capture the next key chord (FR-041). The UI
shows a recording state; the shell captures the chord and checks conflicts.
- **Args**: `{ action: ActionId }`
- **Returns**:
  ```jsonc
  { "ok": true, "chord": "Cmd+Shift+A",
    "conflict": null | { "kind": "pinshot"|"osReserved", "with": "captureWindow" } }
  ```
  The new chord is **not** persisted until `set_settings` saves it; if `conflict`
  is non-null the UI warns before allowing save.

### `open_logs`
Advanced ▸ Logs (FR-044) — reveal the local log file/folder in the OS file
manager. **Args**: `{}` → `{ ok: true, path }`. (Local only; no upload.)

### `check_updates`
Explicit, opt-in update check (mirrors the tray item) — fetch a static version
file **only on this call** and report.
- **Args**: `{}`
- **Returns**: `{ ok: true, current: "0.4.0", latest: "0.4.1"|null, url?: "…releases…" }`
  or `{ ok: false, error: "offline"|"unreachable", message }`.
- **Note**: This is the **sole** outbound call in the entire app, isolated here,
  user-initiated, and default-off (Principle I). It fetches a version string
  only — never sends any user/screenshot data.

## Notes

- **One normal window**: only Settings is a normal window (FR-003); the editor,
  overlay, and pins are transient/borderless surfaces.
- **Persistence**: `Settings.toml` lives in `dirs::config_dir()/PinShot/`,
  human-readable and backup-friendly (FR-045); a missing/corrupt file → defaults
  + rewrite, never a crash (Edge Case).
- **Hotkey scope**: `global` chords are registered with `global-hotkey`; `editor`
  chords apply only while the editor window is focused, so single-key tool
  shortcuts can't fire system-wide.
- **No Share / no telemetry**: there is no command to upload, share, or report
  anywhere in this surface; the only network-capable command is the explicit,
  opt-in `check_updates`.
