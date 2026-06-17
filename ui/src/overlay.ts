import { invoke } from '@tauri-apps/api/core';

// One overlay window per display. It renders that display's frozen frame, lets
// the user drag a selection, shows a magnifier + dimensions + pixel color, and
// reports the selection back to the shell. All physical-pixel/DPI math lives in
// the Rust core; here we only report logical (CSS) coordinates.

interface FramePayload {
  width: number;
  height: number;
  scaleFactor: number;
  originX: number;
  originY: number;
  dataUrl: string;
}

interface CommitResponse {
  output: string;
  path: string | null;
}

const displayId = Number(new URLSearchParams(location.search).get('display') ?? '0');

// Offscreen canvas holding the frozen frame at physical resolution, used for
// pixel-exact color sampling and the magnifier.
const sampler = document.createElement('canvas');
const samplerCtx = sampler.getContext('2d', { willReadFrequently: true });

let scaleFactor = 1;
let dragging = false;
let hasSelection = false;
let startX = 0;
let startY = 0;
let curX = 0;
let curY = 0;
let cursorHex = '#000000';

const selectionEl = document.createElement('div');
const hud = document.createElement('div');
const magCanvas = document.createElement('canvas');
const swatch = document.createElement('span');
const readout = document.createElement('span');

function styleUi(): void {
  document.body.style.background = '#000';

  const bg = document.querySelector<HTMLImageElement>('#frame');
  if (bg) {
    bg.style.cssText =
      'position:fixed;inset:0;width:100vw;height:100vh;display:block;-webkit-user-drag:none;';
  }

  selectionEl.style.cssText =
    'position:fixed;border:1px solid #4f46e5;box-shadow:0 0 0 9999px rgba(0,0,0,0.45);display:none;pointer-events:none;z-index:10;';
  document.body.appendChild(selectionEl);

  magCanvas.width = 120;
  magCanvas.height = 120;
  magCanvas.style.cssText =
    'position:fixed;width:120px;height:120px;border:1px solid #fff;image-rendering:pixelated;display:none;pointer-events:none;z-index:20;background:#000;';
  document.body.appendChild(magCanvas);

  swatch.style.cssText =
    'display:inline-block;width:12px;height:12px;border:1px solid #fff;vertical-align:middle;margin-right:6px;';
  readout.style.cssText = 'vertical-align:middle;';
  hud.style.cssText =
    'position:fixed;padding:4px 8px;background:rgba(0,0,0,0.8);color:#fff;font:12px/1.4 system-ui,sans-serif;border-radius:4px;display:none;pointer-events:none;z-index:20;white-space:nowrap;';
  hud.appendChild(swatch);
  hud.appendChild(readout);
  document.body.appendChild(hud);
}

function physical(v: number): number {
  return Math.round(v * scaleFactor);
}

function rectFromDrag(): { x: number; y: number; width: number; height: number } {
  const x = Math.min(startX, curX);
  const y = Math.min(startY, curY);
  const width = Math.abs(curX - startX);
  const height = Math.abs(curY - startY);
  return { x, y, width, height };
}

function updateColorAt(lx: number, ly: number): void {
  if (!samplerCtx) {
    return;
  }
  const px = Math.min(physical(lx), sampler.width - 1);
  const py = Math.min(physical(ly), sampler.height - 1);
  const data = samplerCtx.getImageData(px, py, 1, 1).data;
  const hex = `#${[data[0], data[1], data[2]]
    .map((c) => c.toString(16).padStart(2, '0'))
    .join('')
    .toUpperCase()}`;
  cursorHex = hex;
  swatch.style.background = hex;
  const rgb = `rgb(${data[0]}, ${data[1]}, ${data[2]})`;
  const dims =
    dragging || hasSelection
      ? `${physical(rectFromDrag().width)}×${physical(rectFromDrag().height)}  `
      : '';
  readout.textContent = `${dims}${hex}  ${rgb}`;
}

function drawMagnifier(lx: number, ly: number): void {
  const ctx = magCanvas.getContext('2d');
  if (!ctx) {
    return;
  }
  const srcSize = 15; // physical px sampled around the cursor
  const px = physical(lx);
  const py = physical(ly);
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, magCanvas.width, magCanvas.height);
  ctx.drawImage(
    sampler,
    px - srcSize / 2,
    py - srcSize / 2,
    srcSize,
    srcSize,
    0,
    0,
    magCanvas.width,
    magCanvas.height,
  );
  // crosshair
  ctx.strokeStyle = 'rgba(79,70,229,0.9)';
  const mid = magCanvas.width / 2;
  ctx.strokeRect(mid - 4, mid - 4, 8, 8);
}

function positionFloaters(lx: number, ly: number): void {
  const offset = 16;
  magCanvas.style.left = `${Math.min(lx + offset, window.innerWidth - 130)}px`;
  magCanvas.style.top = `${Math.min(ly + offset, window.innerHeight - 160)}px`;
  hud.style.left = `${Math.min(lx + offset, window.innerWidth - 220)}px`;
  hud.style.top = `${Math.min(ly + offset + 124, window.innerHeight - 30)}px`;
}

function renderSelection(): void {
  const r = rectFromDrag();
  selectionEl.style.left = `${r.x}px`;
  selectionEl.style.top = `${r.y}px`;
  selectionEl.style.width = `${r.width}px`;
  selectionEl.style.height = `${r.height}px`;
  selectionEl.style.display = 'block';
}

async function commit(output: 'clipboard' | 'file'): Promise<void> {
  const r = rectFromDrag();
  if (r.width === 0 || r.height === 0) {
    return;
  }
  try {
    await invoke<CommitResponse>('commit_selection', { displayId, rect: r, output });
  } catch (e) {
    console.error('commit failed', e);
  }
}

async function cancel(): Promise<void> {
  try {
    await invoke('cancel_capture');
  } catch (e) {
    console.error('cancel failed', e);
  }
}

function onMouseDown(e: MouseEvent): void {
  if (e.button !== 0) {
    return;
  }
  dragging = true;
  hasSelection = false;
  startX = e.clientX;
  startY = e.clientY;
  curX = e.clientX;
  curY = e.clientY;
  renderSelection();
}

function onMouseMove(e: MouseEvent): void {
  curX = e.clientX;
  curY = e.clientY;
  if (dragging) {
    renderSelection();
  }
  updateColorAt(e.clientX, e.clientY);
  drawMagnifier(e.clientX, e.clientY);
  positionFloaters(e.clientX, e.clientY);
  magCanvas.style.display = 'block';
  hud.style.display = 'block';
}

function onMouseUp(e: MouseEvent): void {
  if (e.button !== 0 || !dragging) {
    return;
  }
  dragging = false;
  const r = rectFromDrag();
  hasSelection = r.width > 0 && r.height > 0;
}

function onKeyDown(e: KeyboardEvent): void {
  switch (e.key) {
    case 'Escape':
      void cancel();
      break;
    case 'Enter':
      void commit('clipboard');
      break;
    case 's':
    case 'S':
      void commit('file');
      break;
    case 'c':
    case 'C':
      void invoke('copy_color', { hex: cursorHex }).catch((err) => console.error(err));
      break;
    default:
      break;
  }
}

async function init(): Promise<void> {
  styleUi();
  let frame: FramePayload;
  try {
    frame = await invoke<FramePayload>('get_overlay_frame', { displayId });
  } catch (e) {
    console.error('could not load frame', e);
    return;
  }
  scaleFactor = frame.scaleFactor;

  const img = new Image();
  img.id = 'frame';
  img.src = frame.dataUrl;
  await img.decode().catch(() => undefined);
  document.body.insertBefore(img, document.body.firstChild);
  styleUi(); // re-apply now that #frame exists

  sampler.width = frame.width;
  sampler.height = frame.height;
  samplerCtx?.drawImage(img, 0, 0, frame.width, frame.height);

  window.addEventListener('mousedown', onMouseDown);
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    void cancel();
  });
}

void init();
