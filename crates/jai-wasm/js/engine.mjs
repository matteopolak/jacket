// Host glue for jai_wasm.wasm, shipped in the browser bundle. Browsers, the hosted playground and the Node checks instantiate the exact same Rust compiler (jaic, interpreter backend).
//
// `host` (optional) offers the program procedures the sandbox does not implement:
// `{ functions: { name(args, memory) }, output(text, stream) }`, as webgpu_host.mjs builds them.
// `args` are the call's 64-bit slots as BigInts (pointers are addresses in `memory.buffer`); a
// function returns its result (a number, BigInt or boolean) or a promise of it. A promise
// suspends the module through JSPI, so such programs must run with `playAsync`.
export async function createEngine(wasmBytes, { host } = {}) {
  const module = await WebAssembly.compile(wasmBytes);
  const unknown = WebAssembly.Module.imports(module).filter(i => i.module !== "jai_host");
  if (unknown.length !== 0) throw new Error("This runtime build unexpectedly requires host imports.");
  const jspi = typeof WebAssembly.Suspending === "function" && typeof WebAssembly.promising === "function";
  let instance;
  let pending = null;
  let lastError = "";
  const memory = {
    get buffer() { return instance.exports.memory.buffer; },
    alloc: size => instance.exports.jai_host_alloc(size),
    free: (ptr, size) => instance.exports.jai_host_free(ptr, size),
  };
  const utf8 = new TextDecoder();
  function store(results, count, value) {
    if (count === 0 || value === undefined || value === null) return;
    let bits;
    if (typeof value === "bigint") bits = BigInt.asUintN(64, value);
    else if (typeof value === "boolean") bits = value ? 1n : 0n;
    else if (Number.isInteger(value)) bits = BigInt.asUintN(64, BigInt(value));
    else throw new TypeError(`host function returned ${value}`);
    new BigUint64Array(memory.buffer, results, count)[0] = bits;
  }
  function fail(error) { lastError = String(error?.message ?? error); return 2; }
  const imports = { jai_host: {
    call(name, nameLength, args, count, results, resultCount) {
      const functions = host?.functions;
      if (!functions) return 1;
      const key = utf8.decode(new Uint8Array(memory.buffer, name, nameLength));
      if (!Object.hasOwn(functions, key)) return 1;
      try {
        const value = functions[key](Array.from(new BigUint64Array(memory.buffer, args, count)), memory);
        if (value && typeof value.then === "function") { pending = value; return 3; }
        store(results, resultCount, value);
        return 0;
      } catch (error) { return fail(error); }
    },
    wait: jspi
      ? new WebAssembly.Suspending(async (results, resultCount) => {
          const promise = pending;
          pending = null;
          try { store(results, resultCount, await promise); return 0; } catch (error) { return fail(error); }
        })
      : () => fail("waiting for the page needs JavaScript Promise Integration (WebAssembly.Suspending)"),
    error(buffer, capacity) {
      const bytes = new TextEncoder().encode(lastError).subarray(0, capacity);
      new Uint8Array(memory.buffer, buffer, bytes.length).set(bytes);
      return bytes.length;
    },
    output(data, length, toStderr) {
      host?.output?.(utf8.decode(new Uint8Array(memory.buffer, data, length)), toStderr ? "stderr" : "stdout");
    },
    now_ms: () => performance.now(),
  } };
  instance = await WebAssembly.instantiate(module, imports);
  const api = instance.exports;
  const runAsync = jspi ? WebAssembly.promising(api.jai_play_run) : null;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  const lspExports = ["reset", "begin", "push", "dispatch", "output_len", "output_byte", "diagnostic_len", "diagnostic_byte"];
  const supportsLsp = lspExports.every(name => typeof api[`jai_lsp_${name}`] === "function");
  function readLanguage(kind, limit) {
    const length = api[`jai_lsp_${kind}_len`]();
    if (!Number.isInteger(length) || length < 0 || length > limit) throw new Error("Language response byte limit exceeded.");
    const bytes = new Uint8Array(length);
    for (let i = 0; i < length; i++) { const byte = api[`jai_lsp_${kind}_byte`](i); if (byte < 0 || byte > 255) throw new Error("Invalid language response byte."); bytes[i] = byte; }
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  }
  function languageCheck(status) { if (status !== 0) throw new Error(readLanguage("diagnostic", 2 * 1024 * 1024) || "Language boundary rejected the request."); }
  if (supportsLsp) languageCheck(api.jai_lsp_reset());
  if (!["reset", "push", "finish_file", "run", "output_len", "output_byte", "error_len", "error_byte"].every(name => typeof api[`jai_play_${name}`] === "function")) throw new Error("Compiler module is missing the playground bridge.");
  function playRead(kind) {
    const length = api[`jai_play_${kind}_len`]();
    const bytes = new Uint8Array(length);
    for (let i = 0; i < length; i++) bytes[i] = api[`jai_play_${kind}_byte`](i);
    return decoder.decode(bytes);
  }
  function playCheck(status) { if (status !== 0) throw new Error(playRead("error") || "Playground boundary rejected the request."); }
  function panicMessage() {
    try {
      const length = api.jai_play_panic_len();
      const bytes = new Uint8Array(Math.min(length, 4096));
      for (let i = 0; i < bytes.length; i++) bytes[i] = api.jai_play_panic_byte(i);
      return decoder.decode(bytes);
    } catch { return ""; }
  }
  function playPush(channel, text) { for (const byte of encoder.encode(text)) playCheck(api.jai_play_push(channel, byte)); }
  function prepare(files, main, { budget } = {}) {
      if (typeof main !== "string" || !files || typeof files !== "object") throw new TypeError("play needs a file map and a main path.");
      if (budget !== undefined && (!Number.isSafeInteger(budget) || budget <= 0)) throw new RangeError("budget must be a positive integer.");
      if (typeof api.jai_play_set_budget === "function") playCheck(api.jai_play_set_budget(budget === undefined ? 0 : Math.min(0xffffffff, Math.ceil(budget / 1000))));
      else if (budget !== undefined) throw new Error("This compiler build does not support execution budgets.");
      playCheck(api.jai_play_reset());
      for (const [name, text] of Object.entries(files)) {
        if (typeof text !== "string") throw new TypeError("Every supplied source file must be text.");
        playPush(0, name); playPush(1, text); playCheck(api.jai_play_finish_file());
      }
      playPush(2, main);
  }
  function crashed(error) {
    if (!(error instanceof WebAssembly.RuntimeError)) return error;
    const crash = new Error(`The compiler crashed: ${panicMessage() || error.message}`);
    crash.compilerCrashed = true; // the instance is unusable; callers must create a new engine
    return crash;
  }
  return {
    // Result: { exitCode, stdout, stderr, output: [{ stream: "stdout" | "stderr", text }], rendered, diagnostics }.
    // `budget` bounds the interpreter (basic blocks, rounded up to thousands); a runaway program then
    // fails with "execution budget exhausted" instead of hanging the worker.
    play(files, main, options = {}) {
      prepare(files, main, options);
      try { playCheck(api.jai_play_run()); } catch (error) { throw crashed(error); }
      return JSON.parse(playRead("output"));
    },
    // Like `play`, for programs that wait for the page (WebGPU, animation frames): resolves when
    // the program ends. Needs JSPI; without it this runs synchronously and such waits fail.
    async playAsync(files, main, options = {}) {
      prepare(files, main, options);
      try { playCheck(runAsync ? await runAsync() : api.jai_play_run()); } catch (error) { throw crashed(error); }
      return JSON.parse(playRead("output"));
    },
    jspi,
    ...(supportsLsp ? { lsp(message) {
      const text = JSON.stringify(message);
      if (typeof text !== "string" || text.length > 1024 * 1024) throw new Error("Language message byte limit exceeded.");
      const bytes = encoder.encode(text); if (bytes.length > 1024 * 1024) throw new Error("Language message byte limit exceeded.");
      languageCheck(api.jai_lsp_begin()); for (const byte of bytes) languageCheck(api.jai_lsp_push(byte)); languageCheck(api.jai_lsp_dispatch());
      const result = JSON.parse(readLanguage("output", 2 * 1024 * 1024)); if (!Array.isArray(result)) throw new Error("Invalid language response envelope."); return result;
    } } : {})
  };
}
