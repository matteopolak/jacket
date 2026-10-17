// Instantiates exports.wasm with host functions of our own and calls its exports.
// Usage: node exports.mjs exports.wasm
import { readFileSync } from 'node:fs';

const bytes = readFileSync(process.argv[2]);
const module = await WebAssembly.compile(bytes);
let memory;
const cString = (address) => {
  const view = new Uint8Array(memory.buffer);
  let end = Number(address);
  while (view[end]) end++;
  return new TextDecoder().decode(view.subarray(Number(address), end));
};
const unexpected = (name) => () => {
  throw new Error(`unexpected call to ${name}`);
};
const env = {
  host_report: (label, value) => console.log(`report ${cString(label)} ${value}`),
};
const imports = { env, host_graphics: { draw: (x, y) => x + y } };
// Runtime_Support's own imports (allocator, output) are not used by these exports.
for (const imp of WebAssembly.Module.imports(module)) {
  if (imp.module === 'env' && !(imp.name in env)) env[imp.name] = unexpected(imp.name);
}
const { exports } = await WebAssembly.instantiate(module, imports);
memory = exports.memory;
console.log(`imports ${WebAssembly.Module.imports(module).map((i) => `${i.module}.${i.name}`).sort().join(' ')}`);
console.log(`triangle ${exports.triangle(100n)}`);
console.log(`count_bits ${exports.count_bits(0xffn)}`);
