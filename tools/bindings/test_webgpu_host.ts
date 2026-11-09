// The browser side of stdlib/Extensions/WebGPU (crates/jai-wasm/js/webgpu_host.mjs) against a small mock of
// the WebGPU JavaScript API: argument marshalling, handles, strings, chained structs, sentinels,
// futures and callback records with their modes, mapping, and pointer checks. With a built
// browser bundle (JAI_WASM_DIR=<dir with jai_wasm.wasm>) and JSPI (Node 24 needs the V8 flag
// --experimental-wasm-jspi, which reaches the tests only without process isolation), it also runs
// stdlib/Extensions/WebGPU/tests/driven/bridge.jai in the real engine, where every request suspends the module until the
// mock's promise settles.
//
//   node --test tools/bindings/test_webgpu_host.ts
//   JAI_WASM_DIR=target/ci-wasm node --experimental-wasm-jspi tools/bindings/test_webgpu_host.ts
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import type { HostFunction } from '../browser/play_types.ts';
import { createEngine } from '../../crates/jai-wasm/js/engine.mjs';
import { STRUCTS as GENERATED_STRUCTS } from '../../crates/jai-wasm/js/webgpu_bindings.generated.mjs';
import { createWebGPUHost } from '../../crates/jai-wasm/js/webgpu_host.mjs';

/** One struct of the generated layout tables: byte size, chain tag and `[name, offset, ...]` members. */
interface StructLayout {
  size: number;
  sType?: number;
  members: unknown[][];
}
const STRUCTS: Record<string, StructLayout> = GENERATED_STRUCTS;

/** A call the mock recorded: its name, then the arguments. */
type Call = [string, ...unknown[]];
type MockGPU = ReturnType<typeof mockGPU>;
type Memory = ReturnType<typeof fakeMemory>;
type Setup = ReturnType<typeof setup>;
type MockCanvas = { width?: number; height?: number; getContext(kind?: string): unknown } | null;

/** `createWebGPUHost` is typed for the browser's GPU; the tests hand it partial mocks. */
interface HostOptions {
  gpu?: unknown;
  canvas?: unknown;
  onSurface?(surface: unknown): void;
  output?(text: string): void;
}
interface WebGPUHost {
  functions: Record<string, HostFunction>;
  resize(width: number, height: number): void;
}
const makeHost = createWebGPUHost as unknown as (options: HostOptions) => WebGPUHost;

/** The first call the mock recorded under `name`. */
function recorded(gpu: MockGPU, name: string): Call {
  const call = gpu.calls.find((c) => c[0] === name);
  assert(call, `the mock saw no ${name} call`);
  return call;
}

/** The `sType` tag of a chained struct. */
function sTypeOf(struct: string): number {
  const tag = STRUCTS[struct]?.sType;
  assert(tag !== undefined, `${struct} has no sType`);
  return tag;
}

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const MAX64 = 0xffff_ffff_ffff_ffffn;
const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
// WGPUCallbackMode, and the `how` of jai_webgpu_poll.
const WAIT_ANY_ONLY = 1,
  ALLOW_PROCESS_EVENTS = 2,
  ALLOW_SPONTANEOUS = 3;
const PROCESS_EVENTS = 0n,
  WAIT_ANY = 1n,
  SPONTANEOUS = 2n;

// --- A mock of the WebGPU JavaScript API: records what it was given, settles requests later. ---
// The host reads the error's kind from its constructor's name, so the class keeps the browser's.
class GPUValidationError {
  message: string;
  constructor(message: string) {
    this.message = message;
  }
}
function mockGPU() {
  const calls: Call[] = [];
  const errorScopes: GPUValidationError[][] = [];
  class Buffer {
    declare size: number;
    mapped: ArrayBuffer | null = null;
    constructor(desc: { size: number }) {
      Object.assign(this, desc);
    }
    mapAsync(mode: number, offset: number, size: number) {
      calls.push(['mapAsync', mode, offset, size]);
      return delay(20).then(() => {
        this.mapped = new ArrayBuffer(this.size);
        new Uint8Array(this.mapped).forEach((_, i, bytes) => {
          bytes[i] = i + 1;
        });
      });
    }
    getMappedRange() {
      return this.mapped;
    }
    unmap() {
      calls.push(['unmap', [...new Uint8Array(this.mapped!)]]);
      this.mapped = null;
    }
  }
  const device = {
    label: '',
    queue: {
      label: '',
      submit: (list: unknown[]) => calls.push(['submit', list.length]),
      writeBuffer() {},
    },
    lost: new Promise(() => {}),
    features: new Set(),
    limits: { maxBindGroups: 4 },
    adapterInfo: {
      vendor: 'mock',
      architecture: 'test',
      device: '',
      description: 'Mock adapter',
    },
    addEventListener: (type: string) => calls.push(['addEventListener', type]),
    createBuffer: (desc: { size: number }) => {
      calls.push(['createBuffer', desc]);
      return new Buffer(desc);
    },
    createShaderModule: (desc: { code: string; label?: string }) => {
      calls.push(['createShaderModule', desc]);
      if (!desc.code.includes('@'))
        errorScopes
          .at(-1)
          ?.push(new GPUValidationError('shader parse error: expected a declaration'));
      return { label: desc.label ?? '' };
    },
    createBindGroup: (desc: unknown) => {
      calls.push(['createBindGroup', desc]);
      return { label: '' };
    },
    pushErrorScope: (filter: unknown) => {
      calls.push(['pushErrorScope', filter]);
      errorScopes.push([]);
    },
    popErrorScope: () => delay(5).then(() => errorScopes.pop()![0] ?? null),
  };
  const adapter = {
    info: { vendor: 'mock', architecture: 'test', device: '', description: 'Mock adapter' },
    features: new Set(['timestamp-query']),
    limits: { maxBindGroups: 4 },
    requestDevice: (desc: unknown) => {
      calls.push(['requestDevice', desc]);
      return delay(5).then(() => device);
    },
  };
  return {
    calls,
    adapter,
    device,
    requestAdapter: (options: unknown) => {
      calls.push(['requestAdapter', options]);
      return delay(5).then(() => adapter);
    },
    getPreferredCanvasFormat: () => 'bgra8unorm',
    wgslLanguageFeatures: new Set(),
  };
}

// --- The program's memory: a plain ArrayBuffer with a bump allocator for "C" data. ---
function fakeMemory(size = 1 << 16) {
  const buffer = new ArrayBuffer(size);
  let top = 4096; // [0, 4096) belongs to the test's own structs
  const freed: number[] = [];
  return {
    buffer,
    freed,
    alloc(n: number) {
      const p = top;
      top += (n + 15) & ~15;
      return p;
    },
    free(p: number) {
      freed.push(p);
    },
  };
}
// Lays out C structs by the generated tables' offsets, so the test follows the bindings.
function writer(memory: Memory) {
  const d = new DataView(memory.buffer);
  let next = 64;
  const at = (size: number) => {
    const p = next;
    next += (size + 15) & ~15;
    return p;
  };
  return {
    at,
    d,
    bytes(text: string, terminate = false): [number, number] {
      const encoded = new TextEncoder().encode(text);
      const p = at(encoded.length + 1);
      new Uint8Array(memory.buffer, p, encoded.length).set(encoded);
      if (terminate) d.setUint8(p + encoded.length, 0);
      return [p, encoded.length];
    },
    view(p: number, text: string, { nullTerminated = false } = {}) {
      const [data, length] = this.bytes(text, nullTerminated);
      d.setBigUint64(p, BigInt(data), true);
      d.setBigUint64(p + 8, nullTerminated ? MAX64 : BigInt(length), true);
    },
    member(struct: string, name: string) {
      const found = STRUCTS[struct]?.members.find((m) => m[0] === name);
      assert(found, `no member ${name} in ${struct}`);
      return Number(found[1]);
    },
    callbackInfo(mode: number, callback: bigint, userdata1 = 0n) {
      const p = at(40);
      d.setUint32(p + 8, mode, true);
      d.setBigUint64(p + 16, callback, true);
      d.setBigUint64(p + 24, userdata1, true);
      return p;
    },
  };
}

function setup({
  gpu = mockGPU(),
  canvas = mockCanvas() as MockCanvas,
}: { gpu?: MockGPU; canvas?: MockCanvas } = {}) {
  const memory = fakeMemory();
  const surfaces: unknown[] = [];
  const host = makeHost({ gpu, canvas, onSurface: (s: unknown) => surfaces.push(s) });
  const call = <T = number>(name: string, ...args: (number | bigint)[]) =>
    host.functions[name]!(
      args.map((a) => BigInt(a)),
      memory,
    ) as T;
  const w = writer(memory);
  return { gpu, memory, host, call, w, surfaces };
}
function mockCanvas() {
  return {
    width: 300,
    height: 150,
    getContext: () => ({
      config: undefined as unknown,
      configure(c: unknown) {
        this.config = c;
      },
      unconfigure() {},
    }),
  };
}
// A callback record (Webgpu_Callback_Record in stdlib/Extensions/WebGPU/wasm.jai) as an object.
function readRecord(d: DataView, p: number) {
  return {
    kind: d.getUint32(p, true),
    mode: d.getUint32(p + 4, true),
    future: d.getBigUint64(p + 8, true),
    callback: d.getBigUint64(p + 16, true),
    userdata1: d.getBigUint64(p + 24, true),
    args: [0, 1, 2, 3].map((i) => d.getBigUint64(p + 40 + i * 8, true)),
    message: [d.getBigUint64(p + 72, true), d.getBigUint64(p + 80, true)],
  };
}
async function requestAdapter({ call, w }: Setup, mode = ALLOW_PROCESS_EVENTS) {
  const instance = call('wgpuCreateInstance', 0);
  const future = w.at(8);
  call('wgpuInstanceRequestAdapter', instance, 0, w.callbackInfo(mode, 0xc0den, 0x55n), future);
  return { instance, future: w.d.getBigUint64(future, true) };
}

test('a request becomes a future whose callback record WaitAny runs', async () => {
  const s = setup();
  const { future } = await requestAdapter(s);
  assert.ok(future > 0n);
  const waits = s.w.at(16);
  s.w.d.setBigUint64(waits, future, true);
  const status = await s.call('jai_webgpu_wait', waits, 1, MAX64);
  assert.equal(status, 1); // WGPUWaitStatus_Success
  const record = s.w.at(88);
  assert.equal(s.call('jai_webgpu_poll', record, waits, 1, WAIT_ANY), 1);
  assert.equal(s.w.d.getUint32(waits + 8, true), 1, 'the future is marked completed');
  const r = readRecord(s.w.d, record);
  assert.equal(r.kind, 8); // RequestAdapter
  assert.equal(r.mode, ALLOW_PROCESS_EVENTS);
  assert.equal(r.future, future);
  assert.equal(r.callback, 0xc0den);
  assert.equal(r.userdata1, 0x55n);
  assert.equal(r.args[0], 1n); // WGPURequestAdapterStatus_Success
  assert.ok(r.args[1] > 0n, 'the adapter handle');
  assert.deepEqual(r.message, [0n, 0n]);
  assert.equal(s.call('jai_webgpu_poll', record, waits, 1, WAIT_ANY), 0, 'each record runs once');
});

test('callback modes decide which dispatch runs a record', async () => {
  const modes: [number, string[]][] = [
    [WAIT_ANY_ONLY, ['wait-any']],
    [ALLOW_PROCESS_EVENTS, ['process-events', 'wait-any']],
    [ALLOW_SPONTANEOUS, ['process-events', 'wait-any', 'frame']],
  ];
  const dispatches: [bigint, string][] = [
    [PROCESS_EVENTS, 'process-events'],
    [SPONTANEOUS, 'frame'],
    [WAIT_ANY, 'wait-any'],
  ];
  for (const [mode, runsIn] of modes) {
    for (const [how, name] of dispatches) {
      const s = setup();
      const { future } = await requestAdapter(s, mode);
      await delay(10);
      const waits = s.w.at(16);
      s.w.d.setBigUint64(waits, future, true);
      const ran = s.call(
        'jai_webgpu_poll',
        s.w.at(88),
        how === WAIT_ANY ? waits : 0,
        how === WAIT_ANY ? 1 : 0,
        how,
      );
      assert.equal(ran, runsIn.includes(name) ? 1 : 0, `mode ${mode} in ${name}`);
    }
  }
  // WaitAny on another future does not run an AllowProcessEvents callback.
  const s = setup();
  await requestAdapter(s, ALLOW_PROCESS_EVENTS);
  await delay(10);
  const waits = s.w.at(16);
  s.w.d.setBigUint64(waits, 999n, true);
  assert.equal(s.call('jai_webgpu_poll', s.w.at(88), waits, 1, WAIT_ANY), 0);
});

async function openDevice(s: Setup) {
  const { future } = await requestAdapter(s);
  await delay(10);
  const record = s.w.at(88);
  s.call('jai_webgpu_poll', record, 0, 0, PROCESS_EVENTS);
  const adapter = readRecord(s.w.d, record).args[1];
  const out = s.w.at(8);
  s.call('wgpuAdapterRequestDevice', adapter, 0, s.w.callbackInfo(ALLOW_PROCESS_EVENTS, 1n), out);
  await delay(15);
  assert.equal(s.call('jai_webgpu_poll', record, 0, 0, PROCESS_EVENTS), 1);
  assert.ok(future);
  return readRecord(s.w.d, record).args[1];
}

test('descriptors: strings, flags, handles and reference counts', async () => {
  const s = setup();
  const dev = await openDevice(s);
  const desc = s.w.at(STRUCTS.BufferDescriptor.size);
  s.w.view(desc + s.w.member('BufferDescriptor', 'label'), 'readback');
  s.w.d.setBigUint64(desc + s.w.member('BufferDescriptor', 'usage'), 9n, true); // MapRead | CopyDst
  s.w.d.setBigUint64(desc + s.w.member('BufferDescriptor', 'size'), 16n, true);
  const buffer = s.call('wgpuDeviceCreateBuffer', dev, desc);
  assert.ok(buffer > 0);
  assert.deepEqual(recorded(s.gpu, 'createBuffer')[1], {
    label: 'readback',
    usage: 9,
    size: 16,
    mappedAtCreation: false,
  });
  assert.equal(s.call('wgpuBufferGetSize', buffer), 16, 'attribute getters');
  s.call('wgpuBufferAddRef', buffer);
  s.call('wgpuBufferRelease', buffer);
  assert.equal(s.call('wgpuBufferGetSize', buffer), 16, 'alive while referenced');
  s.call('wgpuBufferRelease', buffer);
  assert.throws(() => s.call('wgpuBufferGetSize', buffer), /not alive/);
});

test('chained structs and null-terminated strings', async () => {
  const s = setup();
  const dev = await openDevice(s);
  const wgsl = s.w.at(STRUCTS.ShaderSourceWGSL.size);
  s.w.d.setUint32(wgsl + 8, sTypeOf('ShaderSourceWGSL'), true);
  s.w.view(wgsl + s.w.member('ShaderSourceWGSL', 'code'), '@compute fn main() {}', {
    nullTerminated: true,
  });
  const desc = s.w.at(STRUCTS.ShaderModuleDescriptor.size);
  s.w.d.setBigUint64(desc, BigInt(wgsl), true);
  s.w.view(desc + s.w.member('ShaderModuleDescriptor', 'label'), 'kernel', {
    nullTerminated: true,
  });
  s.call('wgpuDeviceCreateShaderModule', dev, desc);
  assert.deepEqual(recorded(s.gpu, 'createShaderModule')[1], {
    label: 'kernel',
    code: '@compute fn main() {}',
  });
});

test('sentinels become undefined and bind group entries become resources', async () => {
  const s = setup();
  const dev = await openDevice(s);
  const desc = s.w.at(STRUCTS.BufferDescriptor.size);
  s.w.d.setBigUint64(desc + s.w.member('BufferDescriptor', 'size'), 64n, true);
  const buffer = s.call('wgpuDeviceCreateBuffer', dev, desc);
  const entry = s.w.at(STRUCTS.BindGroupEntry.size);
  s.w.d.setBigUint64(entry + s.w.member('BindGroupEntry', 'buffer'), BigInt(buffer), true);
  s.w.d.setBigUint64(entry + s.w.member('BindGroupEntry', 'size'), MAX64, true); // WGPU_WHOLE_SIZE
  const group = s.w.at(STRUCTS.BindGroupDescriptor.size);
  // An array member: [name, offset of the pointer, ["array", kind, offset of the count]].
  const found = STRUCTS.BindGroupDescriptor.members.find((m) => m[0] === 'entries');
  assert(found, 'BindGroupDescriptor has an entries member');
  const entries = Number(found[1]);
  const count = Number((found[2] as unknown[])[2]);
  s.w.d.setBigUint64(group + entries, BigInt(entry), true);
  s.w.d.setBigUint64(group + count, 1n, true);
  s.call('wgpuDeviceCreateBindGroup', dev, group);
  const made = recorded(s.gpu, 'createBindGroup')[1] as {
    entries: { resource: { buffer: { size: number }; offset: number; size?: number } }[];
  };
  assert.equal(made.entries.length, 1);
  assert.equal(made.entries[0].resource.buffer.size, 64);
  assert.equal(made.entries[0].resource.offset, 0);
  assert.equal(made.entries[0].resource.size, undefined);
});

test("mapping copies the range into the program's memory and back on unmap", async () => {
  const s = setup();
  const dev = await openDevice(s);
  const desc = s.w.at(STRUCTS.BufferDescriptor.size);
  s.w.d.setBigUint64(desc + s.w.member('BufferDescriptor', 'size'), 8n, true);
  const buffer = s.call('wgpuDeviceCreateBuffer', dev, desc);
  const out = s.w.at(8);
  s.call('wgpuBufferMapAsync', buffer, 1, 0, 8, s.w.callbackInfo(ALLOW_PROCESS_EVENTS, 7n), out);
  assert.deepEqual(recorded(s.gpu, 'mapAsync').slice(1), [1, 0, 8]);
  const waits = s.w.at(16);
  s.w.d.setBigUint64(waits, s.w.d.getBigUint64(out, true), true);
  await s.call('jai_webgpu_wait', waits, 1, MAX64);
  const record = s.w.at(88);
  assert.equal(s.call('jai_webgpu_poll', record, 0, 0, PROCESS_EVENTS), 1);
  assert.equal(readRecord(s.w.d, record).args[0], 1n); // WGPUMapAsyncStatus_Success
  const range = s.call('wgpuBufferGetMappedRange', buffer, 0, MAX64);
  assert.deepEqual([...new Uint8Array(s.memory.buffer, range, 8)], [1, 2, 3, 4, 5, 6, 7, 8]);
  new Uint8Array(s.memory.buffer, range, 8)[0] = 42;
  s.call('wgpuBufferUnmap', buffer);
  assert.deepEqual(recorded(s.gpu, 'unmap')[1], [42, 2, 3, 4, 5, 6, 7, 8]);
  assert.ok(s.memory.freed.includes(range), 'the copy is freed');
});

test('error scopes report the error type and message', async () => {
  const s = setup();
  const dev = await openDevice(s);
  s.call('wgpuDevicePushErrorScope', dev, 1);
  const wgsl = s.w.at(STRUCTS.ShaderSourceWGSL.size);
  s.w.d.setUint32(wgsl + 8, sTypeOf('ShaderSourceWGSL'), true);
  s.w.view(wgsl + s.w.member('ShaderSourceWGSL', 'code'), 'nonsense');
  const desc = s.w.at(STRUCTS.ShaderModuleDescriptor.size);
  s.w.d.setBigUint64(desc, BigInt(wgsl), true);
  s.call('wgpuDeviceCreateShaderModule', dev, desc);
  const out = s.w.at(8);
  s.call('wgpuDevicePopErrorScope', dev, s.w.callbackInfo(ALLOW_PROCESS_EVENTS, 3n), out);
  await delay(15);
  const record = s.w.at(88);
  assert.equal(s.call('jai_webgpu_poll', record, 0, 0, PROCESS_EVENTS), 1);
  const r = readRecord(s.w.d, record);
  assert.equal(r.kind, 6); // PopErrorScope
  assert.equal(r.args[0], 1n); // success
  assert.equal(r.args[1], 2n); // WGPUErrorType_Validation
  const [data, length] = r.message;
  assert.equal(
    new TextDecoder().decode(new Uint8Array(s.memory.buffer, Number(data), Number(length))),
    'shader parse error: expected a declaration',
  );
});

test('surfaces tell the page when they are configured', async () => {
  const s = setup();
  const dev = await openDevice(s);
  const surface = s.call('wgpuInstanceCreateSurface', 1, 0);
  const config = s.w.at(STRUCTS.SurfaceConfiguration.size);
  s.w.d.setBigUint64(config + s.w.member('SurfaceConfiguration', 'device'), BigInt(dev), true);
  s.w.d.setUint32(config + s.w.member('SurfaceConfiguration', 'width'), 640, true);
  s.w.d.setUint32(config + s.w.member('SurfaceConfiguration', 'height'), 480, true);
  s.call('wgpuSurfaceConfigure', surface, config);
  s.call('wgpuSurfaceUnconfigure', surface);
  assert.deepEqual(s.surfaces, [{ width: 640, height: 480 }, null]);
});

test('headless GPU capability does not advertise a canvas surface', async () => {
  const s = setup({ canvas: null });
  assert.equal(s.call('jai_webgpu_available'), true);
  assert.equal(s.call('jai_canvas_available'), false);
  const dev = await openDevice(s);
  assert(dev > 0, 'headless adapter/device requests remain supported');
  assert.throws(() => s.call('wgpuInstanceCreateSurface', 1, 0), /no canvas/);
});

test('canvas capability and surface use the same real context', async () => {
  let queried = 0;
  const context: {
    config?: { device?: unknown };
    configure(config: { device?: unknown }): void;
    unconfigure(): void;
  } = {
    configure(config) {
      this.config = config;
    },
    unconfigure() {},
  };
  const canvas = {
    width: 640,
    height: 480,
    getContext(kind?: string) {
      assert.equal(kind, 'webgpu');
      queried++;
      return context;
    },
  };
  const s = setup({ canvas });
  assert.equal(s.call('jai_canvas_available'), true);
  assert.equal(s.call('jai_canvas_available'), true);
  const dev = await openDevice(s);
  const surface = s.call('wgpuInstanceCreateSurface', 1, 0);
  const config = s.w.at(STRUCTS.SurfaceConfiguration.size);
  s.w.d.setBigUint64(config + s.w.member('SurfaceConfiguration', 'device'), BigInt(dev), true);
  s.w.d.setUint32(config + s.w.member('SurfaceConfiguration', 'width'), 640, true);
  s.w.d.setUint32(config + s.w.member('SurfaceConfiguration', 'height'), 480, true);
  s.call('wgpuSurfaceConfigure', surface, config);
  assert.equal(queried, 1);
  assert.equal(context.config?.device, s.gpu.device);
  assert.deepEqual(s.surfaces, [{ width: 640, height: 480 }]);
});

test('an unavailable WebGPU canvas context never becomes a surface handle', () => {
  const s = setup({
    canvas: {
      getContext() {
        return null;
      },
    },
  });
  assert.equal(s.call('jai_webgpu_available'), true);
  assert.equal(s.call('jai_canvas_available'), false);
  assert.throws(() => s.call('wgpuInstanceCreateSurface', 1, 0), /no WebGPU context/);
});

test('host procedures: availability, frames and the canvas size', async () => {
  const s = setup();
  const name = s.w.bytes('jai_webgpu_available');
  assert.equal(s.call('jai_host_provides', ...name), true);
  const missing = s.w.bytes('jai_no_such_thing');
  assert.equal(s.call('jai_host_provides', ...missing), false);
  assert.equal(s.call('jai_webgpu_available'), true);
  const frame = s.call('jai_webgpu_request_frame');
  const waits = s.w.at(16);
  s.w.d.setBigUint64(waits, BigInt(frame), true);
  assert.equal(await s.call('jai_webgpu_wait', waits, 1, MAX64), 1);
  s.host.resize(320, 200);
  const size = s.w.at(8);
  s.call('jai_canvas_size', size, size + 4);
  assert.deepEqual([s.w.d.getUint32(size, true), s.w.d.getUint32(size + 4, true)], [320, 200]);
  const without = makeHost({ gpu: undefined });
  assert.equal(without.functions.jai_webgpu_available([], s.memory), false);
});

test("pointers outside the program's memory are refused", async () => {
  const s = setup();
  const dev = await openDevice(s);
  const outside = s.memory.buffer.byteLength - 8;
  assert.throws(() => s.call('wgpuDeviceCreateBuffer', dev, outside), RangeError);
  assert.throws(
    () => s.call('wgpuDeviceCreateBuffer', dev, 1n << 40n),
    /outside the program's memory/,
  );
  const desc = s.w.at(STRUCTS.BufferDescriptor.size);
  s.w.d.setBigUint64(
    desc + s.w.member('BufferDescriptor', 'label'),
    BigInt(s.memory.buffer.byteLength),
    true,
  );
  s.w.d.setBigUint64(desc + s.w.member('BufferDescriptor', 'label') + 8, 4n, true);
  assert.throws(() => s.call('wgpuDeviceCreateBuffer', dev, desc), /string .* outside/);
  assert.throws(() => s.call('wgpuQueueWriteBuffer', 1, 1, 0, outside, 64), RangeError);
});

// --- The real engine: requests suspend the module (JSPI) until the mock's promises settle. ---
const bundle = process.env.JAI_WASM_DIR;
const jspi = typeof (WebAssembly as { Suspending?: unknown }).Suspending === 'function';
test(
  'a program waits for the page and other threads keep running',
  {
    skip: !bundle
      ? 'JAI_WASM_DIR is not set'
      : !jspi
        ? 'no JSPI (node --experimental-wasm-jspi)'
        : false,
  },
  async () => {
    assert(bundle, 'the skip option covers a missing JAI_WASM_DIR');
    const bytes = await readFile(path.join(bundle, 'jai_wasm.wasm'));
    const gpu = mockGPU();
    const streamed: string[] = [];
    const host = makeHost({ gpu, output: (text: string) => streamed.push(text) });
    const engine = await createEngine(bytes, { host });
    assert.equal(engine.jspi, true);
    const source = await readFile(
      path.join(ROOT, 'stdlib/Extensions/WebGPU/tests/driven/bridge.jai'),
      'utf8',
    );
    const result = await engine.playAsync({ 'main.jai': source }, 'main.jai', {
      budget: 50_000_000,
    });
    assert.equal(result.exitCode, 0, result.stderr + JSON.stringify(result.diagnostics ?? []));
    assert.equal(
      result.stdout,
      [
        'available: true',
        'adapter: Mock adapter',
        'device: true',
        'thread ran while mapping: true',
        'mapped: true 1 16',
        'thread ran during frames: true',
        'error scope: Validation (shader parse error: expected a declaration)',
        'done',
        '',
      ].join('\n'),
    );
    assert.ok(
      streamed.join('').startsWith('available: true\n'),
      'output streams while the program waits',
    );
    const order = gpu.calls.map((c) => c[0]).filter((n) => n !== 'addEventListener');
    assert.deepEqual(order, [
      'requestAdapter',
      'requestDevice',
      'createBuffer',
      'mapAsync',
      'unmap',
      'pushErrorScope',
      'createShaderModule',
    ]);

    // Without a host the same program learns there is no WebGPU instead of failing.
    const plain = await createEngine(bytes);
    const fallback = plain.play(
      {
        'main.jai':
          '#import "Basic";\n#import "Extensions/WebGPU";\nmain :: () { print("%\\n", webgpu_available()); }\n',
      },
      'main.jai',
    );
    assert.equal(fallback.stdout, 'false\n');
  },
);
