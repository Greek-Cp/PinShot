import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

// One floating pin window. It loads its image from the shell, fills the window
// with it, and lets the user drag to move (native OS drag), double-click to
// close, and press C to copy / Esc to close. All pixels and geometry come from
// the Rust core; this page is a thin renderer (Constitution IV).

interface PinImagePayload {
  width: number;
  height: number;
  scaleFactor: number;
  dataUrl: string;
}

const pinId = Number(new URLSearchParams(location.search).get('id') ?? '0');

// Treat two quick presses as a close gesture; a single press starts a drag.
const DOUBLE_CLICK_MS = 300;
let lastDownAt = 0;

async function close(): Promise<void> {
  try {
    await invoke('close_pin', { pinId });
  } catch (e) {
    console.error('close_pin failed', e);
  }
}

async function copy(): Promise<void> {
  try {
    await invoke('copy_pin', { pinId });
  } catch (e) {
    console.error('copy_pin failed', e);
  }
}

async function onMouseDown(e: MouseEvent): Promise<void> {
  if (e.button !== 0) {
    return;
  }
  const now = Date.now();
  if (now - lastDownAt < DOUBLE_CLICK_MS) {
    // Second quick press → close this pin (the chosen close gesture).
    lastDownAt = 0;
    await close();
    return;
  }
  lastDownAt = now;
  // Bring to front, then hand the move to the OS for jank-free dragging.
  try {
    await invoke('raise_pin', { pinId });
    await getCurrentWindow().startDragging();
  } catch (err) {
    console.error('drag failed', err);
  }
}

function onKeyDown(e: KeyboardEvent): void {
  if (e.key === 'Escape') {
    void close();
  } else if (e.key === 'c' || e.key === 'C') {
    void copy();
  }
}

async function init(): Promise<void> {
  document.body.style.background = 'transparent';

  let payload: PinImagePayload;
  try {
    payload = await invoke<PinImagePayload>('get_pin_image', { pinId });
  } catch (e) {
    console.error('could not load pin image', e);
    return;
  }

  const img = new Image();
  img.src = payload.dataUrl;
  img.style.cssText =
    'position:fixed;inset:0;width:100vw;height:100vh;display:block;' +
    'object-fit:fill;-webkit-user-drag:none;user-select:none;pointer-events:none;';
  await img.decode().catch(() => undefined);
  document.body.appendChild(img);

  window.addEventListener('mousedown', (e) => void onMouseDown(e));
  window.addEventListener('keydown', onKeyDown);
  // No context menu on a pin.
  window.addEventListener('contextmenu', (e) => e.preventDefault());
}

void init();
