// The floating horizontal tool row plus a compact contextual strip (stroke
// colour + thickness). Never a sidebar (FR-009). Reports tool/style intents to
// the controller; it holds no document state.

import type { Rgba, Tool } from './types';

const TOOLS: { tool: Tool; label: string; title: string }[] = [
  { tool: 'select', label: '⌖', title: 'Select (V)' },
  { tool: 'rect', label: '▭', title: 'Rectangle (R)' },
  { tool: 'ellipse', label: '◯', title: 'Ellipse (O)' },
  { tool: 'arrow', label: '↗', title: 'Arrow (A)' },
  { tool: 'line', label: '╱', title: 'Line (L)' },
  { tool: 'pencil', label: '✎', title: 'Pencil (D)' },
  { tool: 'highlighter', label: '▰', title: 'Highlighter (H)' },
  { tool: 'text', label: 'T', title: 'Text (T)' },
];

const SWATCHES: Rgba[] = [
  [239, 68, 68, 255], // red
  [37, 99, 235, 255], // blue
  [22, 163, 74, 255], // green
  [234, 179, 8, 255], // yellow
  [17, 24, 39, 255], // near-black
  [255, 255, 255, 255], // white
];

const WIDTHS = [1, 2, 4, 8];

export interface ToolbarCallbacks {
  onTool: (t: Tool) => void;
  onColor: (c: Rgba) => void;
  onWidth: (w: number) => void;
}

export interface ToolbarApi {
  setActive: (t: Tool) => void;
}

export function renderToolbar(
  parent: HTMLElement,
  current: Tool,
  cb: ToolbarCallbacks,
): ToolbarApi {
  const bar = document.createElement('div');
  bar.className = 'pillbar';
  bar.setAttribute('data-tauri-drag-region', '');

  const buttons = new Map<Tool, HTMLButtonElement>();
  for (const t of TOOLS) {
    const btn = document.createElement('button');
    btn.className = 'tool-btn';
    btn.textContent = t.label;
    btn.title = t.title;
    btn.addEventListener('click', () => cb.onTool(t.tool));
    buttons.set(t.tool, btn);
    bar.appendChild(btn);
  }

  bar.appendChild(divider());

  for (const c of SWATCHES) {
    const sw = document.createElement('button');
    sw.className = 'swatch';
    sw.style.background = `rgb(${c[0]}, ${c[1]}, ${c[2]})`;
    sw.title = 'Stroke colour';
    sw.addEventListener('click', () => cb.onColor(c));
    bar.appendChild(sw);
  }

  bar.appendChild(divider());

  for (const w of WIDTHS) {
    const wb = document.createElement('button');
    wb.className = 'tool-btn width-btn';
    wb.textContent = String(w);
    wb.title = `Thickness ${w}`;
    wb.addEventListener('click', () => cb.onWidth(w));
    bar.appendChild(wb);
  }

  parent.appendChild(bar);

  const setActive = (t: Tool): void => {
    for (const [tool, btn] of buttons) {
      btn.classList.toggle('active', tool === t);
    }
  };
  setActive(current);

  return { setActive };
}

function divider(): HTMLElement {
  const d = document.createElement('span');
  d.className = 'divider';
  return d;
}
