// Actual wasm32 compiler-host enum resolution and runtime checks.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";

const binary = process.argv[2];
if (!binary) throw new Error("usage: node tools/check_enum_dependencies_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(binary));
for (const [file, expected] of [
    ["../tests/compiler-semantics/enum-dependencies.jai", 0],
    ["../tests/producer-requests/forward-enum-dependencies.jai", 5],
]) {
    const source = await readFile(new URL(file, import.meta.url), "utf8");
    const result = engine.play({ "main.jai": source }, "main.jai");
    assert.deepEqual(result.diagnostics, [], result.rendered);
    assert.equal(result.exitCode, expected, result.rendered);
}
for (const [source, expected] of [
    ["E :: enum {A::B; B::A;}", "circular dependency"],
    ["E :: enum u8 {A::B; B::256;}", "out of range"],
    ["E :: enum {A::1; A::2;}", "declared more than once"],
]) {
    const result = engine.play({ "main.jai": `${source} main :: () {}` }, "main.jai");
    assert.equal(result.exitCode, null);
    assert(result.diagnostics.length > 0, result.rendered);
    assert(result.rendered.includes(expected), result.rendered);
}
console.log("wasm32 compiler enum dependency runtime and rejection checks passed");
