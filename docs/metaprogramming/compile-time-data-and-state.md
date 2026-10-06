# Compile-time values, globals and runtime info

## What it is

How data produced by `#run` becomes constants of the compiled program, how compile-time changes to globals are undone before `main`, and how `get_runtime_info` exposes the type table.

## How it works

### Freezing `#run` results

After a thunk returns, `read_value` (`sema/consteval.rs`) copies the result out of interpreter memory. Scalars, strings and types become plain `Value`s {#ctdata.1}; aggregates become an `Aggregate` (bytes plus relocations). `freeze` and `freeze_pointer` walk the type and replace each pointer with a relocation:

- a pointer into a global becomes a relocation to that global {#ctdata.2};
- heap memory (say, a `[..]` built with `array_add`) is copied into a new read-only blob, recursively {#ctdata.3};
- integers below `0x10000` cast to pointers stay plain numbers {#ctdata.4};
- a pointer to memory of unknown size (`*void` from `alloc`) fails with `a compile-time value holds a pointer to memory of unknown size` {#ctdata.5};
- union members are copied as raw bytes, without following pointers {#ctdata.6}.

Verified {#ctdata.7}:

```jai
Item :: struct { name: string; tags: [] int; }
make :: () -> Item { arr: [..] int; for 1..4 array_add(*arr, it * it); return .{ "sq", arr }; }
TABLE :: #run make();
// runtime: print("% %\n", TABLE.name, TABLE.tags);   ->   sq [1, 4, 9, 16]
```

### Globals reset before `main`

Changes `#run` code makes to globals do not reach the running program, matching Jai {#ctdata.8}.

```jai
counter := 5;
#no_reset keep := 0;
#run { counter = 99; keep = 7; }
// main sees counter = 5, keep = 7
```

The example above prints `counter = 5, keep = 7` {#ctdata.11}.

`resolve_global_var` (`sema/decls.rs`) adds every user global not marked `#no_reset` to `Program.reset_globals`. `run_program` calls `Interp::reset_globals`, which zeroes each one, copies its `init` bytes and re-applies relocations. `x := #run f();` keeps its value because the result is the initializer {#ctdata.9}. `jaic build` emits only `init` data, so the reset is implicit there.

`#no_reset` globals go to `sema.no_reset_globals`. Before codegen, `bake_no_reset_globals` (`driver.rs`, called from `prepare_compiled_output`) reads each one back from the interpreter with `read_aggregate`, the same path `#run` constants take, and makes that its `init`. An executable therefore sees what compile-time code stored, including strings, arrays and pointers into other globals. chess-jai loads its network weights this way {#ctdata.10}.

### Runtime info

`get_runtime_info()` from the Compiler module reads the `#elsewhere` symbol `__runtime_info`. `sema/runtime_info.rs` creates that global on first reference; `fill_runtime_info` fills it once the program is final with `type_table` (every type descriptor) {#ctdata.12} and `global_data_info` (the data and read-only data segments, which the memory debugger scans) {#ctdata.13}. The layout mirrors `Runtime_Info` in `stdlib/Compiler/workspace.jai`.

## How to change it

- A new kind of value crossing the `#run` boundary: extend `read_value` and `freeze` together.
- New user-visible global storage: push it to `reset_globals` where the `Global` is created.
- Runtime info layout: change `runtime_info.rs` and `stdlib/Compiler/workspace.jai` together.

Tests: `tests/stdlib/compile-time-globals-reset.jai`, `no-reset-globals-baked.jai` (also run natively).

## Dependencies

`ir::Program::reset_globals`, `ir::Reloc`, `Interp::{global_at, reset_globals}`, type descriptors from `sema/typeinfo.rs`.
