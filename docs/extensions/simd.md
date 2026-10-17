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
- Tests: `tests/vectors.jai`; `python3 tools/stdlib_runtime.py --jaic <jaic> --filter SIMD [--modes wasm-interp,wasm-native]`. Interpreted and native results must agree.
- Not done: runtime CPU feature detection and AVX2/AVX-512 paths. Code targets the baseline CPU (SSE2 on x86-64; `core2`, with SSSE3, on macOS).

## Configuration and dependencies

`#module_parameters(VECTOR_BYTES := 16)` is the width code should prefer (set 32 for AVX2 builds). Wasm builds have no `+simd128` by default, so they run scalar code. Depends on `Basic`, `Math`, the `byte_lookup`/`sign_mask` intrinsics and LLVM's vectorizers. Used by [Byte_Search](byte-search.md), [Json](json.md) and the [HTTP codec](http-codec.md).
