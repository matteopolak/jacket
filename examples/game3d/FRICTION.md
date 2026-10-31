# Friction log

Rough edges hit while writing game3d. Fixed ones name the commit subject; open ones say so.

## Fixed in the stdlib

- `meshoptimizer`: the declarations followed an old API, so `meshopt_simplify` and the meshlet functions did not match the linked library. Fixed by `fix(meshoptimizer): match the declarations to the pinned library`; the game now calls the module's `meshopt_simplify`.

## Open: stdlib

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
