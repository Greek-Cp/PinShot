# Pinshot — Roadmap & Project Document

> Screenshot & pin tool sekelas Snipaste. Open source, 100% offline, untuk macOS & Windows.
> OCR dan mode Beautify masih fitur roadmap, belum diimplementasikan saat ini.

---

## 1. Latar Belakang & Masalah yang Ingin Dipecahkan

### Masalah

1. **Snipaste closed-source dan terbatas platform.** Tool screenshot + pin terbaik saat ini (Snipaste) tidak open source, versi macOS-nya masih beta, dan pengembangannya tertutup. Komunitas tidak bisa berkontribusi atau mengaudit kodenya.

2. **Tool yang ada mengorbankan privasi.** Banyak screenshot tool modern (CleanShot, dsb.) mendorong cloud upload, butuh akun, atau mengirim telemetry. Screenshot sering berisi data sangat sensitif (chat, email, dashboard internal, API key) — seharusnya tidak pernah meninggalkan perangkat tanpa izin eksplisit.

3. **Tidak ada solusi open source yang menggabungkan: pin window + OCR + hasil yang siap-share.** Flameshot bagus tapi tidak punya pin & OCR. Shottr bagus tapi closed-source dan Mac-only. ShareX powerful tapi Windows-only dan UX-nya berat.

4. **Workflow "screenshot → ambil teksnya" masih ribet.** Use case harian (copy error message, copy teks dari gambar/video call/PDF ter-protect) belum terlayani dengan baik secara gratis dan offline di kedua platform.

### Solusi

Satu aplikasi ringan yang hidup di tray/menu bar dengan tiga pilar:

| Pilar                           | Deskripsi                                                                                                                                    |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| **Privacy-first**               | 100% offline. Zero telemetry. No account. No cloud. Bisa diaudit (open source).                                                              |
| **OCR built-in (direncanakan)** | Screenshot area → teks langsung bisa di-copy. Fitur ini masih tahap roadmap dan belum tersedia di build saat ini.                            |
| **Beautify (direncanakan)**     | Hasil screenshot otomatis cantik dengan background, padding, dan shadow. Fitur ini masih tahap roadmap dan belum tersedia di build saat ini. |

Di atas fondasi core yang setara Snipaste: capture cepat, pin floating window, color picker, anotasi.

### Target Pengguna

- Developer & engineer (copy error, share snippet, dokumentasi bug)

- Content creator & technical writer (tutorial, sosmed)

- Pekerja kantoran yang sering share screenshot internal (butuh redact & privasi)

- Pengguna Snipaste yang ingin alternatif open source / pengguna Mac yang iri fitur pin

---

## 2. Prinsip Desain

1. **Cepat adalah fitur #1.** Hotkey → overlay muncul

| Komponen             | Pilihan                                                       | Alasan                                                                        |
| -------------------- | ------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| Bahasa inti          | **Rust**                                                      | Performa native, memory-safe, binary kecil, cocok untuk app background 24 jam |
| Framework app        | **Tauri 2.x**                                                 | Multiplatform (macOS + Windows), bundle ~5MB, tray support bawaan             |
| UI overlay & editor  | **Web (TypeScript + Svelte/SolidJS)** via Tauri webview       | Iterasi UI cepat, animasi smooth                                              |
| Screen capture       | **xcap** (Rust crate)                                         | API seragam untuk macOS & Windows, support multi-monitor                      |
| Global hotkey        | **global-hotkey** (crate resmi Tauri)                         | Cross-platform, remappable                                                    |
| Tray / menu bar      | **tray-icon** (crate resmi Tauri)                             | Cross-platform                                                                |
| OCR — macOS          | **Apple Vision framework** (via FFI)                          | Akurasi terbaik, offline, gratis, cepat                                       |
| OCR — Windows        | **Windows.Media.Ocr** (WinRT API)                             | Built-in OS, offline, tanpa dependency besar                                  |
| OCR fallback         | **Tesseract** (opsional, untuk bahasa tambahan)               | Open source, mendukung banyak bahasa                                          |
| Image processing     | **image** + **fast_image_resize** (Rust)                      | Crop, resize, blur/mosaic performan                                           |
| Persistensi settings | File TOML/JSON lokal                                          | Sederhana, transparan, mudah di-backup                                        |
| CI/CD                | **GitHub Actions**                                            | Build matrix macOS + Windows tiap commit                                      |
| Distribusi           | DMG + Homebrew Cask (Mac); MSI/EXE + winget + Scoop (Windows) | Jangkauan maksimal                                                            |

**Catatan arsitektur:**

- Core logic (capture, OCR, image ops) ditulis sebagai Rust library terpisah dari shell Tauri → memudahkan testing & potensi port ke Linux nanti.

- Tidak ada dependency network di core. Fitur "check for update" diisolasi dan opt-in.

---

## 3. Daftar Fitur

Catatan: fitur OCR dan Beautify di bagian ini masih target roadmap, belum tersedia di versi awal aplikasi.

### Core (paritas Snipaste)

- **Capture area**: drag seleksi bebas; auto-detect window/elemen UI saat hover; adjust seleksi dengan handle & arrow keys

- **Magnifier**: zoom pixel + koordinat + ukuran W×H saat seleksi

- **Pin**: hasil capture / isi clipboard jadi floating window always-on-top; geser bebas antar monitor; zoom (scroll), opacity (Ctrl/Cmd+scroll), rotate/flip; click-through mode; hide/show semua pin via hotkey

- **Color picker**: nilai warna pixel di magnifier (HEX/RGB/HSL), tekan C untuk copy; mode standalone

- **Anotasi**: panah, kotak, lingkaran, garis, freehand, teks, blur/mosaic, step numbering, undo/redo

- **Output**: clipboard, save file (PNG/JPG), pin, drag & drop ke app lain

- **Hotkeys**: semua remappable, deteksi konflik

- **Settings**: auto-start, format & folder default, pola nama file, tema light/dark/system

- **History**: recall N screenshot terakhir

### Pembeda 1 — Privacy-First

- 100% offline: tidak ada satu pun request network dari core app

- Zero telemetry, zero analytics, zero account

- Pernyataan privasi eksplisit di README + dapat diverifikasi dari source code

- History & temporary file disimpan lokal terenkripsi-opsional, dengan tombol "clear all"

- Update check opt-in (default mati), hanya fetch file version statis

### Pembeda 2 — OCR Built-in

- Hotkey khusus "Capture Text": seleksi area → teks langsung di clipboard

- Tombol OCR di toolbar hasil capture & di context menu pin (pin lama pun bisa di-OCR)

- Offline penuh: Vision (Mac) / Windows OCR (Win), Tesseract opsional untuk bahasa lain

- Preservasi line break & deteksi kolom sederhana

- Target bahasa awal: English + Indonesian (lalu CJK via Tesseract)

### Pembeda 3 — Beautify

- Toggle "Beautify" di toolbar hasil capture: otomatis tambah background (gradient/solid/wallpaper), padding, rounded corner, shadow

- Preset siap pakai + custom preset tersimpan

- Auto-balance: ukuran canvas mengikuti rasio populer (1:1, 4:3, 16:9, ukuran post Twitter/LinkedIn)

- Watermark opsional milik user (bukan branding app)

- Ekspor langsung ke clipboard/file dalam resolusi 1x/2x

---

## 4. Roadmap

### v0.1 — MVP "Capture & Pin" (Bulan 1–3)

Tujuan: bisa dipakai harian menggantikan tool screenshot bawaan OS.

- [ ] Capture area + auto-detect window + magnifier + color info

- [ ] Output: clipboard & save file

- [ ] Pin dasar: floating window, geser, zoom, opacity, close

- [ ] Global hotkey (belum remappable, default per platform)

- [ ] Tray / menu bar icon dengan menu minimal

- [ ] Multi-monitor & mixed DPI berfungsi benar

- [ ] CI build macOS + Windows

- [ ] Rilis alpha untuk early adopter (GitHub Releases)

**Definition of done:** hotkey → overlay < 100ms; capture benar di setup multi-monitor mixed-DPI.

### v0.2 — "Daily Driver" (Bulan 4–5)

- [ ] Anotasi dasar: panah, kotak, teks, blur/mosaic, undo/redo

- [ ] Hotkey remappable + deteksi konflik

- [ ] Settings window lengkap (format, folder, nama file, tema, auto-start)

- [ ] Pin dari clipboard, hide/show semua pin

- [ ] Color picker standalone

- [ ] Distribusi: Homebrew Cask, winget, Scoop

### v0.3 — "OCR" (Bulan 6–7) 🚀 momen launch publik

- [ ] OCR via Vision (Mac) & Windows OCR (Win)

- [ ] Hotkey "Capture Text" → teks ke clipboard

- [ ] OCR dari pin & dari history

- [ ] Bahasa: EN + ID

- [ ] **Launch:** post ke Hacker News / Product Hunt / r/macapps / r/software dengan demo GIF OCR

### v0.4 — "Beautify" (Bulan 8–9)

- [ ] Engine beautify: background, padding, corner, shadow

- [ ] Preset + rasio sosmed + ekspor 1x/2x

- [ ] Demo GIF kedua untuk gelombang marketing berikutnya

### v0.5 — Polish & Power Features (Bulan 10–12)

- [ ] History panel dengan pencarian (termasuk cari berdasarkan teks hasil OCR!)

- [ ] Step numbering, eraser, marker pada anotasi

- [ ] Pin lanjutan: rotate/flip, click-through, double-click action

- [ ] Tesseract opsional untuk bahasa tambahan

- [ ] Localization framework (mulai: EN, ID)

### v1.0 — Stabil (Bulan 12+)

- [ ] Code signing & notarization (Mac), signing (Windows)

- [ ] Dokumentasi lengkap + website landing page

- [ ] Auto-redact (deteksi email/nomor/API key → blur sekali klik) — kandidat headline v1.x

- [ ] Eksplorasi Linux (X11 dulu)

### Backlog / Ide (tidak dijanjikan)

- Scrolling capture

- Pin workspace/group

- Plugin system

- Screen recording (kemungkinan besar out of scope — fokus tetap still image)

---

## 5. Yang Sengaja TIDAK Dibuat (Non-Goals)

- ❌ Cloud upload / sharing service — bertentangan dengan pilar privacy

- ❌ Akun, login, sync

- ❌ Telemetry & analytics dalam bentuk apa pun

- ❌ Screen recording (setidaknya sampai v1.0)

- ❌ Fitur AI berbasis API online

---

## 6. Risiko & Mitigasi

| Risiko                                                         | Mitigasi                                                                                            |
| -------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Bug multi-monitor / mixed DPI                                  | Jadikan test case wajib sejak v0.1; minta early adopter dengan setup beragam                        |
| Permission Screen Recording di macOS membingungkan user        | Onboarding flow yang memandu langkah demi langkah                                                   |
| Tanpa code signing, SmartScreen/Gatekeeper menakut-nakuti user | Dokumentasikan cara bypass; targetkan signing di v1.0; distribusi via brew/winget mengurangi friksi |
| OCR Windows kalah akurat dari Vision                           | Sediakan Tesseract sebagai fallback opsional                                                        |
| Burnout maintainer solo                                        | Scope MVP ketat; terima kontributor sejak awal; roadmap publik agar ekspektasi jelas                |
| Kompetitor (Snipaste/Shottr) menambah fitur serupa             | Keunggulan struktural: open source + offline tidak bisa mereka tiru tanpa mengubah model bisnis     |

---

## 7. Metrik Keberhasilan

- v0.3 (launch): masuk front page HN atau top 5 Product Hunt hari itu; 1.000+ GitHub stars dalam bulan pertama launch

- v1.0: 10.000+ downloads, 10+ kontributor eksternal, app dipakai harian oleh maintainer sendiri tanpa keluhan

- Kualitas: cold capture latency 99.5%

---

## 8. Lisensi & Komunitas

- **Lisensi:** **GPL-3.0-or-later** (final). Copyleft — bebas dipakai, dipelajari, dimodifikasi, dan di-fork; tapi setiap fork/turunan yang didistribusikan **wajib tetap open source** dengan lisensi yang sama, sehingga tidak bisa dijadikan produk closed-source. Lihat file [LICENSE](LICENSE).

- Kontribusi terbuka: CONTRIBUTING.md, good-first-issue labels, arsitektur terdokumentasi

- Diskusi via GitHub Discussions; roadmap ini hidup sebagai issue ter-pin

---

_Dokumen ini adalah living document — akan diperbarui seiring perkembangan proyek._
