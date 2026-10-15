// Real wasm32 compiler-host source checks with no native foreign provider.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";

const binary = process.argv[2];
if (!binary) throw new Error("usage: node tools/check_storage_aliases_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(binary));
const files = {};
for (const name of ["Backing", "Facade", "Forward"]) {
    files[`modules/${name}/module.jai`] = await readFile(
        new URL(`../tests/compiler-semantics/storage-aliases/${name}/module.jai`, import.meta.url), "utf8");
}
files["main.jai"] = `
    F :: #import "Forward"; R :: #import "Backing";
    #assert type_of(F.cleanroom_alias_counter)==R.Handle && type_of(F.cleanroom_alias_pair)==R.Pair;
    #assert type_info(type_of(F.cleanroom_alias_counter))==type_info(R.Handle);
    #assert type_info(type_of(F.cleanroom_alias_counter)).name=="Handle";
    Probe :: struct(T:Type=type_of(F.cleanroom_alias_counter)) {item:T;}
    #assert Probe()==Probe(F.Handle);
    main :: ()->s32 {return 0;}
`;
const positive = engine.play(files, "main.jai");
assert.deepEqual(positive.diagnostics, [], positive.rendered);
assert.equal(positive.exitCode, 0, positive.rendered);
files["main.jai"] = 'F :: #import "Facade"; read :: ()->F.Handle {return F.cleanroom_alias_counter;} #run read(); main :: () {}';
const negative = engine.play(files, "main.jai");
assert.equal(negative.exitCode, null);
assert(negative.rendered.includes("cleanroom_alias_counter") && negative.rendered.includes("not available"), negative.rendered);
files["main.jai"] = 'original:s64=7; Saved :: original; main :: () {Saved=9;}';
const immutable = engine.play(files, "main.jai");
assert.equal(immutable.exitCode, null);
assert(immutable.rendered.includes("constant"), immutable.rendered);
console.log("wasm32 compiler reexport/type-query, immutable-binding and unavailable-read checks passed");
