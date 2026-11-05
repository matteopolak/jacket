import { defineConfig } from 'astro/config';
import { resolve } from 'node:path';

export default defineConfig({
  output: 'static',
  prefetch: { prefetchAll: false, defaultStrategy: 'hover' },
  base: process.env['JACKET_DOC_BASE'] ?? '/',
  outDir: resolve(process.env['JACKET_DOC_OUT'] ?? 'dist'),
  trailingSlash: 'never',
  build: { format: 'file' },
  compressHTML: false,
  devToolbar: { enabled: false },
});
