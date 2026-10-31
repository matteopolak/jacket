# SIMD

Portable fixed-width vectors: `Vector(T, N)` is `N` lanes of an integer or float type (at most 64 bytes) with lane-wise arithmetic, comparisons that produce a `Mask`, reductions, shuffles and byte table lookups. Results are identical on every target (interpreter, x86-64, arm64, wasm); optimized native builds get vector instructions.

```jai
SIMD :: #import "Extensions/SIMD";
using SIMD;

chunk := load(u8x16, "find the first space", 0);
spaces := eq(chunk, #char " ");                    // Mask(u8, 16)
if any(spaces) print("%\n", first_true(spaces));   // 4
a := splat(f32x4, 1.5) * indices(f32x4);           // 0 1.5 3 4.5
print("%\n", reduce_add(a));                       // 9
```

## How it works

- `Vector :: struct (T: Type, N: int) { lanes: [N] T; }`. `#modify` rejects non-numeric lanes, non-power-of-two counts and vectors over 64 bytes.
- `Mask(T, N)` has one all-ones-or-zeros lane per vector lane, the layout compare instructions produce on SSE, NEON and wasm.
- Most operations are plain loops over `lanes` in `inline` procedures that LLVM vectorizes at `-O1`+. Two use compiler intrinsics ([intrinsics](../language/intrinsics.md)): `lookup` (`byte_lookup`) and `to_bits` of a byte mask (`sign_mask`). `any`/`all`/`none` read the mask as 64-bit words, because a per-lane reduction compiled to a branch per lane.
- Code outside a `#target_features` procedure targets the baseline CPU (SSE2 on x86-64).

## Choosing code by CPU

Detection and dispatch live in `stdlib/internal/cpu`, re-exported here.

```jai
f := cpu_features();            // sse42, avx2, avx512, neon, sve
tier := best_vector_tier();     // .BASELINE, .SSE42, .AVX2, .AVX512 or .SVE
```

- x64 uses `cpuid` plus `xgetbv`, so a feature the OS hasn't enabled reads as absent. arm64 always has `neon`; `sve` comes from `getauxval` on Linux and Android. Other targets report nothing and get the baseline.
- Detection is cached on first call. `limit_vector_tier(.AVX2)` pretends the CPU has nothing above that tier; call it before the first dispatched call.
- A `#target_features "avx2"` procedure is undefined on a CPU without the feature. `pick_target` chooses among versions:

```jai
count_base :: (data: *u8, n: s64) -> s64 { return count_body(u8x16, data, n); }
count_avx2 :: (data: *u8, n: s64) -> s64 #target_features "avx2" { return count_body(u8x32, data, n); }

count_table: type_of(count_base) = count_first;
count_first :: (data: *u8, n: s64) -> s64 {
    count_table = pick_target(count_base, avx2 = count_avx2);
    return count_table(data, n);
}
count :: (data: *u8, n: s64) -> s64 { return count_table(data, n); }
```

The first call fills the table; later calls are one indirect call. Keep `count_body` `inline` so each version is vectorized for its own features.

Gotchas: `lookup` isn't faster in an AVX2 version (it lowers for the build's features). Don't dispatch tiny procedures; the indirect call costs more than it saves under about 64 bytes. To test a version, call it directly; only call AVX2 code where `cpu_features().avx2` is true.

## How to change it

- New operation: write a loop over `lanes` in an `inline` procedure, then check `-O2` vectorizes it (`-emit_ir`, or `otool -tV`/`objdump -d`). If LLVM keeps a per-lane loop or branch, reformulate or add an intrinsic (compiler work).
- Inside `#modify`, `$V: Type` is a variable: use `type_info`, not `size_of(V)` or `V.T`.
- Tests: `jacket test --path stdlib/Extensions/SIMD` (add `--interp`, or `-os wasm --runner "node --no-warnings tools/wasi_run.mjs"`). Interpreted and native results must agree. Dispatch tests: `tests/cpu-dispatch.jai`.

`#module_parameters(VECTOR_BYTES := 16)` is the width code should prefer. Wasm builds run scalar code unless built with `+simd128`. Used by [Byte_Search](byte-search.md), [Json](json.md) and the [HTTP codec](http-codec.md).
