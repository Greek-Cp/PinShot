// The floating action bar below the canvas: Pin · Copy · Save · Undo · Redo ·
// Close. Deliberately NO Share / upload / cloud (FR-047). Reports intents.

export type ExportTarget = 'clipboard' | 'file' | 'pin';

export interface ActionCallbacks {
  onExport: (target: ExportTarget) => void;
  onUndo: () => void;
  onRedo: () => void;
  onClose: () => void;
}

export interface ActionBarApi {
  setUndoRedo: (canUndo: boolean, canRedo: boolean) => void;
}

export function renderActionBar(parent: HTMLElement, cb: ActionCallbacks): ActionBarApi {
  const bar = document.createElement('div');
  bar.className = 'pillbar actionbar';

  const pin = action('📌 Pin', 'Pin (P)', () => cb.onExport('pin'));
  const copy = action('⧉ Copy', 'Copy (C)', () => cb.onExport('clipboard'));
  const save = action('💾 Save', 'Save (S)', () => cb.onExport('file'));
  const undo = action('↶', 'Undo (⌘Z)', () => cb.onUndo());
  const redo = action('↷', 'Redo (⌘⇧Z)', () => cb.onRedo());
  const close = action('✕', 'Close (Esc)', () => cb.onClose());

  bar.append(pin, copy, save, undo, redo, close);
  parent.appendChild(bar);

  return {
    setUndoRedo(canUndo, canRedo) {
      undo.disabled = !canUndo;
      redo.disabled = !canRedo;
    },
  };
}

function action(label: string, title: string, onClick: () => void): HTMLButtonElement {
  const btn = document.createElement('button');
  btn.className = 'action-btn';
  btn.textContent = label;
  btn.title = title;
  btn.addEventListener('click', onClick);
  return btn;
}
