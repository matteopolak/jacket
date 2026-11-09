// @ts-check
// Compiles and runs one example with the browser build of the compiler. It lives in a worker so
// a program that never ends can be stopped by ending the worker, and the page stays responsive.

/**
 * The browser build of the compiler, which is loaded from a URL the page names.
 * @typedef {{ play: (files: Record<string, string>, main: string, options: { budget: number }) => unknown }} Engine
 */

/** @type {Engine | null} */
let engine = null;

const scope = /** @type {DedicatedWorkerGlobalScope} */ (/** @type {unknown} */ (self));

scope.addEventListener('message', async (event) => {
  const { program, engine: engineUrl, wasm } = /** @type {MessageEvent} */ (event).data;
  try {
    if (!engine) {
      const module = await import(engineUrl);
      const bytes = new Uint8Array(await (await fetch(wasm)).arrayBuffer());
      engine = /** @type {Engine} */ (await module.createEngine(bytes));
    }
    const running = engine;
    const started = performance.now();
    // The budget counts interpreter steps, so a loop that never ends stops with a message.
    const result = running.play({ 'main.jai': program }, 'main.jai', { budget: 200_000_000 });
    scope.postMessage({ result, ms: performance.now() - started });
  } catch (error) {
    const failure = /** @type {{ compilerCrashed?: boolean, message?: string } | null} */ (error);
    const crashed = Boolean(failure && failure.compilerCrashed);
    if (crashed) engine = null;
    scope.postMessage({ error: String((failure && failure.message) || error), crashed });
  }
});
