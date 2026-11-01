// Compiles and runs one example with the browser build of the compiler. It lives in a worker so
// a program that never ends can be stopped by ending the worker, and the page stays responsive.
let engine = null;

self.onmessage = async (event) => {
  const { program, engine: engineUrl, wasm } = event.data;
  try {
    if (!engine) {
      const module = await import(engineUrl);
      const bytes = new Uint8Array(await (await fetch(wasm)).arrayBuffer());
      engine = await module.createEngine(bytes);
    }
    const started = performance.now();
    // The budget counts interpreter steps, so a loop that never ends stops with a message.
    const result = engine.play({ 'main.jai': program }, 'main.jai', { budget: 200_000_000 });
    self.postMessage({ result, ms: performance.now() - started });
  } catch (error) {
    const crashed = Boolean(error && error.compilerCrashed);
    if (crashed) engine = null;
    self.postMessage({ error: String((error && error.message) || error), crashed });
  }
};
