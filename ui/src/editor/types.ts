// Wire types for the floating editor, matching the `pinshot-core` serde output
// (Constitution IV: the webview is a thin renderer; the Rust core owns the
// authoritative pixels). Geometry crosses IPC in base-image pixels.

export type Rgba = [number, number, number, number];

export interface Point {
  x: number;
  y: number;
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

// Externally-tagged to match serde's `Geometry` enum (camelCase variant names).
export type Geometry =
  | { rect: Rect }
  | { segment: { a: Point; b: Point } }
  | { path: Point[] }
  | { anchor: Point }
  | { loupe: { center: Point; radius: number } };

export type Tool =
  | 'select'
  | 'rect'
  | 'ellipse'
  | 'arrow'
  | 'line'
  | 'pencil'
  | 'highlighter'
  | 'text';

// AnnotationKind values (serde camelCase). US1 uses the drawing kinds below.
export type AnnotationKind =
  | 'rect'
  | 'ellipse'
  | 'arrow'
  | 'line'
  | 'pencil'
  | 'highlighter'
  | 'text'
  | 'blur'
  | 'pixelate'
  | 'spotlight'
  | 'magnifier'
  | 'stepNumber';

export interface TextStyle {
  content: string;
  size: number;
  color: Rgba;
  background: Rgba | null;
  shadow: boolean;
}

// Mirrors core `Style` (serde camelCase, all fields default-able). The UI sends
// a full object; fields irrelevant to a kind are simply ignored by the core.
export interface Style {
  stroke: Rgba;
  strokeWidth: number;
  fill: Rgba | null;
  opacity: number;
  cornerRadius: number;
  arrowHead: 'none' | 'open' | 'filled';
  dashed: boolean;
  text: TextStyle;
  blurStrength: number;
  pixelateBlock: number;
  spotlightDim: number;
  magnifierZoom: number;
  stepIndex: number;
}

export interface Annotation {
  id: number;
  kind: AnnotationKind;
  geometry: Geometry;
  style: Style;
  z: number;
}

export interface EditorImagePayload {
  width: number;
  height: number;
  scaleFactor: number;
  dataUrl: string;
}

export interface DocResponse {
  revision: number;
  items: Annotation[];
  canUndo: boolean;
  canRedo: boolean;
}

// A reasonable default style; the toolbar mutates stroke / strokeWidth / fill.
export function defaultStyle(): Style {
  return {
    stroke: [239, 68, 68, 255],
    strokeWidth: 4,
    fill: null,
    opacity: 1,
    cornerRadius: 0,
    arrowHead: 'filled',
    dashed: false,
    text: { content: '', size: 24, color: [239, 68, 68, 255], background: null, shadow: false },
    blurStrength: 8,
    pixelateBlock: 12,
    spotlightDim: 0.6,
    magnifierZoom: 2,
    stepIndex: 1,
  };
}

export function rgbaCss(c: Rgba): string {
  return `rgba(${c[0]}, ${c[1]}, ${c[2]}, ${c[3] / 255})`;
}

export function normRect(x0: number, y0: number, x1: number, y1: number): Rect {
  return {
    x: Math.round(Math.min(x0, x1)),
    y: Math.round(Math.min(y0, y1)),
    width: Math.round(Math.abs(x1 - x0)),
    height: Math.round(Math.abs(y1 - y0)),
  };
}
