import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
//import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(),wasm(),topLevelAwait()],
  resolve: {
    alias: {
      // Add alias for your package if needed
      //'@kwarc/ftml-viewer': path.resolve(__dirname, 'node_modules/@kwarc/ftml-viewer')
    }
  },
  optimizeDeps: {
    include: ['@kwarc/ftml-viewer'],
    exclude: ['@kwarc/ftml-viewer-base']
  }
});