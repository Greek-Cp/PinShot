// Main window. PinShot lives in the tray; this window is informational for now.
// The capture flow runs entirely in the overlay windows (see overlay.ts).
const statusEl = document.querySelector<HTMLParagraphElement>('#status');
if (statusEl) {
  statusEl.textContent = 'Press Cmd/Ctrl+Shift+A, or use the tray icon, to capture.';
}
