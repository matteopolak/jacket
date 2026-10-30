// A fixed number of lanes of one type, held in `data`: `SIMD_Array(4, float32)` is four floats in a row.
//
// The struct is just the array, so its size is `count * size_of(type)` with the alignment of one lane, and `data[i]` is lane `i`. The arithmetic, comparison and memory procedures of this module take and return it; a lane is read or written through `data`. The lane count and type are compile-time parameters and add no runtime fields.
//
// ```jai
// v: SIMD_Array(4, s32);
// v.data = .[1, 2, 3, 4];
// assert(size_of(SIMD_Array(4, s32)) == 16);
// assert(horizontal_sum(v + cast(s32) 1) == 14);
// ```
SIMD_Array :: struct(count: s64, type: Type) {
    data: [count] type;
}

// One 16-byte block viewed as every lane layout that fits it: `_u8` (16 lanes) through `_u64` (2 lanes), `_float32` (4) and `_float64` (2). All members share the same bytes, so writing one lane type and reading another reinterprets the bits.
//
// This is a type-punning view and not a vector register; no procedure takes it directly, but each member is a `SIMD_Array` and takes part in all of them. Lanes are in memory order, so on a little-endian target (every supported one) lane 0 of `_u8` is the low byte of lane 0 of `_u32`.
//
// ```jai
// block: Simd_128;
// block._u32.data[0] = 0x04030201;
// assert(block._u8.data[0] == 1 && block._u8.data[3] == 4);
// assert(size_of(Simd_128) == 16);
// ```
Simd_128 :: union {
    _u8: SIMD_Array(16, u8);
    _s8: SIMD_Array(16, s8);
    _u16: SIMD_Array(8, u16);
    _s16: SIMD_Array(8, s16);
    _u32: SIMD_Array(4, u32);
    _s32: SIMD_Array(4, s32);
    _u64: SIMD_Array(2, u64);
    _s64: SIMD_Array(2, s64);
    _float32: SIMD_Array(4, float32);
    _float64: SIMD_Array(2, float64);
}
