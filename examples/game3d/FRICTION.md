# Friction log

Rough edges hit while writing game3d. Fixed ones name the commit subject; open ones say so.

## Fixed in the stdlib

- `Window_Type` is a pointer on macOS and an integer on Linux, so "no window" is `null` in one place and `0` in another. The type stays as it is (it has to match the reference); the Window_Creation docs now say so and show the portable spelling, `no_window: Window_Type;` compared with `WT.INVALID_WINDOW`.
- `Extensions/WebGPU` had no texture readback helper. It has `webgpu_read_texture` now, which does the copy, the 256-byte row padding, the map and the copy out, and returns packed rows. The game, `Extensions/UI` and the headless test use it.
- `Iprof`: the manual mode was written down only in the module's tests. The module now opens with an overview and a runnable example (setup, zones, `update(true)` per frame, report), and `zone_begin`, `zone_end`, `update`, `create_report`, `log_text_report` and `save_csv_report` have doc comments. It also says zones are global to the process and cannot be opened from worker threads.
- `meshoptimizer`: the declarations followed an old API, so `meshopt_simplify` and the meshlet functions did not match the linked library. Fixed by `fix(meshoptimizer): match the declarations to the pinned library`; the game now calls the module's `meshopt_simplify`.

## Fixed in jailint

- `a << 8 | b << 4 | c` reads left to right as `(((a << 8) | b) << 4) | c`, because shifts and bitwise operators share one level in Jai. The `bitwise_precedence` lint of jailint already flags it (with a fix that adds the parentheses); its tests now include this shape. No new lint was needed.

## Open: stdlib

- `Performance_Report` is a compiler plugin (`jaic ... +Performance_Report`), not a runtime profiler; the name suggests the second.
- `WGPUAdapterInfo.vendor` is empty on Metal; only `device` carries the GPU's name.

## Open: compiler

None found. `jaic -check` passes for the game on macOS, Linux and Windows.

## Open: diagnostics and tooling

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
