// Actual wasm32 compiler: missing architectural capabilities must never fabricate results.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";

const binary = process.argv[2];
if (!binary) throw new Error("usage: node tools/check_machine_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(binary));
for (const body of [
    "hi:u32;lo:u32;#asm { rdtsc hi,lo; }",
    "hi:u32;lo:u32;aux:u32;#asm { rdtscp hi,lo,aux; }",
    "v:u64;#asm { rdrand v; }",
    "v:u64;#asm { rdseed v; }",
    "#asm { mfence; }",
    "#asm { lfence; }",
    "#asm { sfence; }",
    "#asm { pause; }",
    "v:u64;pointer:=*v;#asm { prefetcht0 [pointer]; }",
]) {
    const result = engine.play({ "main.jai": `main :: () { ${body} }` }, "main.jai");
    assert.equal(result.exitCode, null);
    assert(result.diagnostics.some(d => d.message.includes("x86-64 target")), result.rendered);
}
const counter = engine.play({
    "main.jai": "get_cpu_cycle_count :: () -> u64 #intrinsic; main :: () { value:=get_cpu_cycle_count(); }",
}, "main.jai");
assert.equal(counter.exitCode, null);
assert(counter.diagnostics.some(d => d.message.includes("no architectural cycle counter")), counter.rendered);

// Even an x64 output selection cannot make an actual wasm compiler execute x86.
const cross = engine.play({ "main.jai": `
C :: #import "Compiler";
#run {
    workspace := C.compiler_create_workspace("x64 instruction from wasm host");
    options := C.get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    options.os_target = .LINUX;
    options.cpu_target = .X64;
    options.runtime_support_definitions = .OMIT;
    C.set_build_options(options, workspace);
    C.add_build_string("probe :: () {hi:u32;lo:u32;#asm {rdtsc hi,lo;}} #run probe(); main :: () {}", workspace);
}
main :: () {}
` }, "main.jai");
assert.equal(cross.exitCode, null);
assert(cross.diagnostics.some(d => d.message.includes("workspace")), cross.rendered);
assert(cross.stderr.includes("x86-64 interpreter host"), cross.stderr);
const valid = engine.play({ "main.jai": "main :: () -> int { return 42; }" }, "main.jai");
assert.deepEqual(valid.diagnostics, [], valid.rendered);
assert.equal(valid.exitCode, 42);
console.log("PASS: actual wasm32 host/target ISA and counter capability diagnostics, x64 child failure and recovery");
