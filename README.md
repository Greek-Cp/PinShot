# PinShot 📌

**PinShot** adalah aplikasi screenshot native 100% untuk macOS (AppKit, SwiftUI, ScreenCaptureKit, Swift 6). Didesain modern, ultra-ringan, minimal, dan mengikuti Apple Human Interface Guidelines (HIG).

---

## ✨ Fitur Utama

- **🚀 100% Native macOS:** Performa tinggi tanpa overhead framework web (Electron/Webview).
- **🔒 Smart Permission Onboarding:** Pengecekan izin Screen Recording saat start dengan panduan interaktif dan tombol langsung ke macOS System Settings.
- **⚡️ Global HotKey (`⌘⇧A`):** Tekan `Command + Shift + A` untuk freeze layar dan membuka overlay capture instan.
- **🖥 Multi-Monitor Continuous Selection:** Seleksi area bebas yang mendukung lintas monitor dengan resolusi dan DPI Retina berbeda.
- **📌 Floating Pin Window:** Jadikan screenshot sebagai referensi mengambang (*always-on-top* di semua Spaces), dapat digeser, di-resize (aspect-ratio terkunci), tombol close (✕) saat hover, dan menu cepat (Copy, Save, Opacity).
- **✏️ Rich In-Place Annotations:** Rectangle, Ellipse, Arrow, Line, Freehand Pen, Highlighter, Text, Step number badges (1-2-3...), Color swatches, dan Undo/Redo (`⌘Z`/`⌘⇧Z`).
- **📋 Instant Copy & Save:** Salin langsung ke clipboard dengan efek suara native (`⌘C`) atau simpan ke file (`⌘S`).
- **🔍 Precision Loupe (Kaca Pembesar):** Zoom pixel level dengan pembacaan kode warna HEX dan RGB.
- **🍸 Menu Bar Native:** Berjalan di latar belakang sebagai menu bar status app tanpa ikon dock yang mengganggu.

---

## 🛠 Struktur Proyek

```
PinShot/
├── Package.swift                     # Swift Package Manager manifest
├── Sources/
│   ├── PinShotCore/                  # Pure logic & cross-platform models
│   │   ├── Geometry.swift            # Selection rect, hit-testing, coordinate mapping
│   │   ├── ColorUtils.swift          # HEX/RGB color parsing, luminances, presets
│   │   ├── HistoryStack.swift        # Generic undo/redo manager
│   │   ├── AnnotationModel.swift     # Annotation shapes, steps, documents
│   │   ├── AnnotationRenderer.swift  # CoreGraphics compositing engine
│   │   └── Settings.swift            # Preferences data models
│   └── PinShot/                      # Native macOS AppKit & SwiftUI App
│       ├── PinShotApp.swift          # Main entry point
│       ├── AppDelegate.swift         # NSApplicationDelegate
│       ├── MenuBarController.swift   # NSStatusItem Menu Bar controller
│       ├── PermissionService.swift   # Screen recording permissions & polling
│       ├── PermissionOnboardingView.swift # Onboarding UI for missing permissions
│       ├── HotKeyService.swift       # Carbon Event global hotkey (⌘⇧A)
│       ├── CaptureService.swift      # ScreenCaptureKit & CGDisplay capture
│       ├── OverlayWindowManager.swift# Multi-screen overlay coordinator
│       ├── OverlayCanvasView.swift   # Area drag selection, resize & drawing
│       ├── FloatingToolbarView.swift # Adaptive Pin/Annotate/Copy/Save toolbar
│       ├── PinWindow.swift           # Floating always-on-top pinned window
│       ├── LoupeView.swift           # Pixel magnifier & color picker
│       ├── PasteboardService.swift   # Clipboard manager
│       └── SoundService.swift        # Audio feedback manager
└── Tests/
    └── PinShotCoreTests/             # Swift Testing suite (100% pass)
```

---

## 🚀 Build & Menjalankan Aplikasi

### Persyaratan:
- macOS 14.0 (Sonoma) atau lebih baru
- Xcode 15+ / Swift 6.0+

### Menjalankan Test:
```bash
swift test
```

### Menjalankan Aplikasi:
```bash
swift run PinShot
```
