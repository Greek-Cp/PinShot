import { listen } from '@tauri-apps/api/event';

// Main window. PinShot lives in the tray; this window is informational. The
// capture flow runs entirely in the overlay windows (see overlay.ts). We do,
// however, surface capture errors here (e.g. the macOS Screen Recording
// permission/relaunch guidance) so they are visible rather than silent.
const statusEl = document.querySelector<HTMLParagraphElement>('#status');
if (statusEl) {
  statusEl.textContent = 'Press Cmd/Ctrl+Shift+A, or use the tray icon, to capture.';
}

const errorEl = document.querySelector<HTMLDivElement>('#error');
void listen<string>('capture://error', (event) => {
  if (errorEl) {
    errorEl.textContent = event.payload;
    errorEl.style.display = 'block';
  }
});
