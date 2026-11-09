// Bounded WASI host: expose only the explicitly supplied corpus/oracle/output directories.
// node wasi_corpus.ts probe.wasm DIRECTORY...
import { readFileSync, statSync } from 'node:fs';
import { resolve } from 'node:path';
import { WASI } from 'node:wasi';

const [modulePath, ...directories] = process.argv.slice(2);
if (!modulePath || directories.length < 2 || directories.length > 3) {
  throw new Error('expected module and two or three probe directories');
}
const preopens: Record<string, string> = {};
const args = ['corpus-probe'];
for (const [index, directory] of directories.entries()) {
  const guest = `/arg${index}`;
  if (!statSync(directory).isDirectory()) throw new Error('probe argument is not a directory');
  preopens[guest] = resolve(directory);
  args.push(guest);
}
const wasi = new WASI({
  version: 'preview1',
  args,
  preopens,
  env: { PWD: '/arg0', TMPDIR: '/arg0' },
  returnOnExit: true,
});
try {
  const module = await WebAssembly.compile(readFileSync(modulePath));
  const instance = await WebAssembly.instantiate(module, {
    wasi_snapshot_preview1: wasi.wasiImport,
  });
  process.exitCode = wasi.start(instance);
} catch (error) {
  console.error(error);
  process.exitCode = 134;
}
