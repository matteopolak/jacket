# Compile-time values, globals and runtime info

## What it is

How data produced by `#run` is turned into constants of the compiled program, how compile-time mutation of
globals is undone before `main`, and how `get_runtime_info` exposes the type table.

## How it works

**Freezing results.** After a thunk returns, `read_value` (`sema/consteval.rs`) copies the result out of
interpreter memory. Scalars, strings and types become plain `Value`s; aggregates become an `Aggregate`
(bytes plus relocations). `freeze` / `freeze_pointer` walk the type and replace every pointer, string data
pointer and array-view pointer with a relocation:

- a pointer into an existing global becomes a relocation to that global;
- heap memory (for example the buffer of a `[..]` built with `array_add`) is copied into a new read-only
  blob, recursively freezing what it points to;
- small integers cast to pointers (below `0x10000`) are kept as plain numbers;
- a pointer to memory of unknown size (for example `*void` from `alloc`) is an error:
  `a compile-time value holds a pointer to memory of unknown size`;
- union members are copied as raw bytes without following pointers.

Verified:

```jai
Item :: struct { name: string; tags: [] int; }
make :: () -> Item { arr: [..] int; for 1..4 array_add(*arr, it * it); return .{ "sq", arr }; }
TABLE :: #run make();
// at runtime: print("% %\n", TABLE.name, TABLE.tags);   ->   sq [1, 4, 9, 16]
```

**Globals do not leak into runtime.** Globals changed by `#run` code are restored before `main`, like `jai`.
`resolve_global_var` (`sema/decls.rs`) lists every user global that is not marked `#no_reset` in
`Program.reset_globals`; `run_program` calls `Interp::reset_globals`, which zeroes each materialized global,
copies its `init` bytes and re-applies relocations. `x := #run f();` keeps its value because the evaluated
result is the initializer. `jaic build` emits only the `init` data, so the reset is implicit there.

A `#no_reset` global goes to `sema.no_reset_globals` instead. Before codegen, `bake_no_reset_globals`
(`driver.rs`, called from `prepare_compiled_output`) reads each materialized one back from the interpreter with
`read_aggregate`, the same path `#run` constants take, and makes that its `init` and relocations. So an
executable sees what compile-time code stored, including strings, arrays and pointers into other globals
(chess-jai loads its network weights this way). Regression: `tests/stdlib/no-reset-globals-baked.jai` (also
run natively).

```jai
counter := 5;
#no_reset keep := 0;
#run { counter = 99; keep = 7; }
// main prints counter = 5, keep = 7
```

Regression: `tests/stdlib/compile-time-globals-reset.jai`.

**Runtime info.** `get_runtime_info()` from the Compiler module reads the `#elsewhere` symbol
`__runtime_info`. `sema/runtime_info.rs` creates that global on first reference and `fill_runtime_info`
fills it once the program is final: `type_table` (every type descriptor) and `global_data_info` with data and
read-only data segments (used by the memory debugger to scan globals). Layouts mirror `Runtime_Info` in
`stdlib/Compiler/workspace.jai`.

## How to change it

- New kind of value that can cross the `#run` boundary: extend `read_value` and `freeze` together.
- New user-visible variable storage: push it to `reset_globals` where the `Global` is created.
- Runtime info layout changes must be made in `runtime_info.rs` and `stdlib/Compiler/workspace.jai` at once.

## Configuration

`#no_reset` on a declaration. Nothing else.

## Dependencies

`ir::Program::reset_globals`, `ir::Reloc`, `Interp::{global_at, reset_globals}`, type descriptors from
`sema/typeinfo.rs`.
