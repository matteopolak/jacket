// Run host-constant assertions in the actual wasm32 compiler, not native emulation.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";

const path = process.argv[2];
if (!path) throw new Error("usage: node tools/check_host_target_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(path));
const source = `
#assert BUILD_OS == .WASM && BUILD_CPU == .CUSTOM;
#assert OS == BUILD_OS && CPU == BUILD_CPU;
#assert type_of(OS) == type_of(BUILD_OS);
#assert type_of(CPU) == type_of(BUILD_CPU);
main :: () -> int {
    if BUILD_OS != .WASM || BUILD_CPU != .CUSTOM return 1;
    return 42;
}
`;
for (let i = 0; i < 2; i++) {
  const result = engine.play({ "main.jai": source }, "main.jai");
  assert.deepEqual(result.diagnostics, [], result.rendered);
  assert.equal(result.exitCode, 42, result.rendered);
}
const crossSource = `
C :: #import "Compiler";
#run {
    workspace := C.compiler_create_workspace("native target from wasm compiler");
    options := C.get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    options.os_target = .LINUX;
    options.cpu_target = .X64;
    options.runtime_support_definitions = .OMIT;
    C.set_build_options(options, workspace);
    C.add_build_string("#assert BUILD_OS == .WASM && BUILD_CPU == .CUSTOM; #assert OS == .LINUX && CPU == .X64; #assert OS != BUILD_OS && CPU != BUILD_CPU; main :: () {}", workspace);
}
main :: () -> int { return 42; }
`;
const cross = engine.play({ "main.jai": crossSource }, "main.jai");
assert.deepEqual(cross.diagnostics, [], cross.rendered);
assert.equal(cross.exitCode, 42, cross.rendered);
const invalidChild = engine.play({
  "main.jai": crossSource.replace("#assert BUILD_OS == .WASM", "#assert BUILD_OS == .MACOS"),
}, "main.jai");
assert.equal(invalidChild.exitCode, null);
assert(invalidChild.diagnostics.some(d => d.message.includes("workspace")), invalidChild.rendered);
assert(invalidChild.stderr.includes("#assert failed"), invalidChild.stderr);
const invalid = engine.play({ "main.jai": "#assert BUILD_OS == .MACOS; main :: () {}" }, "main.jai");
assert.equal(invalid.exitCode, null);
assert(invalid.diagnostics.some(d => d.message.includes("#assert failed")), invalid.rendered);
console.log("PASS: wasm32 compiler host constants, cross-target workspace, repeated execution and invalid assertion diagnostic");
