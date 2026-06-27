import { defineConfig } from 'vite';
import { resolve } from 'node:path';

// Tauri serves the dev frontend on a fixed port and expects a static build in
// `dist/`. No backend, no proxying — the app is fully offline. Pages: the main
// window (index.html), the capture selection overlay (overlay.html), a floating
// pin (pin.html), the floating annotation editor (editor.html), and the
// settings window (settings.html).
export default defineConfig({
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: 'es2022',
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        overlay: resolve(__dirname, 'overlay.html'),
        pin: resolve(__dirname, 'pin.html'),
        editor: resolve(__dirname, 'editor.html'),
        settings: resolve(__dirname, 'settings.html'),
        preview: resolve(__dirname, 'preview.html'),
      },
    },
  },
});
