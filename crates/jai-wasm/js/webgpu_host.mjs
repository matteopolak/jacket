// The browser side of stdlib/WebGPU: host functions for engine.mjs that run a Jai program's
// wgpu* calls on the page's WebGPU (navigator.gpu), plus its canvas and input.
//
// Most calls need no code here: webgpu_bindings.generated.mjs (tools/webgpu_gen.py, from
// webgpu.yml) describes every function's arguments and every struct's layout, and `call` below
// reads them from the module's memory into the dictionaries the JS API takes, calls the method of
// the same name (wgpuDeviceCreateBuffer -> device.createBuffer) and converts the result. OVERRIDES
// covers where the C API and the JS API differ (mapping, writes from memory, surfaces, getters
// that return structs). Objects cross as small integer handles; a callback-taking function
// returns a future and queues its callback for the Jai side (stdlib/WebGPU/wasm.jai) to call.
//
// Usage:
//   const host = createWebGPUHost({ canvas, output, onSurface });
//   const engine = await createEngine(bytes, { host }); await engine.playAsync(files, "main.jai");
// host.input(event) queues a canvas event (see `pollEvent`), host.resize(w, h) sizes the canvas.
// onSurface({ width, height }) is called when the program configures its surface (the page can
// show the canvas then), onSurface(null) when it unconfigures it.
//
// Pointers: the program's pointers are byte addresses in the module's memory (wasm32 today, so
// below 4 GiB; a memory64 module would work the same). Every address that comes from the program
// goes through `addr`/`span`, which reject one outside `memory.buffer` with a RangeError naming
// it, instead of reading the wrong bytes.
import { CALLBACKS, ENUMS, FUNCTIONS, STRUCTS } from "./webgpu_bindings.generated.mjs";

const MAX64 = 0xFFFF_FFFF_FFFF_FFFFn;
const SIZES = { u16: 2, u32: 4, i32: 4, f32: 4, bool: 4, u64: 8, f64: 8, flags: 8, ptr: 8, str: 16, nstr: 16, ostr: 16 };
const FROM_JS = Object.fromEntries(Object.entries(ENUMS).map(([name, table]) => [
  name, new Map(Object.entries(table).map(([value, js]) => [js, Number(value)])),
]));
const enumValue = (name, js) => FROM_JS[name].get(js) ?? 0;
const STYPES = new Map(Object.entries(STRUCTS).filter(([, s]) => s.sType !== undefined).map(([name, s]) => [s.sType, name]));
const RECORD_SIZE = 88; // Webgpu_Callback_Record in stdlib/WebGPU/wasm.jai
const FUTURE_WAIT_INFO_SIZE = 16; // WGPUFutureWaitInfo: future id, completed (WGPUBool), padding
// WGPUCallbackMode, and which modes a dispatch runs (`how` of jai_webgpu_poll).
const WAIT_ANY_ONLY = 1, ALLOW_PROCESS_EVENTS = 2, ALLOW_SPONTANEOUS = 3;
const PROCESS_EVENTS = 0, WAIT_ANY = 1, SPONTANEOUS = 2;

export function createWebGPUHost({ canvas, gpu = globalThis.navigator?.gpu, output, onSurface } = {}) {
  let memory = null;
  let view = null;
  const dv = () => (view && view.buffer === memory.buffer ? view : (view = new DataView(memory.buffer)));
  const scratch = new DataView(new ArrayBuffer(8));
  const utf8 = new TextDecoder();
  const encoder = new TextEncoder();
  const allocations = new Map(); // pointer -> size, for memory this host handed the program

  // --- Handles --------------------------------------------------------------------------------
  const objects = [null];
  const refs = [0];
  const ids = new Map();
  const freeIds = [];
  function put(object) {
    if (object === null || object === undefined) return 0;
    let id = ids.get(object);
    if (id) { refs[id]++; return id; }
    id = freeIds.pop() ?? objects.length;
    objects[id] = object; refs[id] = 1; ids.set(object, id);
    return id;
  }
  function get(id) {
    const object = objects[Number(id)];
    if (id && !object) throw new Error(`WebGPU object handle ${id} is not alive`);
    return object ?? undefined;
  }
  function release(id) {
    id = Number(id);
    if (!objects[id] || --refs[id] > 0) return;
    ids.delete(objects[id]); objects[id] = null; freeIds.push(id);
  }

  // --- Memory ---------------------------------------------------------------------------------
  // An address from the program (a BigInt slot or a number read from memory) as a number, after
  // checking that `size` bytes from it lie in the module's memory.
  function span(address, size, what = "pointer") {
    const a = typeof address === "bigint" ? address : BigInt(address);
    const end = a + BigInt(size);
    if (a < 0n || end > BigInt(memory.buffer.byteLength)) {
      throw new RangeError(`${what} 0x${a.toString(16)} (+${size} bytes) is outside the program's memory (${memory.buffer.byteLength} bytes)`);
    }
    return Number(a);
  }
  const addr = (address, size = 0, what) => span(address, size, what);
  const bytes = (address, size, what) => new Uint8Array(memory.buffer, span(address, size, what), size);
  const ptrAt = p => Number(dv().getBigUint64(p, true));
  function alloc(size) {
    const p = memory.alloc(size);
    if (!p) throw new Error("out of memory");
    allocations.set(p, size);
    return p;
  }
  function free(p) {
    const size = allocations.get(p);
    if (size === undefined) return;
    allocations.delete(p); memory.free(p, size);
  }
  function readString(p) {
    const data = ptrAt(p);
    const length = dv().getBigUint64(p + 8, true);
    if (!data) return undefined;
    if (length === MAX64) {
      const all = new Uint8Array(memory.buffer, addr(data, 0, "string"));
      const n = all.indexOf(0);
      if (n < 0) throw new RangeError(`string at 0x${data.toString(16)} has no terminating zero`);
      return utf8.decode(all.subarray(0, n));
    }
    return utf8.decode(bytes(data, Number(length), "string"));
  }
  function writeString(p, text) {
    const encoded = encoder.encode(text ?? "");
    const data = encoded.length ? alloc(encoded.length) : 0;
    if (data) new Uint8Array(memory.buffer, data, encoded.length).set(encoded);
    dv().setBigUint64(p, BigInt(data), true);
    dv().setBigUint64(p + 8, BigInt(encoded.length), true);
  }
  const sizeOf = kind => typeof kind === "string" ? SIZES[kind]
    : kind[0] === "struct" ? STRUCTS[kind[1]].size : kind[0] === "enum" ? 4 : 8;

  // C -> JS -------------------------------------------------------------------------------------
  function read(kind, p) {
    const d = dv();
    if (typeof kind === "string") switch (kind) {
      case "u16": return d.getUint16(p, true);
      case "u32": return d.getUint32(p, true);
      case "i32": return d.getInt32(p, true);
      case "f32": return d.getFloat32(p, true);
      case "f64": return d.getFloat64(p, true);
      case "bool": return d.getUint32(p, true) !== 0;
      case "u64": case "flags": return Number(d.getBigUint64(p, true));
      case "ptr": return ptrAt(p);
      case "str": case "nstr": case "ostr": return readString(p);
    }
    switch (kind[0]) {
      case "enum": { const v = d.getUint32(p, true); return ENUMS[kind[1]][v]; }
      case "obj": return get(ptrAt(p));
      case "objptr": { const q = ptrAt(p); return q ? get(ptrAt(addr(q, 8))) : undefined; }
      case "struct": return readStruct(kind[1], p);
      case "ptr": { const q = ptrAt(p); return q ? readStruct(kind[1], q) : undefined; }
      case "callback": return readCallbackInfo(kind[1], p);
    }
    throw new Error(`cannot read ${JSON.stringify(kind)}`);
  }
  function readArray(kind, p, count) {
    const out = [];
    const size = sizeOf(kind);
    addr(p, size * count, "array");
    for (let i = 0; i < count; i++) out.push(read(kind, p + i * size));
    return out;
  }
  function isSentinel(kind, p, sentinel) {
    if (sentinel === "max64") return dv().getBigUint64(p, true) === MAX64;
    if (sentinel === "nan") return Number.isNaN(read(kind, p));
    return read(kind, p) === sentinel;
  }
  function readStruct(name, p) {
    const info = STRUCTS[name];
    addr(p, info.size, `WGPU${name}`);
    let out = {};
    for (const [key, off, kind, sentinel] of info.members) {
      if (kind === "chain") { readChain(ptrAt(p + off), out); continue; }
      let value;
      if (kind[0] === "array") {
        const count = Number(dv().getBigUint64(p + kind[2], true));
        const data = ptrAt(p + off);
        value = data && count ? readArray(kind[1], data, count) : [];
      } else {
        if (sentinel !== undefined && isSentinel(kind, p + off, sentinel)) continue;
        value = read(kind, p + off);
      }
      if (value === undefined) continue;
      if (value && value.omitParent) return undefined;
      out[key] = value;
    }
    const fix = STRUCT_FIXUPS[name];
    if (fix) out = fix(out);
    return out;
  }
  // Chained extensions become members of the parent dictionary, which is where the JS API has
  // them (WGPUShaderSourceWGSL.code -> GPUShaderModuleDescriptor.code).
  function readChain(p, out) {
    for (let n = 0; p; p = ptrAt(p), n++) {
      if (n > 64) throw new Error("WebGPU: a nextInChain list loops");
      addr(p, 16, "chained struct");
      const sType = dv().getUint32(p + 8, true);
      const name = STYPES.get(sType);
      if (!name) { console.warn(`WebGPU: chained struct sType ${sType} is not supported`); continue; }
      Object.assign(out, readStruct(name, p));
    }
  }
  function readCallbackInfo(name, p) {
    const d = dv();
    const at = CALLBACKS[name].mode ? 16 : 8;
    addr(p, at + 24, `WGPU${name}CallbackInfo`);
    return {
      mode: CALLBACKS[name].mode ? d.getUint32(p + 8, true) : ALLOW_SPONTANEOUS,
      callback: d.getBigUint64(p + at, true),
      userdata1: d.getBigUint64(p + at + 8, true),
      userdata2: d.getBigUint64(p + at + 16, true),
    };
  }
  const constantsRecord = list => Object.fromEntries((list ?? []).map(c => [c.key, c.value]));
  const STRUCT_FIXUPS = {
    TexelCopyBufferInfo: ({ buffer, layout }) => ({ buffer, ...layout }),
    BindGroupEntry: ({ binding, buffer, offset, size, sampler, textureView, ...rest }) => ({
      binding,
      resource: sampler ?? textureView ?? rest.externalTexture ?? { buffer, offset, size },
    }),
    VertexState: s => ({ ...s, constants: constantsRecord(s.constants) }),
    FragmentState: s => ({ ...s, constants: constantsRecord(s.constants) }),
    ComputeState: s => ({ ...s, constants: constantsRecord(s.constants) }),
    RenderPipelineDescriptor: d => ({ ...d, layout: d.layout ?? "auto" }),
    ComputePipelineDescriptor: d => ({ ...d, layout: d.layout ?? "auto" }),
  };

  // JS -> C -------------------------------------------------------------------------------------
  function write(kind, p, value) {
    const d = dv();
    if (typeof kind === "string") switch (kind) {
      case "u16": return d.setUint16(p, value, true);
      case "u32": return d.setUint32(p, value >>> 0, true);
      case "i32": return d.setInt32(p, value, true);
      case "f32": return d.setFloat32(p, value, true);
      case "f64": return d.setFloat64(p, value, true);
      case "bool": return d.setUint32(p, value ? 1 : 0, true);
      case "u64": case "flags": case "ptr": return d.setBigUint64(p, BigInt(value), true);
      case "str": case "nstr": case "ostr": return writeString(p, value);
    }
    switch (kind[0]) {
      case "enum": return d.setUint32(p, enumValue(kind[1], value), true);
      case "obj": return d.setBigUint64(p, BigInt(put(value)), true);
      case "struct": return writeStruct(kind[1], p, value);
    }
    throw new Error(`cannot write ${JSON.stringify(kind)}`);
  }
  // Writes the members `value` has (a dictionary or a JS API object such as GPUSupportedLimits);
  // the others keep what the program put there. Strings and arrays go to memory this host
  // allocates, which the program releases with the struct's FreeMembers.
  function writeStruct(name, p, value) {
    addr(p, STRUCTS[name].size, `WGPU${name}`);
    for (const [key, off, kind] of STRUCTS[name].members) {
      if (kind === "chain" || value[key] === undefined) continue;
      if (kind[0] === "array") {
        const list = [...value[key]].filter(v => kind[1][0] !== "enum" || FROM_JS[kind[1][1]].has(v));
        const size = sizeOf(kind[1]);
        const data = list.length ? alloc(list.length * size) : 0;
        list.forEach((v, i) => write(kind[1], data + i * size, v));
        dv().setBigUint64(p + off, BigInt(data), true);
        dv().setBigUint64(p + kind[2], BigInt(list.length), true);
      } else write(kind, p + off, value[key]);
    }
  }
  function freeMembers(name, p) {
    addr(p, STRUCTS[name].size, `WGPU${name}`);
    for (const [, off, kind] of STRUCTS[name].members) {
      if (kind === "str" || kind === "ostr" || kind === "nstr" || kind[0] === "array") free(ptrAt(p + off));
    }
  }

  // --- Futures and callbacks ------------------------------------------------------------------
  let nextFuture = 1;
  const futures = new Map(); // id -> { done, settled, internal? }
  const finished = []; // records the Jai side has not run yet
  let lastMessage = 0;
  function newFuture(promise, onSettled) {
    const id = nextFuture++;
    const entry = { done: false };
    entry.settled = Promise.resolve(promise).then(onSettled).then(() => { entry.done = true; });
    futures.set(id, entry);
    return [id, entry];
  }
  function startFuture(callback, info, promise, convert) {
    const cb = CALLBACKS[callback];
    const [id] = newFuture(
      Promise.resolve(promise).then(value => convert.ok(value), error => convert.error(error)),
      ([args, message]) => { finished.push({ kind: cb.id, mode: info.mode, future: id, info, args, message: message ?? "" }); },
    );
    return id;
  }
  function statusOf(callback, which) {
    const name = CALLBACKS[callback].args[0][1];
    for (const js of which) if (FROM_JS[name].has(js)) return FROM_JS[name].get(js);
    return 0;
  }
  // Default conversion: (status, the result object if the callback takes one, message).
  function generic(callback) {
    const kinds = CALLBACKS[callback].args;
    const object = kinds.some(k => k[0] === "obj");
    return {
      ok: value => {
        if (object && !value) return [[statusOf(callback, ["unavailable", "error"]), 0], "not available"];
        return [object ? [statusOf(callback, ["success"]), put(value)] : [statusOf(callback, ["success"])], ""];
      },
      error: error => [object ? [statusOf(callback, ["error", "validation-error", "internal-error"]), 0]
        : [statusOf(callback, ["error", "aborted"])], String(error?.message ?? error)],
    };
  }
  function waitedFutures(futuresPtr, count) {
    if (count) addr(futuresPtr, count * FUTURE_WAIT_INFO_SIZE, "WGPUFutureWaitInfo array");
    const list = [];
    for (let i = 0; i < count; i++) list.push(Number(dv().getBigUint64(futuresPtr + i * FUTURE_WAIT_INFO_SIZE, true)));
    return list;
  }
  // Whether dispatch `how` runs a record: wgpuInstanceProcessEvents runs AllowProcessEvents and
  // AllowSpontaneous callbacks; wgpuInstanceWaitAny the ones of its futures, whatever their mode,
  // and AllowSpontaneous ones; a frame (webgpu_present) only AllowSpontaneous ones.
  function runs(how, record, wanted) {
    if (record.mode === ALLOW_SPONTANEOUS) return true;
    if (how === PROCESS_EVENTS) return record.mode === ALLOW_PROCESS_EVENTS;
    if (how === WAIT_ANY) return wanted.has(record.future);
    return false;
  }
  function pollRecord(record, futuresPtr, count, how) {
    if (lastMessage) { free(lastMessage); lastMessage = 0; }
    const list = waitedFutures(futuresPtr, count);
    const wanted = new Set(list);
    list.forEach((id, i) => {
      const f = futures.get(id);
      if (!f || f.done) dv().setUint32(futuresPtr + i * FUTURE_WAIT_INFO_SIZE + 8, 1, true);
      if (f?.done && f.internal) futures.delete(id);
    });
    const index = finished.findIndex(r => runs(how, r, wanted));
    if (index < 0) return 0;
    const [r] = finished.splice(index, 1);
    futures.delete(r.future);
    const d = dv();
    bytes(record, RECORD_SIZE, "callback record").fill(0);
    d.setUint32(record, r.kind, true);
    d.setUint32(record + 4, r.mode, true);
    d.setBigUint64(record + 8, BigInt(r.future), true);
    d.setBigUint64(record + 16, r.info.callback, true);
    d.setBigUint64(record + 24, r.info.userdata1, true);
    d.setBigUint64(record + 32, r.info.userdata2, true);
    r.args.forEach((v, i) => d.setBigUint64(record + 40 + i * 8, BigInt(v), true));
    const message = encoder.encode(r.message);
    if (message.length) {
      lastMessage = alloc(message.length);
      new Uint8Array(memory.buffer, lastMessage, message.length).set(message);
      d.setBigUint64(record + 72, BigInt(lastMessage), true);
      d.setBigUint64(record + 80, BigInt(message.length), true);
    }
    return 1;
  }
  const turn = () => new Promise(resolve => {
    const channel = new MessageChannel();
    channel.port1.onmessage = () => { channel.port1.close(); resolve(); };
    channel.port2.postMessage(0);
  });
  function waitAny(futuresPtr, count, timeout) {
    const waitStatus = js => enumValue("WaitStatus", js);
    if (count === 0) return turn().then(() => 0);
    const list = waitedFutures(futuresPtr, count).map(id => futures.get(id));
    if (list.some(f => !f || f.done)) return waitStatus("success");
    const race = [...list.map(f => f.settled.then(() => "success"))];
    if (timeout !== MAX64) race.push(new Promise(r => setTimeout(() => r("timed-out"), Number(timeout / 1_000_000n))));
    // Even a zero timeout gives the page one turn, or a polling loop could never see a result.
    return Promise.race(race).then(js => turn().then(() => waitStatus(js)));
  }
  const nextFrame = () => new Promise(resolve =>
    typeof requestAnimationFrame === "function" ? requestAnimationFrame(() => resolve()) : setTimeout(resolve, 16));

  // --- Canvas and input -------------------------------------------------------------------------
  const events = [];
  let size = { width: canvas?.width ?? 0, height: canvas?.height ?? 0 };
  function pollEvent(p) {
    const e = events.shift();
    if (!e) return 0;
    const d = dv();
    bytes(p, 32, "Canvas_Event").fill(0);
    d.setUint32(p, e.type, true);
    d.setUint32(p + 4, e.key ?? 0, true);
    d.setUint32(p + 8, e.pressed ? 1 : 0, true);
    d.setUint32(p + 12, e.modifiers ?? 0, true);
    d.setInt32(p + 16, e.x ?? 0, true);
    d.setInt32(p + 20, e.y ?? 0, true);
    d.setUint32(p + 24, e.utf32 ?? 0, true);
    d.setUint32(p + 28, e.repeat ? 1 : 0, true);
    return 1;
  }

  // --- Calls ------------------------------------------------------------------------------------
  const floatArg = bits => (scratch.setUint32(0, Number(bits & 0xFFFF_FFFFn), true), scratch.getFloat32(0, true));
  const doubleArg = bits => (scratch.setBigUint64(0, bits, true), scratch.getFloat64(0, true));
  function arg(kind, slot) {
    if (typeof kind === "string") switch (kind) {
      case "u16": return Number(slot & 0xFFFFn);
      case "u32": return Number(slot & 0xFFFF_FFFFn);
      case "i32": return Number(BigInt.asIntN(32, slot));
      case "u64": return slot === MAX64 ? undefined : Number(slot); // WGPU_WHOLE_SIZE and friends
      case "flags": return Number(slot);
      case "f32": return floatArg(slot);
      case "f64": return doubleArg(slot);
      case "bool": return slot !== 0n;
      case "ptr": return addr(slot);
      case "str": case "nstr": case "ostr": return readString(addr(slot, 16, "WGPUStringView"));
    }
    switch (kind[0]) {
      case "enum": return ENUMS[kind[1]][Number(slot & 0xFFFF_FFFFn)];
      case "obj": return slot ? get(slot) : undefined;
      case "struct": return readStruct(kind[1], addr(slot));
      case "ptr": return slot ? readStruct(kind[1], addr(slot)) : undefined;
      case "callback": return readCallbackInfo(kind[1], addr(slot));
    }
    throw new Error(`cannot pass ${JSON.stringify(kind)}`);
  }
  function args(f, slots, start) {
    const out = [];
    let i = start;
    for (const kind of f.args) {
      if (kind[0] === "array") {
        const count = Number(slots[i++]); const data = slots[i++];
        out.push(data && count ? readArray(kind[1], addr(data), count) : []);
      } else out.push(arg(kind, slots[i++]));
    }
    return [out, i];
  }
  function result(kind, value) {
    if (kind === undefined || value === undefined) return undefined;
    if (typeof kind === "string") return kind === "bool" ? !!value : value;
    if (kind[0] === "obj") return put(value);
    if (kind[0] === "enum") return enumValue(kind[1], value);
    throw new Error(`cannot return ${JSON.stringify(kind)}`);
  }
  // A callback-taking function writes its WGPUFuture through the last slot.
  const writeFuture = (slot, id) => dv().setBigUint64(addr(slot, 8, "WGPUFuture"), BigInt(id), true);
  const success = 1; // WGPUStatus_Success
  const mappings = new Map(); // GPUBuffer -> [{ pointer, range }]
  const OVERRIDES = {
    wgpuCreateInstance: () => {
      if (!gpu) throw new Error("this browser has no WebGPU (navigator.gpu)");
      return put(gpu);
    },
    wgpuGetInstanceFeatures: ([p]) => { writeStruct("SupportedInstanceFeatures", addr(p), { features: ["timed-wait-any"] }); },
    wgpuGetInstanceLimits: ([p]) => { writeStruct("InstanceLimits", addr(p), { timedWaitAnyMaxCount: 64 }); return success; },
    wgpuHasInstanceFeature: ([feature]) => ENUMS.InstanceFeatureName[Number(feature)] === "timed-wait-any",
    wgpuInstanceHasWGSLLanguageFeature: ([, feature]) => gpu.wgslLanguageFeatures.has(ENUMS.WGSLLanguageFeatureName[Number(feature)]),
    wgpuInstanceGetWGSLLanguageFeatures: ([, p]) => { writeStruct("SupportedWGSLLanguageFeatures", addr(p), { features: [...gpu.wgslLanguageFeatures] }); return success; },
    wgpuInstanceCreateSurface: () => {
      if (!canvas) throw new Error("the page gave the program no canvas");
      return put({ canvas, context: canvas.getContext("webgpu") });
    },
    // The canvas offers plain formats only (no -srgb ones): webgpu_surface_format picks the same
    // kind natively.
    wgpuSurfaceGetCapabilities: ([, , p]) => {
      writeStruct("SurfaceCapabilities", addr(p), {
        usages: 0x1F, // copy src/dst, texture and storage binding, render attachment
        formats: [gpu.getPreferredCanvasFormat(), "rgba8unorm", "rgba16float"],
        presentModes: ["fifo"],
        alphaModes: ["opaque", "premultiplied"],
      });
      return success;
    },
    wgpuSurfaceConfigure: ([surface, p]) => {
      const { canvas: target, context } = get(surface);
      const c = readStruct("SurfaceConfiguration", addr(p));
      target.width = c.width; target.height = c.height;
      context.configure({
        device: c.device, format: c.format, usage: c.usage, viewFormats: c.viewFormats,
        alphaMode: c.alphaMode === "premultiplied" ? "premultiplied" : "opaque",
      });
      onSurface?.({ width: c.width, height: c.height });
    },
    wgpuSurfaceUnconfigure: ([surface]) => { get(surface).context.unconfigure(); onSurface?.(null); },
    wgpuSurfaceGetCurrentTexture: ([surface, p]) => {
      const texture = get(surface).context.getCurrentTexture();
      writeStruct("SurfaceTexture", addr(p), { texture, status: "success-optimal" });
    },
    // The page shows the canvas when the program waits for the next frame (webgpu_present).
    wgpuSurfacePresent: () => success,
    wgpuAdapterGetInfo: ([adapter, p]) => { writeInfo(get(adapter).info, addr(p)); return success; },
    wgpuDeviceGetAdapterInfo: ([device, p]) => { writeInfo(get(device).adapterInfo, addr(p)); return success; },
    wgpuAdapterGetLimits: ([adapter, p]) => { writeStruct("Limits", addr(p), get(adapter).limits); return success; },
    wgpuDeviceGetLimits: ([device, p]) => { writeStruct("Limits", addr(p), get(device).limits); return success; },
    wgpuAdapterGetFeatures: ([adapter, p]) => { writeStruct("SupportedFeatures", addr(p), { features: get(adapter).features }); },
    wgpuDeviceGetFeatures: ([device, p]) => { writeStruct("SupportedFeatures", addr(p), { features: get(device).features }); },
    wgpuAdapterHasFeature: ([adapter, f]) => get(adapter).features.has(ENUMS.FeatureName[Number(f)]),
    wgpuDeviceHasFeature: ([device, f]) => get(device).features.has(ENUMS.FeatureName[Number(f)]),
    wgpuAdapterRequestDevice: ([adapter, p, infoPtr, out]) => {
      const desc = p ? readStruct("DeviceDescriptor", addr(p)) : {};
      const { deviceLostCallbackInfo: lost, uncapturedErrorCallbackInfo: uncaptured, ...rest } = desc;
      const info = readCallbackInfo("RequestDevice", addr(infoPtr));
      const id = startFuture("RequestDevice", info, get(adapter).requestDevice(rest).then(device => {
        // Uncaptured errors may come at any time: they run like AllowSpontaneous callbacks.
        if (uncaptured?.callback) device.addEventListener("uncapturederror", event => {
          finished.push({ kind: CALLBACKS.UncapturedError.id, mode: ALLOW_SPONTANEOUS, future: 0, info: uncaptured,
            args: [put(device), errorType(event.error)], message: event.error.message });
        });
        if (lost?.callback) startFuture("DeviceLost", lost, device.lost, {
          ok: l => [[put(device), enumValue("DeviceLostReason", l.reason === "destroyed" ? "destroyed" : "unknown")], l.message],
          error: e => [[put(device), enumValue("DeviceLostReason", "unknown")], String(e)],
        });
        return device;
      }), generic("RequestDevice"));
      writeFuture(out, id);
    },
    wgpuDevicePopErrorScope: ([device, infoPtr, out]) => {
      const info = readCallbackInfo("PopErrorScope", addr(infoPtr));
      const id = startFuture("PopErrorScope", info, get(device).popErrorScope(), {
        ok: e => [[statusOf("PopErrorScope", ["success"]), errorType(e)], e?.message ?? ""],
        error: e => [[statusOf("PopErrorScope", ["error"]), 0], String(e?.message ?? e)],
      });
      writeFuture(out, id);
    },
    wgpuShaderModuleGetCompilationInfo: ([module, infoPtr, out]) => {
      const info = readCallbackInfo("CompilationInfo", addr(infoPtr));
      const id = startFuture("CompilationInfo", info, get(module).getCompilationInfo(), {
        ok: ci => {
          const p = alloc(STRUCTS.CompilationInfo.size);
          writeStruct("CompilationInfo", p, { messages: ci.messages.map(m => ({
            message: m.message, type: m.type, lineNum: m.lineNum, linePos: m.linePos, offset: m.offset, length: m.length,
          })) });
          return [[statusOf("CompilationInfo", ["success"]), p], ""];
        },
        error: e => [[statusOf("CompilationInfo", ["callback-cancelled"]), 0], String(e)],
      });
      writeFuture(out, id);
    },
    // Writes read the program's memory in place: no copy on this side.
    wgpuQueueWriteBuffer: ([queue, buffer, offset, data, size]) => {
      get(queue).writeBuffer(get(buffer), Number(offset), bytes(data, Number(size), "wgpuQueueWriteBuffer data"));
    },
    wgpuQueueWriteTexture: ([queue, dest, data, size, layout, extent]) => {
      get(queue).writeTexture(readStruct("TexelCopyTextureInfo", addr(dest)),
        bytes(data, Number(size), "wgpuQueueWriteTexture data"),
        readStruct("TexelCopyBufferLayout", addr(layout)), readStruct("Extent3D", addr(extent)));
    },
    // A mapped range lives in an ArrayBuffer the program cannot address: it gets a copy in its
    // memory, written back when it unmaps.
    wgpuBufferGetMappedRange: s => mapRange(s),
    wgpuBufferGetConstMappedRange: s => mapRange(s),
    wgpuBufferUnmap: ([buffer]) => {
      const b = get(buffer);
      for (const { pointer, range } of mappings.get(b) ?? []) {
        new Uint8Array(range).set(new Uint8Array(memory.buffer, pointer, range.byteLength));
        free(pointer);
      }
      mappings.delete(b);
      b.unmap();
    },
    wgpuBufferReadMappedRange: ([buffer, offset, data, size]) => {
      const range = get(buffer).getMappedRange(Number(offset), Number(size));
      bytes(data, Number(size), "wgpuBufferReadMappedRange data").set(new Uint8Array(range));
      return success;
    },
    wgpuBufferWriteMappedRange: ([buffer, offset, data, size]) => {
      const range = get(buffer).getMappedRange(Number(offset), Number(size));
      new Uint8Array(range).set(bytes(data, Number(size), "wgpuBufferWriteMappedRange data"));
      return success;
    },
    wgpuDeviceGetLostFuture: ([device, out]) => {
      const [id] = newFuture(get(device).lost, () => {});
      writeFuture(out, id);
    },
    // Host procedures of stdlib/WebGPU (wasm.jai, module.jai).
    jai_host_provides: ([name, length]) => Object.hasOwn(functions, utf8.decode(bytes(name, Number(length), "name"))),
    jai_webgpu_available: () => !!gpu,
    jai_webgpu_poll: ([record, list, count, how]) => pollRecord(addr(record), addr(list), Number(count), Number(how)),
    jai_webgpu_wait: ([list, count, timeout]) => waitAny(addr(list), Number(count), timeout),
    jai_webgpu_request_frame: () => {
      const [id, entry] = newFuture(nextFrame(), () => {});
      entry.internal = true; // no callback: forgotten once a wait or poll saw it complete
      return id;
    },
    jai_canvas_size: ([w, h]) => {
      dv().setUint32(addr(w, 4), size.width, true);
      dv().setUint32(addr(h, 4), size.height, true);
    },
    jai_canvas_poll_event: ([p]) => pollEvent(addr(p)),
  };
  function mapRange([buffer, offset, size]) {
    const b = get(buffer);
    const range = b.getMappedRange(Number(offset), size === MAX64 ? undefined : Number(size));
    const pointer = alloc(Math.max(range.byteLength, 1));
    new Uint8Array(memory.buffer, pointer, range.byteLength).set(new Uint8Array(range));
    if (!mappings.has(b)) mappings.set(b, []);
    mappings.get(b).push({ pointer, range });
    return pointer;
  }
  function writeInfo(info, p) {
    writeStruct("AdapterInfo", p, {
      vendor: info.vendor, architecture: info.architecture, device: info.device,
      description: info.description || [info.vendor, info.architecture].filter(Boolean).join(" ") || "browser WebGPU",
      backendType: "webgpu", adapterType: "unknown",
      subgroupMinSize: info.subgroupMinSize, subgroupMaxSize: info.subgroupMaxSize,
    });
  }
  function errorType(error) {
    if (!error) return enumValue("ErrorType", "no-error");
    const kind = { GPUValidationError: "validation", GPUOutOfMemoryError: "out-of-memory", GPUInternalError: "internal" }[error.constructor?.name];
    return enumValue("ErrorType", kind ?? "unknown");
  }

  function call(name, slots) {
    const override = OVERRIDES[name];
    if (override) return override(slots);
    const f = FUNCTIONS[name];
    if (f.free) { freeMembers(f.free, addr(slots[0])); return; }
    let i = 0;
    const self = f.self ? get(slots[i++]) : undefined;
    if (f.method === "addRef") { refs[Number(slots[0])]++; return; }
    if (f.method === "release") { release(slots[0]); return; }
    const [values, next] = args(f, slots, i);
    if (f.method === "setLabel") { self.label = values[0] ?? ""; return; }
    if (f.callback) {
      const info = readCallbackInfo(f.callback, addr(slots[next]));
      const id = startFuture(f.callback, info, (async () => self[f.method](...values))(), generic(f.callback));
      writeFuture(slots[next + 1], id);
      return;
    }
    // Getters of attributes: wgpuBufferGetSize -> buffer.size, wgpuDeviceGetQueue -> device.queue.
    if (/^get[A-Z]/.test(f.method) && f.args.length === 0 && typeof self[f.method] !== "function") {
      const property = f.method[3].toLowerCase() + f.method.slice(4);
      return result(f.ret, self[property]);
    }
    if (typeof self?.[f.method] !== "function") throw new Error(`not supported in the browser (no ${f.self ?? "GPU"}.${f.method})`);
    return result(f.ret, self[f.method](...values));
  }

  const functions = {};
  for (const name of [...Object.keys(FUNCTIONS), ...Object.keys(OVERRIDES)]) {
    functions[name] = (slots, mem) => { memory = mem; return call(name, slots); };
  }
  return {
    functions,
    output,
    // `event`: { type, key, pressed, modifiers, x, y, utf32, repeat } (stdlib/Input/wasm.jai).
    input(event) { events.push(event); },
    resize(width, height) {
      size = { width, height };
      events.push({ type: 5, x: width, y: height });
    },
  };
}
