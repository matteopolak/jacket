# Friction log

Rough edges hit while writing game3d. Fixed ones name the commit subject; open ones say so.

## Fixed in the stdlib

Nothing yet.

## Open: stdlib

- `meshoptimizer`: the declarations do not match the library that gets linked. The module follows the 0.11-era inventory (`MESHOPTIMIZER_VERSION :: 110`), but `tools/native/native-libs.json` pins a 1.x build. `meshopt_simplify` is declared with 8 arguments and the library takes 10 (`options`, `result_error`), so a call reads garbage for the last two and the library's own assert fires (`options & meshopt_SimplifyInternalSolve`). `meshopt_simplifySloppy` gained a vertex lock mask, an error limit and `result_error`. `meshopt_buildMeshlets`, `meshopt_computeMeshletBounds` and `meshopt_Meshlet` have different shapes in the 1.x header. Only the functions the module's own test uses (cache, fetch, remap, codecs, strips) are safe today. The game declares the real 10-argument `meshopt_simplify` itself (`simplify_triangles` in `game/meshes.jai`). The proper fix is to re-pin the library to the revision the inventory describes, or to give those functions wrapper bodies; the binding generator only accepts `#c_call` wrappers over a `#foreign` declaration, so the wrappers are not a one-file change.
- `Window_Type` is a pointer on macOS and an integer on Linux, so "no window" is `null` in one place and `0` in another. A zero value of the type (`no_window: Window_Type;`) is the portable spelling; it is not written down anywhere.
- `Extensions/WebGPU` has no texture readback helper. The copy to a buffer, the 256-byte row rule, the map and the copy out are about thirty lines that `tests/driven/headless.jai` and the game both carry.
- `Iprof`: the module has no overview. The manual mode (`MANUAL_MODE = true`) needs `Prof_init_highlevel`, then a name and an `initted` flag for each zone (the tests also set its `hash`), then `zone_begin`/`zone_end`, `update(true)` once per frame and `create_report`/`log_text_report` to read it. That is only written down in the module's tests. Zones are global to the process, so they cannot be opened from the job system's worker threads.
- `Performance_Report` is a compiler plugin (`jaic ... +Performance_Report`), not a runtime profiler; the name suggests the second.
- `WGPUAdapterInfo.vendor` is empty on Metal; only `device` carries the GPU's name.

## Open: compiler

None found. `jaic -check` passes for the game on macOS, Linux and Windows.

## Open: diagnostics and tooling

- `a << 8 | b << 4 | c` groups as `a << (8 | b) << (4 | c)`: `|` binds tighter than `<<`. No warning; the lookup table index went out of range at run time. Parenthesise every shift next to a bitwise operator.
- `crates/jaic-cli/tests/examples.rs` lists every example program by hand (`every_example_is_a_listed_program_or_loaded_by_one`), so a new example fails that test until `("game3d/game/main.jai", NATIVE, NO_WINDOW)` is added to `PROGRAMS`.

## What went smoothly

- Compile time: about 0.4 s for the whole game at the default optimisation level, 1.3 s with `-O2`.
- `Soa` members are plain arrays with a `.data` pointer, which is all the SIMD loads need.
- `Extensions/SIMD` operators and `select` read well, and `Math` has the WebGPU depth range (`depth_range_01`) built in.
- The WebGPU extension needed no changes to draw into a window or an offscreen texture; `webgpu_map_buffer` handled the readback.

## Coverage

| Area | Used | Not used |
|---|---|---|
| Graphics | Extensions/WebGPU, Window_Creation, Input | Simp, GL, Vulkan and Metal directly |
| Meshes | meshoptimizer (remap, cache, overdraw, fetch, simplify, codecs) | meshlets, strips, shadow indices |
| Data | Soa, Math, SIMD, PCG | |
| Threads | Thread (threads, semaphore), Atomics | Thread_Group |
| Profiling | Iprof (manual zones), Performance_Report (build) | the Iprof graphics overlay |
| Media | pl_mpeg video | pl_mpeg audio |
| Files and CLI | Extensions/Assets, Extensions/Image (PNG out), Extensions/Args | |
