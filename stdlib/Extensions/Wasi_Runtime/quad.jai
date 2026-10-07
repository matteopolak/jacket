// IEEE binary128 arithmetic: the soft-float helpers LLVM calls for `fp128` (Extensions/Long_Double's
// Long_Double) on wasm, which has no 128-bit float instructions. WASI builds link no
// compiler-rt, so they are here, under compiler-rt's names and wasm calling convention: a
// binary128 argument is two i64s (low half first) and a binary128 result is written through a
// pointer passed first. Every result is rounded once, to nearest with ties to even, through the
// accumulator `fma` uses (math.jai).

#scope_module

Quad :: struct {
    low: u64;
    high: u64;
}

QUAD_FORMAT :: Fused_Format.{ 113, -16494, 16383 };
QUAD_FRACTION_HIGH: u64 : 0x0000_ffff_ffff_ffff;
QUAD_IMPLICIT_HIGH: u64 : 0x0001_0000_0000_0000;
QUAD_INFINITY_HIGH: u64 : 0x7fff_0000_0000_0000;
QUAD_QUIET_BIT: u64 : 0x0000_8000_0000_0000;
QUAD_DEFAULT_NAN :: Quad.{ 0, 0x7fff_8000_0000_0000 };

Quad_Kind :: enum u8 {
    ZERO;
    FINITE;
    INFINITE;
    NAN;
}

// A binary128 taken apart: a finite one is significand * 2^exponent.
Quad_Parts :: struct {
    kind: Quad_Kind;
    negative: bool;
    high: u64;
    low: u64;
    exponent: s64;
}

quad_parts :: (low: u64, high: u64) -> Quad_Parts #no_context {
    parts: Quad_Parts;
    parts.negative = (high & SIGN_BIT) != 0;
    biased := cast(s64) ((high >> 48) & 0x7fff);
    fraction := high & QUAD_FRACTION_HIGH;
    if biased == 0x7fff {
        parts.kind = ifx fraction != 0 || low != 0 then .NAN else .INFINITE;
        return parts;
    }
    parts.low = low;
    if biased == 0 {
        if fraction == 0 && low == 0 return parts;
        parts.kind = .FINITE;
        parts.high = fraction;
        parts.exponent = QUAD_FORMAT.lowest_exponent;
        return parts;
    }
    parts.kind = .FINITE;
    parts.high = fraction | QUAD_IMPLICIT_HIGH;
    parts.exponent = biased - 16495;
    return parts;
}

// The binary128 holding a rounded value.
quad_from :: (rounded: Rounded) -> Quad #no_context {
    sign := ifx rounded.negative then SIGN_BIT else 0;
    if rounded.infinite return .{ 0, sign | QUAD_INFINITY_HIGH };
    high := rounded.high;
    low := rounded.low;
    exponent := rounded.exponent;
    length := bit_length(high, low);
    if length == 0 return .{ 0, sign };
    // Normalize to 113 bits unless that would go below the smallest exponent (subnormal).
    shift := QUAD_FORMAT.precision - length;
    room := exponent - QUAD_FORMAT.lowest_exponent;
    if shift > room shift = room;
    if shift > 0 {
        high, low = shift_left(high, low, shift);
        exponent -= shift;
    }
    if bit_length(high, low) < QUAD_FORMAT.precision return .{ low, sign | high };
    return .{ low, sign | ((cast(u64) (exponent + 16495)) << 48) | (high & QUAD_FRACTION_HIGH) };
}

quiet :: (low: u64, high: u64) -> Quad #no_context {
    return .{ low, high | QUAD_QUIET_BIT };
}

// Rounds the exact value (high:low) * 2^exponent, which needs at most 128 bits, to binary128.
quad_from_exact :: (negative: bool, high: u64, low: u64, exponent: s64) -> Quad #no_context {
    value: Wide;
    add_shifted(*value, high, low, 0);
    return quad_from(round_wide(*value, exponent, negative, QUAD_FORMAT));
}

quad_add :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> Quad #no_context {
    a := quad_parts(a_low, a_high);
    b := quad_parts(b_low, b_high);
    if a.kind == .NAN return quiet(a_low, a_high);
    if b.kind == .NAN return quiet(b_low, b_high);
    if a.kind == .INFINITE {
        if b.kind == .INFINITE && a.negative != b.negative return QUAD_DEFAULT_NAN;
        return .{ a_low, a_high };
    }
    if b.kind == .INFINITE return .{ b_low, b_high };
    if a.kind == .ZERO {
        // -0 + -0 is -0; +0 + -0 is +0.
        if b.kind == .ZERO return .{ 0, ifx a.negative && b.negative then SIGN_BIT else 0 };
        return .{ b_low, b_high };
    }
    if b.kind == .ZERO return .{ a_low, a_high };

    // An addend more than 116 places below the other's top bit only decides rounding as a
    // nonzero sticky amount: any value below 2^(top - 115) rounds the same, so use 2^(top - 118)
    // and keep the accumulator short.
    a_top := a.exponent + bit_length(a.high, a.low) - 1;
    b_top := b.exponent + bit_length(b.high, b.low) - 1;
    if b_top < a_top - 116 {
        b.high = 0;
        b.low = 1;
        b.exponent = a_top - 118;
    } else if a_top < b_top - 116 {
        a.high = 0;
        a.low = 1;
        a.exponent = b_top - 118;
    }
    base := ifx a.exponent < b.exponent then a.exponent else b.exponent;
    left: Wide;
    right: Wide;
    add_shifted(*left, a.high, a.low, a.exponent - base);
    add_shifted(*right, b.high, b.low, b.exponent - base);
    negative, zero := combine(*left, a.negative, *right, b.negative);
    if zero return .{ 0, 0 };
    return quad_from(round_wide(*left, base, negative, QUAD_FORMAT));
}

quad_multiply :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> Quad #no_context {
    a := quad_parts(a_low, a_high);
    b := quad_parts(b_low, b_high);
    if a.kind == .NAN return quiet(a_low, a_high);
    if b.kind == .NAN return quiet(b_low, b_high);
    negative := a.negative != b.negative;
    sign := ifx negative then SIGN_BIT else 0;
    if a.kind == .INFINITE || b.kind == .INFINITE {
        if a.kind == .ZERO || b.kind == .ZERO return QUAD_DEFAULT_NAN;
        return .{ 0, sign | QUAD_INFINITY_HIGH };
    }
    if a.kind == .ZERO || b.kind == .ZERO return .{ 0, sign };
    // (a.high:a.low) * (b.high:b.low) as four 64-bit products, each a 128-bit term.
    product: Wide;
    high, low := multiply_wide(a.low, b.low);
    add_shifted(*product, high, low, 0);
    high, low = multiply_wide(a.high, b.low);
    add_shifted(*product, high, low, 64);
    high, low = multiply_wide(a.low, b.high);
    add_shifted(*product, high, low, 64);
    high, low = multiply_wide(a.high, b.high);
    add_shifted(*product, high, low, 128);
    return quad_from(round_wide(*product, a.exponent + b.exponent, negative, QUAD_FORMAT));
}

quad_divide :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> Quad #no_context {
    a := quad_parts(a_low, a_high);
    b := quad_parts(b_low, b_high);
    if a.kind == .NAN return quiet(a_low, a_high);
    if b.kind == .NAN return quiet(b_low, b_high);
    negative := a.negative != b.negative;
    sign := ifx negative then SIGN_BIT else 0;
    if a.kind == .INFINITE {
        if b.kind == .INFINITE return QUAD_DEFAULT_NAN;
        return .{ 0, sign | QUAD_INFINITY_HIGH };
    }
    if b.kind == .INFINITE return .{ 0, sign };
    if b.kind == .ZERO {
        if a.kind == .ZERO return QUAD_DEFAULT_NAN;
        return .{ 0, sign | QUAD_INFINITY_HIGH };
    }
    if a.kind == .ZERO return .{ 0, sign };
    // Both significands to exactly 113 bits, then 116 quotient bits by long division:
    // quotient = floor(dividend * 2^115 / divisor), between 2^114 and 2^116.
    a_shift := QUAD_FORMAT.precision - bit_length(a.high, a.low);
    a.high, a.low = shift_left(a.high, a.low, a_shift);
    b_shift := QUAD_FORMAT.precision - bit_length(b.high, b.low);
    b.high, b.low = shift_left(b.high, b.low, b_shift);
    remainder_high, remainder_low := a.high, a.low;
    quotient_high, quotient_low: u64;
    for 1..116 {
        quotient_high, quotient_low = shift_left(quotient_high, quotient_low, 1);
        if remainder_high > b.high || (remainder_high == b.high && remainder_low >= b.low) {
            borrow: u64 = ifx remainder_low < b.low then 1 else 0;
            remainder_low -= b.low;
            remainder_high = remainder_high - b.high - borrow;
            quotient_low |= 1;
        }
        remainder_high, remainder_low = shift_left(remainder_high, remainder_low, 1);
    }
    // A nonzero remainder becomes one more low bit, below every bit that decides rounding.
    quotient_high, quotient_low = shift_left(quotient_high, quotient_low, 1);
    if remainder_high != 0 || remainder_low != 0 quotient_low |= 1;
    exponent := (a.exponent - a_shift) - (b.exponent - b_shift) - 116;
    return quad_from_exact(negative, quotient_high, quotient_low, exponent);
}

// -1, 0 or 1 as a < b, a == b, a > b; 2 when either is NaN.
quad_compare :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> s32 #no_context {
    a := quad_parts(a_low, a_high);
    b := quad_parts(b_low, b_high);
    if a.kind == .NAN || b.kind == .NAN return 2;
    if a.kind == .ZERO && b.kind == .ZERO return 0;
    if a.negative != b.negative return ifx a.negative then -1 else 1;
    // Same sign: the bit patterns order the magnitudes.
    a_magnitude := a_high & ~SIGN_BIT;
    b_magnitude := b_high & ~SIGN_BIT;
    order: s32;
    if a_magnitude != b_magnitude order = ifx a_magnitude < b_magnitude then -1 else 1;
    else if a_low != b_low order = ifx a_low < b_low then -1 else 1;
    return ifx a.negative then -order else order;
}

// Toward zero, saturating at the type's bounds; NaN is 0.
quad_to_integer :: (low: u64, high: u64, minimum: s64, maximum: u64) -> negative: bool, magnitude: u64 #no_context {
    parts := quad_parts(low, high);
    if parts.kind == .NAN || parts.kind == .ZERO return false, 0;
    limit := ifx parts.negative then cast,no_check(u64) (-minimum) else maximum;
    if parts.kind == .INFINITE return parts.negative, limit;
    if parts.exponent + bit_length(parts.high, parts.low) > 64 return parts.negative, limit;
    value_high, value_low: u64;
    if parts.exponent >= 0 {
        value_high, value_low = shift_left(parts.high, parts.low, parts.exponent);
    } else {
        value_high, value_low = shift_right(parts.high, parts.low, -parts.exponent);
    }
    if value_high != 0 || value_low > limit return parts.negative, limit;
    return parts.negative, value_low;
}

// ---------------------------------------------------------------------------------------------
// The compiler-rt entry points.

#program_export "__addtf3"
wasi_addtf3 :: (result: *Quad, a_low: u64, a_high: u64, b_low: u64, b_high: u64) #c_call {
    result.* = quad_add(a_low, a_high, b_low, b_high);
}

#program_export "__subtf3"
wasi_subtf3 :: (result: *Quad, a_low: u64, a_high: u64, b_low: u64, b_high: u64) #c_call {
    result.* = quad_add(a_low, a_high, b_low, b_high ^ SIGN_BIT);
}

#program_export "__multf3"
wasi_multf3 :: (result: *Quad, a_low: u64, a_high: u64, b_low: u64, b_high: u64) #c_call {
    result.* = quad_multiply(a_low, a_high, b_low, b_high);
}

#program_export "__divtf3"
wasi_divtf3 :: (result: *Quad, a_low: u64, a_high: u64, b_low: u64, b_high: u64) #c_call {
    result.* = quad_divide(a_low, a_high, b_low, b_high);
}

#program_export "__negtf2"
wasi_negtf2 :: (result: *Quad, a_low: u64, a_high: u64) #c_call {
    result.* = .{ a_low, a_high ^ SIGN_BIT };
}

// Comparisons: zero for equal (and for `>=`/`<=` when it holds), negative or positive for the
// order, and NaN reported so that the comparison it stands for comes out false.
#program_export "__eqtf2"
wasi_eqtf2 :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> s32 #c_call {
    order := quad_compare(a_low, a_high, b_low, b_high);
    return ifx order == 2 then 1 else order;
}

#program_export "__netf2"
wasi_netf2 :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> s32 #c_call {
    return wasi_eqtf2(a_low, a_high, b_low, b_high);
}

#program_export "__lttf2"
wasi_lttf2 :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> s32 #c_call {
    return wasi_eqtf2(a_low, a_high, b_low, b_high);
}

#program_export "__letf2"
wasi_letf2 :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> s32 #c_call {
    return wasi_eqtf2(a_low, a_high, b_low, b_high);
}

#program_export "__cmptf2"
wasi_cmptf2 :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> s32 #c_call {
    return wasi_eqtf2(a_low, a_high, b_low, b_high);
}

#program_export "__gttf2"
wasi_gttf2 :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> s32 #c_call {
    order := quad_compare(a_low, a_high, b_low, b_high);
    return ifx order == 2 then -1 else order;
}

#program_export "__getf2"
wasi_getf2 :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> s32 #c_call {
    return wasi_gttf2(a_low, a_high, b_low, b_high);
}

#program_export "__unordtf2"
wasi_unordtf2 :: (a_low: u64, a_high: u64, b_low: u64, b_high: u64) -> s32 #c_call {
    return ifx quad_compare(a_low, a_high, b_low, b_high) == 2 then 1 else 0;
}

// Widening is exact.
#program_export "__extenddftf2"
wasi_extenddftf2 :: (result: *Quad, x: float64) #c_call {
    bits := bits_of(x);
    sign := bits & SIGN_BIT;
    if !is_finite(x) {
        // Infinity, or NaN with its payload moved to the top of the wider fraction.
        fraction := bits & FRACTION_BITS;
        result.* = .{ fraction << 60, sign | QUAD_INFINITY_HIGH | (fraction >> 4) };
        return;
    }
    if (bits & ~SIGN_BIT) == 0 {
        result.* = .{ 0, sign };
        return;
    }
    significand, exponent := split_float(x);
    result.* = quad_from_exact(sign != 0, 0, significand, exponent);
}

#program_export "__extendsftf2"
wasi_extendsftf2 :: (result: *Quad, x: float32) #c_call {
    wasi_extenddftf2(result, x);
}

// Narrowing rounds once, straight to the narrower format.
narrow :: (low: u64, high: u64, format: Fused_Format) -> float64 #no_context {
    parts := quad_parts(low, high);
    sign := ifx parts.negative then SIGN_BIT else 0;
    if parts.kind == {
        case .ZERO; return from_bits(sign);
        case .INFINITE; return from_bits(sign | 0x7ff0_0000_0000_0000);
        case .NAN; return from_bits(sign | 0x7ff8_0000_0000_0000 | ((high & QUAD_FRACTION_HIGH) << 4) | (low >> 60));
    }
    value: Wide;
    add_shifted(*value, parts.high, parts.low, 0);
    rounded := round_wide(*value, parts.exponent, parts.negative, format);
    if rounded.infinite return from_bits(sign | 0x7ff0_0000_0000_0000);
    return compose_float(parts.negative, rounded.low, rounded.exponent);
}

#program_export "__trunctfdf2"
wasi_trunctfdf2 :: (low: u64, high: u64) -> float64 #c_call {
    return narrow(low, high, FLOAT64_FORMAT);
}

// The float64 from `narrow` is exactly a float32 here.
#program_export "__trunctfsf2"
wasi_trunctfsf2 :: (low: u64, high: u64) -> float32 #c_call {
    return cast(float32) narrow(low, high, FLOAT32_FORMAT);
}

#program_export "__fixtfdi"
wasi_fixtfdi :: (low: u64, high: u64) -> s64 #c_call {
    negative, magnitude := quad_to_integer(low, high, S64_MIN, cast(u64) S64_MAX);
    if negative return cast,no_check(s64) (0 - magnitude);
    return cast,no_check(s64) magnitude;
}

#program_export "__fixunstfdi"
wasi_fixunstfdi :: (low: u64, high: u64) -> u64 #c_call {
    negative, magnitude := quad_to_integer(low, high, 0, 0xffff_ffff_ffff_ffff);
    if negative return 0;
    return magnitude;
}

#program_export "__fixtfsi"
wasi_fixtfsi :: (low: u64, high: u64) -> s32 #c_call {
    negative, magnitude := quad_to_integer(low, high, -0x8000_0000, 0x7fff_ffff);
    if negative return cast,no_check(s32) (0 - magnitude);
    return cast,no_check(s32) magnitude;
}

#program_export "__fixunstfsi"
wasi_fixunstfsi :: (low: u64, high: u64) -> u32 #c_call {
    negative, magnitude := quad_to_integer(low, high, 0, 0xffff_ffff);
    if negative return 0;
    return cast,no_check(u32) magnitude;
}

// From integers: exact, as 64 bits fit in 113.
#program_export "__floatditf"
wasi_floatditf :: (result: *Quad, x: s64) #c_call {
    magnitude := ifx x < 0 then 0 - cast,no_check(u64) x else cast(u64) x;
    result.* = quad_from_exact(x < 0, 0, magnitude, 0);
}

#program_export "__floatunditf"
wasi_floatunditf :: (result: *Quad, x: u64) #c_call {
    result.* = quad_from_exact(false, 0, x, 0);
}

#program_export "__floatsitf"
wasi_floatsitf :: (result: *Quad, x: s32) #c_call {
    wasi_floatditf(result, x);
}

#program_export "__floatunsitf"
wasi_floatunsitf :: (result: *Quad, x: u32) #c_call {
    wasi_floatunditf(result, x);
}

S64_MIN :: -0x7fff_ffff_ffff_ffff - 1;
S64_MAX :: 0x7fff_ffff_ffff_ffff;
