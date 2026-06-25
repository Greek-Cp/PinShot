import { defineConfig } from 'vite';
import { resolve } from 'node:path';

// Tauri serves the dev frontend on a fixed port and expects a static build in
// `dist/`. No backend, no proxying — the app is fully offline. Three pages: the
// main window (index.html), the capture selection overlay (overlay.html), and a
// floating pin (pin.html).
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
      },
    },
  },
});
