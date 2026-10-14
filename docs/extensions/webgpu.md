# WebGPU

## What it is

`#import "Extensions/WebGPU"` (a [jaic extension](extensions.md), not an official Jai module) gives Jai programs the standard WebGPU C API (`webgpu.h`): every type, enum, struct, callback and `wgpu*` procedure, plus a few helpers for windows, adapters, mapping and error scopes. Natively the procedures call [wgpu-native](https://github.com/gfx-rs/wgpu-native); in the browser playground they are forwarded to the page's WebGPU. Both sides are generated from webgpu-headers' machine-readable spec (`webgpu.yml`), so one program runs in both places.

Examples: `examples/webgpu/triangle.jai` (window, render pipeline), `examples/webgpu/raymarch.jai` (full-screen fragment shader, uniforms, input), `examples/webgpu/compute.jai` (compute shader, storage buffers, mapping). The playground tour ends with a GPU stop (`examples/tour/gpu/raymarch_gpu.jai`).

## Where it works

| Where | Backend | Window surface | Tested |
|---|---|---|---|
| macOS arm64, x64 | Metal | `Window_Creation` window (`CAMetalLayer`) | CI (`stdlib-runtime`, both CPUs): headless test, and the window test presenting to a `Window_Creation` window on the runner VM's paravirtual Metal device, interpreted and built (each a skip if a runner has no Metal device) |
| Linux x64, arm64 | Vulkan | Xlib window (`WGPUSurfaceSourceXlibWindow`) | CI (`stdlib-runtime`): headless test, and the window test presenting to an X11 window on Xvfb, on Mesa's lavapipe, interpreted and built |
| Windows x64, arm64 | D3D12 | HWND (`WGPUSurfaceSourceWindowsHWND`) | CI (`windows-native.yml`): headless test, and the window test presenting to an HWND in the runner's desktop session, on WARP, interpreted and built |
| Browser playground | the browser's WebGPU | the playground's Render tab (`OffscreenCanvas` in the worker) | `tools/test_webgpu_host.mjs` against a mock WebGPU (CI); Chromium by hand |

In the browser a program needs WebGPU in workers and JSPI (`WebAssembly.Suspending`). As of this writing:

| Browser | WebGPU in workers | JSPI | Playground |
|---|---|---|---|
| Chrome, Edge (137+) | yes | yes | draws |
| Firefox | Windows (141+), others behind a pref | not by default | `webgpu_available()` is false; the Render tab says so |
| Safari | 26+ | not by default | same as Firefox |

A program should check `webgpu_available()` first: it is true natively and, in the browser, only when the page passed a WebGPU host and the browser has `navigator.gpu`. The tour's GPU stop prints a skip line instead of failing; natively it opens a `Window_Creation` window of its own and skips only without a window (no display), a surface or an adapter.

## How it works

```
webgpu.yml (pinned) --tools/webgpu_gen.py--> stdlib/Extensions/WebGPU/generated.jai        types + #foreign procs
                                          --> stdlib/Extensions/WebGPU/generated_wasm.jai   callback dispatch (browser)
                                          --> crates/jai-wasm/js/webgpu_bindings.generated.mjs
                                                 struct layouts, enum strings, call descriptors
```

### Helpers (`module.jai`)

- `webgpu_create_surface(instance, window)`: a surface for a `Window_Creation` window. macOS gives the content view a `CAMetalLayer` (`macos.jai`), Windows chains the HWND and module instance (`windows.jai`), Linux the X11 display and window (`linux.jai`). In the browser it is the page's canvas. Natively a null window gives a null surface (and a logged error): check it, because wgpu-native aborts the process (a Rust panic it cannot unwind through the C API, `invalid surface`) when most surface calls get null. The headless test checks this.
- `webgpu_surface_format(surface, adapter, srgb := false)`: the first format the surface supports whose sRGB-ness matches `srgb`, else the preferred one; `.Undefined` for a null surface or adapter. Shaders that write already-encoded colours (most demos, which tone-map and gamma-correct themselves) want `srgb = false`; otherwise the hardware encodes a second time and the picture looks washed out. macOS prefers `bgra8unorm-srgb`, browsers `bgra8unorm` or `rgba8unorm`, so without the helper the same shader looks different in each. `webgpu_format_is_srgb(format)` tells the two apart.
- `webgpu_request_adapter`, `webgpu_request_device`: request and wait (`webgpu_wait`) until the callback ran. The device prints uncaptured errors unless the descriptor brings its own callback.
- `webgpu_map_buffer(instance, buffer, mode, offset, size)`: map and wait; true on success. `WGPU_WHOLE_MAP_SIZE` is replaced by the remaining size, because wgpu-native rejects it.
- `webgpu_pop_error_scope(instance, device) -> WGPUErrorType, message`: pop and wait; the message is a temporary copy.
- `webgpu_present(surface)`: `wgpuSurfacePresent`; in the browser, then wait for the next animation frame.
- `WEBGPU_SURFACE_OCCLUDED`: wgpu-native's extra `wgpuSurfaceGetCurrentTexture` status (`0x0003_0001`, from its `wgpu.h`, so not in the generated enum). Its Metal backend returns it while the window is not visible, the first frame after `create_window` included, instead of blocking for up to a second; skip that frame (handle events, sleep a little) rather than reconfigure. The triangle example and the window test do.
- `webgpu_window_size`, `to_string_view`, `to_string`.

Callback messages (`WGPUStringView`) are only valid during the callback, so the helpers copy them (`Message_Copy`).

### Native

`generated.jai` declares each procedure `#foreign libwgpu` (`#library "libwgpu_native"`). jaic looks for it in its native-libs directory ([native libraries](../tools/native-libs.md)): `jaic -interp` loads the shared library (`.dylib`, `.so`, `.dll`), `jaic` links the static one (`.a`, `.lib`) by path, so built programs do not need wgpu-native next to them. The system libraries the static archive needs are `link_always` in `macos.jai` (Metal, QuartzCore, ...), `linux.jai` (`libm`, `libdl`, `libpthread`, `libgcc_s`) and `windows.jai` (`d3dcompiler`, `ws2_32`, `userenv`, `bcrypt`, `ntdll`, ...).

Release archives ship wgpu-native in `artifacts/native-libs/<platform>/` next to `stdlib/` ([releases](../tools/releases.md)). In a checkout, `python3 tools/fetch_wgpu_native.py` downloads it there.

### Browser

A program runs in the interpreter inside `jai_wasm.wasm`, in a worker that owns an `OffscreenCanvas`.

1. A `#foreign` procedure the sandbox does not implement goes to `crates/jai-wasm/src/host_bridge.rs`, which offers it to the page by name (`jai_host.call` import) with its arguments as 64-bit slots. By-value structs and aggregate results are pointers (the interpreter's C ABI lowering). Interpreter pointers are addresses in the module's memory, so the JS side reads structs in place and `wgpuQueueWriteBuffer` hands WebGPU a view of that memory without a copy. Every address and length is range-checked against the memory first (`span` in `webgpu_host.mjs`, `view` in `engine.mjs`): an out-of-range pointer is a `RangeError` that ends the run, never a read of something else.
2. `crates/jai-wasm/js/webgpu_host.mjs` interprets the generated tables: it reads C structs into the dictionaries the JS API takes (members become camelCase keys, enums their WebIDL strings, chained structs merge into the parent, sentinels such as `WGPU_WHOLE_SIZE` become `undefined`) and calls the method of the same name (`wgpuDeviceCreateBuffer` -> `device.createBuffer`). Objects cross as small integer handles with reference counts. `OVERRIDES` covers where the APIs differ: surfaces (the canvas context), mapping (copies between the mapped `ArrayBuffer` and module memory, written back on unmap), `writeBuffer`/`writeTexture` argument order, getters that return structs, device callbacks, `popErrorScope`.
3. A host function may return a promise. Then the bridge calls the `jai_host.wait` import, a `WebAssembly.Suspending` function: JSPI suspends the whole module, Rust and interpreter stacks included, until the promise settles (`engine.mjs` runs such programs with `playAsync`). After every wait the run's block budget is refilled (`Host::refill_budget`, so for a drawing program the budget bounds the work per frame), the virtual clock advances by the real time that passed, and new output is streamed to the page.
4. JS cannot call a Jai procedure, so callbacks are queued: a callback-taking function returns a `WGPUFuture` and records the settled result; `wgpuInstanceProcessEvents` and `wgpuInstanceWaitAny` are Jai procedures on WASM (`wasm.jai`) that wait (`jai_webgpu_wait`), pull finished records (`jai_webgpu_poll`) and call them (`webgpu_dispatch_record`, generated per callback type).
5. Frames: `webgpu_present` waits on a frame future (`jai_webgpu_request_frame`, an animation frame). The browser shows the canvas when the worker yields. The page learns the canvas size from `onSurface` (called on configure and unconfigure).
6. Input: the page posts canvas events to the worker; `stdlib/Input/wasm.jai` reads them with `jai_canvas_poll_event` and queues keyboard (`Key_Code`), text, mouse buttons, wheel, focus and resize events.

The bridge has no `unsafe` blocks: the host imports are declared `safe fn` with addresses as `usize` in one `#[allow(unsafe_code)]` module, and the memory the page asks for (`jai_host_alloc`) is an owned, zeroed `Box<[u128]>` kept in a table until `jai_host_free`.

### Callback modes

| Mode | Native (wgpu-native) | Browser |
|---|---|---|
| `WaitAnyOnly` | only inside `wgpuInstanceWaitAny` on that future | same |
| `AllowProcessEvents` | inside `wgpuInstanceProcessEvents` (or `WaitAny`) | same |
| `AllowSpontaneous` | may run on any thread at any time, also inside another `wgpu*` call | inside `ProcessEvents`, `WaitAny` or `webgpu_present`, never anywhere else |

The browser cannot run Jai code from JS, so "spontaneous" means "at the next point where the program waits" (`how` argument of `jai_webgpu_poll`: `PROCESS_EVENTS` dispatches modes 2 and 3, `WAIT_ANY` dispatches the futures waited on plus mode 3, `SPONTANEOUS` after a frame dispatches mode 3 only). A program that relies on spontaneous callbacks while it spins without calling any of these never sees them in the browser; natively it does. Uncaptured errors use `AllowSpontaneous` in the browser.

### Waiting and threads

Natively a wait is wgpu-native's `ProcessEvents` in a loop (`webgpu_wait`, which sleeps 1 ms between polls after the first hundred) or `WaitAny`; other interpreted threads keep running because foreign calls release the interpreter.

In the browser all threads share one host thread ([inline threads](../compiler/interpreter-threads.md#inline-threads-sandbox-host-browser)). A wait would suspend the whole module and stop the other threads, so `wasm.jai` first calls `jai_sched_yield_for_wait`: when other threads can run, the waiting thread suspends with `Wait::Host` and they go first. A `WaitAny` with a timeout waits in slices of 2 ms (`WAIT_SLICE_NS`) while other threads may run, so a thread mapping a buffer does not starve a worker thread and the reverse. When no other thread can run, the module suspends for the whole wait.

## Tests

| What | Where | Runs |
|---|---|---|
| `tests/webgpu/headless.jai`: render a triangle into a texture, copy to a mappable buffer, check pixels; compute shader (squares of 256 numbers); texture upload and readback; an invalid shader in an error scope must be a validation error; no window gives a null surface and `webgpu_surface_format` `.Undefined` (not wgpu-native's abort) | `tools/test_webgpu_native.py` (interpreted and built; exit 77 = no adapter, retried with the fallback adapter) | CI `stdlib-runtime` on Linux (lavapipe, adapter required) and macOS (Metal, skip without an adapter); `windows-native.yml` on x64 and arm64 (WARP, adapter required) |
| `tests/webgpu/window.jai`: open a `Window_Creation` window, create its surface (with the adapter asked for one compatible with it), configure it (FIFO), and present `frames N` frames (default 60; CI 120); every frame's surface status must be a success (an `Outdated`/`Timeout` reconfigures, at most 10 times; macOS's `WEBGPU_SURFACE_OCCLUDED` waits), every present `.Success`, the texture the configured size, and no error in a validation or out-of-memory scope or the device's uncaptured-error callback. Where the surface allows `CopySrc` in `BGRA8Unorm`/`RGBA8Unorm`, the last frame is copied out before it is presented and its middle must be the triangle, its corners the clear color. Exit 2: no window (no display) | `tools/test_webgpu_native.py --test window --frames N` (interpreted and built; needs a display, `xvfb-run` on Linux) | CI `stdlib-runtime` on Linux under Xvfb (lavapipe, adapter required) and macOS (skip without an adapter); `windows-native.yml` on x64 and arm64 (WARP, adapter required) |
| Host bridge against a mock WebGPU: struct marshalling, handles and reference counts, strings, chained structs, futures and callback modes, promise suspension, out-of-range pointers; end-to-end `tests/webgpu/bridge.jai` in the real engine (threads keep running during waits) | `tools/test_webgpu_host.mjs` (`node --experimental-wasm-jspi tools/test_webgpu_host.mjs`, run directly so the JSPI flag reaches the tests; set `JAI_WASM_DIR` for the end-to-end tests) | CI `scripting-wasm` |
| Generator rules (enum strings, struct layouts) | `tools/test_webgpu_gen.py` | CI with the other `tools/test_*.py` |
| Packaging: the browser bundle has both JS files and loads them; release archives build the triangle and run the headless test from the archive | `tools/check_browser_release.mjs`, `release.yml` smoke test | release workflows |
| Bridge memory (`jai_host_alloc`/`free`) | `cargo test -p jai-wasm` | CI |

Run locally:

```sh
python3 tools/test_webgpu_native.py --jaic target/debug/jaic [--require-adapter]
python3 tools/test_webgpu_native.py --jaic target/debug/jaic --test window --frames 120   # opens a window
JAI_WASM_DIR=<built bundle> node --experimental-wasm-jspi tools/test_webgpu_host.mjs
```

## How to change it

- New webgpu-headers revision: update `tools/webgpu.json` (revision and `webgpu.yml` sha256, and the wgpu-native tag and asset hashes built from it; `fetch_wgpu_native.py` refuses a release whose bundled `webgpu.yml` differs, ignoring line endings), rerun `python3 tools/webgpu_gen.py`, and check with `--check-header webgpu.h` (same revision), which renders every struct, function, enum value and callback back to C and compares. Then review `OVERRIDES` and `STRUCT_FIXUPS` in `webgpu_host.mjs` against the JS API. The generator emits jaifmt-formatted Jai; keep it that way so `jaifmt --check stdlib` stays clean.
- C and JS APIs differ somewhere new: add an `OVERRIDES` entry (whole call) or a `STRUCT_FIXUPS` entry (one dictionary), not generator special cases.
- New wgpu-native release on a platform: if `jaic` fails with undefined symbols, add the system library to that platform's `link_always` list.
- Fragile spots:
  - webgpu.h is still pre-1.0 and changes between revisions (string views, future-based callbacks and `WGPUBool` all changed in 2024-25); wgpu-native lags webgpu-headers, so the pin must be the revision wgpu-native was built from.
  - wgpu-native deviates from the header in small ways (`WGPU_WHOLE_MAP_SIZE`, empty adapter descriptions on Metal); the helpers paper over the known ones.
  - Browser support depends on JSPI, which only Chromium ships; the JS API also evolves (`GPUAdapter.info`, `requestAdapterInfo` removal).
  - wgpu-native panics (aborting the process, since the panic cannot unwind out of the C call) on `wgpuDevicePushErrorScope(device, .Internal)`; use `.Validation` and `.OutOfMemory` scopes and catch internal errors with the uncaptured-error callback.
  - `WGPUBool` is 4 bytes, not Jai's `bool`. Native callbacks are `#c_call`; use `push_context` to print from one.
  - In the browser, a loop that never calls `ProcessEvents`, `WaitAny` or `webgpu_present` never yields to the page, so nothing settles and nothing is drawn; the run's block budget eventually stops it.

## Configuration

```sh
python3 tools/webgpu_gen.py [--yml webgpu.yml] [--check-header webgpu.h]
python3 tools/fetch_wgpu_native.py [--platform windows-arm64] [--out DIR] [--force]
JAIC_NATIVE_LIBS=<main checkout>/artifacts/native-libs/macos-arm64 jaic examples/webgpu/triangle.jai -interp  # from a worktree
```

Browser: the bundle ships `webgpu_host.mjs` and `webgpu_bindings.generated.mjs` next to `engine.mjs`. An embedder passes `createEngine(bytes, { host: createWebGPUHost({ canvas, output, onSurface }) })`, runs with `playAsync`, and forwards events with `host.input(event)` / `host.resize(w, h)`. Without a host (or without `navigator.gpu`) `webgpu_available()` is false and the `wgpu*` calls are not provided.

## Dependencies

- webgpu-headers `webgpu.yml` (BSD-3-Clause), pinned in `tools/webgpu.json`, downloaded to `artifacts/webgpu/` by the generator; not vendored.
- wgpu-native release zips (MIT/Apache-2.0), pinned by sha256; shipped in release archives.
- Browser: WebGPU in workers (`OffscreenCanvas`, `navigator.gpu`), JSPI, `requestAnimationFrame` in workers.
- Internal: `interp::Host` (`refill_budget`), inline thread scheduler (`jai_sched_yield_for_wait`), `jai-wasm` `host_bridge`, `stdlib/Input`, `Window_Creation`.
