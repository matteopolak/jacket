// Generated shapes for C struct returns: see docs/native/c-abi.md.
#include <string.h>

typedef struct { signed char f0; } R_i8;
R_i8 ret_i8(int seed) {
    R_i8 s;
    memset(&s, 0, sizeof s);
    s.f0 = (signed char) (seed + 1);
    return s;
}
int check_i8(R_i8 s, int seed) {
    return s.f0 == (signed char) (seed + 1);
}
R_i8 call_i8(R_i8 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i8(R_i8 (*cb)(int), int seed) {
    return check_i8(cb(seed), seed);
}

typedef struct { signed char f0; signed char f1; signed char f2; } R_i8x3;
R_i8x3 ret_i8x3(int seed) {
    R_i8x3 s;
    memset(&s, 0, sizeof s);
    s.f0 = (signed char) (seed + 1); s.f1 = (signed char) (seed + 2); s.f2 = (signed char) (seed + 3);
    return s;
}
int check_i8x3(R_i8x3 s, int seed) {
    return s.f0 == (signed char) (seed + 1) && s.f1 == (signed char) (seed + 2) && s.f2 == (signed char) (seed + 3);
}
R_i8x3 call_i8x3(R_i8x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i8x3(R_i8x3 (*cb)(int), int seed) {
    return check_i8x3(cb(seed), seed);
}

typedef struct { short f0; } R_i16;
R_i16 ret_i16(int seed) {
    R_i16 s;
    memset(&s, 0, sizeof s);
    s.f0 = (short) (seed + 1);
    return s;
}
int check_i16(R_i16 s, int seed) {
    return s.f0 == (short) (seed + 1);
}
R_i16 call_i16(R_i16 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i16(R_i16 (*cb)(int), int seed) {
    return check_i16(cb(seed), seed);
}

typedef struct { signed char f0; short f1; } R_i8i16;
R_i8i16 ret_i8i16(int seed) {
    R_i8i16 s;
    memset(&s, 0, sizeof s);
    s.f0 = (signed char) (seed + 1); s.f1 = (short) (seed + 2);
    return s;
}
int check_i8i16(R_i8i16 s, int seed) {
    return s.f0 == (signed char) (seed + 1) && s.f1 == (short) (seed + 2);
}
R_i8i16 call_i8i16(R_i8i16 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i8i16(R_i8i16 (*cb)(int), int seed) {
    return check_i8i16(cb(seed), seed);
}

typedef struct { int f0; } R_i32;
R_i32 ret_i32(int seed) {
    R_i32 s;
    memset(&s, 0, sizeof s);
    s.f0 = (int) (seed + 1);
    return s;
}
int check_i32(R_i32 s, int seed) {
    return s.f0 == (int) (seed + 1);
}
R_i32 call_i32(R_i32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i32(R_i32 (*cb)(int), int seed) {
    return check_i32(cb(seed), seed);
}

typedef struct { long long f0; } R_i64;
R_i64 ret_i64(int seed) {
    R_i64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (long long) (seed + 1);
    return s;
}
int check_i64(R_i64 s, int seed) {
    return s.f0 == (long long) (seed + 1);
}
R_i64 call_i64(R_i64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i64(R_i64 (*cb)(int), int seed) {
    return check_i64(cb(seed), seed);
}

typedef struct { void * f0; } R_ptr;
R_ptr ret_ptr(int seed) {
    R_ptr s;
    memset(&s, 0, sizeof s);
    s.f0 = (void *) (long long) (seed + 1);
    return s;
}
int check_ptr(R_ptr s, int seed) {
    return s.f0 == (void *) (long long) (seed + 1);
}
R_ptr call_ptr(R_ptr (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_ptr(R_ptr (*cb)(int), int seed) {
    return check_ptr(cb(seed), seed);
}

typedef struct { int f0; int f1; } R_i32x2;
R_i32x2 ret_i32x2(int seed) {
    R_i32x2 s;
    memset(&s, 0, sizeof s);
    s.f0 = (int) (seed + 1); s.f1 = (int) (seed + 2);
    return s;
}
int check_i32x2(R_i32x2 s, int seed) {
    return s.f0 == (int) (seed + 1) && s.f1 == (int) (seed + 2);
}
R_i32x2 call_i32x2(R_i32x2 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i32x2(R_i32x2 (*cb)(int), int seed) {
    return check_i32x2(cb(seed), seed);
}

typedef struct { int f0; int f1; int f2; } R_i32x3;
R_i32x3 ret_i32x3(int seed) {
    R_i32x3 s;
    memset(&s, 0, sizeof s);
    s.f0 = (int) (seed + 1); s.f1 = (int) (seed + 2); s.f2 = (int) (seed + 3);
    return s;
}
int check_i32x3(R_i32x3 s, int seed) {
    return s.f0 == (int) (seed + 1) && s.f1 == (int) (seed + 2) && s.f2 == (int) (seed + 3);
}
R_i32x3 call_i32x3(R_i32x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i32x3(R_i32x3 (*cb)(int), int seed) {
    return check_i32x3(cb(seed), seed);
}

typedef struct { int f0; int f1; int f2; int f3; } R_i32x4;
R_i32x4 ret_i32x4(int seed) {
    R_i32x4 s;
    memset(&s, 0, sizeof s);
    s.f0 = (int) (seed + 1); s.f1 = (int) (seed + 2); s.f2 = (int) (seed + 3); s.f3 = (int) (seed + 4);
    return s;
}
int check_i32x4(R_i32x4 s, int seed) {
    return s.f0 == (int) (seed + 1) && s.f1 == (int) (seed + 2) && s.f2 == (int) (seed + 3) && s.f3 == (int) (seed + 4);
}
R_i32x4 call_i32x4(R_i32x4 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i32x4(R_i32x4 (*cb)(int), int seed) {
    return check_i32x4(cb(seed), seed);
}

typedef struct { int f0; int f1; int f2; int f3; int f4; } R_i32x5;
R_i32x5 ret_i32x5(int seed) {
    R_i32x5 s;
    memset(&s, 0, sizeof s);
    s.f0 = (int) (seed + 1); s.f1 = (int) (seed + 2); s.f2 = (int) (seed + 3); s.f3 = (int) (seed + 4); s.f4 = (int) (seed + 5);
    return s;
}
int check_i32x5(R_i32x5 s, int seed) {
    return s.f0 == (int) (seed + 1) && s.f1 == (int) (seed + 2) && s.f2 == (int) (seed + 3) && s.f3 == (int) (seed + 4) && s.f4 == (int) (seed + 5);
}
R_i32x5 call_i32x5(R_i32x5 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i32x5(R_i32x5 (*cb)(int), int seed) {
    return check_i32x5(cb(seed), seed);
}

typedef struct { long long f0; long long f1; } R_i64x2;
R_i64x2 ret_i64x2(int seed) {
    R_i64x2 s;
    memset(&s, 0, sizeof s);
    s.f0 = (long long) (seed + 1); s.f1 = (long long) (seed + 2);
    return s;
}
int check_i64x2(R_i64x2 s, int seed) {
    return s.f0 == (long long) (seed + 1) && s.f1 == (long long) (seed + 2);
}
R_i64x2 call_i64x2(R_i64x2 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i64x2(R_i64x2 (*cb)(int), int seed) {
    return check_i64x2(cb(seed), seed);
}

typedef struct { long long f0; long long f1; long long f2; } R_i64x3;
R_i64x3 ret_i64x3(int seed) {
    R_i64x3 s;
    memset(&s, 0, sizeof s);
    s.f0 = (long long) (seed + 1); s.f1 = (long long) (seed + 2); s.f2 = (long long) (seed + 3);
    return s;
}
int check_i64x3(R_i64x3 s, int seed) {
    return s.f0 == (long long) (seed + 1) && s.f1 == (long long) (seed + 2) && s.f2 == (long long) (seed + 3);
}
R_i64x3 call_i64x3(R_i64x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i64x3(R_i64x3 (*cb)(int), int seed) {
    return check_i64x3(cb(seed), seed);
}

typedef struct { long long f0; long long f1; long long f2; long long f3; } R_i64x4;
R_i64x4 ret_i64x4(int seed) {
    R_i64x4 s;
    memset(&s, 0, sizeof s);
    s.f0 = (long long) (seed + 1); s.f1 = (long long) (seed + 2); s.f2 = (long long) (seed + 3); s.f3 = (long long) (seed + 4);
    return s;
}
int check_i64x4(R_i64x4 s, int seed) {
    return s.f0 == (long long) (seed + 1) && s.f1 == (long long) (seed + 2) && s.f2 == (long long) (seed + 3) && s.f3 == (long long) (seed + 4);
}
R_i64x4 call_i64x4(R_i64x4 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i64x4(R_i64x4 (*cb)(int), int seed) {
    return check_i64x4(cb(seed), seed);
}

typedef struct { signed char f0; signed char f1; signed char f2; signed char f3; signed char f4; signed char f5; signed char f6; signed char f7; signed char f8; } R_i8x9;
R_i8x9 ret_i8x9(int seed) {
    R_i8x9 s;
    memset(&s, 0, sizeof s);
    s.f0 = (signed char) (seed + 1); s.f1 = (signed char) (seed + 2); s.f2 = (signed char) (seed + 3); s.f3 = (signed char) (seed + 4); s.f4 = (signed char) (seed + 5); s.f5 = (signed char) (seed + 6); s.f6 = (signed char) (seed + 7); s.f7 = (signed char) (seed + 8); s.f8 = (signed char) (seed + 9);
    return s;
}
int check_i8x9(R_i8x9 s, int seed) {
    return s.f0 == (signed char) (seed + 1) && s.f1 == (signed char) (seed + 2) && s.f2 == (signed char) (seed + 3) && s.f3 == (signed char) (seed + 4) && s.f4 == (signed char) (seed + 5) && s.f5 == (signed char) (seed + 6) && s.f6 == (signed char) (seed + 7) && s.f7 == (signed char) (seed + 8) && s.f8 == (signed char) (seed + 9);
}
R_i8x9 call_i8x9(R_i8x9 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i8x9(R_i8x9 (*cb)(int), int seed) {
    return check_i8x9(cb(seed), seed);
}

typedef struct { signed char f0; signed char f1; signed char f2; signed char f3; signed char f4; signed char f5; signed char f6; signed char f7; signed char f8; signed char f9; signed char f10; signed char f11; signed char f12; signed char f13; signed char f14; signed char f15; signed char f16; } R_i8x17;
R_i8x17 ret_i8x17(int seed) {
    R_i8x17 s;
    memset(&s, 0, sizeof s);
    s.f0 = (signed char) (seed + 1); s.f1 = (signed char) (seed + 2); s.f2 = (signed char) (seed + 3); s.f3 = (signed char) (seed + 4); s.f4 = (signed char) (seed + 5); s.f5 = (signed char) (seed + 6); s.f6 = (signed char) (seed + 7); s.f7 = (signed char) (seed + 8); s.f8 = (signed char) (seed + 9); s.f9 = (signed char) (seed + 10); s.f10 = (signed char) (seed + 11); s.f11 = (signed char) (seed + 12); s.f12 = (signed char) (seed + 13); s.f13 = (signed char) (seed + 14); s.f14 = (signed char) (seed + 15); s.f15 = (signed char) (seed + 16); s.f16 = (signed char) (seed + 17);
    return s;
}
int check_i8x17(R_i8x17 s, int seed) {
    return s.f0 == (signed char) (seed + 1) && s.f1 == (signed char) (seed + 2) && s.f2 == (signed char) (seed + 3) && s.f3 == (signed char) (seed + 4) && s.f4 == (signed char) (seed + 5) && s.f5 == (signed char) (seed + 6) && s.f6 == (signed char) (seed + 7) && s.f7 == (signed char) (seed + 8) && s.f8 == (signed char) (seed + 9) && s.f9 == (signed char) (seed + 10) && s.f10 == (signed char) (seed + 11) && s.f11 == (signed char) (seed + 12) && s.f12 == (signed char) (seed + 13) && s.f13 == (signed char) (seed + 14) && s.f14 == (signed char) (seed + 15) && s.f15 == (signed char) (seed + 16) && s.f16 == (signed char) (seed + 17);
}
R_i8x17 call_i8x17(R_i8x17 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i8x17(R_i8x17 (*cb)(int), int seed) {
    return check_i8x17(cb(seed), seed);
}

typedef struct { signed char f0; long long f1; } R_i8i64;
R_i8i64 ret_i8i64(int seed) {
    R_i8i64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (signed char) (seed + 1); s.f1 = (long long) (seed + 2);
    return s;
}
int check_i8i64(R_i8i64 s, int seed) {
    return s.f0 == (signed char) (seed + 1) && s.f1 == (long long) (seed + 2);
}
R_i8i64 call_i8i64(R_i8i64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i8i64(R_i8i64 (*cb)(int), int seed) {
    return check_i8i64(cb(seed), seed);
}

typedef struct { int f0; signed char f1; } R_i32i8;
R_i32i8 ret_i32i8(int seed) {
    R_i32i8 s;
    memset(&s, 0, sizeof s);
    s.f0 = (int) (seed + 1); s.f1 = (signed char) (seed + 2);
    return s;
}
int check_i32i8(R_i32i8 s, int seed) {
    return s.f0 == (int) (seed + 1) && s.f1 == (signed char) (seed + 2);
}
R_i32i8 call_i32i8(R_i32i8 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i32i8(R_i32i8 (*cb)(int), int seed) {
    return check_i32i8(cb(seed), seed);
}

typedef struct { short f0; short f1; short f2; } R_i16x3;
R_i16x3 ret_i16x3(int seed) {
    R_i16x3 s;
    memset(&s, 0, sizeof s);
    s.f0 = (short) (seed + 1); s.f1 = (short) (seed + 2); s.f2 = (short) (seed + 3);
    return s;
}
int check_i16x3(R_i16x3 s, int seed) {
    return s.f0 == (short) (seed + 1) && s.f1 == (short) (seed + 2) && s.f2 == (short) (seed + 3);
}
R_i16x3 call_i16x3(R_i16x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i16x3(R_i16x3 (*cb)(int), int seed) {
    return check_i16x3(cb(seed), seed);
}

typedef struct { float f0; } R_f32;
R_f32 ret_f32(int seed) {
    R_f32 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5);
    return s;
}
int check_f32(R_f32 s, int seed) {
    return s.f0 == (float) (seed + 0.5);
}
R_f32 call_f32(R_f32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32(R_f32 (*cb)(int), int seed) {
    return check_f32(cb(seed), seed);
}

typedef struct { float f0; float f1; } R_f32x2;
R_f32x2 ret_f32x2(int seed) {
    R_f32x2 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (float) (seed + 1.5);
    return s;
}
int check_f32x2(R_f32x2 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (float) (seed + 1.5);
}
R_f32x2 call_f32x2(R_f32x2 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32x2(R_f32x2 (*cb)(int), int seed) {
    return check_f32x2(cb(seed), seed);
}

typedef struct { float f0; float f1; float f2; } R_f32x3;
R_f32x3 ret_f32x3(int seed) {
    R_f32x3 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (float) (seed + 1.5); s.f2 = (float) (seed + 2.5);
    return s;
}
int check_f32x3(R_f32x3 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (float) (seed + 1.5) && s.f2 == (float) (seed + 2.5);
}
R_f32x3 call_f32x3(R_f32x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32x3(R_f32x3 (*cb)(int), int seed) {
    return check_f32x3(cb(seed), seed);
}

typedef struct { float f0; float f1; float f2; float f3; } R_f32x4;
R_f32x4 ret_f32x4(int seed) {
    R_f32x4 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (float) (seed + 1.5); s.f2 = (float) (seed + 2.5); s.f3 = (float) (seed + 3.5);
    return s;
}
int check_f32x4(R_f32x4 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (float) (seed + 1.5) && s.f2 == (float) (seed + 2.5) && s.f3 == (float) (seed + 3.5);
}
R_f32x4 call_f32x4(R_f32x4 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32x4(R_f32x4 (*cb)(int), int seed) {
    return check_f32x4(cb(seed), seed);
}

typedef struct { float f0; float f1; float f2; float f3; float f4; } R_f32x5;
R_f32x5 ret_f32x5(int seed) {
    R_f32x5 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (float) (seed + 1.5); s.f2 = (float) (seed + 2.5); s.f3 = (float) (seed + 3.5); s.f4 = (float) (seed + 4.5);
    return s;
}
int check_f32x5(R_f32x5 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (float) (seed + 1.5) && s.f2 == (float) (seed + 2.5) && s.f3 == (float) (seed + 3.5) && s.f4 == (float) (seed + 4.5);
}
R_f32x5 call_f32x5(R_f32x5 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32x5(R_f32x5 (*cb)(int), int seed) {
    return check_f32x5(cb(seed), seed);
}

typedef struct { double f0; } R_f64;
R_f64 ret_f64(int seed) {
    R_f64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (double) (seed + 0.5);
    return s;
}
int check_f64(R_f64 s, int seed) {
    return s.f0 == (double) (seed + 0.5);
}
R_f64 call_f64(R_f64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f64(R_f64 (*cb)(int), int seed) {
    return check_f64(cb(seed), seed);
}

typedef struct { double f0; double f1; } R_f64x2;
R_f64x2 ret_f64x2(int seed) {
    R_f64x2 s;
    memset(&s, 0, sizeof s);
    s.f0 = (double) (seed + 0.5); s.f1 = (double) (seed + 1.5);
    return s;
}
int check_f64x2(R_f64x2 s, int seed) {
    return s.f0 == (double) (seed + 0.5) && s.f1 == (double) (seed + 1.5);
}
R_f64x2 call_f64x2(R_f64x2 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f64x2(R_f64x2 (*cb)(int), int seed) {
    return check_f64x2(cb(seed), seed);
}

typedef struct { double f0; double f1; double f2; } R_f64x3;
R_f64x3 ret_f64x3(int seed) {
    R_f64x3 s;
    memset(&s, 0, sizeof s);
    s.f0 = (double) (seed + 0.5); s.f1 = (double) (seed + 1.5); s.f2 = (double) (seed + 2.5);
    return s;
}
int check_f64x3(R_f64x3 s, int seed) {
    return s.f0 == (double) (seed + 0.5) && s.f1 == (double) (seed + 1.5) && s.f2 == (double) (seed + 2.5);
}
R_f64x3 call_f64x3(R_f64x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f64x3(R_f64x3 (*cb)(int), int seed) {
    return check_f64x3(cb(seed), seed);
}

typedef struct { double f0; double f1; double f2; double f3; } R_f64x4;
R_f64x4 ret_f64x4(int seed) {
    R_f64x4 s;
    memset(&s, 0, sizeof s);
    s.f0 = (double) (seed + 0.5); s.f1 = (double) (seed + 1.5); s.f2 = (double) (seed + 2.5); s.f3 = (double) (seed + 3.5);
    return s;
}
int check_f64x4(R_f64x4 s, int seed) {
    return s.f0 == (double) (seed + 0.5) && s.f1 == (double) (seed + 1.5) && s.f2 == (double) (seed + 2.5) && s.f3 == (double) (seed + 3.5);
}
R_f64x4 call_f64x4(R_f64x4 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f64x4(R_f64x4 (*cb)(int), int seed) {
    return check_f64x4(cb(seed), seed);
}

typedef struct { double f0; double f1; double f2; double f3; double f4; } R_f64x5;
R_f64x5 ret_f64x5(int seed) {
    R_f64x5 s;
    memset(&s, 0, sizeof s);
    s.f0 = (double) (seed + 0.5); s.f1 = (double) (seed + 1.5); s.f2 = (double) (seed + 2.5); s.f3 = (double) (seed + 3.5); s.f4 = (double) (seed + 4.5);
    return s;
}
int check_f64x5(R_f64x5 s, int seed) {
    return s.f0 == (double) (seed + 0.5) && s.f1 == (double) (seed + 1.5) && s.f2 == (double) (seed + 2.5) && s.f3 == (double) (seed + 3.5) && s.f4 == (double) (seed + 4.5);
}
R_f64x5 call_f64x5(R_f64x5 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f64x5(R_f64x5 (*cb)(int), int seed) {
    return check_f64x5(cb(seed), seed);
}

typedef struct { int f0; float f1; } R_i32f32;
R_i32f32 ret_i32f32(int seed) {
    R_i32f32 s;
    memset(&s, 0, sizeof s);
    s.f0 = (int) (seed + 1); s.f1 = (float) (seed + 1.5);
    return s;
}
int check_i32f32(R_i32f32 s, int seed) {
    return s.f0 == (int) (seed + 1) && s.f1 == (float) (seed + 1.5);
}
R_i32f32 call_i32f32(R_i32f32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i32f32(R_i32f32 (*cb)(int), int seed) {
    return check_i32f32(cb(seed), seed);
}

typedef struct { float f0; int f1; } R_f32i32;
R_f32i32 ret_f32i32(int seed) {
    R_f32i32 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (int) (seed + 2);
    return s;
}
int check_f32i32(R_f32i32 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (int) (seed + 2);
}
R_f32i32 call_f32i32(R_f32i32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32i32(R_f32i32 (*cb)(int), int seed) {
    return check_f32i32(cb(seed), seed);
}

typedef struct { long long f0; double f1; } R_i64f64;
R_i64f64 ret_i64f64(int seed) {
    R_i64f64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (long long) (seed + 1); s.f1 = (double) (seed + 1.5);
    return s;
}
int check_i64f64(R_i64f64 s, int seed) {
    return s.f0 == (long long) (seed + 1) && s.f1 == (double) (seed + 1.5);
}
R_i64f64 call_i64f64(R_i64f64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i64f64(R_i64f64 (*cb)(int), int seed) {
    return check_i64f64(cb(seed), seed);
}

typedef struct { double f0; long long f1; } R_f64i64;
R_f64i64 ret_f64i64(int seed) {
    R_f64i64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (double) (seed + 0.5); s.f1 = (long long) (seed + 2);
    return s;
}
int check_f64i64(R_f64i64 s, int seed) {
    return s.f0 == (double) (seed + 0.5) && s.f1 == (long long) (seed + 2);
}
R_f64i64 call_f64i64(R_f64i64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f64i64(R_f64i64 (*cb)(int), int seed) {
    return check_f64i64(cb(seed), seed);
}

typedef struct { float f0; float f1; long long f2; } R_f32x2i64;
R_f32x2i64 ret_f32x2i64(int seed) {
    R_f32x2i64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (float) (seed + 1.5); s.f2 = (long long) (seed + 3);
    return s;
}
int check_f32x2i64(R_f32x2i64 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (float) (seed + 1.5) && s.f2 == (long long) (seed + 3);
}
R_f32x2i64 call_f32x2i64(R_f32x2i64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32x2i64(R_f32x2i64 (*cb)(int), int seed) {
    return check_f32x2i64(cb(seed), seed);
}

typedef struct { long long f0; float f1; float f2; } R_i64f32x2;
R_i64f32x2 ret_i64f32x2(int seed) {
    R_i64f32x2 s;
    memset(&s, 0, sizeof s);
    s.f0 = (long long) (seed + 1); s.f1 = (float) (seed + 1.5); s.f2 = (float) (seed + 2.5);
    return s;
}
int check_i64f32x2(R_i64f32x2 s, int seed) {
    return s.f0 == (long long) (seed + 1) && s.f1 == (float) (seed + 1.5) && s.f2 == (float) (seed + 2.5);
}
R_i64f32x2 call_i64f32x2(R_i64f32x2 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i64f32x2(R_i64f32x2 (*cb)(int), int seed) {
    return check_i64f32x2(cb(seed), seed);
}

typedef struct { double f0; float f1; } R_f64f32;
R_f64f32 ret_f64f32(int seed) {
    R_f64f32 s;
    memset(&s, 0, sizeof s);
    s.f0 = (double) (seed + 0.5); s.f1 = (float) (seed + 1.5);
    return s;
}
int check_f64f32(R_f64f32 s, int seed) {
    return s.f0 == (double) (seed + 0.5) && s.f1 == (float) (seed + 1.5);
}
R_f64f32 call_f64f32(R_f64f32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f64f32(R_f64f32 (*cb)(int), int seed) {
    return check_f64f32(cb(seed), seed);
}

typedef struct { float f0; double f1; } R_f32f64;
R_f32f64 ret_f32f64(int seed) {
    R_f32f64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (double) (seed + 1.5);
    return s;
}
int check_f32f64(R_f32f64 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (double) (seed + 1.5);
}
R_f32f64 call_f32f64(R_f32f64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32f64(R_f32f64 (*cb)(int), int seed) {
    return check_f32f64(cb(seed), seed);
}

typedef struct { signed char f0; float f1; } R_i8f32;
R_i8f32 ret_i8f32(int seed) {
    R_i8f32 s;
    memset(&s, 0, sizeof s);
    s.f0 = (signed char) (seed + 1); s.f1 = (float) (seed + 1.5);
    return s;
}
int check_i8f32(R_i8f32 s, int seed) {
    return s.f0 == (signed char) (seed + 1) && s.f1 == (float) (seed + 1.5);
}
R_i8f32 call_i8f32(R_i8f32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i8f32(R_i8f32 (*cb)(int), int seed) {
    return check_i8f32(cb(seed), seed);
}

typedef struct { float f0; float f1; float f2; int f3; } R_f32x3i32;
R_f32x3i32 ret_f32x3i32(int seed) {
    R_f32x3i32 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (float) (seed + 1.5); s.f2 = (float) (seed + 2.5); s.f3 = (int) (seed + 4);
    return s;
}
int check_f32x3i32(R_f32x3i32 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (float) (seed + 1.5) && s.f2 == (float) (seed + 2.5) && s.f3 == (int) (seed + 4);
}
R_f32x3i32 call_f32x3i32(R_f32x3i32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32x3i32(R_f32x3i32 (*cb)(int), int seed) {
    return check_f32x3i32(cb(seed), seed);
}

typedef struct { double f0; signed char f1; } R_f64i8;
R_f64i8 ret_f64i8(int seed) {
    R_f64i8 s;
    memset(&s, 0, sizeof s);
    s.f0 = (double) (seed + 0.5); s.f1 = (signed char) (seed + 2);
    return s;
}
int check_f64i8(R_f64i8 s, int seed) {
    return s.f0 == (double) (seed + 0.5) && s.f1 == (signed char) (seed + 2);
}
R_f64i8 call_f64i8(R_f64i8 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f64i8(R_f64i8 (*cb)(int), int seed) {
    return check_f64i8(cb(seed), seed);
}

typedef struct { float f0; long long f1; } R_f32i64;
R_f32i64 ret_f32i64(int seed) {
    R_f32i64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (long long) (seed + 2);
    return s;
}
int check_f32i64(R_f32i64 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (long long) (seed + 2);
}
R_f32i64 call_f32i64(R_f32i64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32i64(R_f32i64 (*cb)(int), int seed) {
    return check_f32i64(cb(seed), seed);
}

typedef struct { long long f0; float f1; } R_i64f32;
R_i64f32 ret_i64f32(int seed) {
    R_i64f32 s;
    memset(&s, 0, sizeof s);
    s.f0 = (long long) (seed + 1); s.f1 = (float) (seed + 1.5);
    return s;
}
int check_i64f32(R_i64f32 s, int seed) {
    return s.f0 == (long long) (seed + 1) && s.f1 == (float) (seed + 1.5);
}
R_i64f32 call_i64f32(R_i64f32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_i64f32(R_i64f32 (*cb)(int), int seed) {
    return check_i64f32(cb(seed), seed);
}

typedef struct { double f0; double f1; long long f2; } R_f64x2i64;
R_f64x2i64 ret_f64x2i64(int seed) {
    R_f64x2i64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (double) (seed + 0.5); s.f1 = (double) (seed + 1.5); s.f2 = (long long) (seed + 3);
    return s;
}
int check_f64x2i64(R_f64x2i64 s, int seed) {
    return s.f0 == (double) (seed + 0.5) && s.f1 == (double) (seed + 1.5) && s.f2 == (long long) (seed + 3);
}
R_f64x2i64 call_f64x2i64(R_f64x2i64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f64x2i64(R_f64x2i64 (*cb)(int), int seed) {
    return check_f64x2i64(cb(seed), seed);
}

typedef struct { float f0; signed char f1; signed char f2; signed char f3; } R_f32i8x3;
R_f32i8x3 ret_f32i8x3(int seed) {
    R_f32i8x3 s;
    memset(&s, 0, sizeof s);
    s.f0 = (float) (seed + 0.5); s.f1 = (signed char) (seed + 2); s.f2 = (signed char) (seed + 3); s.f3 = (signed char) (seed + 4);
    return s;
}
int check_f32i8x3(R_f32i8x3 s, int seed) {
    return s.f0 == (float) (seed + 0.5) && s.f1 == (signed char) (seed + 2) && s.f2 == (signed char) (seed + 3) && s.f3 == (signed char) (seed + 4);
}
R_f32i8x3 call_f32i8x3(R_f32i8x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_f32i8x3(R_f32i8x3 (*cb)(int), int seed) {
    return check_f32i8x3(cb(seed), seed);
}

typedef struct { float f0[2]; } R_af32x2;
R_af32x2 ret_af32x2(int seed) {
    R_af32x2 s;
    memset(&s, 0, sizeof s);
    s.f0[0] = (float) (seed + 0.5); s.f0[1] = (float) (seed + 1.5);
    return s;
}
int check_af32x2(R_af32x2 s, int seed) {
    return s.f0[0] == (float) (seed + 0.5) && s.f0[1] == (float) (seed + 1.5);
}
R_af32x2 call_af32x2(R_af32x2 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_af32x2(R_af32x2 (*cb)(int), int seed) {
    return check_af32x2(cb(seed), seed);
}

typedef struct { float f0[3]; } R_af32x3;
R_af32x3 ret_af32x3(int seed) {
    R_af32x3 s;
    memset(&s, 0, sizeof s);
    s.f0[0] = (float) (seed + 0.5); s.f0[1] = (float) (seed + 1.5); s.f0[2] = (float) (seed + 2.5);
    return s;
}
int check_af32x3(R_af32x3 s, int seed) {
    return s.f0[0] == (float) (seed + 0.5) && s.f0[1] == (float) (seed + 1.5) && s.f0[2] == (float) (seed + 2.5);
}
R_af32x3 call_af32x3(R_af32x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_af32x3(R_af32x3 (*cb)(int), int seed) {
    return check_af32x3(cb(seed), seed);
}

typedef struct { double f0[3]; } R_af64x3;
R_af64x3 ret_af64x3(int seed) {
    R_af64x3 s;
    memset(&s, 0, sizeof s);
    s.f0[0] = (double) (seed + 0.5); s.f0[1] = (double) (seed + 1.5); s.f0[2] = (double) (seed + 2.5);
    return s;
}
int check_af64x3(R_af64x3 s, int seed) {
    return s.f0[0] == (double) (seed + 0.5) && s.f0[1] == (double) (seed + 1.5) && s.f0[2] == (double) (seed + 2.5);
}
R_af64x3 call_af64x3(R_af64x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_af64x3(R_af64x3 (*cb)(int), int seed) {
    return check_af64x3(cb(seed), seed);
}

typedef struct { int f0[5]; } R_ai32x5;
R_ai32x5 ret_ai32x5(int seed) {
    R_ai32x5 s;
    memset(&s, 0, sizeof s);
    s.f0[0] = (int) (seed + 1); s.f0[1] = (int) (seed + 2); s.f0[2] = (int) (seed + 3); s.f0[3] = (int) (seed + 4); s.f0[4] = (int) (seed + 5);
    return s;
}
int check_ai32x5(R_ai32x5 s, int seed) {
    return s.f0[0] == (int) (seed + 1) && s.f0[1] == (int) (seed + 2) && s.f0[2] == (int) (seed + 3) && s.f0[3] == (int) (seed + 4) && s.f0[4] == (int) (seed + 5);
}
R_ai32x5 call_ai32x5(R_ai32x5 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_ai32x5(R_ai32x5 (*cb)(int), int seed) {
    return check_ai32x5(cb(seed), seed);
}

typedef struct { signed char f0[3]; } R_ai8x3;
R_ai8x3 ret_ai8x3(int seed) {
    R_ai8x3 s;
    memset(&s, 0, sizeof s);
    s.f0[0] = (signed char) (seed + 1); s.f0[1] = (signed char) (seed + 2); s.f0[2] = (signed char) (seed + 3);
    return s;
}
int check_ai8x3(R_ai8x3 s, int seed) {
    return s.f0[0] == (signed char) (seed + 1) && s.f0[1] == (signed char) (seed + 2) && s.f0[2] == (signed char) (seed + 3);
}
R_ai8x3 call_ai8x3(R_ai8x3 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_ai8x3(R_ai8x3 (*cb)(int), int seed) {
    return check_ai8x3(cb(seed), seed);
}

typedef struct { float f0; float f1; } R_nf32_f0;
typedef struct { R_nf32_f0 f0; float f1; } R_nf32;
R_nf32 ret_nf32(int seed) {
    R_nf32 s;
    memset(&s, 0, sizeof s);
    s.f0.f0 = (float) (seed + 0.5); s.f0.f1 = (float) (seed + 1.5); s.f1 = (float) (seed + 2.5);
    return s;
}
int check_nf32(R_nf32 s, int seed) {
    return s.f0.f0 == (float) (seed + 0.5) && s.f0.f1 == (float) (seed + 1.5) && s.f1 == (float) (seed + 2.5);
}
R_nf32 call_nf32(R_nf32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_nf32(R_nf32 (*cb)(int), int seed) {
    return check_nf32(cb(seed), seed);
}

typedef struct { int f0; float f1; } R_nmix_f0;
typedef struct { R_nmix_f0 f0; double f1; } R_nmix;
R_nmix ret_nmix(int seed) {
    R_nmix s;
    memset(&s, 0, sizeof s);
    s.f0.f0 = (int) (seed + 1); s.f0.f1 = (float) (seed + 1.5); s.f1 = (double) (seed + 2.5);
    return s;
}
int check_nmix(R_nmix s, int seed) {
    return s.f0.f0 == (int) (seed + 1) && s.f0.f1 == (float) (seed + 1.5) && s.f1 == (double) (seed + 2.5);
}
R_nmix call_nmix(R_nmix (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_nmix(R_nmix (*cb)(int), int seed) {
    return check_nmix(cb(seed), seed);
}

typedef struct { float f0; } R_nnf32_f0_f0;
typedef struct { R_nnf32_f0_f0 f0; float f1; } R_nnf32_f0;
typedef struct { R_nnf32_f0 f0; float f1; float f2; } R_nnf32;
R_nnf32 ret_nnf32(int seed) {
    R_nnf32 s;
    memset(&s, 0, sizeof s);
    s.f0.f0.f0 = (float) (seed + 0.5); s.f0.f1 = (float) (seed + 1.5); s.f1 = (float) (seed + 2.5); s.f2 = (float) (seed + 3.5);
    return s;
}
int check_nnf32(R_nnf32 s, int seed) {
    return s.f0.f0.f0 == (float) (seed + 0.5) && s.f0.f1 == (float) (seed + 1.5) && s.f1 == (float) (seed + 2.5) && s.f2 == (float) (seed + 3.5);
}
R_nnf32 call_nnf32(R_nnf32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_nnf32(R_nnf32 (*cb)(int), int seed) {
    return check_nnf32(cb(seed), seed);
}

typedef struct { double f0; } R_nf64x2_f0;
typedef struct { double f0; } R_nf64x2_f1;
typedef struct { R_nf64x2_f0 f0; R_nf64x2_f1 f1; } R_nf64x2;
R_nf64x2 ret_nf64x2(int seed) {
    R_nf64x2 s;
    memset(&s, 0, sizeof s);
    s.f0.f0 = (double) (seed + 0.5); s.f1.f0 = (double) (seed + 1.5);
    return s;
}
int check_nf64x2(R_nf64x2 s, int seed) {
    return s.f0.f0 == (double) (seed + 0.5) && s.f1.f0 == (double) (seed + 1.5);
}
R_nf64x2 call_nf64x2(R_nf64x2 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_nf64x2(R_nf64x2 (*cb)(int), int seed) {
    return check_nf64x2(cb(seed), seed);
}

typedef struct { long long f0; long long f1; } R_nbig_f0;
typedef struct { double f0; double f1; } R_nbig_f1;
typedef struct { R_nbig_f0 f0; R_nbig_f1 f1; } R_nbig;
R_nbig ret_nbig(int seed) {
    R_nbig s;
    memset(&s, 0, sizeof s);
    s.f0.f0 = (long long) (seed + 1); s.f0.f1 = (long long) (seed + 2); s.f1.f0 = (double) (seed + 2.5); s.f1.f1 = (double) (seed + 3.5);
    return s;
}
int check_nbig(R_nbig s, int seed) {
    return s.f0.f0 == (long long) (seed + 1) && s.f0.f1 == (long long) (seed + 2) && s.f1.f0 == (double) (seed + 2.5) && s.f1.f1 == (double) (seed + 3.5);
}
R_nbig call_nbig(R_nbig (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_nbig(R_nbig (*cb)(int), int seed) {
    return check_nbig(cb(seed), seed);
}

typedef union { int f0; float f1; } R_ui32f32;
R_ui32f32 ret_ui32f32(int seed) {
    R_ui32f32 s;
    memset(&s, 0, sizeof s);
    s.f0 = (int) (seed + 1);
    return s;
}
int check_ui32f32(R_ui32f32 s, int seed) {
    return s.f0 == (int) (seed + 1);
}
R_ui32f32 call_ui32f32(R_ui32f32 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_ui32f32(R_ui32f32 (*cb)(int), int seed) {
    return check_ui32f32(cb(seed), seed);
}

typedef union { long long f0; double f1; } R_ui64f64;
R_ui64f64 ret_ui64f64(int seed) {
    R_ui64f64 s;
    memset(&s, 0, sizeof s);
    s.f0 = (long long) (seed + 1);
    return s;
}
int check_ui64f64(R_ui64f64 s, int seed) {
    return s.f0 == (long long) (seed + 1);
}
R_ui64f64 call_ui64f64(R_ui64f64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_ui64f64(R_ui64f64 (*cb)(int), int seed) {
    return check_ui64f64(cb(seed), seed);
}

typedef union { float f0[2]; long long f1; } R_uf32x2i64;
R_uf32x2i64 ret_uf32x2i64(int seed) {
    R_uf32x2i64 s;
    memset(&s, 0, sizeof s);
    s.f0[0] = (float) (seed + 0.5); s.f0[1] = (float) (seed + 1.5);
    return s;
}
int check_uf32x2i64(R_uf32x2i64 s, int seed) {
    return s.f0[0] == (float) (seed + 0.5) && s.f0[1] == (float) (seed + 1.5);
}
R_uf32x2i64 call_uf32x2i64(R_uf32x2i64 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_uf32x2i64(R_uf32x2i64 (*cb)(int), int seed) {
    return check_uf32x2i64(cb(seed), seed);
}

typedef union { double f0[2]; signed char f1[3]; } R_uf64x2i8;
R_uf64x2i8 ret_uf64x2i8(int seed) {
    R_uf64x2i8 s;
    memset(&s, 0, sizeof s);
    s.f0[0] = (double) (seed + 0.5); s.f0[1] = (double) (seed + 1.5);
    return s;
}
int check_uf64x2i8(R_uf64x2i8 s, int seed) {
    return s.f0[0] == (double) (seed + 0.5) && s.f0[1] == (double) (seed + 1.5);
}
R_uf64x2i8 call_uf64x2i8(R_uf64x2i8 (*cb)(int), int seed) {
    return cb(seed);
}
int apply_check_uf64x2i8(R_uf64x2i8 (*cb)(int), int seed) {
    return check_uf64x2i8(cb(seed), seed);
}

