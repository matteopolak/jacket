import { createEngine } from "./engine.mjs";
let enginePromise;
async function engine() {
  enginePromise ??= (async () => {
    const response = await fetch(new URL("./jai_wasm.wasm", import.meta.url));
    if (!response.ok) throw new Error(`Compiler asset could not be loaded (${response.status}).`);
    return createEngine(await response.arrayBuffer());
  })();
  return enginePromise;
}
let queue = Promise.resolve();
self.onmessage = ({ data }) => {
  queue = queue.then(async () => {
    try {
      const runtime = await engine();
      if (data.type === "init") { self.postMessage({ type: "init", capabilities: { languageServer: typeof runtime.lsp === "function" } }); return; }
      if (data.type === "lsp") {
        if (!runtime.lsp) throw new Error("This compiler bundle does not expose the shared language service.");
        self.postMessage({ type: "lsp", id: data.id, messages: runtime.lsp(data.message) }); return;
      }
      self.postMessage({ type: "run-started", id: data.id });
      const result = runtime.run(data.source, data.options);
      self.postMessage({ type: "run", id: data.id, result: { exitCode: result.exitCode.toString(), steps: result.steps.toString() } });
    } catch (error) { self.postMessage({ type: data.type ?? "run", id: data.id, error: error instanceof Error ? error.message : String(error) }); }
  });
};
