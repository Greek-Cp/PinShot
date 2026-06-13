import { defineConfig } from 'vite';

// Tauri serves the dev frontend on a fixed port and expects a static build in
// `dist/`. No backend, no proxying — the app is fully offline.
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
  },
});
