// Host glue for jai_wasm.wasm, shipped in the browser bundle. Browsers, the hosted playground and the Node checks instantiate the exact same Rust compiler (jaic, interpreter backend).
//
// `host` (optional) offers the program procedures the sandbox does not implement:
// `{ functions: { name(args, memory) }, output(text, stream) }`, as webgpu_host.mjs builds them.
// `args` are the call's 64-bit slots as BigInts (pointers are addresses in `memory.buffer`); a
// function returns its result (a number, BigInt or boolean) or a promise of it. A promise
// suspends the module through JSPI, so such programs must run with `playAsync`.
// The engine itself answers `getentropy` from the JavaScript runtime's secure random source
// (`crypto.getRandomValues`); a host function of the same name takes its place.
//
// `assets` (optional) lets Extensions/Assets fetch files: `{ base, limit }`. `load_asset(path)`
// fetches `path` resolved against the `base` URL (a directory), with no credentials and no
// redirects, refusing paths that leave it and responses over `limit` bytes (16 MiB by default).
// The engine also answers `jai_host_provides` for every function it and `host` offer.
export async function createEngine(wasmBytes, { host, assets } = {}) {
  const module = await WebAssembly.compile(wasmBytes);
  const unknown = WebAssembly.Module.imports(module).filter(i => i.module !== "jai_host");
  if (unknown.length !== 0) throw new Error("This runtime build unexpectedly requires host imports.");
  const jspi = typeof WebAssembly.Suspending === "function" && typeof WebAssembly.promising === "function";
  let instance;
  let pending = null;
  let lastError = "";
  // Addresses cross the imports and exports as wasm32 `usize` values, which JavaScript sees as
  // signed i32 numbers (a memory64 build would pass BigInts): `address` makes them the unsigned
  // byte offsets they are, and `view` checks a range against the memory before touching it.
  const address = value => (typeof value === "bigint" ? Number(BigInt.asUintN(64, value)) : value >>> 0);
  function view(Type, at, count, what) {
    const start = address(at);
    const bytes = count * Type.BYTES_PER_ELEMENT;
    const size = instance.exports.memory.buffer.byteLength;
    if (start + bytes > size) throw new RangeError(`${what} at ${start} (+${bytes} bytes) is outside the module's memory (${size} bytes)`);
    return new Type(instance.exports.memory.buffer, start, count);
  }
  const memory = {
    get buffer() { return instance.exports.memory.buffer; },
    alloc: size => address(instance.exports.jai_host_alloc(size)),
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
    view(BigUint64Array, results, count, "results")[0] = bits;
  }
  function fail(error) { lastError = String(error?.message ?? error); return 2; }
  // `getentropy` fills at most 256 bytes and answers 0, or -1 when it cannot.
  const builtins = {
    getentropy([buffer, length]) {
      const count = address(length);
      if (count > 256) return -1;
      globalThis.crypto.getRandomValues(view(Uint8Array, buffer, count, "entropy buffer"));
      return 0;
    },
  };
  if (assets?.base !== undefined) Object.assign(builtins, assetFunctions(assets, view, address));
  const imports = { jai_host: {
    call(name, nameLength, args, count, results, resultCount) {
      const functions = { ...builtins, ...host?.functions };
      functions.jai_host_provides = ([at, length]) => Object.hasOwn(functions, utf8.decode(view(Uint8Array, at, address(length), "name")));
      try {
        const key = utf8.decode(view(Uint8Array, name, address(nameLength), "name"));
        if (!Object.hasOwn(functions, key)) return 1;
        const value = functions[key](Array.from(view(BigUint64Array, args, address(count), "arguments")), memory);
        if (value && typeof value.then === "function") { pending = value; return 3; }
        store(results, address(resultCount), value);
        return 0;
      } catch (error) { return fail(error); }
    },
    wait: jspi
      ? new WebAssembly.Suspending(async (results, resultCount) => {
          const promise = pending;
          pending = null;
          try { store(results, address(resultCount), await promise); return 0; } catch (error) { return fail(error); }
        })
      : () => fail("waiting for the page needs JavaScript Promise Integration (WebAssembly.Suspending)"),
    error(buffer, capacity) {
      const bytes = new TextEncoder().encode(lastError).subarray(0, address(capacity));
      view(Uint8Array, buffer, bytes.length, "error buffer").set(bytes);
      return bytes.length;
    },
    output(data, length, toStderr) {
      host?.output?.(utf8.decode(view(Uint8Array, data, address(length), "output")), toStderr ? "stderr" : "stdout");
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
  // `data` is text (sent as UTF-8) or bytes.
  function playPush(channel, data) { for (const byte of typeof data === "string" ? encoder.encode(data) : data) playCheck(api.jai_play_push(channel, byte)); }
  function prepare(files, main, { budget, args, stdin, libraryRoot } = {}) {
      if (typeof main !== "string" || !files || typeof files !== "object") throw new TypeError("play needs a file map and a main path.");
      if (budget !== undefined && (!Number.isSafeInteger(budget) || budget <= 0)) throw new RangeError("budget must be a positive integer.");
      if (typeof api.jai_play_set_budget === "function") playCheck(api.jai_play_set_budget(budget === undefined ? 0 : Math.min(0xffffffff, Math.ceil(budget / 1000))));
      else if (budget !== undefined) throw new Error("This compiler build does not support execution budgets.");
      playCheck(api.jai_play_reset());
      for (const [name, contents] of Object.entries(files)) {
        if (typeof contents !== "string" && !(contents instanceof Uint8Array)) throw new TypeError("Every supplied file must be text or a Uint8Array.");
        playPush(0, name); playPush(1, contents); playCheck(api.jai_play_finish_file());
      }
      playPush(2, main);
      if (libraryRoot !== undefined) {
        if (typeof libraryRoot !== "string" || !libraryRoot.length) throw new TypeError("libraryRoot must be a nonempty relative directory.");
        if (typeof api.jai_play_accepts_library_root !== "function" || api.jai_play_accepts_library_root() !== 1) throw new Error("This compiler build does not support a supplied library root.");
        playPush(5, libraryRoot);
      }
      // `args`: the program's argv (name first). `stdin`: its whole input; leave it out and the
      // page's `jai_stdin_read` host function supplies input as the program reads (playAsync).
      if (args !== undefined) for (const arg of args) { if (typeof arg !== "string" || arg.includes("\0")) throw new TypeError("Every argument must be text without NUL."); playPush(3, arg); playPush(3, "\0"); }
      if (stdin !== undefined) playPush(4, String(stdin));
  }
  function crashed(error) {
    if (!(error instanceof WebAssembly.RuntimeError)) return error;
    const crash = new Error(`The compiler crashed: ${panicMessage() || error.message}`);
    crash.compilerCrashed = true; // the instance is unusable; callers must create a new engine
    return crash;
  }
  return {
    // `files` maps each workspace path to its contents, as text or a Uint8Array of bytes.
    // Result: { exitCode, stdout, stderr, output: [{ stream: "stdout" | "stderr", text }], rendered, diagnostics }.
    // `budget` bounds the interpreter (basic blocks, rounded up to thousands); a runaway program then
    // fails with "execution budget exhausted" instead of hanging the worker. `args` is the program's
    // argv and `stdin` its whole input (see `prepare`).
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

// Extensions/Assets in the browser: `jai_asset_fetch` resolves to a handle (0 when the path is
// refused or the fetch fails), `jai_asset_size` and `jai_asset_take` hand its bytes over once.
function assetFunctions({ base, limit = 16 * 1024 * 1024 }, view, address) {
  const root = new URL(base, globalThis.location?.href);
  if (!root.pathname.endsWith("/")) root.pathname += "/";
  if (!Number.isSafeInteger(limit) || limit < 0) throw new RangeError("assets.limit must be a byte count.");
  const decoder = new TextDecoder("utf-8", { fatal: true });
  const held = new Map();
  let next = 1;
  // The same rule as asset_path_is_relative in stdlib/Extensions/Assets/module.jai.
  const relative = path => path.length > 0 && !/[\\:\0]/.test(path) && path.split("/").every(part => part !== "" && part !== "..");
  async function fetchAsset(path) {
    if (!relative(path)) return 0;
    const url = new URL(path, root);
    if (url.origin !== root.origin || !url.pathname.startsWith(root.pathname)) return 0;
    try {
      const response = await fetch(url, { credentials: "omit", redirect: "error" });
      if (!response.ok) return 0;
      const declared = Number(response.headers.get("content-length") ?? 0);
      if (declared > limit) return 0;
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (bytes.length > limit) return 0;
      held.set(next, bytes);
      return next++;
    } catch { return 0; }
  }
  return {
    jai_asset_fetch([at, length]) {
      let path;
      try { path = decoder.decode(view(Uint8Array, at, address(length), "asset path")); } catch { return 0; }
      return fetchAsset(path);
    },
    jai_asset_size: ([handle]) => held.get(Number(handle))?.length ?? 0,
    jai_asset_take([handle, at, length]) {
      const bytes = held.get(Number(handle));
      if (!bytes || address(length) !== bytes.length) return 0;
      held.delete(Number(handle));
      view(Uint8Array, at, bytes.length, "asset buffer").set(bytes);
      return 1;
    },
  };
}
