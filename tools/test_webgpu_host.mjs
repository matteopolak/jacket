// The browser side of stdlib/WebGPU (crates/jai-wasm/js/webgpu_host.mjs) against a small mock of
// the WebGPU JavaScript API: argument marshalling, handles, strings, chained structs, sentinels,
// futures and callback records with their modes, mapping, and pointer checks. With a built
// browser bundle (JAI_WASM_DIR=<dir with jai_wasm.wasm>) and JSPI (Node 24 needs the V8 flag
// --experimental-wasm-jspi, which reaches the tests only without process isolation), it also runs
// tests/webgpu/bridge.jai in the real engine, where every request suspends the module until the
// mock's promise settles.
//
//   node --test tools/test_webgpu_host.mjs
//   JAI_WASM_DIR=artifacts/ci-wasm node --experimental-wasm-jspi tools/test_webgpu_host.mjs
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";
import { STRUCTS } from "../crates/jai-wasm/js/webgpu_bindings.generated.mjs";
import { createWebGPUHost } from "../crates/jai-wasm/js/webgpu_host.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const MAX64 = 0xFFFF_FFFF_FFFF_FFFFn;
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
// WGPUCallbackMode, and the `how` of jai_webgpu_poll.
const WAIT_ANY_ONLY = 1, ALLOW_PROCESS_EVENTS = 2, ALLOW_SPONTANEOUS = 3;
const PROCESS_EVENTS = 0n, WAIT_ANY = 1n, SPONTANEOUS = 2n;

// --- A mock of the WebGPU JavaScript API: records what it was given, settles requests later. ---
class GPUValidationError {
  constructor(message) { this.message = message; }
}
function mockGPU() {
  const calls = [];
  const errorScopes = [];
  class Buffer {
    constructor(desc) { Object.assign(this, desc); this.mapped = null; }
    mapAsync(mode, offset, size) {
      calls.push(["mapAsync", mode, offset, size]);
      return delay(20).then(() => {
        this.mapped = new ArrayBuffer(this.size);
        new Uint8Array(this.mapped).forEach((_, i, bytes) => { bytes[i] = i + 1; });
      });
    }
    getMappedRange() { return this.mapped; }
    unmap() { calls.push(["unmap", [...new Uint8Array(this.mapped)]]); this.mapped = null; }
  }
  const device = {
    label: "",
    queue: { label: "", submit: list => calls.push(["submit", list.length]), writeBuffer() {} },
    lost: new Promise(() => {}),
    features: new Set(),
    limits: { maxBindGroups: 4 },
    adapterInfo: { vendor: "mock", architecture: "test", device: "", description: "Mock adapter" },
    addEventListener: (type) => calls.push(["addEventListener", type]),
    createBuffer: desc => { calls.push(["createBuffer", desc]); return new Buffer(desc); },
    createShaderModule: desc => {
      calls.push(["createShaderModule", desc]);
      if (!desc.code.includes("@")) errorScopes.at(-1)?.push(new GPUValidationError("shader parse error: expected a declaration"));
      return { label: desc.label ?? "" };
    },
    createBindGroup: desc => { calls.push(["createBindGroup", desc]); return { label: "" }; },
    pushErrorScope: filter => { calls.push(["pushErrorScope", filter]); errorScopes.push([]); },
    popErrorScope: () => delay(5).then(() => errorScopes.pop()[0] ?? null),
  };
  const adapter = {
    info: { vendor: "mock", architecture: "test", device: "", description: "Mock adapter" },
    features: new Set(["timestamp-query"]),
    limits: { maxBindGroups: 4 },
    requestDevice: desc => { calls.push(["requestDevice", desc]); return delay(5).then(() => device); },
  };
  return {
    calls, adapter, device,
    requestAdapter: options => { calls.push(["requestAdapter", options]); return delay(5).then(() => adapter); },
    getPreferredCanvasFormat: () => "bgra8unorm",
    wgslLanguageFeatures: new Set(),
  };
}

// --- The program's memory: a plain ArrayBuffer with a bump allocator for "C" data. ---
function fakeMemory(size = 1 << 16) {
  const buffer = new ArrayBuffer(size);
  let top = 4096; // [0, 4096) belongs to the test's own structs
  const freed = [];
  return {
    buffer, freed,
    alloc(n) { const p = top; top += (n + 15) & ~15; return p; },
    free(p) { freed.push(p); },
  };
}
// Lays out C structs by the generated tables' offsets, so the test follows the bindings.
function writer(memory) {
  const d = new DataView(memory.buffer);
  let next = 64;
  const at = size => { const p = next; next += (size + 15) & ~15; return p; };
  return {
    at, d,
    bytes(text, terminate = false) {
      const encoded = new TextEncoder().encode(text);
      const p = at(encoded.length + 1);
      new Uint8Array(memory.buffer, p, encoded.length).set(encoded);
      if (terminate) d.setUint8(p + encoded.length, 0);
      return [p, encoded.length];
    },
    view(p, text, { nullTerminated = false } = {}) {
      const [data, length] = this.bytes(text, nullTerminated);
      d.setBigUint64(p, BigInt(data), true);
      d.setBigUint64(p + 8, nullTerminated ? MAX64 : BigInt(length), true);
    },
    member(struct, name) { return STRUCTS[struct].members.find(m => m[0] === name)[1]; },
    callbackInfo(mode, callback, userdata1 = 0n) {
      const p = at(40);
      d.setUint32(p + 8, mode, true);
      d.setBigUint64(p + 16, callback, true);
      d.setBigUint64(p + 24, userdata1, true);
      return p;
    },
  };
}

function setup() {
  const gpu = mockGPU();
  const memory = fakeMemory();
  const surfaces = [];
  const host = createWebGPUHost({ gpu, canvas: mockCanvas(), onSurface: s => surfaces.push(s) });
  const call = (name, ...args) => host.functions[name](args.map(a => BigInt(a)), memory);
  const w = writer(memory);
  return { gpu, memory, host, call, w, surfaces };
}
function mockCanvas() {
  return { width: 300, height: 150, getContext: () => ({ configure(c) { this.config = c; }, unconfigure() {} }) };
}
// A callback record (Webgpu_Callback_Record in stdlib/WebGPU/wasm.jai) as an object.
function readRecord(d, p) {
  return {
    kind: d.getUint32(p, true), mode: d.getUint32(p + 4, true), future: d.getBigUint64(p + 8, true),
    callback: d.getBigUint64(p + 16, true), userdata1: d.getBigUint64(p + 24, true),
    args: [0, 1, 2, 3].map(i => d.getBigUint64(p + 40 + i * 8, true)),
    message: [d.getBigUint64(p + 72, true), d.getBigUint64(p + 80, true)],
  };
}
async function requestAdapter({ call, w }, mode = ALLOW_PROCESS_EVENTS) {
  const instance = call("wgpuCreateInstance", 0);
  const future = w.at(8);
  call("wgpuInstanceRequestAdapter", instance, 0, w.callbackInfo(mode, 0xC0DEn, 0x55n), future);
  return { instance, future: w.d.getBigUint64(future, true) };
}

test("a request becomes a future whose callback record WaitAny runs", async () => {
  const s = setup();
  const { future } = await requestAdapter(s);
  assert.ok(future > 0n);
  const waits = s.w.at(16);
  s.w.d.setBigUint64(waits, future, true);
  const status = await s.call("jai_webgpu_wait", waits, 1, MAX64);
  assert.equal(status, 1); // WGPUWaitStatus_Success
  const record = s.w.at(88);
  assert.equal(s.call("jai_webgpu_poll", record, waits, 1, WAIT_ANY), 1);
  assert.equal(s.w.d.getUint32(waits + 8, true), 1, "the future is marked completed");
  const r = readRecord(s.w.d, record);
  assert.equal(r.kind, 8); // RequestAdapter
  assert.equal(r.mode, ALLOW_PROCESS_EVENTS);
  assert.equal(r.future, future);
  assert.equal(r.callback, 0xC0DEn);
  assert.equal(r.userdata1, 0x55n);
  assert.equal(r.args[0], 1n); // WGPURequestAdapterStatus_Success
  assert.ok(r.args[1] > 0n, "the adapter handle");
  assert.deepEqual(r.message, [0n, 0n]);
  assert.equal(s.call("jai_webgpu_poll", record, waits, 1, WAIT_ANY), 0, "each record runs once");
});

test("callback modes decide which dispatch runs a record", async () => {
  for (const [mode, runsIn] of [
    [WAIT_ANY_ONLY, ["wait-any"]],
    [ALLOW_PROCESS_EVENTS, ["process-events", "wait-any"]],
    [ALLOW_SPONTANEOUS, ["process-events", "wait-any", "frame"]],
  ]) {
    for (const [how, name] of [[PROCESS_EVENTS, "process-events"], [SPONTANEOUS, "frame"], [WAIT_ANY, "wait-any"]]) {
      const s = setup();
      const { future } = await requestAdapter(s, mode);
      await delay(10);
      const waits = s.w.at(16);
      s.w.d.setBigUint64(waits, future, true);
      const ran = s.call("jai_webgpu_poll", s.w.at(88), how === WAIT_ANY ? waits : 0, how === WAIT_ANY ? 1 : 0, how);
      assert.equal(ran, runsIn.includes(name) ? 1 : 0, `mode ${mode} in ${name}`);
    }
  }
  // WaitAny on another future does not run an AllowProcessEvents callback.
  const s = setup();
  await requestAdapter(s, ALLOW_PROCESS_EVENTS);
  await delay(10);
  const waits = s.w.at(16);
  s.w.d.setBigUint64(waits, 999n, true);
  assert.equal(s.call("jai_webgpu_poll", s.w.at(88), waits, 1, WAIT_ANY), 0);
});

async function device(s) {
  const { future } = await requestAdapter(s);
  await delay(10);
  const record = s.w.at(88);
  s.call("jai_webgpu_poll", record, 0, 0, PROCESS_EVENTS);
  const adapter = readRecord(s.w.d, record).args[1];
  const out = s.w.at(8);
  s.call("wgpuAdapterRequestDevice", adapter, 0, s.w.callbackInfo(ALLOW_PROCESS_EVENTS, 1n), out);
  await delay(15);
  assert.equal(s.call("jai_webgpu_poll", record, 0, 0, PROCESS_EVENTS), 1);
  assert.ok(future);
  return readRecord(s.w.d, record).args[1];
}

test("descriptors: strings, flags, handles and reference counts", async () => {
  const s = setup();
  const dev = await device(s);
  const desc = s.w.at(STRUCTS.BufferDescriptor.size);
  s.w.view(desc + s.w.member("BufferDescriptor", "label"), "readback");
  s.w.d.setBigUint64(desc + s.w.member("BufferDescriptor", "usage"), 9n, true); // MapRead | CopyDst
  s.w.d.setBigUint64(desc + s.w.member("BufferDescriptor", "size"), 16n, true);
  const buffer = s.call("wgpuDeviceCreateBuffer", dev, desc);
  assert.ok(buffer > 0);
  assert.deepEqual(s.gpu.calls.find(c => c[0] === "createBuffer")[1],
    { label: "readback", usage: 9, size: 16, mappedAtCreation: false });
  assert.equal(s.call("wgpuBufferGetSize", buffer), 16, "attribute getters");
  s.call("wgpuBufferAddRef", buffer);
  s.call("wgpuBufferRelease", buffer);
  assert.equal(s.call("wgpuBufferGetSize", buffer), 16, "alive while referenced");
  s.call("wgpuBufferRelease", buffer);
  assert.throws(() => s.call("wgpuBufferGetSize", buffer), /not alive/);
});

test("chained structs and null-terminated strings", async () => {
  const s = setup();
  const dev = await device(s);
  const wgsl = s.w.at(STRUCTS.ShaderSourceWGSL.size);
  s.w.d.setUint32(wgsl + 8, STRUCTS.ShaderSourceWGSL.sType, true);
  s.w.view(wgsl + s.w.member("ShaderSourceWGSL", "code"), "@compute fn main() {}", { nullTerminated: true });
  const desc = s.w.at(STRUCTS.ShaderModuleDescriptor.size);
  s.w.d.setBigUint64(desc, BigInt(wgsl), true);
  s.w.view(desc + s.w.member("ShaderModuleDescriptor", "label"), "kernel", { nullTerminated: true });
  s.call("wgpuDeviceCreateShaderModule", dev, desc);
  assert.deepEqual(s.gpu.calls.find(c => c[0] === "createShaderModule")[1], { label: "kernel", code: "@compute fn main() {}" });
});

test("sentinels become undefined and bind group entries become resources", async () => {
  const s = setup();
  const dev = await device(s);
  const desc = s.w.at(STRUCTS.BufferDescriptor.size);
  s.w.d.setBigUint64(desc + s.w.member("BufferDescriptor", "size"), 64n, true);
  const buffer = s.call("wgpuDeviceCreateBuffer", dev, desc);
  const entry = s.w.at(STRUCTS.BindGroupEntry.size);
  s.w.d.setBigUint64(entry + s.w.member("BindGroupEntry", "buffer"), BigInt(buffer), true);
  s.w.d.setBigUint64(entry + s.w.member("BindGroupEntry", "size"), MAX64, true); // WGPU_WHOLE_SIZE
  const group = s.w.at(STRUCTS.BindGroupDescriptor.size);
  // An array member: [name, offset of the pointer, ["array", kind, offset of the count]].
  const [, entries, [, , count]] = STRUCTS.BindGroupDescriptor.members.find(m => m[0] === "entries");
  s.w.d.setBigUint64(group + entries, BigInt(entry), true);
  s.w.d.setBigUint64(group + count, 1n, true);
  s.call("wgpuDeviceCreateBindGroup", dev, group);
  const made = s.gpu.calls.find(c => c[0] === "createBindGroup")[1];
  assert.equal(made.entries.length, 1);
  assert.equal(made.entries[0].resource.buffer.size, 64);
  assert.equal(made.entries[0].resource.offset, 0);
  assert.equal(made.entries[0].resource.size, undefined);
});

test("mapping copies the range into the program's memory and back on unmap", async () => {
  const s = setup();
  const dev = await device(s);
  const desc = s.w.at(STRUCTS.BufferDescriptor.size);
  s.w.d.setBigUint64(desc + s.w.member("BufferDescriptor", "size"), 8n, true);
  const buffer = s.call("wgpuDeviceCreateBuffer", dev, desc);
  const out = s.w.at(8);
  s.call("wgpuBufferMapAsync", buffer, 1, 0, 8, s.w.callbackInfo(ALLOW_PROCESS_EVENTS, 7n), out);
  assert.deepEqual(s.gpu.calls.find(c => c[0] === "mapAsync").slice(1), [1, 0, 8]);
  const waits = s.w.at(16);
  s.w.d.setBigUint64(waits, s.w.d.getBigUint64(out, true), true);
  await s.call("jai_webgpu_wait", waits, 1, MAX64);
  const record = s.w.at(88);
  assert.equal(s.call("jai_webgpu_poll", record, 0, 0, PROCESS_EVENTS), 1);
  assert.equal(readRecord(s.w.d, record).args[0], 1n); // WGPUMapAsyncStatus_Success
  const range = s.call("wgpuBufferGetMappedRange", buffer, 0, MAX64);
  assert.deepEqual([...new Uint8Array(s.memory.buffer, range, 8)], [1, 2, 3, 4, 5, 6, 7, 8]);
  new Uint8Array(s.memory.buffer, range, 8)[0] = 42;
  s.call("wgpuBufferUnmap", buffer);
  assert.deepEqual(s.gpu.calls.find(c => c[0] === "unmap")[1], [42, 2, 3, 4, 5, 6, 7, 8]);
  assert.ok(s.memory.freed.includes(range), "the copy is freed");
});

test("error scopes report the error type and message", async () => {
  const s = setup();
  const dev = await device(s);
  s.call("wgpuDevicePushErrorScope", dev, 1);
  const wgsl = s.w.at(STRUCTS.ShaderSourceWGSL.size);
  s.w.d.setUint32(wgsl + 8, STRUCTS.ShaderSourceWGSL.sType, true);
  s.w.view(wgsl + s.w.member("ShaderSourceWGSL", "code"), "nonsense");
  const desc = s.w.at(STRUCTS.ShaderModuleDescriptor.size);
  s.w.d.setBigUint64(desc, BigInt(wgsl), true);
  s.call("wgpuDeviceCreateShaderModule", dev, desc);
  const out = s.w.at(8);
  s.call("wgpuDevicePopErrorScope", dev, s.w.callbackInfo(ALLOW_PROCESS_EVENTS, 3n), out);
  await delay(15);
  const record = s.w.at(88);
  assert.equal(s.call("jai_webgpu_poll", record, 0, 0, PROCESS_EVENTS), 1);
  const r = readRecord(s.w.d, record);
  assert.equal(r.kind, 6); // PopErrorScope
  assert.equal(r.args[0], 1n); // success
  assert.equal(r.args[1], 2n); // WGPUErrorType_Validation
  const [data, length] = r.message;
  assert.equal(new TextDecoder().decode(new Uint8Array(s.memory.buffer, Number(data), Number(length))), "shader parse error: expected a declaration");
});

test("surfaces tell the page when they are configured", async () => {
  const s = setup();
  const dev = await device(s);
  const surface = s.call("wgpuInstanceCreateSurface", 1, 0);
  const config = s.w.at(STRUCTS.SurfaceConfiguration.size);
  s.w.d.setBigUint64(config + s.w.member("SurfaceConfiguration", "device"), BigInt(dev), true);
  s.w.d.setUint32(config + s.w.member("SurfaceConfiguration", "width"), 640, true);
  s.w.d.setUint32(config + s.w.member("SurfaceConfiguration", "height"), 480, true);
  s.call("wgpuSurfaceConfigure", surface, config);
  s.call("wgpuSurfaceUnconfigure", surface);
  assert.deepEqual(s.surfaces, [{ width: 640, height: 480 }, null]);
});

test("host procedures: availability, frames and the canvas size", async () => {
  const s = setup();
  const name = s.w.bytes("jai_webgpu_available");
  assert.equal(s.call("jai_host_provides", ...name), true);
  const missing = s.w.bytes("jai_no_such_thing");
  assert.equal(s.call("jai_host_provides", ...missing), false);
  assert.equal(s.call("jai_webgpu_available"), true);
  const frame = s.call("jai_webgpu_request_frame");
  const waits = s.w.at(16);
  s.w.d.setBigUint64(waits, BigInt(frame), true);
  assert.equal(await s.call("jai_webgpu_wait", waits, 1, MAX64), 1);
  s.host.resize(320, 200);
  const size = s.w.at(8);
  s.call("jai_canvas_size", size, size + 4);
  assert.deepEqual([s.w.d.getUint32(size, true), s.w.d.getUint32(size + 4, true)], [320, 200]);
  const without = createWebGPUHost({ gpu: undefined });
  assert.equal(without.functions.jai_webgpu_available([], s.memory), false);
});

test("pointers outside the program's memory are refused", async () => {
  const s = setup();
  const dev = await device(s);
  const outside = s.memory.buffer.byteLength - 8;
  assert.throws(() => s.call("wgpuDeviceCreateBuffer", dev, outside), RangeError);
  assert.throws(() => s.call("wgpuDeviceCreateBuffer", dev, 1n << 40n), /outside the program's memory/);
  const desc = s.w.at(STRUCTS.BufferDescriptor.size);
  s.w.d.setBigUint64(desc + s.w.member("BufferDescriptor", "label"), BigInt(s.memory.buffer.byteLength), true);
  s.w.d.setBigUint64(desc + s.w.member("BufferDescriptor", "label") + 8, 4n, true);
  assert.throws(() => s.call("wgpuDeviceCreateBuffer", dev, desc), /string .* outside/);
  assert.throws(() => s.call("wgpuQueueWriteBuffer", 1, 1, 0, outside, 64), RangeError);
});

// --- The real engine: requests suspend the module (JSPI) until the mock's promises settle. ---
const bundle = process.env.JAI_WASM_DIR;
const jspi = typeof WebAssembly.Suspending === "function";
test("a program waits for the page and other threads keep running", { skip: !bundle ? "JAI_WASM_DIR is not set" : !jspi ? "no JSPI (node --experimental-wasm-jspi)" : false }, async () => {
  const bytes = await readFile(path.join(bundle, "jai_wasm.wasm"));
  const gpu = mockGPU();
  const streamed = [];
  const host = createWebGPUHost({ gpu, output: text => streamed.push(text) });
  const engine = await createEngine(bytes, { host });
  assert.equal(engine.jspi, true);
  const source = await readFile(path.join(ROOT, "tests/webgpu/bridge.jai"), "utf8");
  const result = await engine.playAsync({ "main.jai": source }, "main.jai", { budget: 50_000_000 });
  assert.equal(result.exitCode, 0, result.stderr + JSON.stringify(result.diagnostics ?? []));
  assert.equal(result.stdout, [
    "available: true",
    "adapter: Mock adapter",
    "device: true",
    "thread ran while mapping: true",
    "mapped: true 1 16",
    "thread ran during frames: true",
    "error scope: Validation (shader parse error: expected a declaration)",
    "done",
    "",
  ].join("\n"));
  assert.ok(streamed.join("").startsWith("available: true\n"), "output streams while the program waits");
  const order = gpu.calls.map(c => c[0]).filter(n => n !== "addEventListener");
  assert.deepEqual(order, ["requestAdapter", "requestDevice", "createBuffer", "mapAsync", "unmap", "pushErrorScope", "createShaderModule"]);

  // Without a host the same program learns there is no WebGPU instead of failing.
  const plain = await createEngine(bytes);
  const fallback = plain.play({ "main.jai": '#import "Basic";\n#import "WebGPU";\nmain :: () { print("%\\n", webgpu_available()); }\n' }, "main.jai");
  assert.equal(fallback.stdout, "false\n");
});
