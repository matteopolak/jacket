// The exact C math functions: those LLVM calls for operations wasm has no instruction for
// (`round`, whose halves go away from zero where wasm's `nearest` rounds them to even, and
// `fma`), which jaic emits for `#intrinsic` procedures of those names and for the inline assembly
// it emulates (`roundsd`, `vfmadd...`), plus `floor`, `ceil`, `trunc`, `sqrt`, `fabs`,
// `copysign`, `fmin`, `fmax` and `fmod` for programs that declare them `#foreign` from libm.
// WASI builds link no libm, so they are here, with their float32 forms; the transcendental ones
// are in elementary.jai.
//
// Every result is the exact one rounded once: `fma` and `fmod` through a wide accumulator that
// quad.jai's binary128 helpers share.

#scope_module

truncate :: (x: float64) -> float64 #intrinsic "llvm.trunc.f64";
nearest_integer :: (x: float64) -> float64 #intrinsic "llvm.nearbyint.f64";

bits_of :: (x: float64) -> u64 #no_context {
    return (cast(*u64) *x).*;
}

from_bits :: (bits: u64) -> float64 #no_context {
    return (cast(*float64) *bits).*;
}

SIGN_BIT: u64 : 0x8000_0000_0000_0000;
FRACTION_BITS: u64 : 0x000f_ffff_ffff_ffff;
IMPLICIT_BIT: u64 : 0x0010_0000_0000_0000;
ONE: u64 : 1;

is_finite :: (x: float64) -> bool #no_context {
    return (bits_of(x) & 0x7ff0_0000_0000_0000) != 0x7ff0_0000_0000_0000;
}

#program_export "round"
wasi_round :: (x: float64) -> float64 #c_call {
    whole := truncate(x);
    // Exact: below 2^52 the difference is a fraction of x's own bits, above it x is whole
    // (NaN and infinities compare false and come back as they are).
    left := x - whole;
    if left >= 0.5 return whole + 1;
    if left <= -0.5 return whole - 1;
    return whole;
}

#program_export "roundf"
wasi_roundf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_round(x);
}

// ---------------------------------------------------------------------------------------------
// Fused multiply-add: a*b + c is formed exactly as an integer times a power of two, in a
// 4096-bit accumulator (enough for any two float64 exponents apart), then rounded once to
// nearest, ties to even, at the precision and exponent range of the result type.

LIMBS :: 64;
Wide :: [LIMBS] u64;

Fused_Format :: struct {
    precision: s64; // significand bits
    lowest_exponent: s64; // exponent of the least significant bit of the smallest subnormal
    highest_exponent: s64; // exponent of the most significant bit of the largest finite value
}

FLOAT64_FORMAT :: Fused_Format.{ 53, -1074, 1023 };
FLOAT32_FORMAT :: Fused_Format.{ 24, -149, 127 };

// |x| = significand * 2^exponent, for finite nonzero x.
split_float :: (x: float64) -> significand: u64, exponent: s64 #no_context {
    bits := bits_of(x);
    biased := cast(s64) ((bits >> 52) & 0x7ff);
    if biased == 0 return bits & FRACTION_BITS, -1074;
    return (bits & FRACTION_BITS) | IMPLICIT_BIT, biased - 1075;
}

// Adds high:low << shift into `into`.
add_shifted :: (into: *Wide, high: u64, low: u64, shift: s64) #no_context {
    limb := shift / 64;
    bit := cast(u64) (shift % 64);
    pieces: [3] u64;
    pieces[0] = low << bit;
    pieces[1] = high << bit;
    pieces[2] = 0;
    if bit != 0 {
        pieces[1] |= low >> (64 - bit);
        pieces[2] = high >> (64 - bit);
    }
    carry: u64;
    for i: 0..LIMBS - 1 - limb {
        piece := ifx i < 3 then pieces[i] else 0;
        if i >= 3 && carry == 0 break;
        before := into.*[limb + i];
        sum := before + piece;
        next_carry: u64 = ifx sum < before then 1 else 0;
        total := sum + carry;
        if total < sum next_carry = 1;
        into.*[limb + i] = total;
        carry = next_carry;
    }
}

// -1, 0 or 1 as a < b, a == b, a > b.
compare_wide :: (a: *Wide, b: *Wide) -> s64 #no_context {
    for < i: 0..LIMBS - 1 {
        if a.*[i] != b.*[i] return ifx a.*[i] < b.*[i] then -1 else 1;
    }
    return 0;
}

// a -= b, for a >= b.
subtract_wide :: (a: *Wide, b: *Wide) #no_context {
    borrow: u64;
    for i: 0..LIMBS - 1 {
        before := a.*[i];
        difference := before - b.*[i];
        next_borrow: u64 = ifx before < b.*[i] then 1 else 0;
        if difference < borrow next_borrow = 1;
        a.*[i] = difference - borrow;
        borrow = next_borrow;
    }
}

wide_bit :: (a: *Wide, position: s64) -> u64 #no_context {
    if position < 0 || position >= LIMBS * 64 return 0;
    return (a.*[position / 64] >> cast(u64) (position % 64)) & 1;
}

// 64 bits of `a` starting at `position` (>= 0).
wide_bits :: (a: *Wide, position: s64) -> u64 #no_context {
    limb := position / 64;
    if limb >= LIMBS return 0;
    bit := cast(u64) (position % 64);
    result := a.*[limb] >> bit;
    if bit != 0 && limb + 1 < LIMBS result |= a.*[limb + 1] << (64 - bit);
    return result;
}

// Whether any bit of `a` below `position` is set.
any_below :: (a: *Wide, position: s64) -> bool #no_context {
    if position <= 0 return false;
    limb := position / 64;
    if limb >= LIMBS {
        for a.* if it != 0 return true;
        return false;
    }
    for i: 0..limb - 1 if a.*[i] != 0 return true;
    bit := cast(u64) (position % 64);
    if bit == 0 return false;
    return (a.*[limb] & ((ONE << bit) - 1)) != 0;
}

// The float64 holding significand * 2^exponent, which the caller has already rounded to fit.
compose_float :: (negative: bool, significand: u64, exponent: s64) -> float64 #no_context {
    sign := ifx negative then SIGN_BIT else 0;
    if significand == 0 return from_bits(sign);
    m := significand;
    e := exponent;
    while m < IMPLICIT_BIT && e > -1074 {
        m <<= 1;
        e -= 1;
    }
    if m < IMPLICIT_BIT return from_bits(sign | m); // subnormal: e == -1074
    return from_bits(sign | ((cast(u64) (e + 1075)) << 52) | (m & FRACTION_BITS));
}

// A value rounded to a format: significand * 2^exponent, or an overflow to infinity.
Rounded :: struct {
    negative: bool;
    infinite: bool;
    high: u64; // significand, at most 128 bits
    low: u64;
    exponent: s64;
}

// Position of the highest set bit of `a`, -1 when it is zero.
highest_bit :: (a: *Wide) -> s64 #no_context {
    for < i: 0..LIMBS - 1 {
        if a.*[i] != 0 return i * 64 + highest_bit(a.*[i]);
    }
    return -1;
}

highest_bit :: (word: u64) -> s64 #no_context {
    if word == 0 return -1;
    top := 63;
    while (word >> cast(u64) top) == 0 top -= 1;
    return top;
}

bit_length :: (high: u64, low: u64) -> s64 #no_context {
    if high != 0 return 65 + highest_bit(high);
    return 1 + highest_bit(low);
}

shift_left :: (high: u64, low: u64, count: s64) -> u64, u64 #no_context {
    if count == 0 return high, low;
    if count >= 128 return 0, 0;
    if count >= 64 return low << cast(u64) (count - 64), 0;
    return (high << cast(u64) count) | (low >> cast(u64) (64 - count)), low << cast(u64) count;
}

shift_right :: (high: u64, low: u64, count: s64) -> u64, u64 #no_context {
    if count == 0 return high, low;
    if count >= 128 return 0, 0;
    if count >= 64 return 0, high >> cast(u64) (count - 64);
    return high >> cast(u64) count, (low >> cast(u64) count) | (high << cast(u64) (64 - count));
}

// The full 128-bit product of two u64s, from 32-bit halves.
multiply_wide :: (a: u64, b: u64) -> high: u64, low: u64 #no_context {
    a_low := a & 0xffff_ffff;
    a_high := a >> 32;
    b_low := b & 0xffff_ffff;
    b_high := b >> 32;
    low_low := a_low * b_low;
    high_low := a_high * b_low;
    low_high := a_low * b_high;
    high_high := a_high * b_high;
    // The middle column: each part < 2^64, and their sum with the carry-in is kept in two words.
    middle := (low_low >> 32) + (high_low & 0xffff_ffff) + (low_high & 0xffff_ffff);
    low := (middle << 32) | (low_low & 0xffff_ffff);
    high := high_high + (high_low >> 32) + (low_high >> 32) + (middle >> 32);
    return high, low;
}

// Signed sum of two magnitudes, left in `a`: the sum's sign, and whether it is exactly zero.
combine :: (a: *Wide, a_negative: bool, b: *Wide, b_negative: bool) -> negative: bool, zero: bool #no_context {
    if a_negative == b_negative {
        for i: 0..LIMBS - 1 {
            if b.*[i] != 0 add_shifted(a, 0, b.*[i], i * 64);
        }
        return a_negative, false;
    }
    order := compare_wide(a, b);
    if order == 0 return false, true;
    if order > 0 {
        subtract_wide(a, b);
        return a_negative, false;
    }
    subtract_wide(b, a);
    a.* = b.*;
    return b_negative, false;
}

// value * 2^base rounded to nearest, ties to even, at `format`.
round_wide :: (value: *Wide, base: s64, negative: bool, format: Fused_Format) -> Rounded #no_context {
    result: Rounded;
    result.negative = negative;
    top := highest_bit(value);
    if top < 0 return result;
    // The kept bits end at `lsb`, the exponent of the result's last significand bit.
    lsb := base + top - (format.precision - 1);
    if lsb < format.lowest_exponent lsb = format.lowest_exponent;
    drop := lsb - base;
    high, low: u64;
    if drop <= 0 {
        // Fewer bits than the precision: all of them are kept.
        high, low = shift_left(value.*[1], value.*[0], -drop);
    } else {
        kept := top - drop + 1;
        if kept > 0 {
            low = wide_bits(value, drop);
            high = wide_bits(value, drop + 64);
            if kept < 64 {
                low &= (ONE << cast(u64) kept) - 1;
                high = 0;
            } else if kept < 128 {
                high &= (ONE << cast(u64) (kept - 64)) - 1;
            }
        }
        half := wide_bit(value, drop - 1);
        if half != 0 && (any_below(value, drop - 1) || (low & 1) != 0) {
            low += 1;
            if low == 0 high += 1;
        }
        // Rounding up carried to 2^precision: one place up.
        if bit_length(high, low) > format.precision {
            high, low = shift_right(high, low, 1);
            lsb += 1;
        }
    }
    length := bit_length(high, low);
    if length > 0 && lsb + length - 1 > format.highest_exponent result.infinite = true;
    result.high = high;
    result.low = low;
    result.exponent = lsb;
    return result;
}

fused :: (a: float64, b: float64, c: float64, format: Fused_Format) -> float64 #no_context {
    if !is_finite(a) || !is_finite(b) return a * b + c; // the product is exactly inf or NaN
    if !is_finite(c) return c; // finite + c
    if a == 0 || b == 0 return a * b + c; // exact: a zero product, then one rounding
    product_negative := ((bits_of(a) ^ bits_of(b)) & SIGN_BIT) != 0;
    addend_negative := (bits_of(c) & SIGN_BIT) != 0;

    ma, ea := split_float(a);
    mb, eb := split_float(b);
    high, low := multiply_wide(ma, mb);
    product_exponent := ea + eb;

    mc: u64;
    ec := product_exponent;
    if c != 0 mc, ec = split_float(c);

    base := ifx product_exponent < ec then product_exponent else ec;
    product: Wide;
    addend: Wide;
    add_shifted(*product, high, low, product_exponent - base);
    add_shifted(*addend, 0, mc, ec - base);

    negative, zero := combine(*product, product_negative, *addend, addend_negative);
    if zero return 0; // exact cancellation is +0 when rounding to nearest

    rounded := round_wide(*product, base, negative, format);
    if rounded.infinite return from_bits((ifx negative then SIGN_BIT else 0) | 0x7ff0_0000_0000_0000);
    return compose_float(negative, rounded.low, rounded.exponent);
}

#program_export "fma"
wasi_fma :: (a: float64, b: float64, c: float64) -> float64 #c_call {
    return fused(a, b, c, FLOAT64_FORMAT);
}

// Rounded once, straight to float32 precision: the float64 result is exactly a float32.
#program_export "fmaf"
wasi_fmaf :: (a: float32, b: float32, c: float32) -> float32 #c_call {
    return cast(float32) fused(a, b, c, FLOAT32_FORMAT);
}

// ---------------------------------------------------------------------------------------------
// The other exact functions: wasm has instructions for most, and fmod is exact by nature.

floor_instruction :: (x: float64) -> float64 #intrinsic "llvm.floor.f64";
ceil_instruction :: (x: float64) -> float64 #intrinsic "llvm.ceil.f64";
sqrt_instruction :: (x: float64) -> float64 #intrinsic "llvm.sqrt.f64";

#program_export "floor"
wasi_floor :: (x: float64) -> float64 #c_call {
    return floor_instruction(x);
}

#program_export "floorf"
wasi_floorf :: (x: float32) -> float32 #c_call {
    return cast(float32) floor_instruction(x);
}

#program_export "ceil"
wasi_ceil :: (x: float64) -> float64 #c_call {
    return ceil_instruction(x);
}

#program_export "ceilf"
wasi_ceilf :: (x: float32) -> float32 #c_call {
    return cast(float32) ceil_instruction(x);
}

#program_export "trunc"
wasi_trunc :: (x: float64) -> float64 #c_call {
    return truncate(x);
}

#program_export "truncf"
wasi_truncf :: (x: float32) -> float32 #c_call {
    return cast(float32) truncate(x);
}

#program_export "sqrt"
wasi_sqrt :: (x: float64) -> float64 #c_call {
    return sqrt_instruction(x);
}

// The float64 square root of a float32 rounds to the same float32 as a direct one would.
#program_export "sqrtf"
wasi_sqrtf :: (x: float32) -> float32 #c_call {
    return cast(float32) sqrt_instruction(x);
}

#program_export "fabs"
wasi_fabs :: (x: float64) -> float64 #c_call {
    return from_bits(bits_of(x) & ~SIGN_BIT);
}

#program_export "fabsf"
wasi_fabsf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_fabs(x);
}

#program_export "copysign"
wasi_copysign :: (x: float64, sign: float64) -> float64 #c_call {
    return from_bits((bits_of(x) & ~SIGN_BIT) | (bits_of(sign) & SIGN_BIT));
}

#program_export "copysignf"
wasi_copysignf :: (x: float32, sign: float32) -> float32 #c_call {
    return cast(float32) wasi_copysign(x, sign);
}

// A NaN argument gives the other one; of two zeros, fmin takes -0 and fmax +0.
#program_export "fmin"
wasi_fmin :: (a: float64, b: float64) -> float64 #c_call {
    if a != a return b;
    if b != b return a;
    if a == b return from_bits(bits_of(a) | bits_of(b));
    return ifx a < b then a else b;
}

#program_export "fminf"
wasi_fminf :: (a: float32, b: float32) -> float32 #c_call {
    return cast(float32) wasi_fmin(a, b);
}

#program_export "fmax"
wasi_fmax :: (a: float64, b: float64) -> float64 #c_call {
    if a != a return b;
    if b != b return a;
    if a == b return from_bits(bits_of(a) & bits_of(b));
    return ifx a > b then a else b;
}

#program_export "fmaxf"
wasi_fmaxf :: (a: float32, b: float32) -> float32 #c_call {
    return cast(float32) wasi_fmax(a, b);
}

// x - n*y for the n that truncates x/y, which is always exact: the significands' remainder is
// carried down one exponent step at a time.
#program_export "fmod"
wasi_fmod :: (x: float64, y: float64) -> float64 #c_call {
    if !is_finite(x) || y != y || y == 0 return (x * y) / (x * y); // NaN
    if !is_finite(y) || x == 0 return x;
    negative := (bits_of(x) & SIGN_BIT) != 0;
    mx, ex := split_float(x);
    my, ey := split_float(y);
    while mx < IMPLICIT_BIT {
        mx <<= 1;
        ex -= 1;
    }
    while my < IMPLICIT_BIT {
        my <<= 1;
        ey -= 1;
    }
    if ex < ey || (ex == ey && mx < my) return x; // |x| < |y|
    remainder := mx % my;
    for 1..ex - ey {
        remainder <<= 1;
        if remainder >= my remainder -= my;
    }
    if remainder == 0 return from_bits(ifx negative then SIGN_BIT else 0);
    value: Wide;
    add_shifted(*value, 0, remainder, 0);
    rounded := round_wide(*value, ey, negative, FLOAT64_FORMAT);
    return compose_float(negative, rounded.low, rounded.exponent);
}

#program_export "fmodf"
wasi_fmodf :: (x: float32, y: float32) -> float32 #c_call {
    return cast(float32) wasi_fmod(x, y);
}
