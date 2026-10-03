// Node host adapter for the unchanged browser worker and actual compiled Wasm.
import { parentPort, workerData } from "node:worker_threads";
import { readFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
const root = path.resolve(workerData.directory);
globalThis.fetch = async value => {
  const url = new URL(value);
  if (url.protocol !== "file:" || path.resolve(fileURLToPath(url)) !== path.join(root, "jai_wasm.wasm")) throw new Error("Worker probe admits only its real staged compiler asset.");
  return new Response(await readFile(fileURLToPath(url)), { status: 200 });
};
globalThis.self = { postMessage: value => parentPort.postMessage(value), onmessage: undefined };
await import(pathToFileURL(path.join(root, "worker.mjs")).href);
if (typeof self.onmessage !== "function") throw new Error("Staged browser worker did not register its message boundary.");
parentPort.on("message", data => self.onmessage({ data }));
