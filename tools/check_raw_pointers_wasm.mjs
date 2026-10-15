// Actual wasm32 compiler host must preserve its language target's 64-bit pointer values.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";

const binary = process.argv[2];
if (!binary) throw new Error("usage: node tools/check_raw_pointers_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(binary));
const fixture = await readFile(new URL("../tests/compiler-semantics/raw-pointer-constants.jai", import.meta.url), "utf8");
for (let i=0; i<2; ++i) {
    const result=engine.play({"main.jai": fixture+`
        HIGH :: cast(*void) 0x1234567880000001;
        #assert (cast(u64) HIGH)==0x1234567880000001;
        #assert size_of(HKEY)==8;
    `},"main.jai");
    assert.deepEqual(result.diagnostics,[],result.rendered);
    assert.equal(result.exitCode,0,result.rendered);
}
for (const [source, message] of [
    ["HANDLE :: cast(*void)0x80000001; BYTE :: cast(u8)HANDLE; main :: () {}", "out of range"],
    ["read :: () -> u32 {pointer:=cast(*u32)1; return pointer.*;} #run read(); main :: () {}", "null pointer"],
    ["make :: () -> string {value:string;value.count=4;value.data=cast(*u8)0x80000001;return value;} BAD :: #run make(); main :: () {}", "cannot be read"],
]) {
    const result=engine.play({"main.jai":source},"main.jai");
    assert.equal(result.exitCode,null);
    assert(result.diagnostics.some(d=>d.message.includes(message)),result.rendered);
}
console.log("PASS: actual wasm32 compiler raw 64-bit pointer constants, globals/defaults/relocations, repeated execution and invalid-access diagnostics");
