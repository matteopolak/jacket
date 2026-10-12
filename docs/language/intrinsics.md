# Compiler intrinsics

## What it is

Procedures declared `#intrinsic` have no Jai body; the compiler lowers each call to a fixed IR operation chosen by the procedure's name {#intrin.1}. The memory and atomic ones are declared in `stdlib/prelude/module.jai`:

```jai
memset :: (dest: *void, value: u8, count: s64) #intrinsic;
memcpy :: (dest: *void, source: *void, count: s64) #intrinsic;
memcmp :: (a: *void, b: *void, count: s64) -> s16 #must #intrinsic;
compare_and_swap :: (pointer: *$T, old: T, new: T) -> (success: bool, old_value: T) #intrinsic;
```

Two SIMD intrinsics are declared by the library that uses them (`stdlib/Extensions/SIMD`), not in the prelude:

```jai
byte_lookup :: (dest: *u8, table: *u8, indices: *u8, width: s64) #intrinsic;
sign_mask   :: (source: *u8, width: s64) -> u64 #intrinsic;
```

`byte_lookup` is a byte table lookup. `table` is 16 bytes; `indices` and `dest` are `width` bytes, and every 16-byte block of `indices` looks up the same table. The result is the same on every target: `dest[i] = indices[i] < 16 ? table[indices[i]] : 0` for every index 0..255. `sign_mask` returns a `u64` whose bit `i` is the top bit of byte `i` of `source` (`width` bytes). `width` must be a compile-time constant 16, 32 or 64, else `` error: the `width` of `byte_lookup` must be a compile-time constant 16, 32 or 64 ``. The buffers need no alignment; `dest` may alias `indices`, but not `table`.

## How it works

`emit_intrinsic` in `sema/calls.rs` maps the name to an `ir::Intrinsic`: `memcpy`, `memset`, `memcmp`, `compare_and_swap`, `debug_break`, `sqrt`, `sin`, `cos`, `floor`, `ceil`, `round`, `trunc`, `abs`/`fabs`, `rdtsc`/`get_cpu_cycle_count`, `pause`/`mm_pause`, `byte_lookup` (`ir::Intrinsic::ByteLookup(width)`), `sign_mask` (`SignMask(width)`; the width is read from a constant argument by `const_width_arg`). Any other name is `` error: unknown intrinsic `frob` `` {#intrin.2}.

A jaic extension: a bodiless procedure declared `#intrinsic "llvm.<name>"` calls the LLVM intrinsic of that name in native builds, with the procedure's signature as the intrinsic's type (`llvm_intrinsic` in `sema/procs.rs` makes it a foreign procedure whose symbol is the intrinsic's name; `crates/jaic-llvm` declares it without a wasm import). Immediate arguments must be constants at the call. The interpreter has no such intrinsics, so call these only from code that is compiled:

```jai
memory_add_pages :: (memory: s32, pages: s64) -> s64 #intrinsic "llvm.wasm.memory.grow.i64";
bit_count :: (x: u64) -> u64 #intrinsic "llvm.ctpop.i64";
```

`stdlib/Extensions/Wasi_Runtime` uses it for `memory.size`/`memory.grow` ([wasm target](../native/wasm-target.md)).

`compare_and_swap` takes its width (1, 2, 4 or 8 bytes) from the value type {#intrin.3}:

```jai
x: s64 = 5;
ok, old := compare_and_swap(*x, 5, 9);   // true 5, x == 9
ok, old  = compare_and_swap(*x, 5, 1);   // false 9, x stays 9
```

`memcmp` returns the sign of the first differing byte {#intrin.4}. Intrinsics take no implicit context (`has_context` in `sema/procs.rs`) {#intrin.5}.

Taking an intrinsic's address goes through `lower_intrinsic_wrapper` (`sema/procs.rs`), which generates a small function around the op. Only `memcpy`, `memset`, `memcmp` and `debug_break` have real wrappers {#intrin.6}; any other becomes a `Trap`, so don't take pointers to the math intrinsics.

The compiler also emits IR operations that user code never names: `BoundsCheck` for indexing, `IsCompileTime` for `#compile_time`, bit operations for `#asm` lowering (see [SIMD and asm](simd-asm.md)), and `Wide(WideOp, WideFloat)` for every `Long_Double` operation (see [Long_Double](long-double.md)).

### Native lowering of the SIMD intrinsics

`jaic-llvm` (`byte_lookup` and `sign_mask` in `lower.rs`) loads `<N x i8>` vectors and picks the instruction from the target. `SimdFeatures` (`simd_features` in `lib.rs`) is derived from the CPU name and feature string.

| Target | `byte_lookup` |
| --- | --- |
| x86-64 with SSSE3 (macOS x86-64 baseline is `core2`; any CPU or `+ssse3`/`+sse4.1`/`+avx` feature) | `llvm.uadd.sat` of `0x70` on the indices (sets the top bit of every index of 16 or more, so `pshufb` zeroes it), then `pshufb`; width 32/64 with AVX2 uses `llvm.x86.avx2.pshuf.b` on 32-byte chunks |
| aarch64 | `llvm.aarch64.neon.tbl1` (`tbl` already gives 0 for indices of 16 or more) |
| wasm with `+simd128` | `llvm.wasm.swizzle` (also 0 for 16 or more) |
| anything else (Linux x86-64 baseline, wasm without SIMD) | per-lane `extractelement`/`select`/`insertelement` |

`sign_mask` is `icmp slt <N x i8> v, zeroinitializer`, a bitcast of the `<N x i1>` to `iN` and a zero extension to 64 bits on every target (`pmovmskb` on x86). The interpreter has scalar versions of both. `#asm` `pshufb` and `pmovmskb` lower through them ([asm](../compiler/asm.md)).

## How to change it

1. Add an `ir::Intrinsic` variant in `crates/jaic/src/ir.rs`.
2. Implement it in the interpreter (`crates/jaic/src/interp/`) and in `crates/jaic-llvm`.
3. Add the name to `emit_intrinsic`, and to `lower_intrinsic_wrapper` if it must work as a value.
4. Declare it `#intrinsic` in `stdlib/prelude/module.jai` or a stdlib module.

An LLVM intrinsic needs none of this: declare it `#intrinsic "llvm.<name>"` with the signature LLVM expects (check LLVM's `Intrinsics*.td`); a wrong signature fails in LLVM's verifier.

`emit_intrinsic` trusts the declaration and does no further type checking, so keep the declared signature in sync with what the IR op expects.

## Dependencies

`stdlib/prelude/module.jai`, `sema/calls.rs`, `sema/procs.rs`, `ir.rs`, the interpreter and the LLVM backend.
