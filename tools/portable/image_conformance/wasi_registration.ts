// The registered regressions embed fixtures at compile time; their WASI host needs only scratch.
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { basename, isAbsolute, join } from 'node:path';
import { tmpdir } from 'node:os';
import { WASI } from 'node:wasi';
const [binary, ...args] = process.argv.slice(2);
if (!binary) throw new Error('expected a compiled Jacket golden program');
if (args.length > 1 || (args.length && (!isAbsolute(args[0]) || args[0] === '/'))) {
  throw new Error('expected at most one absolute Jacket scratch-directory argument');
}
const work = mkdtempSync(join(tmpdir(), 'jai-image-registration-'));
try {
  // Jacket's $TEMP stays the same guest argument, but aliases fresh owned storage.
  const preopens: Record<string, string> = { '/work': work };
  if (args.length) preopens[args[0]] = work;
  const wasi = new WASI({
    version: 'preview1',
    args: [basename(binary), ...args],
    env: { PWD: '/work', TMPDIR: '/work' },
    preopens,
    returnOnExit: true,
  });
  const module = await WebAssembly.compile(readFileSync(binary));
  const instance = await WebAssembly.instantiate(module, {
    wasi_snapshot_preview1: wasi.wasiImport,
  });
  process.exitCode = wasi.start(instance);
} catch (error) {
  console.error(error);
  process.exitCode = 134;
} finally {
  rmSync(work, { recursive: true, force: true });
}
