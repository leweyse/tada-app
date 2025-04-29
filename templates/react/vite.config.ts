import { fileURLToPath } from 'node:url';

import react from '@vitejs/plugin-react-swc';
import { dirname, resolve } from 'pathe';
import { defineConfig } from 'vite';

export const __dirname = dirname(fileURLToPath(import.meta.url));

// https://vitejs.dev/config/
export default defineConfig({
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  plugins: [react()],
});
