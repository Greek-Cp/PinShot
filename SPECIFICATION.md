# PinShot — Native macOS Screenshot & Pin Application Specification
**Versi Dokumen:** 1.0.0  
**Target Platform:** macOS 14.0+ (Sonoma, Sequoia, and beyond)  
**Arsitektur Bahasa & Framework:** Swift 6, SwiftUI, AppKit, ScreenCaptureKit, CoreGraphics  
**Target Distribusi:** Universal Binary (Apple Silicon & Intel), Sandboxed / Non-Sandboxed Options  

---

## 1. Executive Summary & Visi Produk

**PinShot** adalah aplikasi screenshot native macOS yang ultra-ringan, berkecepatan tinggi, dan berdesain modern mengikuti pedoman *Apple Human Interface Guidelines (HIG)*. PinShot memadukan kenyamanan capture instan, seleksi multi-monitor tanpa jeda, floating annotations, dan fitur utama **Pin to Desktop** (menjadikan hasil tangkapan layar sebagai jendela melayang yang interaktif dan *always-on-top*).

### Nilai Utama (Core Values):
1. **100% Pure Native:** Tanpa webview, tanpa Electron, tanpa runtime overhead. Menggunakan AppKit + SwiftUI + ScreenCaptureKit.
2. **Fluid & Instant:** Waktu respons dari penekanan shortcut hingga overlay muncul < 50ms.
3. **Multi-Monitor Seamlessness:** Mendukung seleksi melintasi batas monitor yang berbeda DPI (Retina & Non-Retina) tanpa distorsi koordinat.
4. **Interactive Pinning:** Screenshot bukan sekadar file pasif, melainkan kanvas referensi terapung yang dapat di-zoom, di-resize, diedit, atau disalin kapan saja.

---

## 2. Arsitektur Sistem & Komponen Utama

```
┌────────────────────────────────────────────────────────────────────────┐
│                              PinShot App                               │
├────────────────────────────────────────────────────────────────────────┤
│ [AppDelegate / App Lifecycle]                                          │
│   ├── PermissionObserver (Screen Recording & Accessibility Polling)     │
│   ├── MenuBarController (NSStatusItem, Popover, Menu Actions)           │
│   └── HotKeyManager (Carbon Global HotKey: ⌘⇧A)                        │
├────────────────────────────────────────────────────────────────────────┤
│ [Capture & Display Engine]                                             │
│   ├── ScreenCaptureManager (ScreenCaptureKit / SCShareableContent)     │
│   ├── MultiScreenCoordinateMapper (Unified Virtual Canvas Coordinate)  │
│   └── OverlayWindowManager (Multi-window NSPanel / Level .screenSaver) │
├────────────────────────────────────────────────────────────────────────┤
│ [Selection & Interaction Layer]                                        │
│   ├── OverlaySelectionView (Crosshair, Selection Rect, Snapping)       │
│   ├── LoupeMagnifierView (Pixel Grid, Color Picker Hex/RGB)            │
│   └── FloatingToolbarView (Pin, Annotate, Copy, Save, Close)           │
├────────────────────────────────────────────────────────────────────────┤
│ [Annotation Engine]                                                    │
│   ├── Vector Canvas (Rect, Ellipse, Arrow, Pen, Highlighter, Text, Step)│
│   ├── Pixel Effects (Blur / Mosaic Caching Shader via CoreImage)       │
│   └── Undo/Redo History Stack                                          │
├────────────────────────────────────────────────────────────────────────┤
│ [Pin Window Management]                                                │
│   ├── PinPanel (NSPanel .floating, .canJoinAllSpaces, borderless)      │
│   ├── PinImageView (Aspect-ratio drag-resize, Zoom, Opacity slider)    │
│   └── PinActionOverlay (Hover close button ✕, Context Menu)            │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Alur Pengguna (User Journey & Lifecycle)

### 3.1 First Launch & Permission Onboarding
1. **Startup Check:**
   - Aplikasi memeriksa izin *Screen Recording* via `CGPreflightScreenCaptureAccess()` dan validasi `SCShareableContent.excludingDesktopWindows(false, onScreenOnly: true)`.
2. **Kondisi Izin Belum Diberikan:**
   - Tampilkan jendela onboarding modern bergaya macOS sheet/window (`PinShot Permission Guide`).
   - Menjelaskan pentingnya izin Screen Recording dengan grafis instruktif.
   - Menyediakan tombol CTA utama: **"Open System Settings"** yang langsung membuka:
     `x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture`
   - Background Polling / `NotificationCenter` mendeteksi kapan izin aktif secara realtime tanpa mengharuskan pengguna me-restart aplikasi jika memungkinkan.
3. **Kondisi Izin Terpenuhi:**
   - Jendela onboarding tertutup otomatis dengan transisi *fade out*.
   - Icon PinShot muncul di Menu Bar macOS (`NSStatusItem`).
   - Aplikasi berjalan di background sebagai accessory application (`LSUIElement = true`).

---

## 4. Spesifikasi Fitur Detail

### 4.1 Menu Bar Application (`NSStatusItem`)
- **Tampilan:**
  - Ikon monokrom SF Symbol yang adaptif terhadap Dark/Light mode (`camera.viewfinder` / custom minimal PinShot icon).
- **Menu Items:**
  - **Capture Area** (`⌘⇧A`)
  - **Capture Fullscreen** (`⌘⇧3` / custom)
  - **Active Pins** (Submenu menampilkan daftar pin aktif dengan opsi *Close All Pins*)
  - ---
  - **Preferences / Settings** (`⌘,`)
  - **Check for Updates...**
  - **About PinShot**
  - ---
  - **Quit PinShot** (`⌘Q`)

---

### 4.2 Global Shortcut Engine
- Menggunakan **Carbon Event HotKeys** API (`RegisterEventHotKey`) untuk performa instan tanpa latency event-loop dan tanpa memerlukan Accessibility Permissions hanya untuk mendengarkan shortcut global.
- Default: `Command + Shift + A` (`⌘⇧A`).
- Mendukung kustomisasi shortcut di menu Preferences dengan validasi bentrokan shortcut sistem.

---

### 4.3 Multi-Monitor Fullscreen Capture & Overlay Engine

#### 4.3.1 Pengambilan Snapshot Layar (ScreenCaptureKit)
- Menggunakan `SCShareableContent` dan `SCScreenshotManager` (atau `CGDisplayCreateImage` untuk instant freeze frame).
- Membekukan frame seluruh layar (*screen freeze*) saat shortcut ditekan untuk mencegah animasi latar belakang bergeser saat pengguna melakukan seleksi.
- Menggabungkan atau memetakan koordinat display dari `NSScreen.screens` ke dalam satu unified canvas coordinate system.

#### 4.3.2 Window Overlay
- Dibuat menggunakan `NSPanel` dengan:
  - `level = .screenSaver` (atau `.statusBar + 1`) agar selalu berada di atas seluruh jendela lain dan menu bar.
  - `styleMask = [.borderless, .nonactivatingPanel]`
  - `collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .ignoresCycle]`
  - `isOpaque = false` dan `backgroundColor = .clear`.
- Satu overlay window per monitor fisik yang disinkronisasi secara real-time via event bridge.

#### 4.3.3 Seleksi Area Bebas (Selection Logic)
- **Multi-Monitor Dragging:** Pengguna dapat menarik mouse melintasi monitor 1 ke monitor 2. Boundary box dihitung secara global menggunakan koordinat virtual unified.
- **Visual Feedback:**
  - Background luar area seleksi diberi efek dimming transparan (hitam 35% opacity).
  - Garis batas seleksi presisi 1px dengan handle resizer di 8 titik (4 sudut + 4 sisi).
  - Tampilan dimensi pixel live `W: 1920 × H: 1080` di dekat kursor atau sudut seleksi.
- **Loupe / Magnifier (Kaca Pembesar):**
  - Lingkaran / kotak zoom 5x–10x di sebelah kursor.
  - Menampilkan pixel grid dengan akurasi Retina 2x/3x.
  - Menampilkan nilai warna pixel di bawah kursor (format HEX `#FFFFFF` dan RGB `rgb(255,255,255)`). Tombol `C` menyalin kode warna secara instan.
- **Smart Window Snapping:**
  - Saat kursor hover di atas jendela tertentu sebelum drag, aplikasi otomatis mendeteksi batas jendela aplikasi di bawah kursor (`SCWindow` bounds) dan menawarkan 1-click select window.

---

### 4.4 Floating Action Toolbar

Setelah area selesai dipilih (mouse release), Floating Toolbar muncul secara halus dengan animasi spring macOS.

#### 4.4.1 Penempatan & Fleksibilitas
- **Smart Auto-Position:** Terletak di bagian bawah kanan area seleksi. Jika area berada di tepi bawah layar, toolbar otomatis pindah ke bagian atas seleksi (atau ke dalam area seleksi jika ruang sangat sempit).
- **Orientasi:** Mendukung toggle **Horizontal** (default) dan **Vertical** (via tombol rotate pada toolbar atau preference).

#### 4.4.2 Tombol & Aksi Toolbar
1. **📌 Pin (`P`):**
   - Mengubah area yang dipilih langsung menjadi Pin Window melayang.
2. **✏️ Annotate (`A` / `E`):**
   - Membuka sub-toolbar anotasi langsung pada area seleksi.
   - Alat anotasi:
     - Persegi & Lingkaran (Kotak/Oval terisi atau outline)
     - Panah & Garis lurus
     - Freehand Pen (Pena halus dengan smoothing bezier)
     - Highlighter (Translucent neon marker)
     - Text Box (Font native San Francisco, auto-grow)
     - Number Step Badge (1, 2, 3...) untuk pembuatan tutorial/dokumentasi.
     - Blur / Mosaic tool (Sensorship informasi rahasia).
     - Color Palette (Preset Apple colors: Red, Orange, Green, Blue, Purple, White, Black + Custom Color Wheel).
     - Stroke Size Picker (Thin, Medium, Thick).
     - Undo (`⌘Z`) & Redo (`⌘⇧Z`).
3. **📋 Copy (`⌘C` / Enter):**
   - Menyalin gambar akhir (beserta anotasi) langsung ke clipboard macOS (`NSPasteboard.general`).
   - Memainkan suara native macOS sound effect (`Tink` / custom feedback).
   - Menutup overlay capture seketika.
4. **💾 Save (`⌘S`):**
   - Menyimpan ke direktori default (misal: `~/Pictures/PinShots` atau `~/Desktop`) atau membuka `NSSavePanel` jika dikonfigurasi.
   - Format: PNG (lossless), JPEG, atau WebP.
5. **✕ Close (`Esc`):**
   - Membatalkan capture dan menutup overlay tanpa menyimpan apa pun.

---

### 4.5 Pinned Floating Window (Fitur Pin Utama)

Pin adalah jendela melayang independen hasil capture yang bertahan di atas semua aplikasi.

```
┌───────────────────────────────────────┐
│                                   [✕] │  <- Close button on hover
│                                       │
│          PINNED SCREENSHOT            │
│            IMAGE CONTENT              │
│                                       │
│                                       │
└───────────────────────────────────────┘
```

#### 4.5.1 Sifat Window Pin
- **Window Level:** `NSWindow.Level.floating` (atau `.popUpMenu`).
- **Style Mask:** `.borderless` dengan rounded corners (10pt) dan bayangan halus khas macOS (`NSWindow.shadow`).
- **Multi-Space Support:** `collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]`, sehingga pin tetap terlihat ketika pengguna berganti virtual desktop / Mission Control / Fullscreen app.
- **Multi-Instance:** Pengguna dapat membuat puluhan pin sekaligus tanpa degradasi performa (memory management berbasis lazy rendering).

#### 4.5.2 Interaksi & Kontrol Pin
- **Move:** Drag dari area mana pun pada gambar untuk memindahkan posisi jendela.
- **Resize:** 
  - Drag dari tepi/sudut window untuk memperbesar/memperkecil dengan menjaga aspect ratio gambar asli.
  - Support pinch-to-zoom pada trackpad.
- **Tombol Close (✕):**
  - Terletak di pojok kanan atas jendela pin.
  - Tampil secara halus saat mouse meng-hover jendela pin (transisi opacity 0.0 → 1.0).
- **Klik & Menu Aksi Cepat (Context Menu / On-Click HUD):**
  - **Klik Kiri:** Memfokuskan pin dan menampilkan action pill mini (Copy, Annotate, Opacity, Close).
  - **Double Click:** Menyesuaikan ukuran ke skala asli 100% (1:1 pixel) atau Fit.
  - **Klik Kanan (Context Menu):**
    - Copy Image (`⌘C`)
    - Save Image As... (`⌘S`)
    - Annotate / Edit
    - Opacity (100%, 75%, 50%, 25% atau slider interaktif)
    - Toggle Click-Through (Mengabaikan klik mouse agar tembus pandang ke aplikasi di bawahnya)
    - Always On Top (Toggle)
    - Close Pin (`⌘W` / `Esc`)
- **Scroll Wheel:** Mengatur zoom scale atau transparansi jendela (bisa dikonfigurasi via setting).

---

## 5. UI/UX & Design Guidelines (Apple HIG Compliant)

| Komponen | Spesifikasi Visual & Interaksi |
| :--- | :--- |
| **Material / Efek Kaca** | `NSVisualEffectView` / SwiftUI `.ultraThinMaterial` dengan material HUD/Sidebar khas macOS |
| **Sudut Melengkung (Corner Radius)** | `12pt` untuk Toolbar & HUD, `10pt` untuk Pin Window, `8pt` untuk Button/Input |
| **Tipografi** | SF Pro Text & SF Pro Rounded untuk angka badge/koordinat |
| **Ikonografi** | Apple SF Symbols 5 / 6 dengan rendering mode `.hierarchical` dan `.palette` |
| **Animasi** | Spring animation responsif: `response: 0.35, dampingFraction: 0.82` |
| **Aksesibilitas** | VoiceOver tags pada semua tombol toolbar, full keyboard accessibility navigation |

---

## 6. Persyaratan Teknis & Optimasi Performa

### 6.1 Concurrency & Swift 6 Safety
- Seluruh state management menggunakan arsitektur Modern Swift Concurrency (`@MainActor`, `actor`, non-blocking async background captures).
- Render grafis anotasi menggunakan hardware-accelerated **Metal** / **CoreGraphics / CoreImage** pipeline untuk menjamin rendering 60/120 FPS (ProMotion displays).

### 6.2 Konsumsi Sumber Daya
- **Idle RAM Usage:** < 25 MB saat berjalan di background menu bar.
- **Capture Latency:** Frame capture ke overlay interaktif < 40ms.
- **Battery Impact:** 0% CPU consumption saat dalam kondisi standby/idle.

---

## 7. Struktur Modul & Kode (Swift Package / Xcode Project Layout)

```
PinShot/
├── App/
│   ├── PinShotApp.swift                # SwiftUI App Lifecycle & Entry point
│   ├── AppDelegate.swift               # NSApplicationDelegate & Global Interceptors
│   └── AppState.swift                  # Observable global state (Active Pins, Permissions)
├── Services/
│   ├── PermissionService.swift         # Screen Recording & Accessibility manager
│   ├── HotKeyService.swift             # Carbon Global HotKey wrapper
│   ├── CaptureService.swift            # ScreenCaptureKit / Multi-monitor capture logic
│   ├── SoundService.swift              # Audio feedback manager
│   └── PasteboardService.swift         # Clipboard management
├── Overlay/
│   ├── MultiScreenOverlayManager.swift # Manages overlay windows on all screens
│   ├── OverlayWindow.swift             # Non-activating screenSaver level NSPanel
│   ├── OverlayCanvasView.swift         # Selection drag, guidelines, loupe view
│   ├── LoupeView.swift                 # Pixel magnifier and RGB/HEX display
│   └── FloatingToolbar/
│       ├── FloatingToolbarWindow.swift # Floating auto-positioning toolbar
│       ├── HorizontalToolbarView.swift # Horizontal button layout
│       └── VerticalToolbarView.swift   # Vertical button layout
├── Annotations/
│   ├── AnnotationEngine.swift          # Core drawing math & state
│   ├── Models/                         # Shape, Line, Arrow, Text, Step, Mosaic
│   ├── Renderers/                      # CoreGraphics/Metal drawing implementations
│   └── Views/AnnotationToolbarView.swift
├── Pin/
│   ├── PinWindowManager.swift          # Pin collection & lifecycle coordinator
│   ├── PinWindow.swift                 # Always-on-top borderless NSPanel
│   ├── PinContentView.swift            # Scalable, zoomable, draggable image view
│   └── PinHoverControlsView.swift      # ✕ close button & quick action pills
├── Settings/
│   ├── SettingsView.swift              # General, Shortcuts, Output, Appearance tabs
│   └── PermissionOnboardingView.swift  # Beautiful onboarding window for missing permissions
└── Resources/
    ├── Assets.xcassets                 # Icons, SF symbols, color sets
    └── Localizable.xcstrings           # Multi-language support (English, Indonesian, etc.)
```

---

## 8. Roadmap Pengembangan & Tahapan Implementasi

1. **Fase 1: Foundation & Menu Bar Core**
   - Setup project Swift/SwiftUI, permission verification service, menu bar status item, dan Carbon global hotkey listener.
2. **Fase 2: Multi-Monitor ScreenCapture Engine & Overlay**
   - Pengambilan frame multi-screen via ScreenCaptureKit, unified coordinate mapping, transparent interactive overlay window dengan crosshair selection dan loupe magnifier.
3. **Fase 3: Floating Action Toolbar & Output**
   - Pembuatan floating toolbar adaptif (Horizontal/Vertical), fungsi Copy to Clipboard, Save to File, dan Dismiss.
4. **Fase 4: Native Pinned Floating Windows**
   - Implementasi `PinWindow` dengan drag, aspect-ratio resize, zoom, opacity adjust, multi-instance handling, dan hover close (✕).
5. **Fase 5: Rich In-Place Annotations**
   - Toolset anotasi lengkap (Shapes, Arrow, Text, Step numbers, Blur/Mosaic) dengan undo/redo.
6. **Fase 6: Polish, Performance & HIG Refinement**
   - Metal rendering optimization, fluid spring transitions, memory leak audit, dan Dark/Light theme testing.
