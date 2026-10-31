# WebGPU

The standard WebGPU C API (`webgpu.h`): every type, enum, struct, callback and `wgpu*` procedure, plus helpers for windows, adapters, mapping and error scopes. Natively the procedures call [wgpu-native](https://github.com/gfx-rs/wgpu-native); in the browser playground they forward to the page's WebGPU. Both sides are generated from webgpu-headers' `webgpu.yml`, so code that only talks to the GPU (`examples/webgpu-compute`) runs in both places. Surfaces differ: `webgpu_create_surface` takes a `Window_Creation` window and is native only, while the playground binds `wgpuInstanceCreateSurface(instance, null)` to its own canvas.

```jai
#import "Extensions/WebGPU";

if !webgpu_available() return;                       // true natively; in the browser only with a WebGPU host
surface := webgpu_create_surface(instance, window);  // null window = null surface: check it
format  := webgpu_surface_format(surface, adapter);  // pass srgb = true only if the shader writes linear colour
// ... adapter/device via webgpu_request_adapter / webgpu_request_device, then render
webgpu_present(surface);
```

Examples: `webgpu-triangle` and `webgpu-raymarch` (native only), `webgpu-compute`, and `textured-cube` (depth, a PNG loaded with [Assets](assets.md) and decoded with [Image](image.md)), which runs natively and in the playground.

| Where | Backend | Surface |
| --- | --- | --- |
| macOS | Metal | `Window_Creation` window (`CAMetalLayer`) |
| Linux | Vulkan | Xlib window |
| Windows | D3D12 | HWND |
| Browser playground | browser WebGPU | the Render tab (`OffscreenCanvas` in the worker); needs WebGPU in workers and JSPI, so Chrome/Edge 137+ draw and Firefox/Safari report `webgpu_available()` false |

## How it works

```
webgpu.yml (pinned) --tools/bindings/webgpu_gen.py--> stdlib/Extensions/WebGPU/generated.jai       types + #foreign procs
                                          --> stdlib/Extensions/WebGPU/generated_wasm.jai  callback dispatch (browser)
                                          --> crates/jai-wasm/js/webgpu_bindings.generated.mjs   struct layouts, enum strings
```

- **Helpers** (`module.jai`): `webgpu_create_surface` (per-OS code in `macos.jai`, `windows.jai`, `linux.jai`; wgpu-native aborts on most calls with a null surface, so check), `webgpu_surface_format` (macOS prefers an sRGB format, browsers don't, so without it the same shader looks different), `webgpu_request_adapter/device`, `webgpu_map_buffer`, `webgpu_read_texture` (copies a 2D texture to the CPU as packed rows; it waits, so use it for screenshots and tests), `webgpu_pop_error_scope`, `webgpu_present`, `webgpu_window_size` (native) and `webgpu_canvas_size` (browser). `WEBGPU_SURFACE_OCCLUDED` is wgpu-native's status for a non-visible Metal window: skip the frame, don't reconfigure. Callback strings are only valid during the callback, so helpers copy them.
- **Native.** `generated.jai` declares procedures `#foreign libwgpu`. `jaic -interp` loads the shared library from the native-libs directory ([native libraries](../tools/native-libs.md)); built programs link the static one, with system libraries listed `link_always` in the per-OS files. Release archives ship wgpu-native ([releases](../tools/releases.md)); in a checkout run `PYTHONPATH=tools python3 tools/bindings/fetch_wgpu_native.py`.
- **Browser.** The program runs in the interpreter inside `jai_wasm.wasm` in a worker owning an `OffscreenCanvas`.
  1. A `#foreign` procedure the sandbox lacks goes to `crates/jai-wasm/src/host_bridge.rs`, which offers it to the page by name with 64-bit slot arguments. Structs are pointers into module memory that JS reads in place; every address and length is range-checked.
  2. `crates/jai-wasm/js/webgpu_host.mjs` interprets the generated tables: C structs become JS dictionaries and it calls the same-named method. Objects cross as reference-counted integer handles. `OVERRIDES` covers API differences (surfaces, mapping, `writeBuffer` argument order, device callbacks).
  3. A host function may return a promise; the bridge calls `jai_host.wait`, a `WebAssembly.Suspending` function, so JSPI suspends the whole module until it settles.
  4. JS can't call Jai, so callbacks are queued: a callback-taking function returns a `WGPUFuture`, and `wgpuInstanceProcessEvents`/`WaitAny` in `wasm.jai` wait, poll finished records and dispatch them.
  5. Input: the page posts canvas events, which `stdlib/Input/wasm.jai` reads.
- **Callback modes.** `WaitAnyOnly` runs only in `WaitAny` on that future; `AllowProcessEvents` in `ProcessEvents` or `WaitAny`; `AllowSpontaneous` may run on any thread natively but in the browser only inside `ProcessEvents`, `WaitAny` or `webgpu_present`. A browser loop that calls none of those never yields to the page.
- **Waiting.** Natively `webgpu_wait` loops `ProcessEvents`. In the browser all threads share one host thread ([inline threads](../compiler/interpreter.md)), so a timed `WaitAny` waits in 2 ms slices while other threads run.

## How to change it

- New webgpu-headers revision: update `tools/bindings/webgpu.json` (revision, `webgpu.yml` sha256, wgpu-native tag and asset hashes built from it), run `PYTHONPATH=tools python3 tools/bindings/webgpu_gen.py`, check with `--check-header webgpu.h`, then review `OVERRIDES` and `STRUCT_FIXUPS` in `webgpu_host.mjs`. Generated Jai must stay jaifmt-formatted. The pin must be the revision wgpu-native was built from, since webgpu.h is pre-1.0.
- C and JS APIs differ somewhere new: add an `OVERRIDES` (whole call) or `STRUCT_FIXUPS` (one dictionary) entry, not generator special cases.
- New wgpu-native release with undefined symbols: add the system library to that platform's `link_always`.
- Gotchas: wgpu-native aborts on `wgpuDevicePushErrorScope(device, .Internal)`; use `.Validation`/`.OutOfMemory` and the uncaptured-error callback. `WGPUBool` is 4 bytes. Native callbacks are `#c_call`: use `push_context` to print. JSPI is Chromium-only.

Commands:

```sh
PYTHONPATH=tools python3 tools/bindings/webgpu_gen.py [--yml webgpu.yml] [--check-header webgpu.h]
PYTHONPATH=tools python3 tools/bindings/fetch_wgpu_native.py [--platform windows-arm64] [--out DIR] [--force]
JAIC_NATIVE_LIBS=<checkout>/target/native-libs/macos-arm64 jaic examples/webgpu-triangle/main.jai -interp   # from a worktree
```

Browser embedders pass `createEngine(bytes, { host: createWebGPUHost({ canvas, output, onSurface }) })`, run with `playAsync`, and forward events with `host.input(event)`/`host.resize(w, h)`. Without a host or `navigator.gpu`, `webgpu_available()` is false.

## Tests

They run in the `stdlib-runtime`, `windows-native` and `scripting-wasm` jobs of `.github/workflows/ci.yml`.

- `stdlib/Extensions/WebGPU/tests/driven/headless.jai` (triangle into a texture with a pixel check, compute, readback, invalid shader in an error scope, null surface) and `window.jai` via `PYTHONPATH=tools python3 tools/bindings/run_webgpu_native.py --jaic target/debug/jaic [--test window --frames 120] [--require-adapter]` (exit 77 means no adapter).
- Host bridge against a mock WebGPU plus `tests/driven/bridge.jai`: `node --experimental-wasm-jspi tools/bindings/test_webgpu_host.mjs` (set `JAI_WASM_DIR` for the end-to-end part).
- Generator rules: `PYTHONPATH=tools python3 tools/bindings/test_webgpu_gen.py`. Browser bundle packaging: `tools/browser/check_browser_release.mjs`. Bridge memory: `cargo test -p jai-wasm`.
