// Actual wasm32 compiler/private ABI checks. Controlled host objects verify the
// protocol; this does not claim hardware GPU rendering or replace the full tour.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";
import { createWebGPUHost } from "../crates/jai-wasm/js/webgpu_host.mjs";

const binary = process.argv[2];
if (!binary) throw new Error("usage: node tools/check_tour_canvas_wasm.mjs <jai_wasm.wasm>");
const bytes = await readFile(binary);
const provider = await readFile(new URL("../examples/tour/gpu/browser_canvas.jai", import.meta.url), "utf8");
const probe = '#if OS == .WASM { #load "browser_canvas.jai"; } else { #import "Input"; #import "Window_Type"; } main :: () -> int { return cast(int) tour_canvas_available(); }';
for (const [name, gpu, canvas, expected] of [
  ["no GPU", null, null, 0],
  ["headless service", { requestAdapter: async () => null }, null, 0],
  ["unavailable canvas context", { requestAdapter: async () => null }, { getContext: () => null }, 0],
  ["provided canvas context", { requestAdapter: async () => null }, { getContext: () => ({}) }, 1],
]) {
  const engine = await createEngine(bytes, { host: createWebGPUHost({ gpu, canvas }) });
  const result = engine.play({ "main.jai": probe, "browser_canvas.jai": provider }, "main.jai");
  assert.deepEqual(result.diagnostics, [], name + ": " + result.rendered);
  assert.equal(result.exitCode, expected, name + ": " + result.rendered);
}

for (const escape of [false, true]) {
  const host = createWebGPUHost({ gpu: { requestAdapter: async () => null }, canvas: { getContext: () => ({}) } });
  host.resize(800, 600);
  host.resize(1024, 768);
  for (const key of [32, 32, 32, 168, 168]) host.input({ type: 1, key, pressed: true });
  host.input({ type: 1, key: 32, pressed: false });
  if (escape) host.input({ type: 1, key: 27, pressed: true });
  const engine = await createEngine(bytes, { host });
  const source = `#import "Basic";
    #load "browser_canvas.jai";
    main :: () {
        assert(tour_canvas_available());
        before := tour_canvas_seconds();
        after := tour_canvas_seconds();
        assert(after >= before);
        width, height := tour_canvas_dimensions();
        assert(width == 1024 && height == 768);
        resizes, quit, pause, clicks := tour_canvas_input();
        assert(resizes == 2 && quit == ${escape} && pause && clicks == 2);
        again, quit_again, pause_again, clicks_again := tour_canvas_input();
        assert(again == 0 && !quit_again && !pause_again && clicks_again == 0);
        print("ok\\n");
    }`;
  const result = engine.play({ "main.jai": source, "browser_canvas.jai": provider }, "main.jai");
  assert.deepEqual(result.diagnostics, [], result.rendered);
  assert.equal(result.exitCode, 0, result.rendered);
  assert.equal(result.stdout, "ok\n");
}

// No embedding provider must diagnose an unavailable private procedure, not
// fabricate a false value for an API that the host does not implement.
const plain = await createEngine(bytes);
const absent = plain.play({ "main.jai": probe, "browser_canvas.jai": provider }, "main.jai");
assert.equal(absent.exitCode, null);
assert(absent.diagnostics.some(d => d.code === "unavailable"), absent.rendered);
console.log("PASS: actual wasm32 canvas capability/32-byte events/dimensions/parity/draining and absent-provider diagnostics (protocol, no GPU rendering)");
