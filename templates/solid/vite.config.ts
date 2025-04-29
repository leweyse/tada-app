import { fileURLToPath } from 'node:url';

import { dirname, resolve } from 'pathe';
import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';

export const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  plugins: [solid()],
});
