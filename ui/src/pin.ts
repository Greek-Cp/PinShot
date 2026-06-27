import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

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

// --- Zoom / resize (Snipaste-style scroll & trackpad pinch) ------------------
const win = getCurrentWindow();
const MIN_SCALE = 0.15;
const MAX_SCALE = 8;
// Logical (CSS px) size the pin was created at; zoom multiplies this.
let baseW = 0;
let baseH = 0;
let scale = 1;
let targetScale = 1;
let zoomQueued = false;

function applyZoom(): void {
  zoomQueued = false;
  scale = targetScale;
  void win.setSize(new LogicalSize(Math.round(baseW * scale), Math.round(baseH * scale)));
}

function queueZoom(): void {
  if (!zoomQueued) {
    zoomQueued = true;
    requestAnimationFrame(applyZoom);
  }
}

function onWheel(e: WheelEvent): void {
  e.preventDefault();
  // macOS trackpad pinch arrives as a wheel event with ctrlKey set; use a finer,
  // continuous factor for it and discrete steps for a real scroll wheel.
  const factor = e.ctrlKey ? Math.exp(-e.deltaY * 0.01) : e.deltaY < 0 ? 1.08 : 1 / 1.08;
  targetScale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, targetScale * factor));
  queueZoom();
}

function resetZoom(): void {
  targetScale = 1;
  queueZoom();
}

// --- Selected state: a border around the pin + a Copy button -----------------
// Clicking a pin selects it: a border frames the whole image so you can tell it
// is active, and a Copy button appears. The border tracks window focus.
const borderEl = document.createElement('div');
const copyBtn = document.createElement('button');

function buildSelectionUi(): void {
  borderEl.style.cssText =
    'position:fixed;inset:0;box-sizing:border-box;border:2px solid #4f46e5;border-radius:4px;' +
    'display:none;pointer-events:none;z-index:10;';
  document.body.appendChild(borderEl);

  copyBtn.textContent = 'Copy';
  copyBtn.style.cssText =
    'position:fixed;right:8px;bottom:8px;display:none;z-index:11;appearance:none;border:0;border-radius:6px;' +
    'padding:5px 12px;font:600 12px/1 system-ui,sans-serif;color:#fff;background:#4f46e5;cursor:pointer;' +
    'box-shadow:0 3px 10px rgba(0,0,0,0.4);';
  copyBtn.addEventListener('mousedown', (e) => e.stopPropagation());
  copyBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    void copyAndFlash();
  });
  document.body.appendChild(copyBtn);
}

function setSelected(on: boolean): void {
  borderEl.style.display = on ? 'block' : 'none';
  copyBtn.style.display = on ? 'block' : 'none';
}

async function copyAndFlash(): Promise<void> {
  await copy();
  copyBtn.textContent = 'Copied ✓';
  setTimeout(() => {
    copyBtn.textContent = 'Copy';
  }, 800);
}

async function savePin(): Promise<void> {
  hideCtxMenu();
  try {
    await invoke('save_pin', { pinId });
  } catch (e) {
    console.error('save_pin failed', e);
  }
}

// --- Right-click context menu (Copy / Save) ----------------------------------
const ctxMenu = document.createElement('div');

function ctxItem(label: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.textContent = label;
  b.style.cssText =
    'display:block;width:100%;text-align:left;appearance:none;border:0;background:none;color:#fff;' +
    'font:13px/1 system-ui,sans-serif;padding:9px 14px;cursor:pointer;border-radius:6px;';
  b.addEventListener('mouseenter', () => (b.style.background = 'rgba(255,255,255,0.12)'));
  b.addEventListener('mouseleave', () => (b.style.background = 'none'));
  b.addEventListener('click', (e) => {
    e.stopPropagation();
    onClick();
  });
  return b;
}

function buildContextMenu(): void {
  ctxMenu.style.cssText =
    'position:fixed;display:none;z-index:20;background:#2c2c2e;border:1px solid rgba(255,255,255,0.12);' +
    'border-radius:10px;box-shadow:0 8px 24px rgba(0,0,0,0.5);padding:4px;min-width:140px;';
  ctxMenu.append(
    ctxItem('Copy', () => void copyAndFlash()),
    ctxItem('Save', () => void savePin()),
  );
  ctxMenu.addEventListener('mousedown', (e) => e.stopPropagation());
  document.body.appendChild(ctxMenu);
}

function showCtxMenu(x: number, y: number): void {
  ctxMenu.style.left = `${Math.min(x, window.innerWidth - 150)}px`;
  ctxMenu.style.top = `${Math.min(y, window.innerHeight - 90)}px`;
  ctxMenu.style.display = 'block';
}

function hideCtxMenu(): void {
  ctxMenu.style.display = 'none';
}

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
  hideCtxMenu();
  if (e.button !== 0) {
    return;
  }
  // Clicking selects the pin (shows its border + Copy button).
  setSelected(true);
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
    void copyAndFlash();
  } else if (e.key === '+' || e.key === '=') {
    targetScale = Math.min(MAX_SCALE, targetScale * 1.1);
    queueZoom();
  } else if (e.key === '-' || e.key === '_') {
    targetScale = Math.max(MIN_SCALE, targetScale / 1.1);
    queueZoom();
  } else if (e.key === '0') {
    resetZoom();
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
  buildSelectionUi();
  buildContextMenu();

  // Record the pin's starting logical size as the zoom baseline.
  try {
    const size = await win.innerSize();
    const sf = await win.scaleFactor();
    baseW = size.width / sf;
    baseH = size.height / sf;
  } catch (e) {
    console.error('could not read pin size', e);
    baseW = payload.width / payload.scaleFactor;
    baseH = payload.height / payload.scaleFactor;
  }

  window.addEventListener('mousedown', (e) => void onMouseDown(e));
  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('wheel', onWheel, { passive: false });
  // Right-click opens the pin's Copy / Save menu.
  window.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    setSelected(true);
    showCtxMenu(e.clientX, e.clientY);
  });

  // The border tracks window focus: a focused pin is "selected".
  window.addEventListener('focus', () => setSelected(true));
  window.addEventListener('blur', () => setSelected(false));
  setSelected(document.hasFocus());
}

void init();
