# Compiler intrinsics

## What it is

A procedure declared `#intrinsic` has no Jai body; the compiler lowers each call to a fixed IR operation chosen by the procedure's name. Memory and atomic ones are declared in `stdlib/prelude/intrinsics.jai`; SIMD and fiber ones by the library that uses them.

```jai
memcpy :: (dest: *void, source: *void, count: s64) #intrinsic;
compare_and_swap :: (pointer: *$T, old: T, new: T) -> (success: bool, old_value: T) #intrinsic;
byte_lookup :: (dest: *u8, table: *u8, indices: *u8, width: s64) #intrinsic;   // stdlib/Extensions/SIMD
sign_mask   :: (source: *u8, width: s64) -> u64 #intrinsic;

x: s64 = 5;
ok, old := compare_and_swap(*x, 5, 9);   // true 5, x == 9
```

## How it works

`emit_intrinsic` in `sema/calls.rs` maps the name to an `ir::Intrinsic` (`memcpy`, `memset`, `memcmp`, `compare_and_swap`, `debug_break`, the math functions, `rdtsc`, `pause`, the `atomic_*_order` family, `byte_lookup`, `sign_mask`). Any other name is ``error: unknown intrinsic `frob` ``. It trusts the declared signature.

- Ordered atomics (wrapped by `stdlib/Atomics/module.jai`, [atomics](../specs/stdlib/modules/atomics.md)) take the memory order as a compile-time last parameter (a compare-exchange packs `success | failure << 4`); invalid load/store/failure orders are errors. They become `AtomicLoad/AtomicStore/AtomicRmw/AtomicCas` with the operand width as a trailing argument.
- `byte_lookup`: `dest[i] = indices[i] < 16 ? table[indices[i]] : 0` on every target; `table` is 16 bytes, `width` must be a constant 16, 32 or 64 (`const_width_arg`). `sign_mask` returns the top bit of each byte as a `u64`. `jaic-llvm`'s `simd` module picks `pshufb`, `tbl1`, `swizzle` or a per-lane fallback from the target features; the interpreter has scalar versions.
- Fibers: `fiber_init`, `fiber_switch`, `fiber_free`, declared by the module that uses them.
- jaic extension: `#intrinsic "llvm.<name>"` on a bodiless procedure calls that LLVM intrinsic in native builds (`llvm_intrinsic` in `sema/procs.rs`; immediates must be constants). The interpreter lacks them, so use only in compiled code. `Wasi_Runtime` uses it for `memory.grow` ([wasm target](../native/wasm-target.md)).
- Intrinsics take no implicit context. Taking an intrinsic's address goes through `lower_intrinsic_wrapper`; only `memcpy`, `memset`, `memcmp` and `debug_break` have real wrappers, any other becomes a `Trap`.
- IR ops users never name: `BoundsCheck`, `IsCompileTime`, `#asm` bit operations ([SIMD and asm](simd-asm.md)), `Wide(WideOp, WideFloat)` for `Long_Double` ([Long_Double](long-double.md)).

## How to change it

1. Add an `ir::Intrinsic` variant in `crates/jaic/src/ir.rs`.
2. Implement it in `crates/jaic/src/interp/` and `crates/jaic-llvm`.
3. Add the name to `emit_intrinsic` (and `lower_intrinsic_wrapper` if it must work as a value).
4. Declare it `#intrinsic` in `stdlib/prelude/intrinsics.jai` or a stdlib module.

An LLVM intrinsic needs none of this; a wrong signature fails in LLVM's verifier.

## Rules

Claims that tests cite by ID (`tools/check_rule_coverage.py`).

- Procedures declared `#intrinsic` have no Jai body; the compiler lowers each call to a fixed IR operation chosen by the procedure's name {#intrin.1}. The memory and atomic ones are declared in `stdlib/prelude/module.jai`:
- `emit_intrinsic` in `sema/calls.rs` maps the name to an `ir::Intrinsic`: `memcpy`, `memset`, `memcmp`, `compare_and_swap`, `debug_break`, `sqrt`, `sin`, `cos`, `floor`, `ceil`, `round`, `trunc`, `abs`/`fabs`, `rdtsc`/`get_cpu_cycle_count`, `pause`/`mm_pause`, the `atomic_*_order` family, `byte_lookup` (`ir::Intrinsic::ByteLookup(width)`), `sign_mask` (`SignMask(width)`; the width is read from a constant argument by `const_width_arg`). Any other name is `` error: unknown intrinsic `frob` `` {#intrin.2}.
- `compare_and_swap` takes its width (1, 2, 4 or 8 bytes) from the value type {#intrin.3}:
- `memcmp` returns the sign of the first differing byte {#intrin.4}. Intrinsics take no implicit context (`has_context` in `sema/procs.rs`) {#intrin.5}.
- Taking an intrinsic's address goes through `lower_intrinsic_wrapper` (`sema/procs.rs`), which generates a small function around the op. Only `memcpy`, `memset`, `memcmp` and `debug_break` have real wrappers {#intrin.6}; any other becomes a `Trap`, so don't take pointers to the math intrinsics.
