# SIMD (jaic extension)

## What it is

`Extensions/SIMD` gives Jai portable fixed-width vectors: `Vector(T, N)` is `N` lanes of an integer or float type, at most 64 bytes, with lane-wise arithmetic, comparisons that produce a `Mask`, reductions, shuffles and byte table lookups. Every operation gives the same result on every target: the interpreter, x86-64, arm64 and wasm. It is a jaic extension, not official Jai, and it may change between releases.

```jai
#import "Basic";
SIMD :: #import "Extensions/SIMD";
using SIMD;

main :: () {
    text := "find the first space";
    chunk := load(u8x16, text, 0);
    spaces := eq(chunk, #char " ");                 // Mask(u8, 16)
    if any(spaces) print("%\n", first_true(spaces)); // 4

    a := splat(f32x4, 1.5) * indices(f32x4);         // 0 1.5 3 4.5
    print("%\n", reduce_add(a));                     // 9
}
```

## How it works

`Vector :: struct (T: Type, N: int) { lanes: [N] T; }`. `#modify` rejects lanes that are not integers or floats, a lane count that is not a power of two, and vectors over 64 bytes. `Mask(T, N)` has one lane per vector lane, as wide as `T`, holding all ones or all zeros. That is the layout compare instructions produce on SSE, NEON and wasm, so a mask never needs converting.

Most operations are plain loops over `lanes` in `inline` procedures. With `-O1` and up, LLVM's vectorizers turn them into vector instructions: a compare + `any` over 64 bytes becomes four `cmeq`/`pcmpeqb` and an OR. Two operations can't be written as loops that LLVM recognises, so they call compiler intrinsics ([intrinsics](../language/intrinsics.md)):

- `lookup(table, indices)` → `byte_lookup`: NEON `tbl`, SSSE3 `pshufb`, wasm `swizzle`.
- `to_bits` of a byte mask (16+ lanes) → `sign_mask`: `pmovmskb`, or the NEON equivalent.

`any`/`all`/`none` read the mask as 64-bit words, not lane by lane. A per-lane reduction compiled into a branch per lane.

| Group | Procedures |
| --- | --- |
| Making | `splat($V, x)`, `indices($V)`, `load($V, *T)`, `load($V, []T, start)`, `load($V, string, start)`, `load_partial($V, ptr, count, fill)`, `store(v, *T)`, `store(v, []T, start)`, `v[i]`, `v[i] = x` |
| Arithmetic | `+ - * /` (vector or scalar operand; integers wrap), unary `-`, `min`, `max`, `abs`, `saturating_add`, `saturating_sub`, `sqrt` |
| Bits | `& \| ^ ~` (integer lanes), `<< >>` by a count (taken modulo the lane width, as wasm does) |
| Compare | `eq ne lt le gt ge` → `Mask` |
| Masks | `& \| ^ ~`, `m[i]`, `to_bits`, `mask_from_bits($M, bits)`, `any`, `all`, `none`, `count_true`, `first_true`, `last_true` (-1 if none), `select(mask, a, b)` |
| Reduce | `reduce_add/min/max/and/or/xor` |
| Rearrange | `shuffle(a, order)`, `shuffle(a, b, order)` (`order` is a compile-time array), `window(a, b, start)`, `reverse`, `lookup(table, indices)` |
| Shape | `convert($V, a)`, `bitcast($V, a)` (vector or mask), `low_half`, `high_half`, `combine` |
| Scalars | `bit_count`, `trailing_zeros`, `leading_zeros` (64 for 0) |

Aliases name the common shapes: `u8x16`, `u8x32`, `u8x64`, `s8x16`, `u16x8`, `s32x4`, `f32x4`, `f64x2`, and so on up to 32 bytes.

## How to change it

- **A new operation:** write it as a loop over `lanes` in an `inline` procedure. Then check that `-O2` vectorizes it: build a small benchmark with `-emit_ir`, or disassemble with `otool -tV`/`objdump -d`. If LLVM keeps a per-lane loop, or a branch per lane, the operation needs either a different formulation (as `any` did) or an intrinsic.
- **A new intrinsic:** the work is in the compiler (`docs/language/intrinsics.md`). Declare it in this module's `#scope_module` and keep a scalar path for shapes it doesn't cover.
- **Checks belong in `#modify`, or in `#assert` in the body.** Inside `#modify`, `$V: Type` is a variable: use `type_info`/`runtime_size`, not `size_of(V)` or `V.T`.
- **Tests:** `tests/vectors.jai` runs in all four stdlib-runtime modes:

  ```sh
  python3 tools/stdlib_runtime.py --jaic <jaic> --filter SIMD [--modes wasm-interp,wasm-native]
  ```

  Interpreted and native results must agree.

Not done yet:
- runtime CPU feature detection, with procedures compiled for several targets;
- AVX2 (32-byte) and AVX-512 code paths chosen at run time.

Today the code is built for the target's baseline CPU (SSE2 on x86-64; `core2`, which has SSSE3, on macOS).

## Configuration

- `#module_parameters(VECTOR_BYTES := 16)`: the width code should prefer when it can choose. 16 bytes suits SSE2, NEON and wasm simd128; set 32 when building for AVX2.
- Native code gets vector instructions only with optimization (`-O1`+). Wasm builds have no `+simd128` by default, so they use scalar code.

## Dependencies

- `Basic`, `Math`.
- The compiler intrinsics `byte_lookup` and `sign_mask`.
- LLVM's loop and SLP vectorizers.
- Used by [Byte_Search](byte-search.md).
