import { invoke } from '@tauri-apps/api/core';
import { EditorCanvas } from './canvas';
import { renderActionBar } from './actionbar';
import { renderToolbar } from './toolbar';
import type { ActionBarApi, ExportTarget } from './actionbar';
import type { ToolbarApi } from './toolbar';
import type {
  AnnotationKind,
  DocResponse,
  EditorImagePayload,
  Geometry,
  Rgba,
  Style,
  Tool,
} from './types';
import { defaultStyle } from './types';

// The editor controller: loads the captured image, wires the floating toolbar,
// canvas, and action bar, and forwards every mutation to the Rust core over IPC.
// The shell owns the authoritative document; this page renders snapshots.

let style: Style = defaultStyle();
let tool: Tool = 'rect';
let canvas: EditorCanvas | null = null;
let toolbar: ToolbarApi | null = null;
let actionbar: ActionBarApi | null = null;

async function refresh(resp?: DocResponse): Promise<void> {
  const doc = resp ?? (await invoke<DocResponse>('editor_get_doc'));
  canvas?.setItems(doc.items);
  actionbar?.setUndoRedo(doc.canUndo, doc.canRedo);
}

async function add(kind: AnnotationKind, geometry: Geometry, st: Style): Promise<void> {
  try {
    await invoke('editor_add', { kind, geometry, style: st });
    await refresh();
  } catch (e) {
    console.error('editor_add failed', e);
  }
}

async function move(id: number, geometry: Geometry): Promise<void> {
  try {
    const resp = await invoke<DocResponse>('editor_update', { id, geometry, style: null });
    await refresh(resp);
  } catch (e) {
    console.error('editor_update failed', e);
  }
}

async function exportTo(target: ExportTarget): Promise<void> {
  try {
    await invoke('editor_export', { target, format: null });
    // The shell closes the editor window on success.
  } catch (e) {
    console.error('editor_export failed', e);
  }
}

async function undo(): Promise<void> {
  try {
    await refresh(await invoke<DocResponse>('editor_undo'));
  } catch (e) {
    console.error('editor_undo failed', e);
  }
}

async function redo(): Promise<void> {
  try {
    await refresh(await invoke<DocResponse>('editor_redo'));
  } catch (e) {
    console.error('editor_redo failed', e);
  }
}

async function close(): Promise<void> {
  try {
    await invoke('editor_close');
  } catch (e) {
    console.error('editor_close failed', e);
  }
}

function setTool(t: Tool): void {
  tool = t;
  canvas?.setTool(t);
  toolbar?.setActive(t);
}

function setColor(c: Rgba): void {
  style = { ...style, stroke: c, text: { ...style.text, color: c } };
  canvas?.setStyle(style);
}

function setWidth(w: number): void {
  style = { ...style, strokeWidth: w };
  canvas?.setStyle(style);
}

function onKeyDown(e: KeyboardEvent): void {
  if (e.key === 'Escape') {
    void close();
    return;
  }
  if (e.metaKey || e.ctrlKey) {
    if (e.key.toLowerCase() === 'z') {
      e.preventDefault();
      void (e.shiftKey ? redo() : undo());
    }
    return;
  }
  switch (e.key.toLowerCase()) {
    case 'v':
      setTool('select');
      break;
    case 'r':
      setTool('rect');
      break;
    case 'o':
      setTool('ellipse');
      break;
    case 'a':
      setTool('arrow');
      break;
    case 'l':
      setTool('line');
      break;
    case 'd':
      setTool('pencil');
      break;
    case 'h':
      setTool('highlighter');
      break;
    case 't':
      setTool('text');
      break;
    case 'p':
      void exportTo('pin');
      break;
    case 'c':
      void exportTo('clipboard');
      break;
    case 's':
      void exportTo('file');
      break;
    default:
      break;
  }
}

async function init(): Promise<void> {
  const toolbarEl = document.getElementById('toolbar');
  const stageEl = document.getElementById('stage');
  const actionEl = document.getElementById('actionbar');
  if (!toolbarEl || !stageEl || !actionEl) {
    throw new Error('editor layout missing');
  }

  let payload: EditorImagePayload;
  try {
    payload = await invoke<EditorImagePayload>('editor_get_image');
  } catch (e) {
    console.error('could not load editor image', e);
    return;
  }

  const img = new Image();
  img.src = payload.dataUrl;
  await img.decode().catch(() => undefined);

  canvas = new EditorCanvas(stageEl, img, style, {
    onCreate: (kind, geometry, st) => void add(kind, geometry, st),
    onMove: (id, geometry) => void move(id, geometry),
    onSelect: () => {},
  });
  toolbar = renderToolbar(toolbarEl, tool, {
    onTool: setTool,
    onColor: setColor,
    onWidth: setWidth,
  });
  actionbar = renderActionBar(actionEl, {
    onExport: (t) => void exportTo(t),
    onUndo: () => void undo(),
    onRedo: () => void redo(),
    onClose: () => void close(),
  });

  canvas.setTool(tool);
  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('contextmenu', (e) => e.preventDefault());
}

void init();
