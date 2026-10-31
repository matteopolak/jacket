# Compiler intrinsics

A procedure declared `#intrinsic` has no Jai body; the compiler lowers each call to a fixed IR operation chosen by the procedure's name {#intrin.1}. Memory and atomic ones are declared in the prelude; SIMD and fiber ones by the library that uses them.

```jai
memcpy :: (dest: *void, source: *void, count: s64) #intrinsic;
compare_and_swap :: (pointer: *$T, old: T, new: T) -> (success: bool, old_value: T) #intrinsic;
byte_lookup :: (dest: *u8, table: *u8, indices: *u8, width: s64) #intrinsic;   // stdlib/Extensions/SIMD
sign_mask   :: (source: *u8, width: s64) -> u64 #intrinsic;

x: s64 = 5;
ok, old := compare_and_swap(*x, 5, 9);   // true 5, x == 9
```

## How it works

- `emit_intrinsic` in `sema/calls/intrinsics.rs` maps the name to an IR op: `memcpy`, `memset`, `memcmp`, `compare_and_swap`, `debug_break`, the math functions (`sqrt`, `sin`, `cos`, `floor`, `ceil`, `round`, `trunc`, `abs`), `rdtsc`, `pause`, the `atomic_*_order` family, `black_box`, `byte_lookup`, `sign_mask`. Any other name is ``error: unknown intrinsic `frob` `` {#intrin.2}. It trusts the declared signature.
- `compare_and_swap` takes its width (1, 2, 4 or 8 bytes) from the value type {#intrin.3}. `memcmp` returns the sign of the first differing byte {#intrin.4}.
- Ordered atomics take the memory order as a compile-time last parameter (a compare-exchange packs `success | failure << 4`); invalid orders are errors.
- `byte_lookup`: `dest[i] = indices[i] < 16 ? table[indices[i]] : 0` on every target; `table` is 16 bytes and `width` is a constant 16, 32 or 64. `sign_mask` returns the top bit of each byte as a `u64`.
- `black_box(value)` (from `Extensions/Testing`) returns its argument and hides it from the optimizer, so benchmarks aren't folded away. It's not in `Basic`, whose exports are fixed.
- `#intrinsic "llvm.<name>"` on a bodiless procedure calls that LLVM intrinsic in native builds (immediates must be constants). The interpreter lacks them, so use them only in compiled code.
- Intrinsics take no implicit context {#intrin.5}.
- Taking an intrinsic's address generates a wrapper. Only `memcpy`, `memset`, `memcmp` and `debug_break` have real ones {#intrin.6}; any other becomes a trap, so don't take pointers to the math intrinsics.

## How to change it

1. Add an `ir::Intrinsic` variant in `crates/jaic/src/ir.rs`.
2. Implement it in `crates/jaic/src/interp/` and `crates/jaic-llvm`.
3. Add the name to `emit_intrinsic` (and `lower_intrinsic_wrapper` if it must work as a value).
4. Declare it `#intrinsic` in the prelude or a stdlib module.

An LLVM intrinsic needs none of this; a wrong signature fails in LLVM's verifier.
