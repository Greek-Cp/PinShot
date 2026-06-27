import { invoke } from '@tauri-apps/api/core';

// One overlay window per display. It renders that display's frozen frame, lets
// the user drag a selection, shows a magnifier + dimensions + pixel color, and
// reports the selection back to the shell. All physical-pixel/DPI math lives in
// the Rust core; here we only report logical (CSS) coordinates.
//
// Performance notes: pointer-driven work (selection, magnifier, color sampling)
// is coalesced into a single requestAnimationFrame tick so a burst of mousemove
// events paints at most once per frame. The dimmed backdrop is drawn with four
// lightweight panels around the selection instead of a giant `box-shadow`
// spread, which is very cheap to repaint while dragging.

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

interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
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
let pointerX = 0;
let pointerY = 0;
let cursorHex = '#000000';
let rafPending = false;

// Smallest selection (logical px) so resize handles stay grabbable and the
// output never collapses to zero (mirrors core `Rect::clamp_min`).
const MIN_SIZE = 8;
// How close (logical px) the pointer must be to an edge to grab its handle.
const HANDLE_HIT = 10;

// Once the initial drag is released the selection becomes an explicit rect the
// user can resize (via handles) or move (drag inside). `dragging` still drives
// the *initial* draw; `adjusting` drives post-release edits.
let sel: Rect = { x: 0, y: 0, width: 0, height: 0 };
let adjusting: 'resize' | 'move' | null = null;
// Which edges a resize drag moves, and the move-mode grab offset.
let grab = { left: false, right: false, top: false, bottom: false };
let moveOffX = 0;
let moveOffY = 0;

// Eight resize handles (4 corners + 4 edges), shown once a selection exists.
const HANDLES = [
  { id: 'nw', cursor: 'nwse-resize', edges: { left: true, top: true } },
  { id: 'n', cursor: 'ns-resize', edges: { top: true } },
  { id: 'ne', cursor: 'nesw-resize', edges: { right: true, top: true } },
  { id: 'e', cursor: 'ew-resize', edges: { right: true } },
  { id: 'se', cursor: 'nwse-resize', edges: { right: true, bottom: true } },
  { id: 's', cursor: 'ns-resize', edges: { bottom: true } },
  { id: 'sw', cursor: 'nesw-resize', edges: { left: true, bottom: true } },
  { id: 'w', cursor: 'ew-resize', edges: { left: true } },
] as const;
const handleEls: HTMLDivElement[] = [];

// Backdrop dim is four panels framing the selection (cheap to repaint) instead
// of one element with a huge box-shadow spread.
const dimTop = document.createElement('div');
const dimBottom = document.createElement('div');
const dimLeft = document.createElement('div');
const dimRight = document.createElement('div');
const dimPanels = [dimTop, dimBottom, dimLeft, dimRight];

const selectionEl = document.createElement('div');
const sizeBadge = document.createElement('div');
const magCanvas = document.createElement('canvas');
const hud = document.createElement('div');
const swatch = document.createElement('span');
const readout = document.createElement('span');
const hint = document.createElement('div');
const toolbar = document.createElement('div');

const ACCENT = '#4f46e5';

function makeButton(label: string, onClick: () => void): HTMLButtonElement {
  const button = document.createElement('button');
  button.textContent = label;
  button.style.cssText =
    'appearance:none;border:0;border-radius:6px;padding:6px 12px;font:600 12px/1 system-ui,sans-serif;' +
    'color:#fff;background:rgba(255,255,255,0.12);cursor:pointer;';
  button.addEventListener('mouseenter', () => {
    button.style.background = 'rgba(255,255,255,0.24)';
  });
  button.addEventListener('mouseleave', () => {
    button.style.background = 'rgba(255,255,255,0.12)';
  });
  // Don't let a click on the toolbar start a fresh selection underneath it.
  button.addEventListener('mousedown', (e) => e.stopPropagation());
  button.addEventListener('click', (e) => {
    e.stopPropagation();
    onClick();
  });
  return button;
}

function buildUi(): void {
  document.body.style.background = '#000';
  // Dragging a selection must not trigger the webview's native text/element
  // selection — that paints the blue highlight over the frame image and HUD
  // text. Suppress it for the whole overlay; PinShot draws its own rectangle.
  document.body.style.userSelect = 'none';
  (document.body.style as CSSStyleDeclaration & { webkitUserSelect: string }).webkitUserSelect =
    'none';
  document.body.style.cursor = 'crosshair';

  for (const panel of dimPanels) {
    panel.style.cssText =
      'position:fixed;background:rgba(0,0,0,0.5);display:none;pointer-events:none;z-index:5;';
    document.body.appendChild(panel);
  }

  selectionEl.style.cssText =
    `position:fixed;border:1px solid ${ACCENT};box-shadow:inset 0 0 0 1px rgba(255,255,255,0.4);` +
    'display:none;pointer-events:none;z-index:10;';
  document.body.appendChild(selectionEl);

  for (const h of HANDLES) {
    const el = document.createElement('div');
    el.style.cssText =
      `position:fixed;width:10px;height:10px;margin:-5px 0 0 -5px;background:#fff;` +
      `border:1px solid ${ACCENT};border-radius:2px;display:none;pointer-events:none;` +
      `z-index:12;cursor:${h.cursor};`;
    handleEls.push(el);
    document.body.appendChild(el);
  }

  sizeBadge.style.cssText =
    'position:fixed;padding:2px 6px;background:rgba(0,0,0,0.8);color:#fff;' +
    'font:11px/1.4 ui-monospace,monospace;border-radius:4px;display:none;pointer-events:none;z-index:21;';
  document.body.appendChild(sizeBadge);

  magCanvas.width = 120;
  magCanvas.height = 120;
  magCanvas.style.cssText =
    'position:fixed;width:120px;height:120px;border:1px solid #fff;border-radius:6px;' +
    'image-rendering:pixelated;display:none;pointer-events:none;z-index:20;background:#000;';
  document.body.appendChild(magCanvas);

  swatch.style.cssText =
    'display:inline-block;width:12px;height:12px;border:1px solid #fff;border-radius:2px;vertical-align:middle;margin-right:6px;';
  readout.style.cssText = 'vertical-align:middle;';
  hud.style.cssText =
    'position:fixed;padding:4px 8px;background:rgba(0,0,0,0.8);color:#fff;' +
    'font:12px/1.4 system-ui,sans-serif;border-radius:4px;display:none;pointer-events:none;z-index:20;white-space:nowrap;';
  hud.appendChild(swatch);
  hud.appendChild(readout);
  document.body.appendChild(hud);

  hint.textContent =
    'Drag to select  ·  drag edges to resize  ·  ⌘/Ctrl+A all  ·  ↵ Copy  ·  S Save  ·  P Pin  ·  Esc Cancel';
  hint.style.cssText =
    'position:fixed;left:50%;top:24px;transform:translateX(-50%);padding:8px 14px;' +
    'background:rgba(0,0,0,0.7);color:#fff;font:13px/1 system-ui,sans-serif;border-radius:8px;' +
    'pointer-events:none;z-index:30;white-space:nowrap;';
  document.body.appendChild(hint);

  toolbar.style.cssText =
    'position:fixed;display:none;gap:6px;padding:6px;background:rgba(20,20,22,0.92);' +
    'border:1px solid rgba(255,255,255,0.12);border-radius:10px;pointer-events:auto;z-index:30;' +
    'box-shadow:0 6px 24px rgba(0,0,0,0.5);';
  toolbar.appendChild(makeButton('Pin P', () => void pinSelection()));
  toolbar.appendChild(makeButton('Copy ↵', () => void commit('clipboard')));
  toolbar.appendChild(makeButton('Save S', () => void commit('file')));
  toolbar.appendChild(makeButton('Cancel', () => void cancel()));
  // Guard the toolbar container too (clicks land on padding/gaps).
  toolbar.addEventListener('mousedown', (e) => e.stopPropagation());
  document.body.appendChild(toolbar);
}

function physical(v: number): number {
  return Math.round(v * scaleFactor);
}

function setBox(el: HTMLElement, x: number, y: number, w: number, h: number): void {
  el.style.left = `${x}px`;
  el.style.top = `${y}px`;
  el.style.width = `${Math.max(0, w)}px`;
  el.style.height = `${Math.max(0, h)}px`;
}

function rectFromDrag(): Rect {
  const x = Math.min(startX, curX);
  const y = Math.min(startY, curY);
  return { x, y, width: Math.abs(curX - startX), height: Math.abs(curY - startY) };
}

// The selection in effect right now: the live drag while drawing, otherwise the
// committed (and possibly resized/moved) rectangle.
function currentRect(): Rect {
  return dragging ? rectFromDrag() : sel;
}

type Grab = { left: boolean; right: boolean; top: boolean; bottom: boolean };

// Classify a pointer position against the current selection: which edge handles
// it grabs, whether it is inside (move), or outside (start a new selection).
function hitTest(px: number, py: number): Grab | 'inside' | 'outside' {
  if (!hasSelection) {
    return 'outside';
  }
  const { x, y, width, height } = sel;
  const right = x + width;
  const bottom = y + height;
  const withinX = px >= x - HANDLE_HIT && px <= right + HANDLE_HIT;
  const withinY = py >= y - HANDLE_HIT && py <= bottom + HANDLE_HIT;
  const grabbed: Grab = {
    left: Math.abs(px - x) <= HANDLE_HIT && withinY,
    right: Math.abs(px - right) <= HANDLE_HIT && withinY,
    top: Math.abs(py - y) <= HANDLE_HIT && withinX,
    bottom: Math.abs(py - bottom) <= HANDLE_HIT && withinX,
  };
  if (grabbed.left || grabbed.right || grabbed.top || grabbed.bottom) {
    return grabbed;
  }
  if (px > x && px < right && py > y && py < bottom) {
    return 'inside';
  }
  return 'outside';
}

function cursorFor(px: number, py: number): string {
  const hit = hitTest(px, py);
  if (hit === 'inside') {
    return 'move';
  }
  if (hit === 'outside') {
    return 'crosshair';
  }
  if ((hit.left && hit.top) || (hit.right && hit.bottom)) {
    return 'nwse-resize';
  }
  if ((hit.right && hit.top) || (hit.left && hit.bottom)) {
    return 'nesw-resize';
  }
  return hit.left || hit.right ? 'ew-resize' : 'ns-resize';
}

function applyResize(px: number, py: number): void {
  const w = window.innerWidth;
  const h = window.innerHeight;
  let left = sel.x;
  let top = sel.y;
  let right = sel.x + sel.width;
  let bottom = sel.y + sel.height;
  // Each grabbed edge follows the pointer but cannot cross within MIN_SIZE of
  // its opposite edge — this also prevents inversion, so no handle swap needed.
  if (grab.left) left = Math.min(px, right - MIN_SIZE);
  if (grab.right) right = Math.max(px, left + MIN_SIZE);
  if (grab.top) top = Math.min(py, bottom - MIN_SIZE);
  if (grab.bottom) bottom = Math.max(py, top + MIN_SIZE);
  left = Math.max(0, Math.min(left, w));
  right = Math.max(0, Math.min(right, w));
  top = Math.max(0, Math.min(top, h));
  bottom = Math.max(0, Math.min(bottom, h));
  sel = {
    x: left,
    y: top,
    width: Math.max(MIN_SIZE, right - left),
    height: Math.max(MIN_SIZE, bottom - top),
  };
}

function applyMove(px: number, py: number): void {
  const x = Math.max(0, Math.min(px - moveOffX, window.innerWidth - sel.width));
  const y = Math.max(0, Math.min(py - moveOffY, window.innerHeight - sel.height));
  sel = { ...sel, x, y };
}

function positionHandles(r: Rect): void {
  const cx = r.x + r.width / 2;
  const cy = r.y + r.height / 2;
  const right = r.x + r.width;
  const bottom = r.y + r.height;
  const points: Record<string, [number, number]> = {
    nw: [r.x, r.y],
    n: [cx, r.y],
    ne: [right, r.y],
    e: [right, cy],
    se: [right, bottom],
    s: [cx, bottom],
    sw: [r.x, bottom],
    w: [r.x, cy],
  };
  HANDLES.forEach((handle, i) => {
    const [hx, hy] = points[handle.id];
    handleEls[i].style.left = `${hx}px`;
    handleEls[i].style.top = `${hy}px`;
  });
}

function showHandles(visible: boolean): void {
  for (const el of handleEls) {
    el.style.display = visible ? 'block' : 'none';
  }
}

function updateColorAt(lx: number, ly: number): void {
  if (!samplerCtx) {
    return;
  }
  const px = Math.max(0, Math.min(physical(lx), sampler.width - 1));
  const py = Math.max(0, Math.min(physical(ly), sampler.height - 1));
  const data = samplerCtx.getImageData(px, py, 1, 1).data;
  const hex = `#${[data[0], data[1], data[2]]
    .map((c) => c.toString(16).padStart(2, '0'))
    .join('')
    .toUpperCase()}`;
  cursorHex = hex;
  swatch.style.background = hex;
  readout.textContent = `${hex}  rgb(${data[0]}, ${data[1]}, ${data[2]})`;
}

function drawMagnifier(lx: number, ly: number): void {
  const ctx = magCanvas.getContext('2d');
  if (!ctx) {
    return;
  }
  const srcSize = 15; // physical px sampled around the cursor
  const px = physical(lx);
  const py = physical(ly);
  // Clamp the sampled window so it stays inside the frame (no edge artifacts).
  const sx = Math.max(0, Math.min(px - srcSize / 2, sampler.width - srcSize));
  const sy = Math.max(0, Math.min(py - srcSize / 2, sampler.height - srcSize));
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, magCanvas.width, magCanvas.height);
  ctx.drawImage(sampler, sx, sy, srcSize, srcSize, 0, 0, magCanvas.width, magCanvas.height);
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

function renderDim(r: Rect): void {
  const w = window.innerWidth;
  const h = window.innerHeight;
  setBox(dimTop, 0, 0, w, r.y);
  setBox(dimBottom, 0, r.y + r.height, w, h - (r.y + r.height));
  setBox(dimLeft, 0, r.y, r.x, r.height);
  setBox(dimRight, r.x + r.width, r.y, w - (r.x + r.width), r.height);
}

function showDim(visible: boolean): void {
  for (const panel of dimPanels) {
    panel.style.display = visible ? 'block' : 'none';
  }
}

function renderSelection(r: Rect): void {
  setBox(selectionEl, r.x, r.y, r.width, r.height);
  selectionEl.style.display = 'block';
  sizeBadge.textContent = `${physical(r.width)} × ${physical(r.height)}`;
  const by = r.y > 24 ? r.y - 22 : r.y + 6;
  sizeBadge.style.left = `${r.x}px`;
  sizeBadge.style.top = `${by}px`;
  sizeBadge.style.display = 'block';
}

function positionToolbar(r: Rect): void {
  const tw = 300;
  const left = Math.max(8, Math.min(r.x + r.width - tw, window.innerWidth - tw - 8));
  let top = r.y + r.height + 10;
  if (top > window.innerHeight - 52) {
    top = Math.max(8, r.y - 52);
  }
  toolbar.style.left = `${left}px`;
  toolbar.style.top = `${top}px`;
}

function update(): void {
  rafPending = false;
  const active = dragging || hasSelection;
  const r = currentRect();

  if (active) {
    renderSelection(r);
    renderDim(r);
    showDim(true);
    hint.style.display = 'none';
  } else {
    selectionEl.style.display = 'none';
    sizeBadge.style.display = 'none';
    showDim(false);
    hint.style.display = 'block';
  }

  // Resize handles appear once a selection exists and track it through edits.
  if (hasSelection && !dragging) {
    positionHandles(r);
    showHandles(true);
  } else {
    showHandles(false);
  }

  // Magnifier + color readout track the cursor while drawing or resizing (both
  // benefit from pixel precision); a settled selection gives way to the toolbar.
  if (!hasSelection || dragging || adjusting === 'resize') {
    updateColorAt(pointerX, pointerY);
    drawMagnifier(pointerX, pointerY);
    positionFloaters(pointerX, pointerY);
    magCanvas.style.display = 'block';
    hud.style.display = 'block';
  } else {
    magCanvas.style.display = 'none';
    hud.style.display = 'none';
  }

  if (hasSelection && !dragging && !adjusting) {
    positionToolbar(r);
    toolbar.style.display = 'flex';
  } else {
    toolbar.style.display = 'none';
  }
}

function scheduleRender(): void {
  if (rafPending) {
    return;
  }
  rafPending = true;
  requestAnimationFrame(update);
}

async function commit(output: 'clipboard' | 'file'): Promise<void> {
  const r = currentRect();
  if (r.width < 1 || r.height < 1) {
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

async function pinSelection(): Promise<void> {
  const r = currentRect();
  if (r.width < 1 || r.height < 1) {
    return;
  }
  try {
    await invoke('create_pin', { displayId, rect: r });
  } catch (e) {
    console.error('create_pin failed', e);
  }
}

async function editSelection(): Promise<void> {
  const r = currentRect();
  if (r.width < 1 || r.height < 1) {
    return;
  }
  try {
    await invoke('edit_selection', { displayId, rect: r });
  } catch (e) {
    console.error('edit_selection failed', e);
  }
}

function selectAll(): void {
  sel = { x: 0, y: 0, width: window.innerWidth, height: window.innerHeight };
  dragging = false;
  adjusting = null;
  hasSelection = true;
  scheduleRender();
}

function onMouseDown(e: MouseEvent): void {
  if (e.button !== 0 || toolbar.contains(e.target as Node)) {
    return;
  }
  pointerX = e.clientX;
  pointerY = e.clientY;
  const hit = hitTest(e.clientX, e.clientY);

  if (hit !== 'inside' && hit !== 'outside') {
    // Grabbed an edge/corner handle → resize the existing selection.
    e.preventDefault();
    adjusting = 'resize';
    grab = hit;
    scheduleRender();
    return;
  }
  if (hit === 'inside') {
    // Pressed inside → move the whole selection.
    e.preventDefault();
    adjusting = 'move';
    moveOffX = e.clientX - sel.x;
    moveOffY = e.clientY - sel.y;
    scheduleRender();
    return;
  }
  // Outside any selection → start drawing a fresh one.
  dragging = true;
  hasSelection = false;
  adjusting = null;
  startX = e.clientX;
  startY = e.clientY;
  curX = e.clientX;
  curY = e.clientY;
  scheduleRender();
}

function onMouseMove(e: MouseEvent): void {
  pointerX = e.clientX;
  pointerY = e.clientY;
  if (dragging) {
    curX = e.clientX;
    curY = e.clientY;
  } else if (adjusting === 'resize') {
    applyResize(e.clientX, e.clientY);
  } else if (adjusting === 'move') {
    applyMove(e.clientX, e.clientY);
  } else {
    // Hover feedback: crosshair / move / resize cursor over the selection.
    document.body.style.cursor = cursorFor(e.clientX, e.clientY);
  }
  scheduleRender();
}

function onMouseUp(e: MouseEvent): void {
  if (e.button !== 0) {
    return;
  }
  if (dragging) {
    dragging = false;
    const r = rectFromDrag();
    // A tiny drag is really a click; don't trap the user in a 1px selection.
    hasSelection = r.width > 2 && r.height > 2;
    if (hasSelection) {
      sel = r;
    }
  } else if (adjusting) {
    adjusting = null;
  }
  scheduleRender();
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
    case 'p':
    case 'P':
      void pinSelection();
      break;
    case 'e':
    case 'E':
      void editSelection();
      break;
    case 'c':
    case 'C':
      void invoke('copy_color', { hex: cursorHex }).catch((err) => console.error(err));
      break;
    case 'a':
    case 'A':
      if (e.metaKey || e.ctrlKey) {
        e.preventDefault();
        selectAll();
      }
      break;
    default:
      break;
  }
}

async function init(): Promise<void> {
  buildUi();
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
  img.style.cssText =
    'position:fixed;inset:0;width:100vw;height:100vh;display:block;-webkit-user-drag:none;';
  img.src = frame.dataUrl;
  await img.decode().catch(() => undefined);
  document.body.insertBefore(img, document.body.firstChild);

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

  scheduleRender();
}

void init();
