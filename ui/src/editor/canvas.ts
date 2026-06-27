// The editor canvas: renders the captured base image plus a live preview of the
// annotation stack, and turns pointer gestures into draft annotations. It holds
// no authoritative state — it draws the `items` the shell returns and reports
// intents (create / move / select) back to the controller (Constitution IV).

import type { Annotation, AnnotationKind, Geometry, Point, Rect, Style, Tool } from './types';
import { normRect, rgbaCss } from './types';

export interface CanvasCallbacks {
  onCreate: (kind: AnnotationKind, geometry: Geometry, style: Style) => void;
  onMove: (id: number, geometry: Geometry) => void;
  onSelect: (id: number | null) => void;
}

interface Draft {
  kind: AnnotationKind;
  start: Point;
  cur: Point;
  points: Point[];
}

const DRAW_TOOLS: ReadonlyMap<Tool, AnnotationKind> = new Map([
  ['rect', 'rect'],
  ['ellipse', 'ellipse'],
  ['arrow', 'arrow'],
  ['line', 'line'],
  ['pencil', 'pencil'],
  ['highlighter', 'highlighter'],
]);

export class EditorCanvas {
  private readonly canvas: HTMLCanvasElement;
  private readonly ctx: CanvasRenderingContext2D;
  private readonly base: HTMLImageElement;
  private readonly cb: CanvasCallbacks;

  private items: Annotation[] = [];
  private tool: Tool = 'rect';
  private style: Style;
  private shiftDown = false;

  private draft: Draft | null = null;
  private selectedId: number | null = null;
  private moving: { id: number; from: Point; geom: Geometry } | null = null;

  constructor(parent: HTMLElement, base: HTMLImageElement, style: Style, cb: CanvasCallbacks) {
    this.base = base;
    this.style = style;
    this.cb = cb;

    this.canvas = document.createElement('canvas');
    this.canvas.width = base.naturalWidth;
    this.canvas.height = base.naturalHeight;
    this.canvas.className = 'editor-canvas';
    parent.appendChild(this.canvas);
    const ctx = this.canvas.getContext('2d');
    if (!ctx) {
      throw new Error('2d canvas context unavailable');
    }
    this.ctx = ctx;

    this.canvas.addEventListener('pointerdown', (e) => this.onDown(e));
    this.canvas.addEventListener('pointermove', (e) => this.onMove(e));
    window.addEventListener('pointerup', (e) => this.onUp(e));
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Shift') this.shiftDown = true;
    });
    window.addEventListener('keyup', (e) => {
      if (e.key === 'Shift') this.shiftDown = false;
    });

    this.render();
  }

  setItems(items: Annotation[]): void {
    this.items = items;
    if (this.selectedId !== null && !items.some((a) => a.id === this.selectedId)) {
      this.selectedId = null;
    }
    this.render();
  }

  setTool(tool: Tool): void {
    this.tool = tool;
    if (tool !== 'select') {
      this.selectedId = null;
    }
    this.render();
  }

  setStyle(style: Style): void {
    this.style = style;
  }

  selected(): number | null {
    return this.selectedId;
  }

  private toImage(e: PointerEvent): Point {
    const r = this.canvas.getBoundingClientRect();
    const sx = this.canvas.width / r.width;
    const sy = this.canvas.height / r.height;
    return { x: (e.clientX - r.left) * sx, y: (e.clientY - r.top) * sy };
  }

  private onDown(e: PointerEvent): void {
    if (e.button !== 0) {
      return;
    }
    const p = this.toImage(e);

    if (this.tool === 'select') {
      const id = this.hitTest(p);
      this.selectedId = id;
      this.cb.onSelect(id);
      if (id !== null) {
        const a = this.items.find((it) => it.id === id);
        if (a) {
          this.moving = { id, from: p, geom: a.geometry };
        }
      }
      this.render();
      return;
    }

    if (this.tool === 'text') {
      const content = window.prompt('Text:')?.trim();
      if (content) {
        const style: Style = { ...this.style, text: { ...this.style.text, content } };
        this.cb.onCreate('text', { anchor: { x: Math.round(p.x), y: Math.round(p.y) } }, style);
      }
      return;
    }

    const kind = DRAW_TOOLS.get(this.tool);
    if (kind) {
      this.canvas.setPointerCapture(e.pointerId);
      this.draft = { kind, start: p, cur: p, points: [p] };
      this.render();
    }
  }

  private onMove(e: PointerEvent): void {
    const p = this.toImage(e);
    if (this.draft) {
      this.draft.cur = this.constrain(this.draft.start, p);
      this.draft.points.push(p);
      this.render();
    } else if (this.moving) {
      const dx = p.x - this.moving.from.x;
      const dy = p.y - this.moving.from.y;
      this.renderWithPreview(translate(this.moving.geom, dx, dy));
    }
  }

  private onUp(e: PointerEvent): void {
    if (this.draft) {
      const d = this.draft;
      this.draft = null;
      this.commitDraft(d);
      this.render();
    } else if (this.moving) {
      const p = this.toImage(e);
      const dx = p.x - this.moving.from.x;
      const dy = p.y - this.moving.from.y;
      const id = this.moving.id;
      const geom = translate(this.moving.geom, dx, dy);
      this.moving = null;
      if (Math.abs(dx) > 1 || Math.abs(dy) > 1) {
        this.cb.onMove(id, geom);
      }
    }
  }

  private constrain(start: Point, cur: Point): Point {
    if (!this.shiftDown) {
      return cur;
    }
    // Shift: square shapes / 45°-snapped lines.
    if (this.tool === 'line' || this.tool === 'arrow') {
      const dx = cur.x - start.x;
      const dy = cur.y - start.y;
      if (Math.abs(dx) > Math.abs(dy)) {
        return { x: cur.x, y: start.y };
      }
      return { x: start.x, y: cur.y };
    }
    const side = Math.max(Math.abs(cur.x - start.x), Math.abs(cur.y - start.y));
    return {
      x: start.x + Math.sign(cur.x - start.x) * side,
      y: start.y + Math.sign(cur.y - start.y) * side,
    };
  }

  private commitDraft(d: Draft): void {
    const style = this.style;
    if (d.kind === 'pencil' || d.kind === 'highlighter') {
      if (d.points.length < 2) {
        return;
      }
      const points = d.points.map((p) => ({ x: Math.round(p.x), y: Math.round(p.y) }));
      this.cb.onCreate(d.kind, { path: points }, style);
      return;
    }
    if (d.kind === 'arrow' || d.kind === 'line') {
      if (dist(d.start, d.cur) < 2) {
        return;
      }
      this.cb.onCreate(d.kind, { segment: { a: round(d.start), b: round(d.cur) } }, style);
      return;
    }
    // rect / ellipse
    const r = normRect(d.start.x, d.start.y, d.cur.x, d.cur.y);
    if (r.width < 2 || r.height < 2) {
      return;
    }
    this.cb.onCreate(d.kind, { rect: r }, style);
  }

  private hitTest(p: Point): number | null {
    for (let i = this.items.length - 1; i >= 0; i--) {
      const a = this.items[i];
      const b = boundsOf(a.geometry);
      const pad = Math.max(a.style.strokeWidth, 6);
      if (
        p.x >= b.x - pad &&
        p.y >= b.y - pad &&
        p.x <= b.x + b.width + pad &&
        p.y <= b.y + b.height + pad
      ) {
        return a.id;
      }
    }
    return null;
  }

  private render(): void {
    this.renderWithPreview(null);
  }

  private renderWithPreview(movedSelected: Geometry | null): void {
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    ctx.drawImage(this.base, 0, 0, this.canvas.width, this.canvas.height);

    for (const a of this.items) {
      const geom = movedSelected && a.id === this.selectedId ? movedSelected : a.geometry;
      drawAnnotation(ctx, a.kind, geom, a.style);
      if (a.id === this.selectedId) {
        drawSelection(ctx, geom);
      }
    }

    if (this.draft) {
      const style = this.style;
      if (this.draft.kind === 'pencil' || this.draft.kind === 'highlighter') {
        drawAnnotation(ctx, this.draft.kind, { path: this.draft.points }, style);
      } else if (this.draft.kind === 'arrow' || this.draft.kind === 'line') {
        drawAnnotation(
          ctx,
          this.draft.kind,
          { segment: { a: this.draft.start, b: this.draft.cur } },
          style,
        );
      } else {
        const r = normRect(
          this.draft.start.x,
          this.draft.start.y,
          this.draft.cur.x,
          this.draft.cur.y,
        );
        drawAnnotation(ctx, this.draft.kind, { rect: r }, style);
      }
    }
  }
}

function round(p: Point): Point {
  return { x: Math.round(p.x), y: Math.round(p.y) };
}

function dist(a: Point, b: Point): number {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

function translate(g: Geometry, dx: number, dy: number): Geometry {
  if ('rect' in g) {
    return { rect: { ...g.rect, x: g.rect.x + dx, y: g.rect.y + dy } };
  }
  if ('segment' in g) {
    return {
      segment: {
        a: { x: g.segment.a.x + dx, y: g.segment.a.y + dy },
        b: { x: g.segment.b.x + dx, y: g.segment.b.y + dy },
      },
    };
  }
  if ('path' in g) {
    return { path: g.path.map((p) => ({ x: p.x + dx, y: p.y + dy })) };
  }
  if ('anchor' in g) {
    return { anchor: { x: g.anchor.x + dx, y: g.anchor.y + dy } };
  }
  return {
    loupe: {
      center: { x: g.loupe.center.x + dx, y: g.loupe.center.y + dy },
      radius: g.loupe.radius,
    },
  };
}

function boundsOf(g: Geometry): Rect {
  if ('rect' in g) {
    return g.rect;
  }
  if ('segment' in g) {
    return normRect(g.segment.a.x, g.segment.a.y, g.segment.b.x, g.segment.b.y);
  }
  if ('path' in g) {
    const xs = g.path.map((p) => p.x);
    const ys = g.path.map((p) => p.y);
    const minx = Math.min(...xs);
    const miny = Math.min(...ys);
    return { x: minx, y: miny, width: Math.max(...xs) - minx, height: Math.max(...ys) - miny };
  }
  if ('anchor' in g) {
    return { x: g.anchor.x, y: g.anchor.y, width: 40, height: 24 };
  }
  return {
    x: g.loupe.center.x - g.loupe.radius,
    y: g.loupe.center.y - g.loupe.radius,
    width: g.loupe.radius * 2,
    height: g.loupe.radius * 2,
  };
}

function drawAnnotation(
  ctx: CanvasRenderingContext2D,
  kind: AnnotationKind,
  g: Geometry,
  style: Style,
): void {
  ctx.save();
  ctx.globalAlpha = style.opacity;
  ctx.lineWidth = style.strokeWidth;
  ctx.strokeStyle = rgbaCss(style.stroke);
  ctx.lineJoin = 'round';
  ctx.lineCap = 'round';

  switch (kind) {
    case 'rect':
    case 'blur':
    case 'pixelate':
    case 'spotlight': {
      if (!('rect' in g)) break;
      if (style.fill) {
        ctx.fillStyle = rgbaCss(style.fill);
        ctx.fillRect(g.rect.x, g.rect.y, g.rect.width, g.rect.height);
      }
      if (kind !== 'rect') {
        ctx.setLineDash([6, 4]);
      }
      ctx.strokeRect(g.rect.x, g.rect.y, g.rect.width, g.rect.height);
      break;
    }
    case 'ellipse': {
      if (!('rect' in g)) break;
      const rx = g.rect.width / 2;
      const ry = g.rect.height / 2;
      ctx.beginPath();
      ctx.ellipse(g.rect.x + rx, g.rect.y + ry, rx, ry, 0, 0, Math.PI * 2);
      if (style.fill) {
        ctx.fillStyle = rgbaCss(style.fill);
        ctx.fill();
      }
      ctx.stroke();
      break;
    }
    case 'line':
    case 'arrow': {
      if (!('segment' in g)) break;
      const { a, b } = g.segment;
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();
      if (kind === 'arrow' && style.arrowHead !== 'none') {
        drawArrowHead(ctx, a, b, style.strokeWidth);
      }
      break;
    }
    case 'pencil':
    case 'highlighter':
    case 'magnifier': {
      if (!('path' in g)) {
        if ('loupe' in g) {
          ctx.beginPath();
          ctx.ellipse(
            g.loupe.center.x,
            g.loupe.center.y,
            g.loupe.radius,
            g.loupe.radius,
            0,
            0,
            Math.PI * 2,
          );
          ctx.stroke();
        }
        break;
      }
      if (kind === 'highlighter') {
        ctx.globalAlpha = 0.4 * style.opacity;
        ctx.lineWidth = Math.max(style.strokeWidth, 8);
      }
      ctx.beginPath();
      g.path.forEach((p, i) => (i === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)));
      ctx.stroke();
      break;
    }
    case 'text': {
      if (!('anchor' in g)) break;
      const t = style.text;
      ctx.font = `${t.size}px sans-serif`;
      ctx.textBaseline = 'top';
      if (t.background) {
        const w = ctx.measureText(t.content).width;
        ctx.fillStyle = rgbaCss(t.background);
        ctx.fillRect(g.anchor.x - 4, g.anchor.y - 2, w + 8, t.size + 6);
      }
      ctx.fillStyle = rgbaCss(t.color);
      ctx.fillText(t.content, g.anchor.x, g.anchor.y);
      break;
    }
    case 'stepNumber': {
      if (!('anchor' in g)) break;
      ctx.fillStyle = rgbaCss(style.stroke);
      ctx.beginPath();
      ctx.ellipse(g.anchor.x, g.anchor.y, 12, 12, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = 'white';
      ctx.font = 'bold 14px sans-serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(String(style.stepIndex), g.anchor.x, g.anchor.y);
      break;
    }
  }
  ctx.restore();
}

function drawArrowHead(
  ctx: CanvasRenderingContext2D,
  from: Point,
  tip: Point,
  width: number,
): void {
  const angle = Math.atan2(tip.y - from.y, tip.x - from.x);
  const size = Math.max(width * 4, 10);
  for (const off of [Math.PI - 0.5, Math.PI + 0.5]) {
    ctx.beginPath();
    ctx.moveTo(tip.x, tip.y);
    ctx.lineTo(tip.x + Math.cos(angle + off) * size, tip.y + Math.sin(angle + off) * size);
    ctx.stroke();
  }
}

function drawSelection(ctx: CanvasRenderingContext2D, g: Geometry): void {
  const b = boundsOf(g);
  ctx.save();
  ctx.setLineDash([5, 4]);
  ctx.strokeStyle = 'rgba(79, 70, 229, 0.95)';
  ctx.lineWidth = 1.5;
  ctx.strokeRect(b.x - 3, b.y - 3, b.width + 6, b.height + 6);
  ctx.restore();
}
