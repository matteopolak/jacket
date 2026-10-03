// The browser and Node verification harness instantiate the exact same Rust VM.
export async function createEngine(wasmBytes) {
  const module = await WebAssembly.compile(wasmBytes);
  if (WebAssembly.Module.imports(module).length !== 0) {
    throw new Error("This runtime build unexpectedly requires host imports.");
  }
  const instance = await WebAssembly.instantiate(module, {});
  const api = instance.exports;
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
  return {
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
