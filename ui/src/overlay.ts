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
// The whole frozen frame's pixels, read back once. Color sampling then becomes a
// plain array index per frame instead of a getImageData readback (which stalls).
let fullData: ImageData | null = null;

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

// --- Inline annotation (Snipaste-style) ---------------------------------------
// After a selection exists, the user can pick a tool and draw directly over the
// frozen frame. Annotations live in logical (CSS) coords; on output we composite
// them with the cropped frame into a PNG and hand that to the shell.
type Tool = 'select' | 'rect' | 'ellipse' | 'arrow' | 'line' | 'pen' | 'text' | 'blur' | 'mosaic';

interface Point {
  x: number;
  y: number;
}

interface Annotation {
  tool: Tool;
  points: Point[];
  color: string;
  width: number;
  text?: string;
}

let activeTool: Tool = 'select';
const annotations: Annotation[] = [];
let draft: Annotation | null = null;
let drawingAnno = false;
let currentColor = '#ff3b30';
let currentWidth = 3;
let textInput: HTMLInputElement | null = null;

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

// Backdrop dimming uses a "static dim + clipped reveal" model: one full-screen
// dim layer that never changes (composited once), and a second copy of the frame
// clipped to the selection so the selected region shows through bright. Dragging
// only moves a small GPU clip — no full-screen blend/repaint per frame.
const dimEl = document.createElement('div');
const revealWrap = document.createElement('div');
let revealImg: HTMLImageElement | null = null;
// Module-level handle to the frozen frame image, used as the magnifier source
// (a GPU-decoded <img> instead of the CPU-backed willReadFrequently sampler).
let frameImg: HTMLImageElement | null = null;

const selectionEl = document.createElement('div');
const sizeBadge = document.createElement('div');
const magCanvas = document.createElement('canvas');
const hud = document.createElement('div');
const swatch = document.createElement('span');
const readout = document.createElement('span');
const hint = document.createElement('div');
const toolbar = document.createElement('div');
// Canvas layer that holds the live annotation drawing, above the frame but
// below the selection chrome. Drawn in logical coords (ctx scaled by dpr).
const annoCanvas = document.createElement('canvas');
const annoCtx = annoCanvas.getContext('2d');
// Committed annotations are rasterised once into this offscreen cache and only
// rebuilt when the set changes; each frame just blits the cache + the live draft.
// This keeps expensive Blur/Mosaic from re-rendering on every mouse move.
const committedCanvas = document.createElement('canvas');
const committedCtx = committedCanvas.getContext('2d');
let committedDirty = true;
const dpr = window.devicePixelRatio || 1;
// Tool buttons, kept so the active one can be highlighted on change.
const toolButtons = new Map<Tool, HTMLButtonElement>();

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

function setTool(tool: Tool): void {
  activeTool = tool;
  for (const [t, btn] of toolButtons) {
    btn.style.background = t === tool ? ACCENT : 'rgba(255,255,255,0.12)';
  }
  document.body.style.cursor = tool === 'select' ? 'crosshair' : 'crosshair';
}

// A compact tool button (icon glyph). Selecting it switches the active tool.
function makeToolButton(tool: Tool, glyph: string, title: string): HTMLButtonElement {
  const button = document.createElement('button');
  button.textContent = glyph;
  button.title = title;
  button.style.cssText =
    'appearance:none;border:0;border-radius:6px;width:30px;height:30px;font:600 15px/1 system-ui,sans-serif;' +
    'color:#fff;background:rgba(255,255,255,0.12);cursor:pointer;';
  button.addEventListener('mousedown', (e) => e.stopPropagation());
  button.addEventListener('click', (e) => {
    e.stopPropagation();
    setTool(tool);
  });
  toolButtons.set(tool, button);
  return button;
}

function makeSeparator(): HTMLSpanElement {
  const sep = document.createElement('span');
  sep.style.cssText =
    'width:1px;align-self:stretch;background:rgba(255,255,255,0.18);margin:2px 2px;';
  return sep;
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

  // Static full-screen dim, composited once and never resized.
  dimEl.style.cssText =
    'position:fixed;inset:0;background:rgba(0,0,0,0.5);display:none;pointer-events:none;z-index:5;';
  document.body.appendChild(dimEl);

  // Clip window that reveals the bright frame inside the selection. `transform`
  // creates a containing block so the fixed-position inner image is clipped here.
  revealWrap.style.cssText =
    'position:fixed;left:0;top:0;overflow:hidden;display:none;pointer-events:none;z-index:6;' +
    'transform:translate(0,0);will-change:transform,width,height;';
  document.body.appendChild(revealWrap);

  selectionEl.style.cssText =
    `position:fixed;border:1px solid ${ACCENT};` + 'display:none;pointer-events:none;z-index:10;';
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
    'Drag to select  ·  pick a tool to annotate  ·  ⌘/Ctrl+Z undo  ·  ↵ Copy  ·  S Save  ·  P Pin  ·  Esc Cancel';
  hint.style.cssText =
    'position:fixed;left:50%;top:24px;transform:translateX(-50%);padding:8px 14px;' +
    'background:rgba(0,0,0,0.7);color:#fff;font:13px/1 system-ui,sans-serif;border-radius:8px;' +
    'pointer-events:none;z-index:30;white-space:nowrap;';
  document.body.appendChild(hint);

  // Annotation drawing layer: above the frame, below the selection chrome.
  // Hidden during plain area-selection so its full-screen retina layer doesn't
  // add compositing cost to every mouse move; shown only while annotating.
  annoCanvas.style.cssText =
    'position:fixed;inset:0;width:100vw;height:100vh;pointer-events:none;z-index:8;display:none;';
  document.body.appendChild(annoCanvas);

  toolbar.style.cssText =
    'position:fixed;display:none;align-items:center;gap:4px;padding:6px;background:rgba(20,20,22,0.92);' +
    'border:1px solid rgba(255,255,255,0.12);border-radius:10px;pointer-events:auto;z-index:30;' +
    'box-shadow:0 6px 24px rgba(0,0,0,0.5);';

  // Annotation tools.
  toolbar.appendChild(makeToolButton('select', '⤢', 'Select / move (Esc tool)'));
  toolbar.appendChild(makeToolButton('rect', '▭', 'Rectangle'));
  toolbar.appendChild(makeToolButton('ellipse', '◯', 'Ellipse'));
  toolbar.appendChild(makeToolButton('arrow', '↗', 'Arrow'));
  toolbar.appendChild(makeToolButton('line', '╱', 'Line'));
  toolbar.appendChild(makeToolButton('pen', '✎', 'Pen'));
  toolbar.appendChild(makeToolButton('text', 'A', 'Text'));
  toolbar.appendChild(makeToolButton('blur', '▒', 'Blur'));
  toolbar.appendChild(makeToolButton('mosaic', '▦', 'Mosaic'));
  toolbar.appendChild(makeSeparator());

  // Color + thickness.
  const color = document.createElement('input');
  color.type = 'color';
  color.value = currentColor;
  color.title = 'Color';
  color.style.cssText =
    'width:28px;height:28px;padding:0;border:0;border-radius:6px;background:none;cursor:pointer;';
  color.addEventListener('mousedown', (e) => e.stopPropagation());
  color.addEventListener('input', () => {
    currentColor = color.value;
  });
  toolbar.appendChild(color);

  const width = document.createElement('input');
  width.type = 'range';
  width.min = '1';
  width.max = '12';
  width.value = String(currentWidth);
  width.title = 'Thickness';
  width.style.cssText = 'width:70px;cursor:pointer;';
  width.addEventListener('mousedown', (e) => e.stopPropagation());
  width.addEventListener('input', () => {
    currentWidth = Number(width.value);
  });
  toolbar.appendChild(width);
  toolbar.appendChild(makeSeparator());

  // Output actions.
  toolbar.appendChild(makeButton('Pin', () => void pinSelection()));
  toolbar.appendChild(makeButton('Copy', () => void commit('clipboard')));
  toolbar.appendChild(makeButton('Save', () => void commit('file')));
  toolbar.appendChild(makeButton('✕', () => void cancel()));
  // Guard the toolbar container too (clicks land on padding/gaps).
  toolbar.addEventListener('mousedown', (e) => e.stopPropagation());
  document.body.appendChild(toolbar);

  setTool('select');
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

// Toggle visibility without redundant writes: setting `display` to its current
// value still dirties style, so skip the write when it would be a no-op. Reading
// inline `style.display` is cheap (no layout flush).
function setShown(el: HTMLElement, show: boolean, shownValue = 'block'): void {
  const want = show ? shownValue : 'none';
  if (el.style.display !== want) {
    el.style.display = want;
  }
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
    setShown(el, visible);
  }
}

function updateColorAt(lx: number, ly: number): void {
  if (!fullData) {
    return;
  }
  const px = Math.max(0, Math.min(physical(lx), sampler.width - 1));
  const py = Math.max(0, Math.min(physical(ly), sampler.height - 1));
  const i = (py * sampler.width + px) * 4;
  const data = fullData.data;
  const r = data[i];
  const g = data[i + 1];
  const b = data[i + 2];
  const hex = `#${[r, g, b]
    .map((c) => c.toString(16).padStart(2, '0'))
    .join('')
    .toUpperCase()}`;
  cursorHex = hex;
  swatch.style.background = hex;
  readout.textContent = `${hex}  rgb(${r}, ${g}, ${b})`;
}

function drawMagnifier(lx: number, ly: number): void {
  const ctx = magCanvas.getContext('2d');
  if (!ctx || !frameImg) {
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
  // Source the GPU-decoded <img> (not the CPU-backed sampler) to avoid a slow
  // software read each frame.
  ctx.drawImage(frameImg, sx, sy, srcSize, srcSize, 0, 0, magCanvas.width, magCanvas.height);
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

// Move/size the reveal clip to the selection. The wrapper is positioned with a
// GPU transform; the inner image counter-translates so it stays pixel-aligned
// with the (viewport-fixed) background frame, then gets clipped by the wrapper.
function renderDim(r: Rect): void {
  revealWrap.style.transform = `translate(${r.x}px, ${r.y}px)`;
  revealWrap.style.width = `${r.width}px`;
  revealWrap.style.height = `${r.height}px`;
  if (revealImg) {
    revealImg.style.transform = `translate(${-r.x}px, ${-r.y}px)`;
  }
}

function showDim(visible: boolean): void {
  setShown(dimEl, visible);
  setShown(revealWrap, visible);
}

function renderSelection(r: Rect): void {
  setBox(selectionEl, r.x, r.y, r.width, r.height);
  setShown(selectionEl, true);
  sizeBadge.textContent = `${physical(r.width)} × ${physical(r.height)}`;
  const by = r.y > 24 ? r.y - 22 : r.y + 6;
  sizeBadge.style.left = `${r.x}px`;
  sizeBadge.style.top = `${by}px`;
  setShown(sizeBadge, true);
}

function positionToolbar(r: Rect): void {
  const tw = 560;
  const left = Math.max(8, Math.min(r.x + r.width - tw, window.innerWidth - tw - 8));
  let top = r.y + r.height + 10;
  if (top > window.innerHeight - 52) {
    top = Math.max(8, r.y - 52);
  }
  toolbar.style.left = `${left}px`;
  toolbar.style.top = `${top}px`;
}

// Normalise a two-point drag into a logical rect.
function rectOf(pts: Point[]): { x: number; y: number; w: number; h: number } {
  const a = pts[0];
  const b = pts[pts.length - 1];
  return {
    x: Math.min(a.x, b.x),
    y: Math.min(a.y, b.y),
    w: Math.abs(b.x - a.x),
    h: Math.abs(b.y - a.y),
  };
}

// Clamp a pointer to the current selection so annotations never spill outside it.
function clampToSel(px: number, py: number): Point {
  return {
    x: Math.max(sel.x, Math.min(px, sel.x + sel.width)),
    y: Math.max(sel.y, Math.min(py, sel.y + sel.height)),
  };
}

function drawArrow(ctx: CanvasRenderingContext2D, p0: Point, p1: Point, w: number): void {
  ctx.beginPath();
  ctx.moveTo(p0.x, p0.y);
  ctx.lineTo(p1.x, p1.y);
  ctx.stroke();
  const ang = Math.atan2(p1.y - p0.y, p1.x - p0.x);
  const head = Math.max(10, w * 4);
  ctx.beginPath();
  ctx.moveTo(p1.x, p1.y);
  ctx.lineTo(p1.x - head * Math.cos(ang - Math.PI / 6), p1.y - head * Math.sin(ang - Math.PI / 6));
  ctx.moveTo(p1.x, p1.y);
  ctx.lineTo(p1.x - head * Math.cos(ang + Math.PI / 6), p1.y - head * Math.sin(ang + Math.PI / 6));
  ctx.stroke();
}

// Blur/mosaic read the *original* frozen pixels from the physical-res `sampler`,
// so they work identically on the on-screen canvas and the export composite.
function drawBlur(ctx: CanvasRenderingContext2D, a: Annotation): void {
  const r = rectOf(a.points);
  if (r.w < 1 || r.h < 1) {
    return;
  }
  ctx.save();
  ctx.beginPath();
  ctx.rect(r.x, r.y, r.w, r.h);
  ctx.clip();
  ctx.filter = `blur(${Math.max(3, a.width * 2)}px)`;
  ctx.drawImage(
    sampler,
    r.x * scaleFactor,
    r.y * scaleFactor,
    r.w * scaleFactor,
    r.h * scaleFactor,
    r.x,
    r.y,
    r.w,
    r.h,
  );
  ctx.restore();
}

function drawMosaic(ctx: CanvasRenderingContext2D, a: Annotation): void {
  const r = rectOf(a.points);
  if (r.w < 1 || r.h < 1) {
    return;
  }
  const block = Math.max(4, a.width * 3);
  const downW = Math.max(1, Math.round(r.w / block));
  const downH = Math.max(1, Math.round(r.h / block));
  const small = document.createElement('canvas');
  small.width = downW;
  small.height = downH;
  const sctx = small.getContext('2d');
  if (!sctx) {
    return;
  }
  sctx.imageSmoothingEnabled = false;
  sctx.drawImage(
    sampler,
    r.x * scaleFactor,
    r.y * scaleFactor,
    r.w * scaleFactor,
    r.h * scaleFactor,
    0,
    0,
    downW,
    downH,
  );
  const prev = ctx.imageSmoothingEnabled;
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(small, 0, 0, downW, downH, r.x, r.y, r.w, r.h);
  ctx.imageSmoothingEnabled = prev;
}

// Paint a single annotation in logical coords; the caller sets the ctx transform
// (dpr for the screen, scaleFactor+offset for the export composite).
function paintAnnotation(ctx: CanvasRenderingContext2D, a: Annotation): void {
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';
  ctx.strokeStyle = a.color;
  ctx.fillStyle = a.color;
  ctx.lineWidth = a.width;
  const pts = a.points;
  const twoPoint = pts.length >= 2;
  switch (a.tool) {
    case 'rect': {
      if (!twoPoint) break;
      const r = rectOf(pts);
      ctx.strokeRect(r.x, r.y, r.w, r.h);
      break;
    }
    case 'ellipse': {
      if (!twoPoint) break;
      const r = rectOf(pts);
      ctx.beginPath();
      ctx.ellipse(r.x + r.w / 2, r.y + r.h / 2, r.w / 2, r.h / 2, 0, 0, Math.PI * 2);
      ctx.stroke();
      break;
    }
    case 'line': {
      if (!twoPoint) break;
      ctx.beginPath();
      ctx.moveTo(pts[0].x, pts[0].y);
      ctx.lineTo(pts[pts.length - 1].x, pts[pts.length - 1].y);
      ctx.stroke();
      break;
    }
    case 'arrow': {
      if (!twoPoint) break;
      drawArrow(ctx, pts[0], pts[pts.length - 1], a.width);
      break;
    }
    case 'pen': {
      if (pts.length < 2) break;
      ctx.beginPath();
      ctx.moveTo(pts[0].x, pts[0].y);
      for (let i = 1; i < pts.length; i++) {
        ctx.lineTo(pts[i].x, pts[i].y);
      }
      ctx.stroke();
      break;
    }
    case 'text': {
      ctx.font = `${Math.max(14, a.width * 6)}px system-ui,-apple-system,sans-serif`;
      ctx.textBaseline = 'top';
      ctx.fillText(a.text ?? '', pts[0].x, pts[0].y);
      break;
    }
    case 'blur':
      drawBlur(ctx, a);
      break;
    case 'mosaic':
      drawMosaic(ctx, a);
      break;
    default:
      break;
  }
}

// Rebuild the committed-annotation cache (only when the set actually changed).
function renderCommitted(): void {
  if (!committedCtx) {
    return;
  }
  committedCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
  committedCtx.clearRect(0, 0, window.innerWidth, window.innerHeight);
  for (const a of annotations) {
    paintAnnotation(committedCtx, a);
  }
  committedDirty = false;
}

// Compose the visible layer: cached committed annotations + the live draft.
function renderAnnotations(): void {
  if (!annoCtx) {
    return;
  }
  if (committedDirty) {
    renderCommitted();
  }
  annoCtx.setTransform(1, 0, 0, 1, 0, 0);
  annoCtx.clearRect(0, 0, annoCanvas.width, annoCanvas.height);
  annoCtx.drawImage(committedCanvas, 0, 0);
  if (draft) {
    annoCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
    if (draft.tool === 'blur' || draft.tool === 'mosaic') {
      // Cheap placeholder while dragging; the real pixel effect renders on commit.
      const r = rectOf(draft.points);
      annoCtx.fillStyle = 'rgba(120,120,120,0.5)';
      annoCtx.fillRect(r.x, r.y, r.w, r.h);
      annoCtx.strokeStyle = 'rgba(255,255,255,0.7)';
      annoCtx.lineWidth = 1;
      annoCtx.strokeRect(r.x, r.y, r.w, r.h);
    } else {
      paintAnnotation(annoCtx, draft);
    }
  }
}

// Composite the cropped frozen region + annotations into a PNG data URL at the
// selection's physical resolution, for the annotated output commands.
function renderToDataUrl(): string {
  const physW = Math.max(1, Math.round(sel.width * scaleFactor));
  const physH = Math.max(1, Math.round(sel.height * scaleFactor));
  const out = document.createElement('canvas');
  out.width = physW;
  out.height = physH;
  const octx = out.getContext('2d');
  if (!octx) {
    return '';
  }
  octx.drawImage(
    sampler,
    Math.round(sel.x * scaleFactor),
    Math.round(sel.y * scaleFactor),
    physW,
    physH,
    0,
    0,
    physW,
    physH,
  );
  octx.setTransform(scaleFactor, 0, 0, scaleFactor, -sel.x * scaleFactor, -sel.y * scaleFactor);
  for (const a of annotations) {
    paintAnnotation(octx, a);
  }
  octx.setTransform(1, 0, 0, 1, 0, 0);
  return out.toDataURL('image/png');
}

// Floating text-entry box for the Text tool; commits an annotation on Enter/blur.
function startTextInput(p: Point): void {
  if (textInput) {
    textInput.blur();
  }
  const input = document.createElement('input');
  input.type = 'text';
  const size = Math.max(14, currentWidth * 6);
  input.style.cssText =
    `position:fixed;left:${p.x}px;top:${p.y}px;z-index:40;background:rgba(0,0,0,0.5);` +
    `border:1px solid ${ACCENT};border-radius:4px;padding:2px 4px;outline:none;` +
    `color:${currentColor};font:${size}px system-ui,sans-serif;min-width:80px;`;
  input.addEventListener('mousedown', (e) => e.stopPropagation());
  const color = currentColor;
  const widthAtStart = currentWidth;
  const commitText = (): void => {
    const value = input.value.trim();
    if (value) {
      annotations.push({ tool: 'text', points: [p], color, width: widthAtStart, text: value });
      committedDirty = true;
    }
    input.remove();
    if (textInput === input) {
      textInput = null;
    }
    scheduleRender();
  };
  input.addEventListener('keydown', (e) => {
    e.stopPropagation();
    if (e.key === 'Enter') {
      commitText();
    } else if (e.key === 'Escape') {
      input.remove();
      if (textInput === input) {
        textInput = null;
      }
    }
  });
  input.addEventListener('blur', commitText);
  textInput = input;
  document.body.appendChild(input);
  input.focus();
}

function update(): void {
  rafPending = false;
  const active = dragging || hasSelection;
  const r = currentRect();

  if (active) {
    renderSelection(r);
    renderDim(r);
    showDim(true);
    setShown(hint, false);
  } else {
    setShown(selectionEl, false);
    setShown(sizeBadge, false);
    showDim(false);
    setShown(hint, true);
  }

  // Resize handles appear once a selection exists and track it through edits.
  if (hasSelection && !dragging) {
    positionHandles(r);
    showHandles(true);
  } else {
    showHandles(false);
  }

  // Color readout tracks the cursor while drawing/resizing. The magnifier canvas
  // is the only per-frame canvas work, so during a fast initial drag we skip it
  // (it's most useful for fine edge placement while resizing, not while sweeping).
  if (!hasSelection || dragging || adjusting === 'resize') {
    updateColorAt(pointerX, pointerY);
    positionFloaters(pointerX, pointerY);
    setShown(hud, true);
    if (!dragging) {
      drawMagnifier(pointerX, pointerY);
      setShown(magCanvas, true);
    } else {
      setShown(magCanvas, false);
    }
  } else {
    setShown(magCanvas, false);
    setShown(hud, false);
  }

  if (hasSelection && !dragging && !adjusting) {
    positionToolbar(r);
    setShown(toolbar, true, 'flex');
  } else {
    setShown(toolbar, false);
  }

  // Keep the annotation layer out of the compositor entirely while the user is
  // just selecting an area (no annotations, Select tool) — that path must stay
  // as light as the original overlay.
  const showAnno = annotations.length > 0 || draft !== null || activeTool !== 'select';
  setShown(annoCanvas, showAnno);

  // Only repaint the annotation layer when there's a live draft or the committed
  // set changed — hovering with a settled selection does no annotation work.
  if (showAnno && (draft || committedDirty)) {
    renderAnnotations();
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
    if (annotations.length > 0) {
      await invoke<CommitResponse>('commit_annotated', { output, png: renderToDataUrl() });
    } else {
      await invoke<CommitResponse>('commit_selection', { displayId, rect: r, output });
    }
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
    if (annotations.length > 0) {
      await invoke('pin_annotated', { displayId, rect: r, png: renderToDataUrl() });
    } else {
      await invoke('create_pin', { displayId, rect: r });
    }
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

  // Tool active + a selection exists → draw an annotation instead of reselecting.
  if (activeTool !== 'select' && hasSelection) {
    e.preventDefault();
    const p = clampToSel(e.clientX, e.clientY);
    if (activeTool === 'text') {
      startTextInput(p);
      return;
    }
    drawingAnno = true;
    draft = {
      tool: activeTool,
      points: [p, { x: p.x, y: p.y }],
      color: currentColor,
      width: currentWidth,
    };
    scheduleRender();
    return;
  }

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
  if (drawingAnno && draft) {
    const p = clampToSel(e.clientX, e.clientY);
    if (draft.tool === 'pen') {
      draft.points.push(p);
    } else {
      draft.points[1] = p;
    }
    scheduleRender();
    return;
  }
  if (dragging) {
    curX = e.clientX;
    curY = e.clientY;
  } else if (adjusting === 'resize') {
    applyResize(e.clientX, e.clientY);
  } else if (adjusting === 'move') {
    applyMove(e.clientX, e.clientY);
  } else if (activeTool !== 'select') {
    document.body.style.cursor = 'crosshair';
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
  if (drawingAnno) {
    drawingAnno = false;
    if (draft) {
      const r = rectOf(draft.points);
      const meaningful = draft.tool === 'pen' ? draft.points.length > 2 : r.w > 2 || r.h > 2;
      if (meaningful) {
        annotations.push(draft);
        committedDirty = true;
      }
    }
    draft = null;
    scheduleRender();
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
  // While the Text tool's input box is focused, keys belong to it.
  if (textInput) {
    return;
  }
  switch (e.key) {
    case 'Escape':
      void cancel();
      break;
    case 'z':
    case 'Z':
      if (e.metaKey || e.ctrlKey) {
        e.preventDefault();
        if (annotations.length > 0) {
          annotations.pop();
          committedDirty = true;
          scheduleRender();
        }
      }
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
  // Promote the frame to its own GPU layer so the chrome above it composites
  // without ever forcing the full-screen image to repaint.
  img.style.cssText =
    'position:fixed;inset:0;width:100vw;height:100vh;display:block;-webkit-user-drag:none;' +
    'transform:translateZ(0);';
  img.src = frame.dataUrl;
  await img.decode().catch(() => undefined);
  document.body.insertBefore(img, document.body.firstChild);
  frameImg = img;

  // Bright copy of the frame revealed inside the selection clip. position:absolute
  // + transform keeps it on its own layer and pixel-aligned with the background.
  const reveal = new Image();
  reveal.src = frame.dataUrl;
  reveal.style.cssText =
    'position:absolute;left:0;top:0;width:100vw;height:100vh;display:block;-webkit-user-drag:none;' +
    'transform:translateZ(0);';
  await reveal.decode().catch(() => undefined);
  revealWrap.appendChild(reveal);
  revealImg = reveal;

  sampler.width = frame.width;
  sampler.height = frame.height;
  samplerCtx?.drawImage(img, 0, 0, frame.width, frame.height);
  // One readback up front; per-frame color sampling then indexes this buffer.
  fullData = samplerCtx?.getImageData(0, 0, sampler.width, sampler.height) ?? null;

  // Annotation canvas backs the full viewport at device resolution for crisp
  // strokes; the context is drawn in logical coords (scaled by dpr per render).
  annoCanvas.width = Math.round(window.innerWidth * dpr);
  annoCanvas.height = Math.round(window.innerHeight * dpr);
  committedCanvas.width = annoCanvas.width;
  committedCanvas.height = annoCanvas.height;

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
