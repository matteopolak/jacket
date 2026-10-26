// Folded constants, `#run` and compiled code agree on shifts, division and narrowing.
// No stdlib imports.
shr_s8 :: (v: s8, n: u32) -> s8 #no_context {
    return v >> n;
}

shr_s64 :: (v: s64, n: u32) -> s64 #no_context {
    return v >> n;
}

shr_u16 :: (v: u16, n: u32) -> u16 #no_context {
    return v >> n;
}

shl_u8 :: (v: u8, n: u32) -> u8 #no_context {
    return v << n;
}

shl_s32 :: (v: s32, n: u32) -> s32 #no_context {
    return v << n;
}

div_s8 :: (a: s8, b: s8) -> s8 #no_context {
    return a / b;
}

rem_s8 :: (a: s8, b: s8) -> s8 #no_context {
    return a % b;
}

div_s64 :: (a: s64, b: s64) -> s64 #no_context {
    return a / b;
}

mul_u8 :: (a: u8, b: u8) -> u8 #no_context {
    return a * b;
}

add_s16 :: (a: s16, b: s16) -> s16 #no_context {
    return a + b;
}

to_s8 :: (x: float64) -> s8 #no_context {
    return cast(s8, x);
}

to_u8 :: (x: float64) -> u8 #no_context {
    return cast(u8, x);
}

to_s32 :: (x: float64) -> s32 #no_context {
    return cast(s32, x);
}

to_u64 :: (x: float64) -> u64 #no_context {
    return cast(u64, x);
}

MIN8 :: cast(s8) -128;

// Each case: the folded constant and the same call run by the interpreter.
FOLD_SHR_S8_BIG :: MIN8 >> 200;
RUN_SHR_S8_BIG :: #run shr_s8(MIN8, 200);
FOLD_SHR_S8_EDGE :: MIN8 >> 8;
RUN_SHR_S8_EDGE :: #run shr_s8(MIN8, 8);
FOLD_SHR_S8_WIDE :: MIN8 >> 128;
RUN_SHR_S8_WIDE :: #run shr_s8(MIN8, 128);
FOLD_SHR_S64_WIDE :: (cast(s64) -5) >> 130;
RUN_SHR_S64_WIDE :: #run shr_s64(-5, 130);
FOLD_SHR_U16_BIG :: (cast(u16) 0xffff) >> 16;
RUN_SHR_U16_BIG :: #run shr_u16(0xffff, 16);
FOLD_SHL_U8_BIG :: (cast(u8) 0xff) << 8;
RUN_SHL_U8_BIG :: #run shl_u8(0xff, 8);
FOLD_SHL_S32_SIGN :: (cast(s32) 1) << 31;
RUN_SHL_S32_SIGN :: #run shl_s32(1, 31);
FOLD_DIV_S8_MIN :: MIN8 / -1;
RUN_DIV_S8_MIN :: #run div_s8(MIN8, -1);
FOLD_REM_S8_MIN :: MIN8 % -1;
RUN_REM_S8_MIN :: #run rem_s8(MIN8, -1);
FOLD_DIV_S64_MIN :: (cast(s64) 0x8000000000000000) / -1;
RUN_DIV_S64_MIN :: #run div_s64(cast(s64) 0x8000000000000000, -1);
FOLD_MUL_U8 :: (cast(u8) 200) * (cast(u8) 3);
RUN_MUL_U8 :: #run mul_u8(200, 3);
FOLD_ADD_S16 :: (cast(s16) 32767) + (cast(s16) 1);
RUN_ADD_S16 :: #run add_s16(32767, 1);
FOLD_TO_S8 :: cast(s8, 300.0);
RUN_TO_S8 :: #run to_s8(300.0);
FOLD_TO_U8 :: cast(u8, -1.0);
RUN_TO_U8 :: #run to_u8(-1.0);
FOLD_TO_S32 :: cast(s32, 1e10);
RUN_TO_S32 :: #run to_s32(1e10);
FOLD_TO_U64 :: cast(u64, 1e30);
RUN_TO_U64 :: #run to_u64(1e30);

#assert(FOLD_SHR_S8_BIG == RUN_SHR_S8_BIG && FOLD_SHR_S8_BIG == -1);
#assert(FOLD_SHR_S8_EDGE == RUN_SHR_S8_EDGE && FOLD_SHR_S8_EDGE == -1);
#assert(FOLD_SHR_S8_WIDE == RUN_SHR_S8_WIDE && FOLD_SHR_S8_WIDE == -1);
#assert(FOLD_SHR_S64_WIDE == RUN_SHR_S64_WIDE && FOLD_SHR_S64_WIDE == -1);
#assert(FOLD_SHR_U16_BIG == RUN_SHR_U16_BIG && FOLD_SHR_U16_BIG == 0);
#assert(FOLD_SHL_U8_BIG == RUN_SHL_U8_BIG && FOLD_SHL_U8_BIG == 0);
#assert(FOLD_SHL_S32_SIGN == RUN_SHL_S32_SIGN);
#assert(FOLD_DIV_S8_MIN == RUN_DIV_S8_MIN && FOLD_DIV_S8_MIN == -128);
#assert(FOLD_REM_S8_MIN == RUN_REM_S8_MIN && FOLD_REM_S8_MIN == 0);
#assert(FOLD_DIV_S64_MIN == RUN_DIV_S64_MIN);
#assert(FOLD_MUL_U8 == RUN_MUL_U8 && FOLD_MUL_U8 == 88);
#assert(FOLD_ADD_S16 == RUN_ADD_S16 && FOLD_ADD_S16 == -32768);
#assert(FOLD_TO_S8 == RUN_TO_S8 && FOLD_TO_S8 == 127);
#assert(FOLD_TO_U8 == RUN_TO_U8 && FOLD_TO_U8 == 0);
#assert(FOLD_TO_S32 == RUN_TO_S32 && FOLD_TO_S32 == 2147483647);
#assert(FOLD_TO_U64 == RUN_TO_U64 && FOLD_TO_U64 == 18446744073709551615);

// The compiled calls against the folded constants.
#program_export "integer_constant_arithmetic_check"
check :: () -> s32 #c_call {
    if shr_s8(MIN8, 200) != FOLD_SHR_S8_BIG return 1;
    if shr_s8(MIN8, 8) != FOLD_SHR_S8_EDGE return 2;
    if shr_s8(MIN8, 128) != FOLD_SHR_S8_WIDE return 3;
    if shr_s64(-5, 130) != FOLD_SHR_S64_WIDE return 4;
    if shr_u16(0xffff, 16) != FOLD_SHR_U16_BIG return 5;
    if shl_u8(0xff, 8) != FOLD_SHL_U8_BIG return 6;
    if shl_s32(1, 31) != FOLD_SHL_S32_SIGN return 7;
    if div_s8(MIN8, -1) != FOLD_DIV_S8_MIN return 8;
    if rem_s8(MIN8, -1) != FOLD_REM_S8_MIN return 9;
    if div_s64(cast(s64) 0x8000000000000000, -1) != FOLD_DIV_S64_MIN return 10;
    if mul_u8(200, 3) != FOLD_MUL_U8 return 11;
    if add_s16(32767, 1) != FOLD_ADD_S16 return 12;
    if to_s8(300.0) != FOLD_TO_S8 return 13;
    if to_u8(-1.0) != FOLD_TO_U8 return 14;
    if to_s32(1e10) != FOLD_TO_S32 return 15;
    if to_u64(1e30) != FOLD_TO_U64 return 16;

    return 0;
}

main :: () -> s32 {
    return check();
}
