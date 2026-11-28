// @ts-check
// Compiles and runs programs with the browser build of the compiler. It lives in a worker so a
// program that never ends can be stopped by ending the worker, and the page stays responsive.
//
// A request is `{ id, engine, wasm }` plus one of:
//   `program`: the text of one example, run as `main.jai`;
//   `files`, `main`, `play`: a workspace, the file to run and `{ budget, args, stdin }`;
//   `lsp`: one language-server message (or a list of them), answered as `{ id, lsp }` with the
//          messages the server sent back;
//   `warm`: only load the compiler.
// `assets` (a URL) is where the program's `load_asset` calls fetch from. `modules` (a URL) is a
// JSON map of the library's files, supplied beside every program so it can import them.
// The reply is `{ id, result, ms }`, `{ id, ready }` for a warm-up, or `{ id, error, crashed }`.

/**
 * The browser build of the compiler, which is loaded from a URL the page names.
 * @typedef {{ play: (files: Record<string, string>, main: string, options: { budget: number, args?: string[], stdin?: string }) => unknown }} Engine
 */

/** @type {Engine | null} */
let engine = null;
/** The library files last fetched, by URL. @type {{ url: string, files: Record<string, string> } | null} */
let library = null;
/** What the loaded engine was made for, so a request for other assets gets its own. */
let loaded = '';

const scope = /** @type {DedicatedWorkerGlobalScope} */ (/** @type {unknown} */ (self));

scope.addEventListener('message', async (event) => {
  const request = /** @type {MessageEvent} */ (event).data;
  const { id, engine: engineUrl, wasm, assets } = request;
  try {
    const key = engineUrl + '\n' + (assets || '');
    if (!engine || loaded !== key) {
      const module = await import(engineUrl);
      const bytes = new Uint8Array(await (await fetch(wasm)).arrayBuffer());
      engine = /** @type {Engine} */ (
        await module.createEngine(bytes, assets ? { assets: { base: assets } } : {})
      );
      loaded = key;
    }
    if (request.warm) {
      scope.postMessage({ id, ready: true });
      return;
    }
    if (request.lsp !== undefined) {
      const server = /** @type {{ lsp?: (message: unknown) => unknown[] }} */ (engine);
      if (!server.lsp) throw new Error('This compiler build has no language server.');
      /** @type {unknown[]} */
      const messages = [].concat(request.lsp);
      scope.postMessage({
        id,
        lsp: messages.flatMap((m) => /** @type {unknown[]} */ (server.lsp?.(m))),
      });
      return;
    }
    const running = engine;
    const started = performance.now();
    // The budget counts interpreter steps, so a loop that never ends stops with a message.
    if (request.modules && library?.url !== request.modules) {
      const response = await fetch(request.modules);
      if (!response.ok) throw new Error('The library modules could not be loaded.');
      library = { url: request.modules, files: await response.json() };
    }
    const files = {
      ...(request.modules ? library?.files : {}),
      ...(request.files || { 'main.jai': request.program }),
    };
    const play = { budget: 200_000_000, ...request.play };
    const result = running.play(files, request.main || 'main.jai', play);
    scope.postMessage({ id, result, ms: performance.now() - started });
  } catch (error) {
    const failure = /** @type {{ compilerCrashed?: boolean, message?: string } | null} */ (error);
    const crashed = Boolean(failure && failure.compilerCrashed);
    if (crashed) engine = null;
    scope.postMessage({ id, error: String((failure && failure.message) || error), crashed });
  }
});
