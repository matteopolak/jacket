TAU :: cast(float32) 6.283185;
TAU64 :: 6.28318530717958648;
PI :: cast(float32) 3.1415927;
PI64 :: 3.141592653589793;
SQRT2 :: 1.41421356237309504880;
SQRTH :: 0.70710678118654752440;
DP1 :: 7.85398125648498535156E-1;
DP2 :: 3.77489470793079817668E-8;
DP3 :: 2.69515142907905952645E-15;
LOSSTH: float64 : 1.073741824e9;
ASIN_AND_ACOS_ERROR_REPORTING_LIMIT :: 1.000001;
FLOAT16_MAX: float : 65504.0;
FLOAT32_MIN :: 0h00800000;
FLOAT32_MAX :: 0h7F7FFFFF;
FLOAT32_INFINITY :: 0h7F800000;
FLOAT32_NAN :: 0h7FBFFFFF;
FLOAT64_MIN :: 0h00100000_00000000;
FLOAT64_MAX :: 0h7FEFFFFF_FFFFFFFF;
FLOAT64_INFINITY :: 0h7FF00000_00000000;
FLOAT64_NAN :: 0h7FF7FFFF_FFFFFFFF;
S8_MIN: s8 : -128;
S8_MAX: s8 : 127;
U8_MAX: u8 : 255;
S16_MIN: s16 : -32768;
S16_MAX: s16 : 32767;
U16_MAX: u16 : 0xffff;
S32_MIN: s32 : 0x8000_0000;
S32_MAX: s32 : 0x7fff_ffff;
U32_MAX: u32 : 0xffff_ffff;
S64_MIN: s64 : 0x8000_0000_0000_0000;
S64_MAX: s64 : 0x7fff_ffff_ffff_ffff;
U64_MAX: u64 : 0xffff_ffff_ffff_ffff;
ceil :: (x: float64) -> float64 #intrinsic; @UseResult
floor :: (x: float64) -> float64 #intrinsic; @UseResult

ceil :: (f: float) -> float #intrinsic; @UseResult

floor :: (f: float) -> float #intrinsic; @UseResult

sqrt :: inline (x: float) -> float #intrinsic; @UseResult

sqrt :: (x: float64) -> float64 #intrinsic; @UseResult

sin :: (x: float32) -> float32 #intrinsic; @UseResult

cos :: (x: float32) -> float32 #intrinsic; @UseResult

sin :: (x: float64) -> float64 #intrinsic; @UseResult
cos :: (x: float64) -> float64 #intrinsic; @UseResult

sin_or_cos :: (x: float64, $IS_SIN: bool) -> float64 {
    #if IS_SIN return sin(x);
    else return cos(x);
}

Clamp :: inline (pointer: *$T, low: T, high: T) {
    if pointer.* < low pointer.* = low;
    if pointer.* > high pointer.* = high;
}

abs :: inline (x: $T/ .[s64, s32, s16, s8]) -> T {
    if x >= 0 return x;
    assert(x != cast,trunc(T) ((cast(u64) 1) << (size_of(T) * 8 - 1)));
    return -x;
} @UseResult

abs :: inline (x: float64) -> float64 {
    u: Bits64;
    u.f = x;
    u.u &= 0x7fff_ffff_ffff_ffff;
    return u.f;
} @UseResult

abs :: inline (x: float32) -> float32 {
    u: Bits32;
    u.f = x;
    u.u &= 0x7fff_ffff;
    return u.f;
} @UseResult

is_nan_is_inf :: (x: float64) -> (isnan: bool, isinf: bool) {
    return x != x, abs(x) == FLOAT64_INFINITY;
}

is_nan_is_inf :: (x: float32) -> (isnan: bool, isinf: bool) {
    return x != x, abs(x) == FLOAT32_INFINITY;
}

isnan :: (x: float64) -> bool {
    return x != x;
}

isnan :: (x: float32) -> bool {
    return x != x;
}

isinf :: (x: float64) -> bool {
    return abs(x) == FLOAT64_INFINITY;
}

isinf :: (x: float32) -> bool {
    return abs(x) == FLOAT32_INFINITY;
}

is_finite :: (x: float64) -> bool {
    return !isnan(x) && !isinf(x);
}

is_finite :: (x: float) -> bool {
    return !isnan(x) && !isinf(x);
}

frac :: inline (x: $T) -> T #modify {
    return T == float32 || T == float64;
} {
    return x - floor(x);
}

fmod_cycling :: inline (x: float64, y: float64) -> float64 {
    if y == 0 {
        print("fmod_cycling: zero divisor\n", to_standard_error = true);
        return FLOAT64_INFINITY;
    }

    return x - floor(x / y) * y;
}

fmod_cycling :: inline (x: float, divisor: float) -> float {
    return cast(float) fmod_cycling(cast(float64) x, cast(float64) divisor);
}

saturate :: (f: float) -> float {
    return clamp(f, 0.0, 1.0);
}

saturate :: (f: float64) -> float64 {
    return clamp(f, 0.0, 1.0);
}

square :: (a: $T) -> T {
    return a * a;
}

reduc :: (x: float64) -> float64 {
    return floor(16 * x) / 16;
}

lerp :: inline (a: float, b: float, t: float) -> float {
    return a + (b - a) * t;
} @UseResult

lerp :: inline (a: float64, b: float64, t: float64) -> float64 {
    return a + (b - a) * t;
} @UseResult

lerp :: (a: $R, b: R, t: float) -> R #modify {
    return R != float32 && R != float64;
} {
    return a + (b - a) * t;
} @UseResult

signbit :: inline (x: float64) -> u64 {
    u: Bits64;
    u.f = x;
    return u.u >> 63;
}

frexp :: (x: float64) -> (result: float64, pow2: s32) {
    if x == 0 || !is_finite(x) return x, 0;

    u: Bits64;
    u.f = x;
    e := cast(s32) ((u.u >> 52) & 0x7ff);
    adjust: s32 = 0;

    if e == 0 {
        u.f = x * 18014398509481984.0;
        e = cast(s32) ((u.u >> 52) & 0x7ff);
        adjust = -54;
    }

    u.u = (u.u & 0x800f_ffff_ffff_ffff) | 0x3fe0_0000_0000_0000;

    return u.f, e - 1022 + adjust;
}

ldexp :: (x: float64, pw2: s32) -> float64 {
    if x == 0 || !is_finite(x) return x;

    n := pw2;
    r := x;

    // Chunking keeps the final underflow rounding in one multiplication.
    while n > 1023 {
        r *= 8.98846567431158e307;
        n -= 1023;
        if isinf(r) return r;
    }

    while n < -1022 {
        r *= 2.2250738585072014e-308;
        n += 1022;
        if r == 0 return r;
    }

    u: Bits64;
    u.u = (cast(u64) (n + 1023)) << 52;

    return r * u.f;
}

log :: (x: float64) -> float64 {
    if isnan(x) || x < 0 return FLOAT64_NAN;
    if x == 0 return -FLOAT64_INFINITY;
    if isinf(x) return x;

    m, e := frexp(x);
    if m < SQRTH {
        m *= 2;
        e -= 1;
    }

    z := (m - 1) / (m + 1);
    z2 := z * z;
    term := z;
    total := z;

    for k: 1..24 {
        term *= z2;
        total += term / cast(float64) (2 * k + 1);
    }

    return 2 * total + cast(float64) e * 0.693147180559945309417;
}

log :: inline (x: float) -> float {
    return cast(float) log(cast(float64) x);
}

exp :: (x: float64) -> float64 {
    if isnan(x) return x;
    if x > 709.782712893384 return FLOAT64_INFINITY;
    if x < -745.133219101941 return 0;

    n := cast(s32) floor(x * 1.44269504088896340736 + 0.5);
    r := (x - cast(float64) n * 0.6931471803691238) - cast(float64) n * 1.9082149292705877e-10;
    term: float64 = 1;
    total: float64 = 1;

    for k: 1..20 {
        term *= r / cast(float64) k;
        total += term;
    }

    return ldexp(total, n);
}

exp :: inline (x: float) -> float {
    return cast(float) exp(cast(float64) x);
}

pow :: (x: float64, y: float64) -> float64 {
    if y == 0 || x == 1 return 1;
    if isnan(x) || isnan(y) return FLOAT64_NAN;

    if isinf(y) {
        a := abs(x);
        if a == 1 return 1;
        if (a > 1) == (y > 0) return FLOAT64_INFINITY;
        return 0;
    }

    if floor(y) == y && abs(y) < 2147483648.0 {
        n := cast(u32) abs(y);
        base := x;

        if y < 0 base = 1 / x;

        result: float64 = 1;

        while n > 0 {
            if (n & 1) != 0 result *= base;
            n >>= 1;
            if n > 0 base *= base;
        }

        return result;
    }

    neg := signbit(x) != 0;
    odd := false;

    if neg {
        if floor(y) != y && x != 0 && !isinf(x) return FLOAT64_NAN;
        odd = abs(y) < 9007199254740992.0 && fmod_cycling(abs(y), 2.0) == 1;
    }

    r: float64;

    if x == 0 {
        if y > 0 r = 0;
        else r = FLOAT64_INFINITY;
    } else r = exp(y * log(abs(x)));

    if odd return -r;

    return r;
}

pow :: inline (x: float, power: float) -> float {
    return cast(float) pow(cast(float64) x, cast(float64) power);
}

atan :: inline (x: float64) -> float64 {
    if isnan(x) || x == 0 return x;

    a := abs(x);
    base: float64 = 0;
    flip := a > 1;

    if flip a = 1 / a;

    if a > 0.414213562373095 {
        a = (a - 1) / (a + 1);
        base = PI64 / 4;
    }

    term := a;
    total := a;
    aa := -a * a;

    for k: 1..28 {
        term *= aa;
        total += term / cast(float64) (2 * k + 1);
    }

    total += base;
    if flip total = PI64 / 2 - total;
    if x < 0 return -total;

    return total;
}

atan :: inline (y: float32) -> float32 {
    return cast(float) atan(cast(float64) y);
}

atan2 :: (y: float64, x: float64) -> float64 {
    if isnan(x) || isnan(y) return FLOAT64_NAN;

    if y == 0 {
        if signbit(x) != 0 {
            if signbit(y) != 0 return -PI64;
            return PI64;
        }

        return y;
    }

    if x == 0 {
        if y < 0 return -PI64 / 2;
        return PI64 / 2;
    }

    if isinf(x) && isinf(y) {
        a := PI64 / 4;
        if x < 0 a = 3 * a;
        if y < 0 a = -a;
        return a;
    }

    a := atan(y / x);

    if x < 0 {
        if y < 0 a -= PI64;
        else a += PI64;
    }

    return a;
}

atan2 :: (y: float, x: float) -> float {
    return cast(float) atan2(cast(float64) y, cast(float64) x);
}

asin :: (x: float64) -> float64 {
    if abs(x) > 1 return FLOAT64_NAN;
    return atan2(x, sqrt((1 - x) * (1 + x)));
}

acos :: (x: float64) -> float64 {
    if abs(x) > 1 return FLOAT64_NAN;
    return atan2(sqrt((1 - x) * (1 + x)), x);
}

asin :: inline (theta: float) -> float {
    return cast(float) asin(cast(float64) theta);
}

acos :: inline (theta: float) -> float {
    return cast(float) acos(cast(float64) theta);
}

tan :: (x: float64) -> float64 {
    return sin(x) / cos(x);
} @UseResult

tan :: inline (theta: float) -> float {
    return sin(theta) / cos(theta);
} @UseResult

cot :: (x: float64) -> float64 {
    return cos(x) / sin(x);
}

half_to_float :: (h: u16) -> float {
    sign := (cast(u32) (h & 0x8000)) << 16;
    e := cast(u32) ((h >> 10) & 31);
    m := cast(u32) (h & 1023);
    u: Bits32;

    if e == 0 {
        if m == 0 {
            u.u = sign;
            return u.f;
        }

        // 2^-24 is exactly representable in float32; this is intentional
        // binary32 arithmetic for a half subnormal's integer significand.
        f := cast(float) m * cast(float) 5.9604644775390625e-8;
        if sign != 0 return -f;

        return f;
    }

    if e == 31 u.u = sign | 0x7f800000 | (m << 13);
    else u.u = sign | ((e + 112) << 23) | (m << 13);

    return u.f;
}

#scope_file
Bits32 :: union {
    f: float32;
    u: u32;
}

Bits64 :: union {
    f: float64;
    u: u64;
}

#import "Basic";
