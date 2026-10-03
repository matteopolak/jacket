import { createEngine } from "./engine.mjs";
let engine;
self.onmessage = async ({ data }) => {
  try {
    engine ??= await createEngine(await (await fetch("./jai_wasm.wasm")).arrayBuffer());
    const result = engine.run(data.source, data.options);
    self.postMessage({ result: { exitCode: result.exitCode.toString(), steps: result.steps.toString() } });
  } catch (error) {
    self.postMessage({ error: error instanceof Error ? error.message : String(error) });
  }
};
