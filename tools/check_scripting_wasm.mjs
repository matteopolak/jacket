#!/usr/bin/env node
// Execute the actual wasm module; a native-only test cannot satisfy this gate.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";
const path = process.argv[2];
if (!path) throw new Error("usage: node tools/check_scripting_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(path));
const fixtures = [
  ["integer entry", { "main.jai": "main :: () -> int { return 42; }" }, 42],
  ["negative result", { "main.jai": "main :: () -> int { return -7; }" }, -7],
  ["runtime phase and compile-time source run", { "main.jai": "seed :: #run answer(); answer :: () -> int { if #compile_time return 40; return 900; } main :: () -> int { if #compile_time return 700; return seed + 2; }" }, 42],
  ["wasm target", { "main.jai": "main :: () -> int { if OS == .WASM return 42; return 1; }" }, 42],
  ["source bundle", { "main.jai": '#load "helper.jai"; main :: () -> int { return answer; }', "helper.jai": "answer :: 42;" }, 42],
  ["nested source import", { "main.jai": '#load "lib/one.jai"; main :: () -> int { return answer; }', "lib/one.jai": '#load "../two.jai"; answer :: result;', "two.jai": "result :: 42;" }, 42],
  ["isolated globals", { "main.jai": "counter: int = 41; main :: () -> int { counter += 1; return counter; }" }, 42],
  ["void entry", { "main.jai": "main :: () {}" }, 0],
  ["standard library output", { "main.jai": '#import "Basic"; main :: () { print("hi %\\n", 42); }' }, 0, "hi 42\n"],
];
for (const [name, files, expected, stdout] of fixtures) {
  for (const label of [name, `${name}, repeat run`]) {
    const result = engine.play(files, "main.jai");
    assert.deepEqual(result.diagnostics, [], label);
    assert.equal(result.exitCode, expected, label);
    if (stdout !== undefined) assert.equal(result.stdout, stdout, label);
  }
}
const missing = engine.play({ "main.jai": "main :: () -> int { return missing; }" }, "main.jai");
assert.equal(missing.exitCode, null);
assert(missing.diagnostics.some(item => /missing/.test(item.message)), "Unknown names are reported as diagnostics");
console.log(`PASS: real WebAssembly compiler (${fixtures.length} fixtures, repeated runs and diagnostics)`);
