// A bounded filesystem host for one JPEG probe. The outer OS sandbox keeps /input read-only.
// node wasi_probe.mjs probe.wasm CORPUS WORK RELATIVE_INPUT OUTPUT_NAME
import { readFileSync } from 'node:fs';
import { resolve, relative, isAbsolute, sep } from 'node:path';
import { WASI } from 'node:wasi';

const [modulePath, corpus, work, input, output] = process.argv.slice(2);
if (!modulePath || !corpus || !work || !input || !output) {
  throw new Error('expected module, corpus, work, relative input and output name');
}
const inputRelative = relative(resolve(corpus), resolve(corpus, input));
const outputRelative = relative(resolve(work), resolve(work, output));
const escapes = text => isAbsolute(text) || text === '..' || text.startsWith(`..${sep}`);
if (escapes(inputRelative) || escapes(outputRelative)) {
  throw new Error('probe path escapes its configured directory');
}
const guest = text => text.split(sep).join('/');
const wasi = new WASI({
  version: 'preview1',
  args: ['jpeg-probe', `/input/${guest(inputRelative)}`, `/output/${guest(outputRelative)}`],
  env: { PWD: '/output', TMPDIR: '/output' },
  preopens: { '/input': resolve(corpus), '/output': resolve(work) },
  returnOnExit: true,
});
try {
  const module = await WebAssembly.compile(readFileSync(modulePath));
  const instance = await WebAssembly.instantiate(module, { wasi_snapshot_preview1: wasi.wasiImport });
  process.exitCode = wasi.start(instance);
} catch (error) {
  console.error(error);
  process.exitCode = 134;
}
