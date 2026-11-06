import { defineConfig } from 'astro/config';
import { resolve } from 'node:path';
import {documentationPublication} from './src/lib/dev-publication.ts';

// Resolve the admitted base before Astro normalizes routes and creates Vite's configuration.
let base=process.env['JACKET_DOC_BASE'];
if(base===undefined&&process.env['JACKET_DOC_EXPORT']){
  const {bundle}=await import('./src/lib/loader.ts');base=(await bundle()).site.pageBase;
}

export default defineConfig({
  output: 'static',
  integrations: [documentationPublication()],
  prefetch: { prefetchAll: false, defaultStrategy: 'hover' },
  base: base ?? '/',
  outDir: resolve(process.env['JACKET_DOC_OUT'] ?? 'dist'),
  trailingSlash: 'never',
  build: { format: 'file' },
  compressHTML: false,
  devToolbar: { enabled: false },
});
