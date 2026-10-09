// Foreign calls returning C structs of every shape, and #c_call procedures returning them to C
// (interpreter and native). Generated; see docs/native/c-abi.md.
#import "Basic";

returns :: #library "libreturns";

failures := 0;

fail :: (what: string) {
    failures += 1;
    print("FAIL %\n", what);
}

R_i8 :: struct {
    f0: s8;
}

ret_i8 :: (seed: s32) -> R_i8 #foreign returns;
check_i8 :: (r: R_i8, seed: s32) -> s32 #foreign returns;
call_i8 :: (cb: (seed: s32) -> R_i8 #c_call, seed: s32) -> R_i8 #foreign returns;
apply_check_i8 :: (cb: (seed: s32) -> R_i8 #c_call, seed: s32) -> s32 #foreign returns;

make_i8 :: (seed: s32) -> R_i8 #c_call {
    r: R_i8;
    r.f0 = cast(s8) (seed + 1);
    return r;
}

is_i8 :: (r: R_i8, seed: s32) -> bool {
    return r.f0 == cast(s8) (seed + 1);
}

R_i8x3 :: struct {
    f0: s8;
    f1: s8;
    f2: s8;
}

ret_i8x3 :: (seed: s32) -> R_i8x3 #foreign returns;
check_i8x3 :: (r: R_i8x3, seed: s32) -> s32 #foreign returns;
call_i8x3 :: (cb: (seed: s32) -> R_i8x3 #c_call, seed: s32) -> R_i8x3 #foreign returns;
apply_check_i8x3 :: (cb: (seed: s32) -> R_i8x3 #c_call, seed: s32) -> s32 #foreign returns;

make_i8x3 :: (seed: s32) -> R_i8x3 #c_call {
    r: R_i8x3;
    r.f0 = cast(s8) (seed + 1);
    r.f1 = cast(s8) (seed + 2);
    r.f2 = cast(s8) (seed + 3);
    return r;
}

is_i8x3 :: (r: R_i8x3, seed: s32) -> bool {
    return r.f0 == cast(s8) (seed + 1) && r.f1 == cast(s8) (seed + 2) && r.f2 == cast(s8) (seed + 3);
}

R_i16 :: struct {
    f0: s16;
}

ret_i16 :: (seed: s32) -> R_i16 #foreign returns;
check_i16 :: (r: R_i16, seed: s32) -> s32 #foreign returns;
call_i16 :: (cb: (seed: s32) -> R_i16 #c_call, seed: s32) -> R_i16 #foreign returns;
apply_check_i16 :: (cb: (seed: s32) -> R_i16 #c_call, seed: s32) -> s32 #foreign returns;

make_i16 :: (seed: s32) -> R_i16 #c_call {
    r: R_i16;
    r.f0 = cast(s16) (seed + 1);
    return r;
}

is_i16 :: (r: R_i16, seed: s32) -> bool {
    return r.f0 == cast(s16) (seed + 1);
}

R_i8i16 :: struct {
    f0: s8;
    f1: s16;
}

ret_i8i16 :: (seed: s32) -> R_i8i16 #foreign returns;
check_i8i16 :: (r: R_i8i16, seed: s32) -> s32 #foreign returns;
call_i8i16 :: (cb: (seed: s32) -> R_i8i16 #c_call, seed: s32) -> R_i8i16 #foreign returns;
apply_check_i8i16 :: (cb: (seed: s32) -> R_i8i16 #c_call, seed: s32) -> s32 #foreign returns;

make_i8i16 :: (seed: s32) -> R_i8i16 #c_call {
    r: R_i8i16;
    r.f0 = cast(s8) (seed + 1);
    r.f1 = cast(s16) (seed + 2);
    return r;
}

is_i8i16 :: (r: R_i8i16, seed: s32) -> bool {
    return r.f0 == cast(s8) (seed + 1) && r.f1 == cast(s16) (seed + 2);
}

R_i32 :: struct {
    f0: s32;
}

ret_i32 :: (seed: s32) -> R_i32 #foreign returns;
check_i32 :: (r: R_i32, seed: s32) -> s32 #foreign returns;
call_i32 :: (cb: (seed: s32) -> R_i32 #c_call, seed: s32) -> R_i32 #foreign returns;
apply_check_i32 :: (cb: (seed: s32) -> R_i32 #c_call, seed: s32) -> s32 #foreign returns;

make_i32 :: (seed: s32) -> R_i32 #c_call {
    r: R_i32;
    r.f0 = cast(s32) (seed + 1);
    return r;
}

is_i32 :: (r: R_i32, seed: s32) -> bool {
    return r.f0 == cast(s32) (seed + 1);
}

R_i64 :: struct {
    f0: s64;
}

ret_i64 :: (seed: s32) -> R_i64 #foreign returns;
check_i64 :: (r: R_i64, seed: s32) -> s32 #foreign returns;
call_i64 :: (cb: (seed: s32) -> R_i64 #c_call, seed: s32) -> R_i64 #foreign returns;
apply_check_i64 :: (cb: (seed: s32) -> R_i64 #c_call, seed: s32) -> s32 #foreign returns;

make_i64 :: (seed: s32) -> R_i64 #c_call {
    r: R_i64;
    r.f0 = cast(s64) (seed + 1);
    return r;
}

is_i64 :: (r: R_i64, seed: s32) -> bool {
    return r.f0 == cast(s64) (seed + 1);
}

R_ptr :: struct {
    f0: *void;
}

ret_ptr :: (seed: s32) -> R_ptr #foreign returns;
check_ptr :: (r: R_ptr, seed: s32) -> s32 #foreign returns;
call_ptr :: (cb: (seed: s32) -> R_ptr #c_call, seed: s32) -> R_ptr #foreign returns;
apply_check_ptr :: (cb: (seed: s32) -> R_ptr #c_call, seed: s32) -> s32 #foreign returns;

make_ptr :: (seed: s32) -> R_ptr #c_call {
    r: R_ptr;
    r.f0 = cast(*void) cast(s64) (seed + 1);
    return r;
}

is_ptr :: (r: R_ptr, seed: s32) -> bool {
    return r.f0 == cast(*void) cast(s64) (seed + 1);
}

R_i32x2 :: struct {
    f0: s32;
    f1: s32;
}

ret_i32x2 :: (seed: s32) -> R_i32x2 #foreign returns;
check_i32x2 :: (r: R_i32x2, seed: s32) -> s32 #foreign returns;
call_i32x2 :: (cb: (seed: s32) -> R_i32x2 #c_call, seed: s32) -> R_i32x2 #foreign returns;
apply_check_i32x2 :: (cb: (seed: s32) -> R_i32x2 #c_call, seed: s32) -> s32 #foreign returns;

make_i32x2 :: (seed: s32) -> R_i32x2 #c_call {
    r: R_i32x2;
    r.f0 = cast(s32) (seed + 1);
    r.f1 = cast(s32) (seed + 2);
    return r;
}

is_i32x2 :: (r: R_i32x2, seed: s32) -> bool {
    return r.f0 == cast(s32) (seed + 1) && r.f1 == cast(s32) (seed + 2);
}

R_i32x3 :: struct {
    f0: s32;
    f1: s32;
    f2: s32;
}

ret_i32x3 :: (seed: s32) -> R_i32x3 #foreign returns;
check_i32x3 :: (r: R_i32x3, seed: s32) -> s32 #foreign returns;
call_i32x3 :: (cb: (seed: s32) -> R_i32x3 #c_call, seed: s32) -> R_i32x3 #foreign returns;
apply_check_i32x3 :: (cb: (seed: s32) -> R_i32x3 #c_call, seed: s32) -> s32 #foreign returns;

make_i32x3 :: (seed: s32) -> R_i32x3 #c_call {
    r: R_i32x3;
    r.f0 = cast(s32) (seed + 1);
    r.f1 = cast(s32) (seed + 2);
    r.f2 = cast(s32) (seed + 3);
    return r;
}

is_i32x3 :: (r: R_i32x3, seed: s32) -> bool {
    return r.f0 == cast(s32) (seed + 1) && r.f1 == cast(s32) (seed + 2) && r.f2 == cast(s32) (seed + 3);
}

R_i32x4 :: struct {
    f0: s32;
    f1: s32;
    f2: s32;
    f3: s32;
}

ret_i32x4 :: (seed: s32) -> R_i32x4 #foreign returns;
check_i32x4 :: (r: R_i32x4, seed: s32) -> s32 #foreign returns;
call_i32x4 :: (cb: (seed: s32) -> R_i32x4 #c_call, seed: s32) -> R_i32x4 #foreign returns;
apply_check_i32x4 :: (cb: (seed: s32) -> R_i32x4 #c_call, seed: s32) -> s32 #foreign returns;

make_i32x4 :: (seed: s32) -> R_i32x4 #c_call {
    r: R_i32x4;
    r.f0 = cast(s32) (seed + 1);
    r.f1 = cast(s32) (seed + 2);
    r.f2 = cast(s32) (seed + 3);
    r.f3 = cast(s32) (seed + 4);
    return r;
}

is_i32x4 :: (r: R_i32x4, seed: s32) -> bool {
    return r.f0 == cast(s32) (seed + 1) && r.f1 == cast(s32) (seed + 2) && r.f2 == cast(s32) (seed + 3) && r.f3 == cast(s32) (seed + 4);
}

R_i32x5 :: struct {
    f0: s32;
    f1: s32;
    f2: s32;
    f3: s32;
    f4: s32;
}

ret_i32x5 :: (seed: s32) -> R_i32x5 #foreign returns;
check_i32x5 :: (r: R_i32x5, seed: s32) -> s32 #foreign returns;
call_i32x5 :: (cb: (seed: s32) -> R_i32x5 #c_call, seed: s32) -> R_i32x5 #foreign returns;
apply_check_i32x5 :: (cb: (seed: s32) -> R_i32x5 #c_call, seed: s32) -> s32 #foreign returns;

make_i32x5 :: (seed: s32) -> R_i32x5 #c_call {
    r: R_i32x5;
    r.f0 = cast(s32) (seed + 1);
    r.f1 = cast(s32) (seed + 2);
    r.f2 = cast(s32) (seed + 3);
    r.f3 = cast(s32) (seed + 4);
    r.f4 = cast(s32) (seed + 5);
    return r;
}

is_i32x5 :: (r: R_i32x5, seed: s32) -> bool {
    return r.f0 == cast(s32) (seed + 1) && r.f1 == cast(s32) (seed + 2) && r.f2 == cast(s32) (seed + 3) && r.f3 == cast(s32) (seed + 4) && r.f4 == cast(s32) (seed + 5);
}

R_i64x2 :: struct {
    f0: s64;
    f1: s64;
}

ret_i64x2 :: (seed: s32) -> R_i64x2 #foreign returns;
check_i64x2 :: (r: R_i64x2, seed: s32) -> s32 #foreign returns;
call_i64x2 :: (cb: (seed: s32) -> R_i64x2 #c_call, seed: s32) -> R_i64x2 #foreign returns;
apply_check_i64x2 :: (cb: (seed: s32) -> R_i64x2 #c_call, seed: s32) -> s32 #foreign returns;

make_i64x2 :: (seed: s32) -> R_i64x2 #c_call {
    r: R_i64x2;
    r.f0 = cast(s64) (seed + 1);
    r.f1 = cast(s64) (seed + 2);
    return r;
}

is_i64x2 :: (r: R_i64x2, seed: s32) -> bool {
    return r.f0 == cast(s64) (seed + 1) && r.f1 == cast(s64) (seed + 2);
}

R_i64x3 :: struct {
    f0: s64;
    f1: s64;
    f2: s64;
}

ret_i64x3 :: (seed: s32) -> R_i64x3 #foreign returns;
check_i64x3 :: (r: R_i64x3, seed: s32) -> s32 #foreign returns;
call_i64x3 :: (cb: (seed: s32) -> R_i64x3 #c_call, seed: s32) -> R_i64x3 #foreign returns;
apply_check_i64x3 :: (cb: (seed: s32) -> R_i64x3 #c_call, seed: s32) -> s32 #foreign returns;

make_i64x3 :: (seed: s32) -> R_i64x3 #c_call {
    r: R_i64x3;
    r.f0 = cast(s64) (seed + 1);
    r.f1 = cast(s64) (seed + 2);
    r.f2 = cast(s64) (seed + 3);
    return r;
}

is_i64x3 :: (r: R_i64x3, seed: s32) -> bool {
    return r.f0 == cast(s64) (seed + 1) && r.f1 == cast(s64) (seed + 2) && r.f2 == cast(s64) (seed + 3);
}

R_i64x4 :: struct {
    f0: s64;
    f1: s64;
    f2: s64;
    f3: s64;
}

ret_i64x4 :: (seed: s32) -> R_i64x4 #foreign returns;
check_i64x4 :: (r: R_i64x4, seed: s32) -> s32 #foreign returns;
call_i64x4 :: (cb: (seed: s32) -> R_i64x4 #c_call, seed: s32) -> R_i64x4 #foreign returns;
apply_check_i64x4 :: (cb: (seed: s32) -> R_i64x4 #c_call, seed: s32) -> s32 #foreign returns;

make_i64x4 :: (seed: s32) -> R_i64x4 #c_call {
    r: R_i64x4;
    r.f0 = cast(s64) (seed + 1);
    r.f1 = cast(s64) (seed + 2);
    r.f2 = cast(s64) (seed + 3);
    r.f3 = cast(s64) (seed + 4);
    return r;
}

is_i64x4 :: (r: R_i64x4, seed: s32) -> bool {
    return r.f0 == cast(s64) (seed + 1) && r.f1 == cast(s64) (seed + 2) && r.f2 == cast(s64) (seed + 3) && r.f3 == cast(s64) (seed + 4);
}

R_i8x9 :: struct {
    f0: s8;
    f1: s8;
    f2: s8;
    f3: s8;
    f4: s8;
    f5: s8;
    f6: s8;
    f7: s8;
    f8: s8;
}

ret_i8x9 :: (seed: s32) -> R_i8x9 #foreign returns;
check_i8x9 :: (r: R_i8x9, seed: s32) -> s32 #foreign returns;
call_i8x9 :: (cb: (seed: s32) -> R_i8x9 #c_call, seed: s32) -> R_i8x9 #foreign returns;
apply_check_i8x9 :: (cb: (seed: s32) -> R_i8x9 #c_call, seed: s32) -> s32 #foreign returns;

make_i8x9 :: (seed: s32) -> R_i8x9 #c_call {
    r: R_i8x9;
    r.f0 = cast(s8) (seed + 1);
    r.f1 = cast(s8) (seed + 2);
    r.f2 = cast(s8) (seed + 3);
    r.f3 = cast(s8) (seed + 4);
    r.f4 = cast(s8) (seed + 5);
    r.f5 = cast(s8) (seed + 6);
    r.f6 = cast(s8) (seed + 7);
    r.f7 = cast(s8) (seed + 8);
    r.f8 = cast(s8) (seed + 9);
    return r;
}

is_i8x9 :: (r: R_i8x9, seed: s32) -> bool {
    return r.f0 == cast(s8) (seed + 1) && r.f1 == cast(s8) (seed + 2) && r.f2 == cast(s8) (seed + 3) && r.f3 == cast(s8) (seed + 4) && r.f4 == cast(s8) (seed + 5) && r.f5 == cast(s8) (seed + 6) && r.f6 == cast(s8) (seed + 7) && r.f7 == cast(s8) (seed + 8) && r.f8 == cast(s8) (seed + 9);
}

R_i8x17 :: struct {
    f0: s8;
    f1: s8;
    f2: s8;
    f3: s8;
    f4: s8;
    f5: s8;
    f6: s8;
    f7: s8;
    f8: s8;
    f9: s8;
    f10: s8;
    f11: s8;
    f12: s8;
    f13: s8;
    f14: s8;
    f15: s8;
    f16: s8;
}

ret_i8x17 :: (seed: s32) -> R_i8x17 #foreign returns;
check_i8x17 :: (r: R_i8x17, seed: s32) -> s32 #foreign returns;
call_i8x17 :: (cb: (seed: s32) -> R_i8x17 #c_call, seed: s32) -> R_i8x17 #foreign returns;
apply_check_i8x17 :: (cb: (seed: s32) -> R_i8x17 #c_call, seed: s32) -> s32 #foreign returns;

make_i8x17 :: (seed: s32) -> R_i8x17 #c_call {
    r: R_i8x17;
    r.f0 = cast(s8) (seed + 1);
    r.f1 = cast(s8) (seed + 2);
    r.f2 = cast(s8) (seed + 3);
    r.f3 = cast(s8) (seed + 4);
    r.f4 = cast(s8) (seed + 5);
    r.f5 = cast(s8) (seed + 6);
    r.f6 = cast(s8) (seed + 7);
    r.f7 = cast(s8) (seed + 8);
    r.f8 = cast(s8) (seed + 9);
    r.f9 = cast(s8) (seed + 10);
    r.f10 = cast(s8) (seed + 11);
    r.f11 = cast(s8) (seed + 12);
    r.f12 = cast(s8) (seed + 13);
    r.f13 = cast(s8) (seed + 14);
    r.f14 = cast(s8) (seed + 15);
    r.f15 = cast(s8) (seed + 16);
    r.f16 = cast(s8) (seed + 17);
    return r;
}

is_i8x17 :: (r: R_i8x17, seed: s32) -> bool {
    return r.f0 == cast(s8) (seed + 1) && r.f1 == cast(s8) (seed + 2) && r.f2 == cast(s8) (seed + 3) && r.f3 == cast(s8) (seed + 4) && r.f4 == cast(s8) (seed + 5) && r.f5 == cast(s8) (seed + 6) && r.f6 == cast(s8) (seed + 7) && r.f7 == cast(s8) (seed + 8) && r.f8 == cast(s8) (seed + 9) && r.f9 == cast(s8) (seed + 10) && r.f10 == cast(s8) (seed + 11) && r.f11 == cast(s8) (seed + 12) && r.f12 == cast(s8) (seed + 13) && r.f13 == cast(s8) (seed + 14) && r.f14 == cast(s8) (seed + 15) && r.f15 == cast(s8) (seed + 16) && r.f16 == cast(s8) (seed + 17);
}

R_i8i64 :: struct {
    f0: s8;
    f1: s64;
}

ret_i8i64 :: (seed: s32) -> R_i8i64 #foreign returns;
check_i8i64 :: (r: R_i8i64, seed: s32) -> s32 #foreign returns;
call_i8i64 :: (cb: (seed: s32) -> R_i8i64 #c_call, seed: s32) -> R_i8i64 #foreign returns;
apply_check_i8i64 :: (cb: (seed: s32) -> R_i8i64 #c_call, seed: s32) -> s32 #foreign returns;

make_i8i64 :: (seed: s32) -> R_i8i64 #c_call {
    r: R_i8i64;
    r.f0 = cast(s8) (seed + 1);
    r.f1 = cast(s64) (seed + 2);
    return r;
}

is_i8i64 :: (r: R_i8i64, seed: s32) -> bool {
    return r.f0 == cast(s8) (seed + 1) && r.f1 == cast(s64) (seed + 2);
}

R_i32i8 :: struct {
    f0: s32;
    f1: s8;
}

ret_i32i8 :: (seed: s32) -> R_i32i8 #foreign returns;
check_i32i8 :: (r: R_i32i8, seed: s32) -> s32 #foreign returns;
call_i32i8 :: (cb: (seed: s32) -> R_i32i8 #c_call, seed: s32) -> R_i32i8 #foreign returns;
apply_check_i32i8 :: (cb: (seed: s32) -> R_i32i8 #c_call, seed: s32) -> s32 #foreign returns;

make_i32i8 :: (seed: s32) -> R_i32i8 #c_call {
    r: R_i32i8;
    r.f0 = cast(s32) (seed + 1);
    r.f1 = cast(s8) (seed + 2);
    return r;
}

is_i32i8 :: (r: R_i32i8, seed: s32) -> bool {
    return r.f0 == cast(s32) (seed + 1) && r.f1 == cast(s8) (seed + 2);
}

R_i16x3 :: struct {
    f0: s16;
    f1: s16;
    f2: s16;
}

ret_i16x3 :: (seed: s32) -> R_i16x3 #foreign returns;
check_i16x3 :: (r: R_i16x3, seed: s32) -> s32 #foreign returns;
call_i16x3 :: (cb: (seed: s32) -> R_i16x3 #c_call, seed: s32) -> R_i16x3 #foreign returns;
apply_check_i16x3 :: (cb: (seed: s32) -> R_i16x3 #c_call, seed: s32) -> s32 #foreign returns;

make_i16x3 :: (seed: s32) -> R_i16x3 #c_call {
    r: R_i16x3;
    r.f0 = cast(s16) (seed + 1);
    r.f1 = cast(s16) (seed + 2);
    r.f2 = cast(s16) (seed + 3);
    return r;
}

is_i16x3 :: (r: R_i16x3, seed: s32) -> bool {
    return r.f0 == cast(s16) (seed + 1) && r.f1 == cast(s16) (seed + 2) && r.f2 == cast(s16) (seed + 3);
}

R_f32 :: struct {
    f0: float32;
}

ret_f32 :: (seed: s32) -> R_f32 #foreign returns;
check_f32 :: (r: R_f32, seed: s32) -> s32 #foreign returns;
call_f32 :: (cb: (seed: s32) -> R_f32 #c_call, seed: s32) -> R_f32 #foreign returns;
apply_check_f32 :: (cb: (seed: s32) -> R_f32 #c_call, seed: s32) -> s32 #foreign returns;

make_f32 :: (seed: s32) -> R_f32 #c_call {
    r: R_f32;
    r.f0 = cast(float32) (seed + 0.5);
    return r;
}

is_f32 :: (r: R_f32, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5);
}

R_f32x2 :: struct {
    f0: float32;
    f1: float32;
}

ret_f32x2 :: (seed: s32) -> R_f32x2 #foreign returns;
check_f32x2 :: (r: R_f32x2, seed: s32) -> s32 #foreign returns;
call_f32x2 :: (cb: (seed: s32) -> R_f32x2 #c_call, seed: s32) -> R_f32x2 #foreign returns;
apply_check_f32x2 :: (cb: (seed: s32) -> R_f32x2 #c_call, seed: s32) -> s32 #foreign returns;

make_f32x2 :: (seed: s32) -> R_f32x2 #c_call {
    r: R_f32x2;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(float32) (seed + 1.5);
    return r;
}

is_f32x2 :: (r: R_f32x2, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(float32) (seed + 1.5);
}

R_f32x3 :: struct {
    f0: float32;
    f1: float32;
    f2: float32;
}

ret_f32x3 :: (seed: s32) -> R_f32x3 #foreign returns;
check_f32x3 :: (r: R_f32x3, seed: s32) -> s32 #foreign returns;
call_f32x3 :: (cb: (seed: s32) -> R_f32x3 #c_call, seed: s32) -> R_f32x3 #foreign returns;
apply_check_f32x3 :: (cb: (seed: s32) -> R_f32x3 #c_call, seed: s32) -> s32 #foreign returns;

make_f32x3 :: (seed: s32) -> R_f32x3 #c_call {
    r: R_f32x3;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(float32) (seed + 1.5);
    r.f2 = cast(float32) (seed + 2.5);
    return r;
}

is_f32x3 :: (r: R_f32x3, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(float32) (seed + 1.5) && r.f2 == cast(float32) (seed + 2.5);
}

R_f32x4 :: struct {
    f0: float32;
    f1: float32;
    f2: float32;
    f3: float32;
}

ret_f32x4 :: (seed: s32) -> R_f32x4 #foreign returns;
check_f32x4 :: (r: R_f32x4, seed: s32) -> s32 #foreign returns;
call_f32x4 :: (cb: (seed: s32) -> R_f32x4 #c_call, seed: s32) -> R_f32x4 #foreign returns;
apply_check_f32x4 :: (cb: (seed: s32) -> R_f32x4 #c_call, seed: s32) -> s32 #foreign returns;

make_f32x4 :: (seed: s32) -> R_f32x4 #c_call {
    r: R_f32x4;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(float32) (seed + 1.5);
    r.f2 = cast(float32) (seed + 2.5);
    r.f3 = cast(float32) (seed + 3.5);
    return r;
}

is_f32x4 :: (r: R_f32x4, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(float32) (seed + 1.5) && r.f2 == cast(float32) (seed + 2.5) && r.f3 == cast(float32) (seed + 3.5);
}

R_f32x5 :: struct {
    f0: float32;
    f1: float32;
    f2: float32;
    f3: float32;
    f4: float32;
}

ret_f32x5 :: (seed: s32) -> R_f32x5 #foreign returns;
check_f32x5 :: (r: R_f32x5, seed: s32) -> s32 #foreign returns;
call_f32x5 :: (cb: (seed: s32) -> R_f32x5 #c_call, seed: s32) -> R_f32x5 #foreign returns;
apply_check_f32x5 :: (cb: (seed: s32) -> R_f32x5 #c_call, seed: s32) -> s32 #foreign returns;

make_f32x5 :: (seed: s32) -> R_f32x5 #c_call {
    r: R_f32x5;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(float32) (seed + 1.5);
    r.f2 = cast(float32) (seed + 2.5);
    r.f3 = cast(float32) (seed + 3.5);
    r.f4 = cast(float32) (seed + 4.5);
    return r;
}

is_f32x5 :: (r: R_f32x5, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(float32) (seed + 1.5) && r.f2 == cast(float32) (seed + 2.5) && r.f3 == cast(float32) (seed + 3.5) && r.f4 == cast(float32) (seed + 4.5);
}

R_f64 :: struct {
    f0: float64;
}

ret_f64 :: (seed: s32) -> R_f64 #foreign returns;
check_f64 :: (r: R_f64, seed: s32) -> s32 #foreign returns;
call_f64 :: (cb: (seed: s32) -> R_f64 #c_call, seed: s32) -> R_f64 #foreign returns;
apply_check_f64 :: (cb: (seed: s32) -> R_f64 #c_call, seed: s32) -> s32 #foreign returns;

make_f64 :: (seed: s32) -> R_f64 #c_call {
    r: R_f64;
    r.f0 = cast(float64) (seed + 0.5);
    return r;
}

is_f64 :: (r: R_f64, seed: s32) -> bool {
    return r.f0 == cast(float64) (seed + 0.5);
}

R_f64x2 :: struct {
    f0: float64;
    f1: float64;
}

ret_f64x2 :: (seed: s32) -> R_f64x2 #foreign returns;
check_f64x2 :: (r: R_f64x2, seed: s32) -> s32 #foreign returns;
call_f64x2 :: (cb: (seed: s32) -> R_f64x2 #c_call, seed: s32) -> R_f64x2 #foreign returns;
apply_check_f64x2 :: (cb: (seed: s32) -> R_f64x2 #c_call, seed: s32) -> s32 #foreign returns;

make_f64x2 :: (seed: s32) -> R_f64x2 #c_call {
    r: R_f64x2;
    r.f0 = cast(float64) (seed + 0.5);
    r.f1 = cast(float64) (seed + 1.5);
    return r;
}

is_f64x2 :: (r: R_f64x2, seed: s32) -> bool {
    return r.f0 == cast(float64) (seed + 0.5) && r.f1 == cast(float64) (seed + 1.5);
}

R_f64x3 :: struct {
    f0: float64;
    f1: float64;
    f2: float64;
}

ret_f64x3 :: (seed: s32) -> R_f64x3 #foreign returns;
check_f64x3 :: (r: R_f64x3, seed: s32) -> s32 #foreign returns;
call_f64x3 :: (cb: (seed: s32) -> R_f64x3 #c_call, seed: s32) -> R_f64x3 #foreign returns;
apply_check_f64x3 :: (cb: (seed: s32) -> R_f64x3 #c_call, seed: s32) -> s32 #foreign returns;

make_f64x3 :: (seed: s32) -> R_f64x3 #c_call {
    r: R_f64x3;
    r.f0 = cast(float64) (seed + 0.5);
    r.f1 = cast(float64) (seed + 1.5);
    r.f2 = cast(float64) (seed + 2.5);
    return r;
}

is_f64x3 :: (r: R_f64x3, seed: s32) -> bool {
    return r.f0 == cast(float64) (seed + 0.5) && r.f1 == cast(float64) (seed + 1.5) && r.f2 == cast(float64) (seed + 2.5);
}

R_f64x4 :: struct {
    f0: float64;
    f1: float64;
    f2: float64;
    f3: float64;
}

ret_f64x4 :: (seed: s32) -> R_f64x4 #foreign returns;
check_f64x4 :: (r: R_f64x4, seed: s32) -> s32 #foreign returns;
call_f64x4 :: (cb: (seed: s32) -> R_f64x4 #c_call, seed: s32) -> R_f64x4 #foreign returns;
apply_check_f64x4 :: (cb: (seed: s32) -> R_f64x4 #c_call, seed: s32) -> s32 #foreign returns;

make_f64x4 :: (seed: s32) -> R_f64x4 #c_call {
    r: R_f64x4;
    r.f0 = cast(float64) (seed + 0.5);
    r.f1 = cast(float64) (seed + 1.5);
    r.f2 = cast(float64) (seed + 2.5);
    r.f3 = cast(float64) (seed + 3.5);
    return r;
}

is_f64x4 :: (r: R_f64x4, seed: s32) -> bool {
    return r.f0 == cast(float64) (seed + 0.5) && r.f1 == cast(float64) (seed + 1.5) && r.f2 == cast(float64) (seed + 2.5) && r.f3 == cast(float64) (seed + 3.5);
}

R_f64x5 :: struct {
    f0: float64;
    f1: float64;
    f2: float64;
    f3: float64;
    f4: float64;
}

ret_f64x5 :: (seed: s32) -> R_f64x5 #foreign returns;
check_f64x5 :: (r: R_f64x5, seed: s32) -> s32 #foreign returns;
call_f64x5 :: (cb: (seed: s32) -> R_f64x5 #c_call, seed: s32) -> R_f64x5 #foreign returns;
apply_check_f64x5 :: (cb: (seed: s32) -> R_f64x5 #c_call, seed: s32) -> s32 #foreign returns;

make_f64x5 :: (seed: s32) -> R_f64x5 #c_call {
    r: R_f64x5;
    r.f0 = cast(float64) (seed + 0.5);
    r.f1 = cast(float64) (seed + 1.5);
    r.f2 = cast(float64) (seed + 2.5);
    r.f3 = cast(float64) (seed + 3.5);
    r.f4 = cast(float64) (seed + 4.5);
    return r;
}

is_f64x5 :: (r: R_f64x5, seed: s32) -> bool {
    return r.f0 == cast(float64) (seed + 0.5) && r.f1 == cast(float64) (seed + 1.5) && r.f2 == cast(float64) (seed + 2.5) && r.f3 == cast(float64) (seed + 3.5) && r.f4 == cast(float64) (seed + 4.5);
}

R_i32f32 :: struct {
    f0: s32;
    f1: float32;
}

ret_i32f32 :: (seed: s32) -> R_i32f32 #foreign returns;
check_i32f32 :: (r: R_i32f32, seed: s32) -> s32 #foreign returns;
call_i32f32 :: (cb: (seed: s32) -> R_i32f32 #c_call, seed: s32) -> R_i32f32 #foreign returns;
apply_check_i32f32 :: (cb: (seed: s32) -> R_i32f32 #c_call, seed: s32) -> s32 #foreign returns;

make_i32f32 :: (seed: s32) -> R_i32f32 #c_call {
    r: R_i32f32;
    r.f0 = cast(s32) (seed + 1);
    r.f1 = cast(float32) (seed + 1.5);
    return r;
}

is_i32f32 :: (r: R_i32f32, seed: s32) -> bool {
    return r.f0 == cast(s32) (seed + 1) && r.f1 == cast(float32) (seed + 1.5);
}

R_f32i32 :: struct {
    f0: float32;
    f1: s32;
}

ret_f32i32 :: (seed: s32) -> R_f32i32 #foreign returns;
check_f32i32 :: (r: R_f32i32, seed: s32) -> s32 #foreign returns;
call_f32i32 :: (cb: (seed: s32) -> R_f32i32 #c_call, seed: s32) -> R_f32i32 #foreign returns;
apply_check_f32i32 :: (cb: (seed: s32) -> R_f32i32 #c_call, seed: s32) -> s32 #foreign returns;

make_f32i32 :: (seed: s32) -> R_f32i32 #c_call {
    r: R_f32i32;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(s32) (seed + 2);
    return r;
}

is_f32i32 :: (r: R_f32i32, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(s32) (seed + 2);
}

R_i64f64 :: struct {
    f0: s64;
    f1: float64;
}

ret_i64f64 :: (seed: s32) -> R_i64f64 #foreign returns;
check_i64f64 :: (r: R_i64f64, seed: s32) -> s32 #foreign returns;
call_i64f64 :: (cb: (seed: s32) -> R_i64f64 #c_call, seed: s32) -> R_i64f64 #foreign returns;
apply_check_i64f64 :: (cb: (seed: s32) -> R_i64f64 #c_call, seed: s32) -> s32 #foreign returns;

make_i64f64 :: (seed: s32) -> R_i64f64 #c_call {
    r: R_i64f64;
    r.f0 = cast(s64) (seed + 1);
    r.f1 = cast(float64) (seed + 1.5);
    return r;
}

is_i64f64 :: (r: R_i64f64, seed: s32) -> bool {
    return r.f0 == cast(s64) (seed + 1) && r.f1 == cast(float64) (seed + 1.5);
}

R_f64i64 :: struct {
    f0: float64;
    f1: s64;
}

ret_f64i64 :: (seed: s32) -> R_f64i64 #foreign returns;
check_f64i64 :: (r: R_f64i64, seed: s32) -> s32 #foreign returns;
call_f64i64 :: (cb: (seed: s32) -> R_f64i64 #c_call, seed: s32) -> R_f64i64 #foreign returns;
apply_check_f64i64 :: (cb: (seed: s32) -> R_f64i64 #c_call, seed: s32) -> s32 #foreign returns;

make_f64i64 :: (seed: s32) -> R_f64i64 #c_call {
    r: R_f64i64;
    r.f0 = cast(float64) (seed + 0.5);
    r.f1 = cast(s64) (seed + 2);
    return r;
}

is_f64i64 :: (r: R_f64i64, seed: s32) -> bool {
    return r.f0 == cast(float64) (seed + 0.5) && r.f1 == cast(s64) (seed + 2);
}

R_f32x2i64 :: struct {
    f0: float32;
    f1: float32;
    f2: s64;
}

ret_f32x2i64 :: (seed: s32) -> R_f32x2i64 #foreign returns;
check_f32x2i64 :: (r: R_f32x2i64, seed: s32) -> s32 #foreign returns;
call_f32x2i64 :: (cb: (seed: s32) -> R_f32x2i64 #c_call, seed: s32) -> R_f32x2i64 #foreign returns;
apply_check_f32x2i64 :: (cb: (seed: s32) -> R_f32x2i64 #c_call, seed: s32) -> s32 #foreign returns;

make_f32x2i64 :: (seed: s32) -> R_f32x2i64 #c_call {
    r: R_f32x2i64;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(float32) (seed + 1.5);
    r.f2 = cast(s64) (seed + 3);
    return r;
}

is_f32x2i64 :: (r: R_f32x2i64, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(float32) (seed + 1.5) && r.f2 == cast(s64) (seed + 3);
}

R_i64f32x2 :: struct {
    f0: s64;
    f1: float32;
    f2: float32;
}

ret_i64f32x2 :: (seed: s32) -> R_i64f32x2 #foreign returns;
check_i64f32x2 :: (r: R_i64f32x2, seed: s32) -> s32 #foreign returns;
call_i64f32x2 :: (cb: (seed: s32) -> R_i64f32x2 #c_call, seed: s32) -> R_i64f32x2 #foreign returns;
apply_check_i64f32x2 :: (cb: (seed: s32) -> R_i64f32x2 #c_call, seed: s32) -> s32 #foreign returns;

make_i64f32x2 :: (seed: s32) -> R_i64f32x2 #c_call {
    r: R_i64f32x2;
    r.f0 = cast(s64) (seed + 1);
    r.f1 = cast(float32) (seed + 1.5);
    r.f2 = cast(float32) (seed + 2.5);
    return r;
}

is_i64f32x2 :: (r: R_i64f32x2, seed: s32) -> bool {
    return r.f0 == cast(s64) (seed + 1) && r.f1 == cast(float32) (seed + 1.5) && r.f2 == cast(float32) (seed + 2.5);
}

R_f64f32 :: struct {
    f0: float64;
    f1: float32;
}

ret_f64f32 :: (seed: s32) -> R_f64f32 #foreign returns;
check_f64f32 :: (r: R_f64f32, seed: s32) -> s32 #foreign returns;
call_f64f32 :: (cb: (seed: s32) -> R_f64f32 #c_call, seed: s32) -> R_f64f32 #foreign returns;
apply_check_f64f32 :: (cb: (seed: s32) -> R_f64f32 #c_call, seed: s32) -> s32 #foreign returns;

make_f64f32 :: (seed: s32) -> R_f64f32 #c_call {
    r: R_f64f32;
    r.f0 = cast(float64) (seed + 0.5);
    r.f1 = cast(float32) (seed + 1.5);
    return r;
}

is_f64f32 :: (r: R_f64f32, seed: s32) -> bool {
    return r.f0 == cast(float64) (seed + 0.5) && r.f1 == cast(float32) (seed + 1.5);
}

R_f32f64 :: struct {
    f0: float32;
    f1: float64;
}

ret_f32f64 :: (seed: s32) -> R_f32f64 #foreign returns;
check_f32f64 :: (r: R_f32f64, seed: s32) -> s32 #foreign returns;
call_f32f64 :: (cb: (seed: s32) -> R_f32f64 #c_call, seed: s32) -> R_f32f64 #foreign returns;
apply_check_f32f64 :: (cb: (seed: s32) -> R_f32f64 #c_call, seed: s32) -> s32 #foreign returns;

make_f32f64 :: (seed: s32) -> R_f32f64 #c_call {
    r: R_f32f64;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(float64) (seed + 1.5);
    return r;
}

is_f32f64 :: (r: R_f32f64, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(float64) (seed + 1.5);
}

R_i8f32 :: struct {
    f0: s8;
    f1: float32;
}

ret_i8f32 :: (seed: s32) -> R_i8f32 #foreign returns;
check_i8f32 :: (r: R_i8f32, seed: s32) -> s32 #foreign returns;
call_i8f32 :: (cb: (seed: s32) -> R_i8f32 #c_call, seed: s32) -> R_i8f32 #foreign returns;
apply_check_i8f32 :: (cb: (seed: s32) -> R_i8f32 #c_call, seed: s32) -> s32 #foreign returns;

make_i8f32 :: (seed: s32) -> R_i8f32 #c_call {
    r: R_i8f32;
    r.f0 = cast(s8) (seed + 1);
    r.f1 = cast(float32) (seed + 1.5);
    return r;
}

is_i8f32 :: (r: R_i8f32, seed: s32) -> bool {
    return r.f0 == cast(s8) (seed + 1) && r.f1 == cast(float32) (seed + 1.5);
}

R_f32x3i32 :: struct {
    f0: float32;
    f1: float32;
    f2: float32;
    f3: s32;
}

ret_f32x3i32 :: (seed: s32) -> R_f32x3i32 #foreign returns;
check_f32x3i32 :: (r: R_f32x3i32, seed: s32) -> s32 #foreign returns;
call_f32x3i32 :: (cb: (seed: s32) -> R_f32x3i32 #c_call, seed: s32) -> R_f32x3i32 #foreign returns;
apply_check_f32x3i32 :: (cb: (seed: s32) -> R_f32x3i32 #c_call, seed: s32) -> s32 #foreign returns;

make_f32x3i32 :: (seed: s32) -> R_f32x3i32 #c_call {
    r: R_f32x3i32;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(float32) (seed + 1.5);
    r.f2 = cast(float32) (seed + 2.5);
    r.f3 = cast(s32) (seed + 4);
    return r;
}

is_f32x3i32 :: (r: R_f32x3i32, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(float32) (seed + 1.5) && r.f2 == cast(float32) (seed + 2.5) && r.f3 == cast(s32) (seed + 4);
}

R_f64i8 :: struct {
    f0: float64;
    f1: s8;
}

ret_f64i8 :: (seed: s32) -> R_f64i8 #foreign returns;
check_f64i8 :: (r: R_f64i8, seed: s32) -> s32 #foreign returns;
call_f64i8 :: (cb: (seed: s32) -> R_f64i8 #c_call, seed: s32) -> R_f64i8 #foreign returns;
apply_check_f64i8 :: (cb: (seed: s32) -> R_f64i8 #c_call, seed: s32) -> s32 #foreign returns;

make_f64i8 :: (seed: s32) -> R_f64i8 #c_call {
    r: R_f64i8;
    r.f0 = cast(float64) (seed + 0.5);
    r.f1 = cast(s8) (seed + 2);
    return r;
}

is_f64i8 :: (r: R_f64i8, seed: s32) -> bool {
    return r.f0 == cast(float64) (seed + 0.5) && r.f1 == cast(s8) (seed + 2);
}

R_f32i64 :: struct {
    f0: float32;
    f1: s64;
}

ret_f32i64 :: (seed: s32) -> R_f32i64 #foreign returns;
check_f32i64 :: (r: R_f32i64, seed: s32) -> s32 #foreign returns;
call_f32i64 :: (cb: (seed: s32) -> R_f32i64 #c_call, seed: s32) -> R_f32i64 #foreign returns;
apply_check_f32i64 :: (cb: (seed: s32) -> R_f32i64 #c_call, seed: s32) -> s32 #foreign returns;

make_f32i64 :: (seed: s32) -> R_f32i64 #c_call {
    r: R_f32i64;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(s64) (seed + 2);
    return r;
}

is_f32i64 :: (r: R_f32i64, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(s64) (seed + 2);
}

R_i64f32 :: struct {
    f0: s64;
    f1: float32;
}

ret_i64f32 :: (seed: s32) -> R_i64f32 #foreign returns;
check_i64f32 :: (r: R_i64f32, seed: s32) -> s32 #foreign returns;
call_i64f32 :: (cb: (seed: s32) -> R_i64f32 #c_call, seed: s32) -> R_i64f32 #foreign returns;
apply_check_i64f32 :: (cb: (seed: s32) -> R_i64f32 #c_call, seed: s32) -> s32 #foreign returns;

make_i64f32 :: (seed: s32) -> R_i64f32 #c_call {
    r: R_i64f32;
    r.f0 = cast(s64) (seed + 1);
    r.f1 = cast(float32) (seed + 1.5);
    return r;
}

is_i64f32 :: (r: R_i64f32, seed: s32) -> bool {
    return r.f0 == cast(s64) (seed + 1) && r.f1 == cast(float32) (seed + 1.5);
}

R_f64x2i64 :: struct {
    f0: float64;
    f1: float64;
    f2: s64;
}

ret_f64x2i64 :: (seed: s32) -> R_f64x2i64 #foreign returns;
check_f64x2i64 :: (r: R_f64x2i64, seed: s32) -> s32 #foreign returns;
call_f64x2i64 :: (cb: (seed: s32) -> R_f64x2i64 #c_call, seed: s32) -> R_f64x2i64 #foreign returns;
apply_check_f64x2i64 :: (cb: (seed: s32) -> R_f64x2i64 #c_call, seed: s32) -> s32 #foreign returns;

make_f64x2i64 :: (seed: s32) -> R_f64x2i64 #c_call {
    r: R_f64x2i64;
    r.f0 = cast(float64) (seed + 0.5);
    r.f1 = cast(float64) (seed + 1.5);
    r.f2 = cast(s64) (seed + 3);
    return r;
}

is_f64x2i64 :: (r: R_f64x2i64, seed: s32) -> bool {
    return r.f0 == cast(float64) (seed + 0.5) && r.f1 == cast(float64) (seed + 1.5) && r.f2 == cast(s64) (seed + 3);
}

R_f32i8x3 :: struct {
    f0: float32;
    f1: s8;
    f2: s8;
    f3: s8;
}

ret_f32i8x3 :: (seed: s32) -> R_f32i8x3 #foreign returns;
check_f32i8x3 :: (r: R_f32i8x3, seed: s32) -> s32 #foreign returns;
call_f32i8x3 :: (cb: (seed: s32) -> R_f32i8x3 #c_call, seed: s32) -> R_f32i8x3 #foreign returns;
apply_check_f32i8x3 :: (cb: (seed: s32) -> R_f32i8x3 #c_call, seed: s32) -> s32 #foreign returns;

make_f32i8x3 :: (seed: s32) -> R_f32i8x3 #c_call {
    r: R_f32i8x3;
    r.f0 = cast(float32) (seed + 0.5);
    r.f1 = cast(s8) (seed + 2);
    r.f2 = cast(s8) (seed + 3);
    r.f3 = cast(s8) (seed + 4);
    return r;
}

is_f32i8x3 :: (r: R_f32i8x3, seed: s32) -> bool {
    return r.f0 == cast(float32) (seed + 0.5) && r.f1 == cast(s8) (seed + 2) && r.f2 == cast(s8) (seed + 3) && r.f3 == cast(s8) (seed + 4);
}

R_af32x2 :: struct {
    f0: [2] float32;
}

ret_af32x2 :: (seed: s32) -> R_af32x2 #foreign returns;
check_af32x2 :: (r: R_af32x2, seed: s32) -> s32 #foreign returns;
call_af32x2 :: (cb: (seed: s32) -> R_af32x2 #c_call, seed: s32) -> R_af32x2 #foreign returns;
apply_check_af32x2 :: (cb: (seed: s32) -> R_af32x2 #c_call, seed: s32) -> s32 #foreign returns;

make_af32x2 :: (seed: s32) -> R_af32x2 #c_call {
    r: R_af32x2;
    r.f0[0] = cast(float32) (seed + 0.5);
    r.f0[1] = cast(float32) (seed + 1.5);
    return r;
}

is_af32x2 :: (r: R_af32x2, seed: s32) -> bool {
    return r.f0[0] == cast(float32) (seed + 0.5) && r.f0[1] == cast(float32) (seed + 1.5);
}

R_af32x3 :: struct {
    f0: [3] float32;
}

ret_af32x3 :: (seed: s32) -> R_af32x3 #foreign returns;
check_af32x3 :: (r: R_af32x3, seed: s32) -> s32 #foreign returns;
call_af32x3 :: (cb: (seed: s32) -> R_af32x3 #c_call, seed: s32) -> R_af32x3 #foreign returns;
apply_check_af32x3 :: (cb: (seed: s32) -> R_af32x3 #c_call, seed: s32) -> s32 #foreign returns;

make_af32x3 :: (seed: s32) -> R_af32x3 #c_call {
    r: R_af32x3;
    r.f0[0] = cast(float32) (seed + 0.5);
    r.f0[1] = cast(float32) (seed + 1.5);
    r.f0[2] = cast(float32) (seed + 2.5);
    return r;
}

is_af32x3 :: (r: R_af32x3, seed: s32) -> bool {
    return r.f0[0] == cast(float32) (seed + 0.5) && r.f0[1] == cast(float32) (seed + 1.5) && r.f0[2] == cast(float32) (seed + 2.5);
}

R_af64x3 :: struct {
    f0: [3] float64;
}

ret_af64x3 :: (seed: s32) -> R_af64x3 #foreign returns;
check_af64x3 :: (r: R_af64x3, seed: s32) -> s32 #foreign returns;
call_af64x3 :: (cb: (seed: s32) -> R_af64x3 #c_call, seed: s32) -> R_af64x3 #foreign returns;
apply_check_af64x3 :: (cb: (seed: s32) -> R_af64x3 #c_call, seed: s32) -> s32 #foreign returns;

make_af64x3 :: (seed: s32) -> R_af64x3 #c_call {
    r: R_af64x3;
    r.f0[0] = cast(float64) (seed + 0.5);
    r.f0[1] = cast(float64) (seed + 1.5);
    r.f0[2] = cast(float64) (seed + 2.5);
    return r;
}

is_af64x3 :: (r: R_af64x3, seed: s32) -> bool {
    return r.f0[0] == cast(float64) (seed + 0.5) && r.f0[1] == cast(float64) (seed + 1.5) && r.f0[2] == cast(float64) (seed + 2.5);
}

R_ai32x5 :: struct {
    f0: [5] s32;
}

ret_ai32x5 :: (seed: s32) -> R_ai32x5 #foreign returns;
check_ai32x5 :: (r: R_ai32x5, seed: s32) -> s32 #foreign returns;
call_ai32x5 :: (cb: (seed: s32) -> R_ai32x5 #c_call, seed: s32) -> R_ai32x5 #foreign returns;
apply_check_ai32x5 :: (cb: (seed: s32) -> R_ai32x5 #c_call, seed: s32) -> s32 #foreign returns;

make_ai32x5 :: (seed: s32) -> R_ai32x5 #c_call {
    r: R_ai32x5;
    r.f0[0] = cast(s32) (seed + 1);
    r.f0[1] = cast(s32) (seed + 2);
    r.f0[2] = cast(s32) (seed + 3);
    r.f0[3] = cast(s32) (seed + 4);
    r.f0[4] = cast(s32) (seed + 5);
    return r;
}

is_ai32x5 :: (r: R_ai32x5, seed: s32) -> bool {
    return r.f0[0] == cast(s32) (seed + 1) && r.f0[1] == cast(s32) (seed + 2) && r.f0[2] == cast(s32) (seed + 3) && r.f0[3] == cast(s32) (seed + 4) && r.f0[4] == cast(s32) (seed + 5);
}

R_ai8x3 :: struct {
    f0: [3] s8;
}

ret_ai8x3 :: (seed: s32) -> R_ai8x3 #foreign returns;
check_ai8x3 :: (r: R_ai8x3, seed: s32) -> s32 #foreign returns;
call_ai8x3 :: (cb: (seed: s32) -> R_ai8x3 #c_call, seed: s32) -> R_ai8x3 #foreign returns;
apply_check_ai8x3 :: (cb: (seed: s32) -> R_ai8x3 #c_call, seed: s32) -> s32 #foreign returns;

make_ai8x3 :: (seed: s32) -> R_ai8x3 #c_call {
    r: R_ai8x3;
    r.f0[0] = cast(s8) (seed + 1);
    r.f0[1] = cast(s8) (seed + 2);
    r.f0[2] = cast(s8) (seed + 3);
    return r;
}

is_ai8x3 :: (r: R_ai8x3, seed: s32) -> bool {
    return r.f0[0] == cast(s8) (seed + 1) && r.f0[1] == cast(s8) (seed + 2) && r.f0[2] == cast(s8) (seed + 3);
}

R_nf32_f0 :: struct {
    f0: float32;
    f1: float32;
}

R_nf32 :: struct {
    f0: R_nf32_f0;
    f1: float32;
}

ret_nf32 :: (seed: s32) -> R_nf32 #foreign returns;
check_nf32 :: (r: R_nf32, seed: s32) -> s32 #foreign returns;
call_nf32 :: (cb: (seed: s32) -> R_nf32 #c_call, seed: s32) -> R_nf32 #foreign returns;
apply_check_nf32 :: (cb: (seed: s32) -> R_nf32 #c_call, seed: s32) -> s32 #foreign returns;

make_nf32 :: (seed: s32) -> R_nf32 #c_call {
    r: R_nf32;
    r.f0.f0 = cast(float32) (seed + 0.5);
    r.f0.f1 = cast(float32) (seed + 1.5);
    r.f1 = cast(float32) (seed + 2.5);
    return r;
}

is_nf32 :: (r: R_nf32, seed: s32) -> bool {
    return r.f0.f0 == cast(float32) (seed + 0.5) && r.f0.f1 == cast(float32) (seed + 1.5) && r.f1 == cast(float32) (seed + 2.5);
}

R_nmix_f0 :: struct {
    f0: s32;
    f1: float32;
}

R_nmix :: struct {
    f0: R_nmix_f0;
    f1: float64;
}

ret_nmix :: (seed: s32) -> R_nmix #foreign returns;
check_nmix :: (r: R_nmix, seed: s32) -> s32 #foreign returns;
call_nmix :: (cb: (seed: s32) -> R_nmix #c_call, seed: s32) -> R_nmix #foreign returns;
apply_check_nmix :: (cb: (seed: s32) -> R_nmix #c_call, seed: s32) -> s32 #foreign returns;

make_nmix :: (seed: s32) -> R_nmix #c_call {
    r: R_nmix;
    r.f0.f0 = cast(s32) (seed + 1);
    r.f0.f1 = cast(float32) (seed + 1.5);
    r.f1 = cast(float64) (seed + 2.5);
    return r;
}

is_nmix :: (r: R_nmix, seed: s32) -> bool {
    return r.f0.f0 == cast(s32) (seed + 1) && r.f0.f1 == cast(float32) (seed + 1.5) && r.f1 == cast(float64) (seed + 2.5);
}

R_nnf32_f0_f0 :: struct {
    f0: float32;
}

R_nnf32_f0 :: struct {
    f0: R_nnf32_f0_f0;
    f1: float32;
}

R_nnf32 :: struct {
    f0: R_nnf32_f0;
    f1: float32;
    f2: float32;
}

ret_nnf32 :: (seed: s32) -> R_nnf32 #foreign returns;
check_nnf32 :: (r: R_nnf32, seed: s32) -> s32 #foreign returns;
call_nnf32 :: (cb: (seed: s32) -> R_nnf32 #c_call, seed: s32) -> R_nnf32 #foreign returns;
apply_check_nnf32 :: (cb: (seed: s32) -> R_nnf32 #c_call, seed: s32) -> s32 #foreign returns;

make_nnf32 :: (seed: s32) -> R_nnf32 #c_call {
    r: R_nnf32;
    r.f0.f0.f0 = cast(float32) (seed + 0.5);
    r.f0.f1 = cast(float32) (seed + 1.5);
    r.f1 = cast(float32) (seed + 2.5);
    r.f2 = cast(float32) (seed + 3.5);
    return r;
}

is_nnf32 :: (r: R_nnf32, seed: s32) -> bool {
    return r.f0.f0.f0 == cast(float32) (seed + 0.5) && r.f0.f1 == cast(float32) (seed + 1.5) && r.f1 == cast(float32) (seed + 2.5) && r.f2 == cast(float32) (seed + 3.5);
}

R_nf64x2_f0 :: struct {
    f0: float64;
}

R_nf64x2_f1 :: struct {
    f0: float64;
}

R_nf64x2 :: struct {
    f0: R_nf64x2_f0;
    f1: R_nf64x2_f1;
}

ret_nf64x2 :: (seed: s32) -> R_nf64x2 #foreign returns;
check_nf64x2 :: (r: R_nf64x2, seed: s32) -> s32 #foreign returns;
call_nf64x2 :: (cb: (seed: s32) -> R_nf64x2 #c_call, seed: s32) -> R_nf64x2 #foreign returns;
apply_check_nf64x2 :: (cb: (seed: s32) -> R_nf64x2 #c_call, seed: s32) -> s32 #foreign returns;

make_nf64x2 :: (seed: s32) -> R_nf64x2 #c_call {
    r: R_nf64x2;
    r.f0.f0 = cast(float64) (seed + 0.5);
    r.f1.f0 = cast(float64) (seed + 1.5);
    return r;
}

is_nf64x2 :: (r: R_nf64x2, seed: s32) -> bool {
    return r.f0.f0 == cast(float64) (seed + 0.5) && r.f1.f0 == cast(float64) (seed + 1.5);
}

R_nbig_f0 :: struct {
    f0: s64;
    f1: s64;
}

R_nbig_f1 :: struct {
    f0: float64;
    f1: float64;
}

R_nbig :: struct {
    f0: R_nbig_f0;
    f1: R_nbig_f1;
}

ret_nbig :: (seed: s32) -> R_nbig #foreign returns;
check_nbig :: (r: R_nbig, seed: s32) -> s32 #foreign returns;
call_nbig :: (cb: (seed: s32) -> R_nbig #c_call, seed: s32) -> R_nbig #foreign returns;
apply_check_nbig :: (cb: (seed: s32) -> R_nbig #c_call, seed: s32) -> s32 #foreign returns;

make_nbig :: (seed: s32) -> R_nbig #c_call {
    r: R_nbig;
    r.f0.f0 = cast(s64) (seed + 1);
    r.f0.f1 = cast(s64) (seed + 2);
    r.f1.f0 = cast(float64) (seed + 2.5);
    r.f1.f1 = cast(float64) (seed + 3.5);
    return r;
}

is_nbig :: (r: R_nbig, seed: s32) -> bool {
    return r.f0.f0 == cast(s64) (seed + 1) && r.f0.f1 == cast(s64) (seed + 2) && r.f1.f0 == cast(float64) (seed + 2.5) && r.f1.f1 == cast(float64) (seed + 3.5);
}

R_ui32f32 :: union {
    f0: s32;
    f1: float32;
}

ret_ui32f32 :: (seed: s32) -> R_ui32f32 #foreign returns;
check_ui32f32 :: (r: R_ui32f32, seed: s32) -> s32 #foreign returns;
call_ui32f32 :: (cb: (seed: s32) -> R_ui32f32 #c_call, seed: s32) -> R_ui32f32 #foreign returns;
apply_check_ui32f32 :: (cb: (seed: s32) -> R_ui32f32 #c_call, seed: s32) -> s32 #foreign returns;

make_ui32f32 :: (seed: s32) -> R_ui32f32 #c_call {
    r: R_ui32f32;
    r.f0 = cast(s32) (seed + 1);
    return r;
}

is_ui32f32 :: (r: R_ui32f32, seed: s32) -> bool {
    return r.f0 == cast(s32) (seed + 1);
}

R_ui64f64 :: union {
    f0: s64;
    f1: float64;
}

ret_ui64f64 :: (seed: s32) -> R_ui64f64 #foreign returns;
check_ui64f64 :: (r: R_ui64f64, seed: s32) -> s32 #foreign returns;
call_ui64f64 :: (cb: (seed: s32) -> R_ui64f64 #c_call, seed: s32) -> R_ui64f64 #foreign returns;
apply_check_ui64f64 :: (cb: (seed: s32) -> R_ui64f64 #c_call, seed: s32) -> s32 #foreign returns;

make_ui64f64 :: (seed: s32) -> R_ui64f64 #c_call {
    r: R_ui64f64;
    r.f0 = cast(s64) (seed + 1);
    return r;
}

is_ui64f64 :: (r: R_ui64f64, seed: s32) -> bool {
    return r.f0 == cast(s64) (seed + 1);
}

R_uf32x2i64 :: union {
    f0: [2] float32;
    f1: s64;
}

ret_uf32x2i64 :: (seed: s32) -> R_uf32x2i64 #foreign returns;
check_uf32x2i64 :: (r: R_uf32x2i64, seed: s32) -> s32 #foreign returns;
call_uf32x2i64 :: (cb: (seed: s32) -> R_uf32x2i64 #c_call, seed: s32) -> R_uf32x2i64 #foreign returns;
apply_check_uf32x2i64 :: (cb: (seed: s32) -> R_uf32x2i64 #c_call, seed: s32) -> s32 #foreign returns;

make_uf32x2i64 :: (seed: s32) -> R_uf32x2i64 #c_call {
    r: R_uf32x2i64;
    r.f0[0] = cast(float32) (seed + 0.5);
    r.f0[1] = cast(float32) (seed + 1.5);
    return r;
}

is_uf32x2i64 :: (r: R_uf32x2i64, seed: s32) -> bool {
    return r.f0[0] == cast(float32) (seed + 0.5) && r.f0[1] == cast(float32) (seed + 1.5);
}

R_uf64x2i8 :: union {
    f0: [2] float64;
    f1: [3] s8;
}

ret_uf64x2i8 :: (seed: s32) -> R_uf64x2i8 #foreign returns;
check_uf64x2i8 :: (r: R_uf64x2i8, seed: s32) -> s32 #foreign returns;
call_uf64x2i8 :: (cb: (seed: s32) -> R_uf64x2i8 #c_call, seed: s32) -> R_uf64x2i8 #foreign returns;
apply_check_uf64x2i8 :: (cb: (seed: s32) -> R_uf64x2i8 #c_call, seed: s32) -> s32 #foreign returns;

make_uf64x2i8 :: (seed: s32) -> R_uf64x2i8 #c_call {
    r: R_uf64x2i8;
    r.f0[0] = cast(float64) (seed + 0.5);
    r.f0[1] = cast(float64) (seed + 1.5);
    return r;
}

is_uf64x2i8 :: (r: R_uf64x2i8, seed: s32) -> bool {
    return r.f0[0] == cast(float64) (seed + 0.5) && r.f0[1] == cast(float64) (seed + 1.5);
}

main :: () {
    if !is_i8(ret_i8(7), 7) fail("i8: foreign return");
    if !check_i8(make_i8(7), 7) fail("i8: argument made in Jai");
    if !apply_check_i8(make_i8, 7) fail("i8: callback return read by C");
    if !is_i8(call_i8(make_i8, 9), 9) fail("i8: callback return round trip");
    if !is_i8x3(ret_i8x3(7), 7) fail("i8x3: foreign return");
    if !check_i8x3(make_i8x3(7), 7) fail("i8x3: argument made in Jai");
    if !apply_check_i8x3(make_i8x3, 7) fail("i8x3: callback return read by C");
    if !is_i8x3(call_i8x3(make_i8x3, 9), 9) fail("i8x3: callback return round trip");
    if !is_i16(ret_i16(7), 7) fail("i16: foreign return");
    if !check_i16(make_i16(7), 7) fail("i16: argument made in Jai");
    if !apply_check_i16(make_i16, 7) fail("i16: callback return read by C");
    if !is_i16(call_i16(make_i16, 9), 9) fail("i16: callback return round trip");
    if !is_i8i16(ret_i8i16(7), 7) fail("i8i16: foreign return");
    if !check_i8i16(make_i8i16(7), 7) fail("i8i16: argument made in Jai");
    if !apply_check_i8i16(make_i8i16, 7) fail("i8i16: callback return read by C");
    if !is_i8i16(call_i8i16(make_i8i16, 9), 9) fail("i8i16: callback return round trip");
    if !is_i32(ret_i32(7), 7) fail("i32: foreign return");
    if !check_i32(make_i32(7), 7) fail("i32: argument made in Jai");
    if !apply_check_i32(make_i32, 7) fail("i32: callback return read by C");
    if !is_i32(call_i32(make_i32, 9), 9) fail("i32: callback return round trip");
    if !is_i64(ret_i64(7), 7) fail("i64: foreign return");
    if !check_i64(make_i64(7), 7) fail("i64: argument made in Jai");
    if !apply_check_i64(make_i64, 7) fail("i64: callback return read by C");
    if !is_i64(call_i64(make_i64, 9), 9) fail("i64: callback return round trip");
    if !is_ptr(ret_ptr(7), 7) fail("ptr: foreign return");
    if !check_ptr(make_ptr(7), 7) fail("ptr: argument made in Jai");
    if !apply_check_ptr(make_ptr, 7) fail("ptr: callback return read by C");
    if !is_ptr(call_ptr(make_ptr, 9), 9) fail("ptr: callback return round trip");
    if !is_i32x2(ret_i32x2(7), 7) fail("i32x2: foreign return");
    if !check_i32x2(make_i32x2(7), 7) fail("i32x2: argument made in Jai");
    if !apply_check_i32x2(make_i32x2, 7) fail("i32x2: callback return read by C");
    if !is_i32x2(call_i32x2(make_i32x2, 9), 9) fail("i32x2: callback return round trip");
    if !is_i32x3(ret_i32x3(7), 7) fail("i32x3: foreign return");
    if !check_i32x3(make_i32x3(7), 7) fail("i32x3: argument made in Jai");
    if !apply_check_i32x3(make_i32x3, 7) fail("i32x3: callback return read by C");
    if !is_i32x3(call_i32x3(make_i32x3, 9), 9) fail("i32x3: callback return round trip");
    if !is_i32x4(ret_i32x4(7), 7) fail("i32x4: foreign return");
    if !check_i32x4(make_i32x4(7), 7) fail("i32x4: argument made in Jai");
    if !apply_check_i32x4(make_i32x4, 7) fail("i32x4: callback return read by C");
    if !is_i32x4(call_i32x4(make_i32x4, 9), 9) fail("i32x4: callback return round trip");
    if !is_i32x5(ret_i32x5(7), 7) fail("i32x5: foreign return");
    if !check_i32x5(make_i32x5(7), 7) fail("i32x5: argument made in Jai");
    if !apply_check_i32x5(make_i32x5, 7) fail("i32x5: callback return read by C");
    if !is_i32x5(call_i32x5(make_i32x5, 9), 9) fail("i32x5: callback return round trip");
    if !is_i64x2(ret_i64x2(7), 7) fail("i64x2: foreign return");
    if !check_i64x2(make_i64x2(7), 7) fail("i64x2: argument made in Jai");
    if !apply_check_i64x2(make_i64x2, 7) fail("i64x2: callback return read by C");
    if !is_i64x2(call_i64x2(make_i64x2, 9), 9) fail("i64x2: callback return round trip");
    if !is_i64x3(ret_i64x3(7), 7) fail("i64x3: foreign return");
    if !check_i64x3(make_i64x3(7), 7) fail("i64x3: argument made in Jai");
    if !apply_check_i64x3(make_i64x3, 7) fail("i64x3: callback return read by C");
    if !is_i64x3(call_i64x3(make_i64x3, 9), 9) fail("i64x3: callback return round trip");
    if !is_i64x4(ret_i64x4(7), 7) fail("i64x4: foreign return");
    if !check_i64x4(make_i64x4(7), 7) fail("i64x4: argument made in Jai");
    if !apply_check_i64x4(make_i64x4, 7) fail("i64x4: callback return read by C");
    if !is_i64x4(call_i64x4(make_i64x4, 9), 9) fail("i64x4: callback return round trip");
    if !is_i8x9(ret_i8x9(7), 7) fail("i8x9: foreign return");
    if !check_i8x9(make_i8x9(7), 7) fail("i8x9: argument made in Jai");
    if !apply_check_i8x9(make_i8x9, 7) fail("i8x9: callback return read by C");
    if !is_i8x9(call_i8x9(make_i8x9, 9), 9) fail("i8x9: callback return round trip");
    if !is_i8x17(ret_i8x17(7), 7) fail("i8x17: foreign return");
    if !check_i8x17(make_i8x17(7), 7) fail("i8x17: argument made in Jai");
    if !apply_check_i8x17(make_i8x17, 7) fail("i8x17: callback return read by C");
    if !is_i8x17(call_i8x17(make_i8x17, 9), 9) fail("i8x17: callback return round trip");
    if !is_i8i64(ret_i8i64(7), 7) fail("i8i64: foreign return");
    if !check_i8i64(make_i8i64(7), 7) fail("i8i64: argument made in Jai");
    if !apply_check_i8i64(make_i8i64, 7) fail("i8i64: callback return read by C");
    if !is_i8i64(call_i8i64(make_i8i64, 9), 9) fail("i8i64: callback return round trip");
    if !is_i32i8(ret_i32i8(7), 7) fail("i32i8: foreign return");
    if !check_i32i8(make_i32i8(7), 7) fail("i32i8: argument made in Jai");
    if !apply_check_i32i8(make_i32i8, 7) fail("i32i8: callback return read by C");
    if !is_i32i8(call_i32i8(make_i32i8, 9), 9) fail("i32i8: callback return round trip");
    if !is_i16x3(ret_i16x3(7), 7) fail("i16x3: foreign return");
    if !check_i16x3(make_i16x3(7), 7) fail("i16x3: argument made in Jai");
    if !apply_check_i16x3(make_i16x3, 7) fail("i16x3: callback return read by C");
    if !is_i16x3(call_i16x3(make_i16x3, 9), 9) fail("i16x3: callback return round trip");
    if !is_f32(ret_f32(7), 7) fail("f32: foreign return");
    if !check_f32(make_f32(7), 7) fail("f32: argument made in Jai");
    if !apply_check_f32(make_f32, 7) fail("f32: callback return read by C");
    if !is_f32(call_f32(make_f32, 9), 9) fail("f32: callback return round trip");
    if !is_f32x2(ret_f32x2(7), 7) fail("f32x2: foreign return");
    if !check_f32x2(make_f32x2(7), 7) fail("f32x2: argument made in Jai");
    if !apply_check_f32x2(make_f32x2, 7) fail("f32x2: callback return read by C");
    if !is_f32x2(call_f32x2(make_f32x2, 9), 9) fail("f32x2: callback return round trip");
    if !is_f32x3(ret_f32x3(7), 7) fail("f32x3: foreign return");
    if !check_f32x3(make_f32x3(7), 7) fail("f32x3: argument made in Jai");
    if !apply_check_f32x3(make_f32x3, 7) fail("f32x3: callback return read by C");
    if !is_f32x3(call_f32x3(make_f32x3, 9), 9) fail("f32x3: callback return round trip");
    if !is_f32x4(ret_f32x4(7), 7) fail("f32x4: foreign return");
    if !check_f32x4(make_f32x4(7), 7) fail("f32x4: argument made in Jai");
    if !apply_check_f32x4(make_f32x4, 7) fail("f32x4: callback return read by C");
    if !is_f32x4(call_f32x4(make_f32x4, 9), 9) fail("f32x4: callback return round trip");
    if !is_f32x5(ret_f32x5(7), 7) fail("f32x5: foreign return");
    if !check_f32x5(make_f32x5(7), 7) fail("f32x5: argument made in Jai");
    if !apply_check_f32x5(make_f32x5, 7) fail("f32x5: callback return read by C");
    if !is_f32x5(call_f32x5(make_f32x5, 9), 9) fail("f32x5: callback return round trip");
    if !is_f64(ret_f64(7), 7) fail("f64: foreign return");
    if !check_f64(make_f64(7), 7) fail("f64: argument made in Jai");
    if !apply_check_f64(make_f64, 7) fail("f64: callback return read by C");
    if !is_f64(call_f64(make_f64, 9), 9) fail("f64: callback return round trip");
    if !is_f64x2(ret_f64x2(7), 7) fail("f64x2: foreign return");
    if !check_f64x2(make_f64x2(7), 7) fail("f64x2: argument made in Jai");
    if !apply_check_f64x2(make_f64x2, 7) fail("f64x2: callback return read by C");
    if !is_f64x2(call_f64x2(make_f64x2, 9), 9) fail("f64x2: callback return round trip");
    if !is_f64x3(ret_f64x3(7), 7) fail("f64x3: foreign return");
    if !check_f64x3(make_f64x3(7), 7) fail("f64x3: argument made in Jai");
    if !apply_check_f64x3(make_f64x3, 7) fail("f64x3: callback return read by C");
    if !is_f64x3(call_f64x3(make_f64x3, 9), 9) fail("f64x3: callback return round trip");
    if !is_f64x4(ret_f64x4(7), 7) fail("f64x4: foreign return");
    if !check_f64x4(make_f64x4(7), 7) fail("f64x4: argument made in Jai");
    if !apply_check_f64x4(make_f64x4, 7) fail("f64x4: callback return read by C");
    if !is_f64x4(call_f64x4(make_f64x4, 9), 9) fail("f64x4: callback return round trip");
    if !is_f64x5(ret_f64x5(7), 7) fail("f64x5: foreign return");
    if !check_f64x5(make_f64x5(7), 7) fail("f64x5: argument made in Jai");
    if !apply_check_f64x5(make_f64x5, 7) fail("f64x5: callback return read by C");
    if !is_f64x5(call_f64x5(make_f64x5, 9), 9) fail("f64x5: callback return round trip");
    if !is_i32f32(ret_i32f32(7), 7) fail("i32f32: foreign return");
    if !check_i32f32(make_i32f32(7), 7) fail("i32f32: argument made in Jai");
    if !apply_check_i32f32(make_i32f32, 7) fail("i32f32: callback return read by C");
    if !is_i32f32(call_i32f32(make_i32f32, 9), 9) fail("i32f32: callback return round trip");
    if !is_f32i32(ret_f32i32(7), 7) fail("f32i32: foreign return");
    if !check_f32i32(make_f32i32(7), 7) fail("f32i32: argument made in Jai");
    if !apply_check_f32i32(make_f32i32, 7) fail("f32i32: callback return read by C");
    if !is_f32i32(call_f32i32(make_f32i32, 9), 9) fail("f32i32: callback return round trip");
    if !is_i64f64(ret_i64f64(7), 7) fail("i64f64: foreign return");
    if !check_i64f64(make_i64f64(7), 7) fail("i64f64: argument made in Jai");
    if !apply_check_i64f64(make_i64f64, 7) fail("i64f64: callback return read by C");
    if !is_i64f64(call_i64f64(make_i64f64, 9), 9) fail("i64f64: callback return round trip");
    if !is_f64i64(ret_f64i64(7), 7) fail("f64i64: foreign return");
    if !check_f64i64(make_f64i64(7), 7) fail("f64i64: argument made in Jai");
    if !apply_check_f64i64(make_f64i64, 7) fail("f64i64: callback return read by C");
    if !is_f64i64(call_f64i64(make_f64i64, 9), 9) fail("f64i64: callback return round trip");
    if !is_f32x2i64(ret_f32x2i64(7), 7) fail("f32x2i64: foreign return");
    if !check_f32x2i64(make_f32x2i64(7), 7) fail("f32x2i64: argument made in Jai");
    if !apply_check_f32x2i64(make_f32x2i64, 7) fail("f32x2i64: callback return read by C");
    if !is_f32x2i64(call_f32x2i64(make_f32x2i64, 9), 9) fail("f32x2i64: callback return round trip");
    if !is_i64f32x2(ret_i64f32x2(7), 7) fail("i64f32x2: foreign return");
    if !check_i64f32x2(make_i64f32x2(7), 7) fail("i64f32x2: argument made in Jai");
    if !apply_check_i64f32x2(make_i64f32x2, 7) fail("i64f32x2: callback return read by C");
    if !is_i64f32x2(call_i64f32x2(make_i64f32x2, 9), 9) fail("i64f32x2: callback return round trip");
    if !is_f64f32(ret_f64f32(7), 7) fail("f64f32: foreign return");
    if !check_f64f32(make_f64f32(7), 7) fail("f64f32: argument made in Jai");
    if !apply_check_f64f32(make_f64f32, 7) fail("f64f32: callback return read by C");
    if !is_f64f32(call_f64f32(make_f64f32, 9), 9) fail("f64f32: callback return round trip");
    if !is_f32f64(ret_f32f64(7), 7) fail("f32f64: foreign return");
    if !check_f32f64(make_f32f64(7), 7) fail("f32f64: argument made in Jai");
    if !apply_check_f32f64(make_f32f64, 7) fail("f32f64: callback return read by C");
    if !is_f32f64(call_f32f64(make_f32f64, 9), 9) fail("f32f64: callback return round trip");
    if !is_i8f32(ret_i8f32(7), 7) fail("i8f32: foreign return");
    if !check_i8f32(make_i8f32(7), 7) fail("i8f32: argument made in Jai");
    if !apply_check_i8f32(make_i8f32, 7) fail("i8f32: callback return read by C");
    if !is_i8f32(call_i8f32(make_i8f32, 9), 9) fail("i8f32: callback return round trip");
    if !is_f32x3i32(ret_f32x3i32(7), 7) fail("f32x3i32: foreign return");
    if !check_f32x3i32(make_f32x3i32(7), 7) fail("f32x3i32: argument made in Jai");
    if !apply_check_f32x3i32(make_f32x3i32, 7) fail("f32x3i32: callback return read by C");
    if !is_f32x3i32(call_f32x3i32(make_f32x3i32, 9), 9) fail("f32x3i32: callback return round trip");
    if !is_f64i8(ret_f64i8(7), 7) fail("f64i8: foreign return");
    if !check_f64i8(make_f64i8(7), 7) fail("f64i8: argument made in Jai");
    if !apply_check_f64i8(make_f64i8, 7) fail("f64i8: callback return read by C");
    if !is_f64i8(call_f64i8(make_f64i8, 9), 9) fail("f64i8: callback return round trip");
    if !is_f32i64(ret_f32i64(7), 7) fail("f32i64: foreign return");
    if !check_f32i64(make_f32i64(7), 7) fail("f32i64: argument made in Jai");
    if !apply_check_f32i64(make_f32i64, 7) fail("f32i64: callback return read by C");
    if !is_f32i64(call_f32i64(make_f32i64, 9), 9) fail("f32i64: callback return round trip");
    if !is_i64f32(ret_i64f32(7), 7) fail("i64f32: foreign return");
    if !check_i64f32(make_i64f32(7), 7) fail("i64f32: argument made in Jai");
    if !apply_check_i64f32(make_i64f32, 7) fail("i64f32: callback return read by C");
    if !is_i64f32(call_i64f32(make_i64f32, 9), 9) fail("i64f32: callback return round trip");
    if !is_f64x2i64(ret_f64x2i64(7), 7) fail("f64x2i64: foreign return");
    if !check_f64x2i64(make_f64x2i64(7), 7) fail("f64x2i64: argument made in Jai");
    if !apply_check_f64x2i64(make_f64x2i64, 7) fail("f64x2i64: callback return read by C");
    if !is_f64x2i64(call_f64x2i64(make_f64x2i64, 9), 9) fail("f64x2i64: callback return round trip");
    if !is_f32i8x3(ret_f32i8x3(7), 7) fail("f32i8x3: foreign return");
    if !check_f32i8x3(make_f32i8x3(7), 7) fail("f32i8x3: argument made in Jai");
    if !apply_check_f32i8x3(make_f32i8x3, 7) fail("f32i8x3: callback return read by C");
    if !is_f32i8x3(call_f32i8x3(make_f32i8x3, 9), 9) fail("f32i8x3: callback return round trip");
    if !is_af32x2(ret_af32x2(7), 7) fail("af32x2: foreign return");
    if !check_af32x2(make_af32x2(7), 7) fail("af32x2: argument made in Jai");
    if !apply_check_af32x2(make_af32x2, 7) fail("af32x2: callback return read by C");
    if !is_af32x2(call_af32x2(make_af32x2, 9), 9) fail("af32x2: callback return round trip");
    if !is_af32x3(ret_af32x3(7), 7) fail("af32x3: foreign return");
    if !check_af32x3(make_af32x3(7), 7) fail("af32x3: argument made in Jai");
    if !apply_check_af32x3(make_af32x3, 7) fail("af32x3: callback return read by C");
    if !is_af32x3(call_af32x3(make_af32x3, 9), 9) fail("af32x3: callback return round trip");
    if !is_af64x3(ret_af64x3(7), 7) fail("af64x3: foreign return");
    if !check_af64x3(make_af64x3(7), 7) fail("af64x3: argument made in Jai");
    if !apply_check_af64x3(make_af64x3, 7) fail("af64x3: callback return read by C");
    if !is_af64x3(call_af64x3(make_af64x3, 9), 9) fail("af64x3: callback return round trip");
    if !is_ai32x5(ret_ai32x5(7), 7) fail("ai32x5: foreign return");
    if !check_ai32x5(make_ai32x5(7), 7) fail("ai32x5: argument made in Jai");
    if !apply_check_ai32x5(make_ai32x5, 7) fail("ai32x5: callback return read by C");
    if !is_ai32x5(call_ai32x5(make_ai32x5, 9), 9) fail("ai32x5: callback return round trip");
    if !is_ai8x3(ret_ai8x3(7), 7) fail("ai8x3: foreign return");
    if !check_ai8x3(make_ai8x3(7), 7) fail("ai8x3: argument made in Jai");
    if !apply_check_ai8x3(make_ai8x3, 7) fail("ai8x3: callback return read by C");
    if !is_ai8x3(call_ai8x3(make_ai8x3, 9), 9) fail("ai8x3: callback return round trip");
    if !is_nf32(ret_nf32(7), 7) fail("nf32: foreign return");
    if !check_nf32(make_nf32(7), 7) fail("nf32: argument made in Jai");
    if !apply_check_nf32(make_nf32, 7) fail("nf32: callback return read by C");
    if !is_nf32(call_nf32(make_nf32, 9), 9) fail("nf32: callback return round trip");
    if !is_nmix(ret_nmix(7), 7) fail("nmix: foreign return");
    if !check_nmix(make_nmix(7), 7) fail("nmix: argument made in Jai");
    if !apply_check_nmix(make_nmix, 7) fail("nmix: callback return read by C");
    if !is_nmix(call_nmix(make_nmix, 9), 9) fail("nmix: callback return round trip");
    if !is_nnf32(ret_nnf32(7), 7) fail("nnf32: foreign return");
    if !check_nnf32(make_nnf32(7), 7) fail("nnf32: argument made in Jai");
    if !apply_check_nnf32(make_nnf32, 7) fail("nnf32: callback return read by C");
    if !is_nnf32(call_nnf32(make_nnf32, 9), 9) fail("nnf32: callback return round trip");
    if !is_nf64x2(ret_nf64x2(7), 7) fail("nf64x2: foreign return");
    if !check_nf64x2(make_nf64x2(7), 7) fail("nf64x2: argument made in Jai");
    if !apply_check_nf64x2(make_nf64x2, 7) fail("nf64x2: callback return read by C");
    if !is_nf64x2(call_nf64x2(make_nf64x2, 9), 9) fail("nf64x2: callback return round trip");
    if !is_nbig(ret_nbig(7), 7) fail("nbig: foreign return");
    if !check_nbig(make_nbig(7), 7) fail("nbig: argument made in Jai");
    if !apply_check_nbig(make_nbig, 7) fail("nbig: callback return read by C");
    if !is_nbig(call_nbig(make_nbig, 9), 9) fail("nbig: callback return round trip");
    if !is_ui32f32(ret_ui32f32(7), 7) fail("ui32f32: foreign return");
    if !check_ui32f32(make_ui32f32(7), 7) fail("ui32f32: argument made in Jai");
    if !apply_check_ui32f32(make_ui32f32, 7) fail("ui32f32: callback return read by C");
    if !is_ui32f32(call_ui32f32(make_ui32f32, 9), 9) fail("ui32f32: callback return round trip");
    if !is_ui64f64(ret_ui64f64(7), 7) fail("ui64f64: foreign return");
    if !check_ui64f64(make_ui64f64(7), 7) fail("ui64f64: argument made in Jai");
    if !apply_check_ui64f64(make_ui64f64, 7) fail("ui64f64: callback return read by C");
    if !is_ui64f64(call_ui64f64(make_ui64f64, 9), 9) fail("ui64f64: callback return round trip");
    if !is_uf32x2i64(ret_uf32x2i64(7), 7) fail("uf32x2i64: foreign return");
    if !check_uf32x2i64(make_uf32x2i64(7), 7) fail("uf32x2i64: argument made in Jai");
    if !apply_check_uf32x2i64(make_uf32x2i64, 7) fail("uf32x2i64: callback return read by C");
    if !is_uf32x2i64(call_uf32x2i64(make_uf32x2i64, 9), 9) fail("uf32x2i64: callback return round trip");
    if !is_uf64x2i8(ret_uf64x2i8(7), 7) fail("uf64x2i8: foreign return");
    if !check_uf64x2i8(make_uf64x2i8(7), 7) fail("uf64x2i8: argument made in Jai");
    if !apply_check_uf64x2i8(make_uf64x2i8, 7) fail("uf64x2i8: callback return read by C");
    if !is_uf64x2i8(call_uf64x2i8(make_uf64x2i8, 9), 9) fail("uf64x2i8: callback return round trip");
    if failures == 0 print("ok\n");
}
