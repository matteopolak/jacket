// Reviewed public layouts; compile-time parameters do not add runtime fields.
SIMD_Array :: struct(count: s64, type: Type) {
    data: [count] type;
}

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
