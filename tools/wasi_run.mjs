// Runs a WASI command module built by `jaic build -os wasm` under node's WASI (preview 1), with
// this process's stdin, stdout, stderr and environment, and exits with the module's exit code.
// The module sees the host's file system: `/` is pre-opened, and `PWD` names this process's
// working directory, which Wasi_Runtime starts from (WASI itself has no working directory).
// Usage: node --no-warnings tools/wasi_run.mjs module.wasm [args...]
// Needs node 24 or newer: jaic's modules use 64-bit memory (Memory64).
//
// A trap (`unreachable`, an out-of-bounds access, a stack overflow) prints `wasm trap: ...` and
// exits 134, like an abort. A module that imports something the host lacks prints
// `wasm link error: ...` and exits 127. See docs/native/wasm-target.md.
import { readFileSync } from 'node:fs';
import { WASI } from 'node:wasi';

const [path, ...args] = process.argv.slice(2);
if (!path) {
  console.error('usage: node tools/wasi_run.mjs module.wasm [args...]');
  process.exit(2);
}
const wasi = new WASI({
  version: 'preview1',
  args: [path, ...args],
  env: { ...process.env, PWD: process.cwd() },
  preopens: { '/': '/' },
  returnOnExit: true,
});
const module = await WebAssembly.compile(readFileSync(path));
// Name every import WASI does not provide (an `env` import is a C function nothing defined),
// rather than node's error about the first missing import module.
const provided = wasi.getImportObject();
const missing = WebAssembly.Module.imports(module)
  .filter((i) => !(i.module in provided) || !(i.name in provided[i.module]))
  .map((i) => `${i.module}.${i.name}`);
if (missing.length) {
  console.error(`wasm link error: missing imports ${missing.join(', ')}`);
  process.exit(127);
}
let instance;
try {
  instance = await WebAssembly.instantiate(module, provided);
} catch (e) {
  if (!(e instanceof WebAssembly.LinkError)) throw e;
  console.error(`wasm link error: ${e.message}`);
  process.exit(127);
}
try {
  process.exitCode = wasi.start(instance);
} catch (e) {
  if (!(e instanceof WebAssembly.RuntimeError || e instanceof RangeError)) throw e;
  console.error(`wasm trap: ${e.message}`);
  process.exitCode = 134;
}
