#!/usr/bin/env node
// Execute the actual wasm module; a native-only test cannot satisfy this gate.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../web/scripting-runtime/engine.mjs";
const path = process.argv[2];
if (!path) throw new Error("usage: node tools/check_scripting_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(path));
const fixtures = [
  ["integer entry", "main :: () -> int { return 42; }", {}, 42n],
  ["runtime phase and compile-time source run", "seed :: #run answer(); answer :: () -> int { if #compile_time return 40; return 900; } main :: () -> int { if #compile_time return 700; return seed + 2; }", {}, 42n],
  ["wasm32 layout", "main :: () -> int { return size_of(*int) + 38; }", {}, 42n],
  ["arguments", 'main :: (args: []string) -> int { if args.count != 2 return 1; if args[0] != "two words" return 2; if args[1] != "" return 3; return 42; }', { arguments: ["two words", ""] }, 42n],
  ["source bundle", '#load "helper.jai"; main :: () -> int { return answer; }', { files: { "helper.jai": "answer :: 42;" } }, 42n],
  ["isolated globals", "counter: int = 41; main :: () -> int { counter += 1; return counter; }", {}, 42n],
  ["void entry", "main :: () {}", {}, 0n],
];
for (const [name, source, options, expected] of fixtures) {
  assert.equal(engine.run(source, options).exitCode, expected, name);
  assert.equal(engine.run(source, options).exitCode, expected, `${name}, repeat run`);
}
assert.throws(() => engine.run("main :: () { while true {} }", { fuel: 100 }), /Fuel/);
assert.throws(() => engine.run("main :: () -> int { return missing; }"), /missing/);
assert.throws(() => engine.run("main :: () {}", { arguments: ["extra"] }), /no arguments/);
console.log(`PASS: real WebAssembly interpreter (${fixtures.length} fixtures, repeated runs, limits and diagnostics)`);
