import { invoke } from '@tauri-apps/api/core';

interface SelectionRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

// Smoke-test the core ↔ shell ↔ webview wiring: ask the Rust side to normalise
// a drag selection and show the result. Outside a Tauri runtime (e.g. plain
// `vite preview`) the invoke simply rejects, which we report rather than crash.
async function showSelectionDemo(): Promise<void> {
  const status = document.querySelector<HTMLParagraphElement>('#status');
  if (!status) {
    return;
  }
  try {
    const rect = await invoke<SelectionRect>('selection_rect', {
      x0: 100,
      y0: 80,
      x1: 20,
      y1: 10,
    });
    status.textContent = `Selection: ${rect.width}×${rect.height} at (${rect.x}, ${rect.y})`;
  } catch {
    status.textContent = 'Offline screenshot & pin tool (running outside Tauri).';
  }
}

void showSelectionDemo();
