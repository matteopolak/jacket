// The registered regressions embed fixtures at compile time; their WASI host needs only scratch.
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { basename, join } from 'node:path';
import { tmpdir } from 'node:os';
import { WASI } from 'node:wasi';
const [binary, ...args] = process.argv.slice(2);
if (!binary) throw new Error('expected a compiled Jacket golden program');
const work = mkdtempSync(join(tmpdir(), 'jai-image-registration-'));
try {
  const wasi = new WASI({
    version: 'preview1', args: [basename(binary), ...args],
    env: { PWD: '/work', TMPDIR: '/work' }, preopens: { '/work': work },
    returnOnExit: true,
  });
  const module = await WebAssembly.compile(readFileSync(binary));
  const instance = await WebAssembly.instantiate(module, { wasi_snapshot_preview1: wasi.wasiImport });
  process.exitCode = wasi.start(instance);
} catch (error) {
  console.error(error);
  process.exitCode = 134;
} finally {
  rmSync(work, { recursive: true, force: true });
}
