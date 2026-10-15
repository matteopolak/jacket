// Exercise actual wasm32 compiler message records, including canonical result types.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";
const binary = process.argv[2];
if (!binary) throw new Error("usage: node tools/check_typed_nodes_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(binary));
const source = await readFile(new URL("../tests/compiler-semantics/typed-expression-message.jai", import.meta.url), "utf8");
for (let i=0; i<2; i++) {
    const result = engine.play({"main.jai":source}, "main.jai");
    assert.deepEqual(result.diagnostics, [], result.rendered);
    assert.equal(result.exitCode, 0, result.rendered);
}
const invalid = engine.play({"main.jai": source.replace("node.type != type_info(u8)", "node.type != type_info(u16)")}, "main.jai");
assert.equal(invalid.exitCode, null);
assert(invalid.diagnostics.some(d => d.message.includes("Expected canonical u8")), invalid.rendered);
console.log("PASS: actual wasm32 compiler checked binary canonical result types, repeated execution");
