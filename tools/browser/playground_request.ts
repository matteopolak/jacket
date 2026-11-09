// Build one browser request from a source workspace. No compiler or import parsing.
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import type { CatalogueCase, PlayFiles, RequestSize } from './play_types.ts';

export async function collectPlayFiles(dir: string, base: string, files: PlayFiles): Promise<void> {
  const entries = await readdir(dir, { withFileTypes: true });
  entries.sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
  for (const entry of entries) {
    // AppleDouble files are filesystem sidecars, not checked-in inputs.
    if (entry.name.startsWith('._')) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) await collectPlayFiles(full, base, files);
    else if (entry.isFile()) {
      const relative = path.relative(base, full).replaceAll('\\', '/');
      // Every file is sent as its bytes, so binary fixtures arrive intact.
      files[relative] = new Uint8Array(await readFile(full));
    }
  }
}

export async function buildPlayRequest(
  root: string,
  test: Pick<CatalogueCase, 'path'>,
): Promise<{ files: PlayFiles; main: string; request: RequestSize }> {
  const parts = test.path.split('/');
  const base = path.join(root, ...parts.slice(0, 2));
  const files: PlayFiles = {};
  await collectPlayFiles(base, base, files);
  const main = path.relative(base, path.join(root, test.path)).replaceAll('\\', '/');
  return { files, main, request: playRequestSize(files, main) };
}

export function playRequestSize(files: PlayFiles, main: string): RequestSize {
  let pathBytes = 0,
    sourceBytes = 0;
  for (const [name, contents] of Object.entries(files)) {
    pathBytes += Buffer.byteLength(name, 'utf8');
    sourceBytes +=
      typeof contents === 'string' ? Buffer.byteLength(contents, 'utf8') : contents.length;
  }
  const mainBytes = Buffer.byteLength(main, 'utf8');
  return {
    fileCount: Object.keys(files).length,
    pathBytes,
    sourceBytes,
    mainBytes,
    totalBytes: pathBytes + sourceBytes + mainBytes,
  };
}
