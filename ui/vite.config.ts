import { defineConfig } from 'vite';
import { resolve } from 'node:path';

// Tauri serves the dev frontend on a fixed port and expects a static build in
// `dist/`. No backend, no proxying — the app is fully offline. Two pages: the
// main window (index.html) and the capture selection overlay (overlay.html).
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
      },
    },
  },
});
