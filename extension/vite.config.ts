import { defineConfig } from 'vite';
import { copyFileSync } from 'node:fs';
import { resolve } from 'node:path';
export default defineConfig({
  root: resolve('extension'),
  plugins: [{ name: 'local-manifest', closeBundle() { copyFileSync('extension/manifest.json', 'extension/dist/manifest.json'); } }],
  build: { outDir: 'dist', emptyOutDir: true, rollupOptions: {
    input: { background: resolve('extension/src/background.ts'), popup: resolve('extension/popup.html') },
    output: { entryFileNames: '[name].js' },
  } },
});
