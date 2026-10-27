# SIMD (`Extensions/SIMD`)

## What it is

Portable fixed-width vectors: `Vector(T, N)` is `N` lanes of an integer or float type (at most 64 bytes) with lane-wise arithmetic, comparisons that produce a `Mask`, reductions, shuffles and byte table lookups. Results are identical on every target (interpreter, x86-64, arm64, wasm); native optimized builds get vector instructions.

```jai
SIMD :: #import "Extensions/SIMD";
using SIMD;

chunk := load(u8x16, "find the first space", 0);
spaces := eq(chunk, #char " ");                    // Mask(u8, 16)
if any(spaces) print("%\n", first_true(spaces));   // 4
a := splat(f32x4, 1.5) * indices(f32x4);           // 0 1.5 3 4.5
print("%\n", reduce_add(a));                       // 9
```

Groups: make (`splat`, `indices`, `load`, `load_partial`, `store`), arithmetic (`+ - * /`, `min`, `max`, `abs`, `saturating_add/sub`, `sqrt`), bits (`& | ^ ~`, shifts by count modulo lane width), compare (`eq ne lt le gt ge` to `Mask`), masks (`to_bits`, `mask_from_bits`, `any`, `all`, `none`, `count_true`, `first_true`, `last_true`, `select`), `reduce_*`, rearrange (`shuffle`, `window`, `reverse`, `lookup`), shape (`convert`, `bitcast`, `low_half`, `high_half`, `combine`), scalars (`bit_count`, `trailing_zeros`, `leading_zeros`). Aliases: `u8x16`, `u8x32`, `u8x64`, `s8x16`, `u16x8`, `s32x4`, `f32x4`, `f64x2`, up to 32 bytes.

## How it works

`Vector :: struct (T: Type, N: int) { lanes: [N] T; }`; `#modify` rejects non-numeric lanes, non-power-of-two counts and vectors over 64 bytes. `Mask(T, N)` has one lane per vector lane, as wide as `T`, all ones or all zeros: the layout compare instructions produce on SSE, NEON and wasm, so no conversion is needed.

Most operations are plain loops over `lanes` in `inline` procedures that LLVM's vectorizers turn into vector instructions at `-O1`+. Two cannot be written as recognizable loops and use compiler intrinsics ([intrinsics](../language/intrinsics.md)): `lookup` calls `byte_lookup` (NEON `tbl`, SSSE3 `pshufb`, wasm `swizzle`) and `to_bits` of a byte mask calls `sign_mask` (`pmovmskb`). `any`/`all`/`none` read the mask as 64-bit words, because a per-lane reduction compiled to a branch per lane.

## How to change it

- **New operation:** write a loop over `lanes` in an `inline` procedure, then check `-O2` vectorizes it (`-emit_ir`, or `otool -tV`/`objdump -d`). If LLVM keeps a per-lane loop or a branch per lane, reformulate (as `any` was) or add an intrinsic.
- **New intrinsic:** the work is in the compiler; declare it in this module's `#scope_module` and keep a scalar path for shapes it does not cover.
- Checks go in `#modify` or `#assert`. Inside `#modify`, `$V: Type` is a variable: use `type_info`/`runtime_size`, not `size_of(V)` or `V.T`.
- Tests: `tests/vectors.jai`; `jacket test --path stdlib/Extensions/SIMD` (add `--interp`, or `-os wasm --runner "node --no-warnings tools/wasi_run.mjs"`, for the other modes). Interpreted and native results must agree.
- Code outside a `#target_features` procedure targets the baseline CPU (SSE2 on x86-64; `core2`, with SSSE3, on macOS). Wider code goes through [choosing code by CPU](#choosing-code-by-cpu).

## Choosing code by CPU

Detection and dispatch live in `stdlib/internal/cpu` and are re-exported by this module.

```jai
f := cpu_features();            // Cpu_Features: sse42, avx2, avx512, neon, sve
tier := best_vector_tier();     // .BASELINE, .SSE42, .AVX2, .AVX512 or .SVE
```

- **x64:** `cpuid` gives SSE4.2, AVX, AVX2 and AVX-512 F/DQ/BW/VL; `xgetbv` confirms the operating system saves the YMM (AVX, AVX2) and opmask/ZMM (AVX-512) registers, so a feature the OS has not enabled reads as absent. Each tier implies those below it.
- **arm64:** `neon` is always true (it is the baseline, not a tier). `sve` is read from `getauxval` on Linux and Android, and false elsewhere.
- Other targets (wasm) report no features, so dispatch picks the baseline version.
- Detection runs on the first call and is cached. `limit_vector_tier(.AVX2)` pretends the CPU has nothing above that tier, for tests and for comparing versions on one machine; call it before the first call of any dispatched procedure.

A procedure compiled for a feature is `#target_features "avx2"` (see [directives](../language/directives-and-notes.md#target_features)); calling it where the CPU lacks the feature is undefined. `pick_target` chooses among versions:

```jai
count_body :: inline ($V: Type, data: *u8, n: s64) -> s64 { ... }    // written once, any vector type
count_base :: (data: *u8, n: s64) -> s64 { return count_body(u8x16, data, n); }
count_avx2 :: (data: *u8, n: s64) -> s64 #target_features "avx2" { return count_body(u8x32, data, n); }

count_table: type_of(count_base) = count_first;
count_first :: (data: *u8, n: s64) -> s64 {
    count_table = pick_target(count_base, avx2 = count_avx2);   // avx512 =, sse42 =, sve = also exist
    return count_table(data, n);
}
count :: (data: *u8, n: s64) -> s64 { return count_table(data, n); }
```

The table starts at the stub, the first call fills it, and later calls are one indirect call with no feature check. Two racing first calls store the same pointer. `pick_target` takes the highest tier that has both a version and the CPU feature, else the baseline version. The body is `inline` so each version gets the vectorizer of its own features; a baseline helper inlined into an AVX2 procedure is compiled for AVX2.

- **Gotchas:** vector intrinsics (`lookup`, `to_bits` of a byte mask) still lower for the build's features, not the procedure's, so `lookup` is not faster in an AVX2 version (the baseline x86-64 target has no `pshufb`). Versions that are tiny should not be dispatched: the indirect call costs more than it saves under about 64 bytes of input.
- **Testing a version:** call it directly. A `#target_features` procedure on another architecture is the plain code, so a 32-byte version can be tested on any machine; only call it on x64 where `cpu_features().avx2` is true. `Byte_Search/tests/variants.jai` loads the module with `#load` to reach its private versions.
- Tests: `tests/cpu-dispatch.jai`.

## Configuration and dependencies

`#module_parameters(VECTOR_BYTES := 16)` is the width code should prefer (set 32 for AVX2 builds). Wasm builds have no `+simd128` by default, so they run scalar code. Depends on `Basic`, the `byte_lookup`/`sign_mask` intrinsics and LLVM's vectorizers. Used by [Byte_Search](byte-search.md), [Json](json.md) and the [HTTP codec](http-codec.md).
