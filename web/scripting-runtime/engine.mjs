// The browser and Node verification harness instantiate the exact same Rust VM.
export async function createEngine(wasmBytes) {
  const module = await WebAssembly.compile(wasmBytes);
  if (WebAssembly.Module.imports(module).length !== 0) {
    throw new Error("This runtime build unexpectedly requires host imports.");
  }
  const instance = await WebAssembly.instantiate(module, {});
  const api = instance.exports;
  const required = ["jai_script_reset", "jai_script_push", "jai_script_finish_source", "jai_script_finish_argument", "jai_script_run", "jai_script_has_result", "jai_script_exit_code", "jai_script_steps", "jai_script_diagnostic_len", "jai_script_diagnostic_byte"];
  if (required.some(name => typeof api[name] !== "function")) throw new Error("Compiler module is missing the runtime bridge.");
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  function diagnostic() {
    const bytes = new Uint8Array(api.jai_script_diagnostic_len());
    for (let i = 0; i < bytes.length; i++) bytes[i] = api.jai_script_diagnostic_byte(i);
    return decoder.decode(bytes);
  }
  function check(status) {
    if (status !== 0) throw new Error(diagnostic() || "Runtime boundary rejected the request.");
  }
  function push(channel, text) {
    for (const byte of encoder.encode(text)) check(api.jai_script_push(channel, byte));
  }
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
  const supportsPlay = ["reset", "push", "finish_file", "run", "output_len", "output_byte", "error_len", "error_byte"].every(name => typeof api[`jai_play_${name}`] === "function");
  function playRead(kind) {
    const length = api[`jai_play_${kind}_len`]();
    const bytes = new Uint8Array(length);
    for (let i = 0; i < length; i++) bytes[i] = api[`jai_play_${kind}_byte`](i);
    return decoder.decode(bytes);
  }
  function playCheck(status) { if (status !== 0) throw new Error(playRead("error") || "Playground boundary rejected the request."); }
  function playPush(channel, text) { for (const byte of encoder.encode(text)) playCheck(api.jai_play_push(channel, byte)); }
  return {
    ...(supportsPlay ? { play(files, main) {
      if (typeof main !== "string" || !files || typeof files !== "object") throw new TypeError("play needs a file map and a main path.");
      playCheck(api.jai_play_reset());
      for (const [name, text] of Object.entries(files)) {
        if (typeof text !== "string") throw new TypeError("Every supplied source file must be text.");
        playPush(0, name); playPush(1, text); playCheck(api.jai_play_finish_file());
      }
      playPush(2, main);
      playCheck(api.jai_play_run());
      return JSON.parse(playRead("output"));
    } } : {}),
    ...(supportsLsp ? { lsp(message) {
      const text = JSON.stringify(message);
      if (typeof text !== "string" || text.length > 1024 * 1024) throw new Error("Language message byte limit exceeded.");
      const bytes = encoder.encode(text); if (bytes.length > 1024 * 1024) throw new Error("Language message byte limit exceeded.");
      languageCheck(api.jai_lsp_begin()); for (const byte of bytes) languageCheck(api.jai_lsp_push(byte)); languageCheck(api.jai_lsp_dispatch());
      const result = JSON.parse(readLanguage("output", 2 * 1024 * 1024)); if (!Array.isArray(result)) throw new Error("Invalid language response envelope."); return result;
    } } : {}),
    run(source, { arguments: args = [], files = {}, fuel = 1_000_000 } = {}) {
      if (typeof source !== "string" || !Array.isArray(args) || args.some(arg => typeof arg !== "string")) {
        throw new TypeError("Source must be text and arguments must be an array of strings.");
      }
      if (!Number.isInteger(fuel) || fuel < 0 || fuel > 0xffffffff) {
        throw new RangeError("Fuel must be an unsigned 32-bit integer.");
      }
      check(api.jai_script_reset());
      for (const [name, text] of Object.entries({ ...files, "main.jai": source })) {
        if (typeof text !== "string") throw new TypeError("Every supplied source file must be text.");
        push(2, name);
        push(0, text);
        check(api.jai_script_finish_source());
      }
      for (const arg of args) {
        push(1, arg);
        check(api.jai_script_finish_argument());
      }
      check(api.jai_script_run(fuel));
      if (api.jai_script_has_result() !== 1) throw new Error("Runtime produced no result.");
      return { exitCode: api.jai_script_exit_code(), steps: api.jai_script_steps() };
    },
  };
}
