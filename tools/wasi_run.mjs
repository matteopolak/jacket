// Runs a WASI command module built by `jaic build -os wasm` under node's WASI (preview 1), with
// this process's stdin, stdout, stderr and environment, and exits with the module's exit code.
// Usage: node --no-warnings tools/wasi_run.mjs module.wasm [args...]
// Needs node 24 or newer: jaic's modules use 64-bit memory (Memory64).
// See docs/native/wasm-target.md.
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
  env: process.env,
  returnOnExit: true,
});
const module = await WebAssembly.compile(readFileSync(path));
const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
process.exitCode = wasi.start(instance);
