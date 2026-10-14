import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";

const path = process.argv[2];
if (!path) throw new Error("usage: node tools/run_foundations_wasm.mjs program.wasm");
const module = await WebAssembly.compile(readFileSync(path));
const unavailable = WebAssembly.Module.imports(module).filter(item => item.module !== "wasi_snapshot_preview1");
if (unavailable.length) throw new Error(`non-WASI imports: ${JSON.stringify(unavailable)}`);
const wasi = new WASI({ version: "preview1", args: [path], returnOnExit: true });
const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
process.exitCode = wasi.start(instance);
