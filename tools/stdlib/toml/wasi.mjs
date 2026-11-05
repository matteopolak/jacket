// TOML protocol command under the existing Node WASI runtime. Only its explicit test-mode
// environment variable is forwarded; program arguments select semantic encoding.
// Example: node --no-warnings wasi.mjs adapter.wasm encode < tagged.json
import {readFileSync} from 'node:fs';
import {WASI} from 'node:wasi';
const [path, ...args] = process.argv.slice(2);
if (!path) throw new Error('usage: node wasi.mjs adapter.wasm [encode]');
try {
  const module = await WebAssembly.compile(readFileSync(path));
  if (WebAssembly.Module.imports(module).some(item => item.module !== 'wasi_snapshot_preview1')) {
    throw new Error('TOML protocol adapter has non-WASI imports');
  }
  const wasi = new WASI({version: 'preview1', args: [path, ...args],
    env: {TOML_CONFORMANCE_STATEMENTS: process.env.TOML_CONFORMANCE_STATEMENTS || '0'}, returnOnExit: true});
  const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
  process.exitCode = wasi.start(instance);

} catch (error) {
  console.error(String(error.stack || error));
  process.exitCode = 70; // A trap/host failure must never count as an expected parse rejection (1).
}
