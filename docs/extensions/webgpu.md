# WebGPU (`Extensions/WebGPU`)

## What it is

The standard WebGPU C API (`webgpu.h`): every type, enum, struct, callback and `wgpu*` procedure, plus helpers for windows, adapters, mapping and error scopes. Natively the procedures call [wgpu-native](https://github.com/gfx-rs/wgpu-native); in the browser playground they forward to the page's WebGPU. Both sides are generated from webgpu-headers' `webgpu.yml`, so one program runs in both places.

```jai
#import "Extensions/WebGPU";

if !webgpu_available() return;                       // true natively; in the browser only with a WebGPU host
surface := webgpu_create_surface(instance, window);  // null window = null surface: check it
format  := webgpu_surface_format(surface, adapter);  // pass srgb = true only if the shader writes linear colour
// ... adapter/device via webgpu_request_adapter / webgpu_request_device, then render
webgpu_present(surface);
```

Examples: `examples/webgpu/triangle.jai` (window, pipeline), `raymarch.jai` (fragment shader, uniforms, input), `compute.jai` (storage buffers, mapping); the playground tour ends with `examples/tour/gpu/raymarch_gpu.jai`.

| Where | Backend | Surface |
| --- | --- | --- |
| macOS | Metal | `Window_Creation` window (`CAMetalLayer`) |
| Linux | Vulkan | Xlib window |
| Windows | D3D12 | HWND |
| Browser playground | browser WebGPU | the Render tab (`OffscreenCanvas` in the worker); needs WebGPU in workers and JSPI, so Chrome/Edge 137+ draw and Firefox/Safari report `webgpu_available()` false |

## How it works

```
webgpu.yml (pinned) --tools/webgpu_gen.py--> stdlib/Extensions/WebGPU/generated.jai       types + #foreign procs
                                          --> stdlib/Extensions/WebGPU/generated_wasm.jai  callback dispatch (browser)
                                          --> crates/jai-wasm/js/webgpu_bindings.generated.mjs   struct layouts, enum strings
```

**Helpers** (`module.jai`): `webgpu_create_surface` (per-OS code in `macos.jai`, `windows.jai`, `linux.jai`; wgpu-native aborts on most calls with a null surface, so check); `webgpu_surface_format` (first format whose sRGB-ness matches; macOS prefers `bgra8unorm-srgb`, browsers `bgra8unorm`, so without it the same shader looks different); `webgpu_request_adapter/device`, `webgpu_map_buffer` (replaces `WGPU_WHOLE_MAP_SIZE`, which wgpu-native rejects), `webgpu_pop_error_scope`, `webgpu_present` (in the browser also waits for the next animation frame), `webgpu_window_size`. `WEBGPU_SURFACE_OCCLUDED` is wgpu-native's extra status for a non-visible Metal window: skip the frame, do not reconfigure. Callback strings are only valid during the callback, so helpers copy them.

**Native:** `generated.jai` declares procedures `#foreign libwgpu`. `jaic -interp` loads the shared library from the native-libs directory ([native libraries](../tools/native-libs.md)); built programs link the static one (`.a`/`.lib`), with system libraries listed `link_always` in `macos.jai`, `linux.jai`, `windows.jai`. Release archives ship wgpu-native in `native-libs/<platform>/` ([releases](../tools/releases.md)); in a checkout run `python3 tools/fetch_wgpu_native.py`.

**Browser:** the program runs in the interpreter inside `jai_wasm.wasm` in a worker owning an `OffscreenCanvas`.

1. A `#foreign` procedure the sandbox lacks goes to `crates/jai-wasm/src/host_bridge.rs`, which offers it to the page by name (`jai_host.call`) with 64-bit slot arguments. Structs are pointers into module memory, which JS reads in place (`wgpuQueueWriteBuffer` hands WebGPU a view without a copy); every address and length is range-checked (`span` in `webgpu_host.mjs`, `view` in `engine.mjs`).
2. `crates/jai-wasm/js/webgpu_host.mjs` interprets the generated tables: C structs become JS dictionaries (camelCase keys, enum strings, chained structs merged, sentinels like `WGPU_WHOLE_SIZE` become `undefined`) and calls the same-named method. Objects cross as reference-counted integer handles. `OVERRIDES` covers API differences (surfaces, mapping, `writeBuffer` argument order, struct getters, device callbacks, `popErrorScope`).
3. A host function may return a promise: the bridge calls `jai_host.wait`, a `WebAssembly.Suspending` function, so JSPI suspends the whole module until it settles. After each wait the run's block budget is refilled and output streams to the page.
4. JS cannot call Jai, so callbacks are queued: a callback-taking function returns a `WGPUFuture`; `wgpuInstanceProcessEvents`/`WaitAny` are Jai procedures in `wasm.jai` that wait (`jai_webgpu_wait`), poll finished records and dispatch them (`webgpu_dispatch_record`).
5. Input: the page posts canvas events; `stdlib/Input/wasm.jai` reads them with `jai_canvas_poll_event`.

**Callback modes:** `WaitAnyOnly` runs only in `WaitAny` on that future; `AllowProcessEvents` in `ProcessEvents` or `WaitAny`; `AllowSpontaneous` may run on any thread natively, but in the browser only inside `ProcessEvents`, `WaitAny` or `webgpu_present`. A browser loop that never calls one of those never yields to the page, so nothing settles or draws.

**Waiting and threads:** natively `webgpu_wait` loops `ProcessEvents` (sleeping 1 ms after the first hundred polls); other interpreted threads keep running. In the browser all threads share one host thread ([inline threads](../compiler/interpreter.md)), so `wasm.jai` first calls `jai_sched_yield_for_wait`; a timed `WaitAny` waits in 2 ms slices while other threads can run.

## How to change it

- **New webgpu-headers revision:** update `tools/webgpu.json` (revision and `webgpu.yml` sha256, wgpu-native tag and asset hashes built from it; `fetch_wgpu_native.py` refuses a release whose bundled `webgpu.yml` differs), run `python3 tools/webgpu_gen.py`, check with `--check-header webgpu.h`, then review `OVERRIDES` and `STRUCT_FIXUPS` in `webgpu_host.mjs`. Generated Jai must stay jaifmt-formatted.
- **C and JS APIs differ somewhere new:** add an `OVERRIDES` (whole call) or `STRUCT_FIXUPS` (one dictionary) entry, not generator special cases.
- **New wgpu-native release with undefined symbols:** add the system library to that platform's `link_always`.
- Gotchas: webgpu.h is pre-1.0 and wgpu-native lags it, so the pin must be the revision wgpu-native was built from. wgpu-native panics (aborting) on `wgpuDevicePushErrorScope(device, .Internal)`: use `.Validation`/`.OutOfMemory` and the uncaptured-error callback. `WGPUBool` is 4 bytes. Native callbacks are `#c_call`: use `push_context` to print. JSPI is Chromium-only.

## Configuration

```sh
python3 tools/webgpu_gen.py [--yml webgpu.yml] [--check-header webgpu.h]
python3 tools/fetch_wgpu_native.py [--platform windows-arm64] [--out DIR] [--force]
JAIC_NATIVE_LIBS=<checkout>/target/native-libs/macos-arm64 jaic examples/webgpu/triangle.jai -interp   # from a worktree
```

Browser embedders pass `createEngine(bytes, { host: createWebGPUHost({ canvas, output, onSurface }) })`, run with `playAsync`, and forward events with `host.input(event)`/`host.resize(w, h)`. Without a host or `navigator.gpu`, `webgpu_available()` is false.

## Dependencies and tests

webgpu-headers `webgpu.yml` (pinned, downloaded to `target/webgpu/`, not vendored), wgpu-native release zips (pinned by sha256), `interp::Host`, the inline thread scheduler, `jai-wasm` `host_bridge`, `stdlib/Input`, `Window_Creation`. Tests run in the `stdlib-runtime`, `windows-native` and `scripting-wasm` jobs of `.github/workflows/ci.yml`:

- `stdlib/Extensions/WebGPU/tests/headless.jai` (triangle into a texture and pixel check, compute, texture readback, invalid shader in an error scope, null surface) and `window.jai` (window, surface, N presented frames) via `python3 tools/test_webgpu_native.py --jaic target/debug/jaic [--test window --frames 120] [--require-adapter]` (exit 77 means no adapter).
- Host bridge against a mock WebGPU plus `stdlib/Extensions/WebGPU/tests/bridge.jai`: `node --experimental-wasm-jspi tools/test_webgpu_host.mjs` (set `JAI_WASM_DIR` for the end-to-end part).
- Generator rules: `python3 tools/test_webgpu_gen.py`. Browser bundle packaging: `tools/check_browser_release.mjs`. Bridge memory: `cargo test -p jai-wasm`.
