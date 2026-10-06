// Math scalars at run time: the elementary functions against known values (relative
// tolerance), their float32 overloads, rounding, IEEE special values (NaN, infinities,
// negative zero, subnormals) and the small generic helpers.
#import "Basic";
#import "Math";

near :: (actual: float64, expected: float64, tolerance := 1e-13) -> bool {
    return abs(actual - expected) <= tolerance * abs(expected) + 1e-300;
}
near32 :: (actual: float32, expected: float64, tolerance := 1e-6) -> bool {
    return abs(cast(float64) actual - expected) <= tolerance * abs(expected) + 1e-30;
}
bits_of :: (x: float64) -> u64 {
    return (cast(*u64) *x).*;
}
is_negative_zero :: (x: float64) -> bool {
    return bits_of(x) == 0x8000_0000_0000_0000;
}

main :: () {
    NEG_ZERO: float64 = -0.0;
    INF := FLOAT64_INFINITY;
    NAN := FLOAT64_NAN;

    // Classification.
    nan_flag, inf_flag := is_nan_is_inf(NAN);
    assert(nan_flag && !inf_flag);
    nan_flag, inf_flag = is_nan_is_inf(-INF);
    assert(!nan_flag && inf_flag);
    nan_flag, inf_flag = is_nan_is_inf(cast(float64) 1.0);
    assert(!nan_flag && !inf_flag);
    nan_flag, inf_flag = is_nan_is_inf(FLOAT32_NAN);
    assert(nan_flag && !inf_flag);
    nan_flag, inf_flag = is_nan_is_inf(-FLOAT32_INFINITY);
    assert(!nan_flag && inf_flag);
    assert(isnan(NAN) && !isnan(INF) && isnan(FLOAT32_NAN) && !isnan(cast(float32) 0));
    assert(isinf(-INF) && !isinf(NAN) && isinf(FLOAT32_INFINITY) && !isinf(FLOAT32_MAX));
    assert(is_finite(FLOAT64_MAX) && !is_finite(INF) && !is_finite(NAN) && is_finite(FLOAT32_MIN) && !is_finite(-FLOAT32_INFINITY));
    assert(abs(NEG_ZERO) == 0 && bits_of(abs(NEG_ZERO)) == 0, "abs clears the sign of -0");
    assert(abs(cast(float32) -2.5) == 2.5 && abs(-INF) == INF && abs(cast(s32) -5) == 5 && abs(-9) == 9);

    // Constants are the IEEE limits.
    assert(float64_bits(FLOAT64_MAX) == 0x7fef_ffff_ffff_ffff && float32_bits(FLOAT32_MAX) == 0x7f7f_ffff);
    assert(float64_bits(FLOAT64_MIN) == 0x0010_0000_0000_0000 && float32_bits(FLOAT32_MIN) == 0x0080_0000);
    assert(near(PI64, 3.141592653589793, 1e-16) && near(TAU64, 2 * PI64, 1e-16) && near32(PI, PI64) && near32(TAU, TAU64));
    assert(S8_MIN == -128 && U16_MAX == 65535 && S32_MAX == 2147483647 && U64_MAX == 0xffff_ffff_ffff_ffff && S64_MIN == -9223372036854775807 - 1);
    assert(float64_from_bits(float64_bits(1.5)) == 1.5 && float32_from_bits(0x3f80_0000) == 1);

    // floor / ceil / round / frac.
    assert(floor(cast(float64) 2.7) == 2 && floor(cast(float64) -2.7) == -3 && floor(cast(float64) -3) == -3);
    assert(ceil(cast(float64) 2.1) == 3 && ceil(cast(float64) -2.1) == -2 && ceil(cast(float64) 4) == 4);
    assert(is_negative_zero(ceil(cast(float64) -0.5)), "ceil(-0.5) is -0");
    assert(is_negative_zero(floor(NEG_ZERO)) && floor(INF) == INF && isnan(ceil(NAN)));
    assert(floor(cast(float64) 4503599627370497.0) == 4503599627370497.0, "integers above 2^52 are their own floor");
    assert(round(cast(float64) 2.5) == 3 && round(cast(float64) -2.5) == -3 && round(cast(float64) 2.4999) == 2);
    assert(round(cast(float64) 0.49999999999999994) == 0, "the largest double below 0.5 rounds down: %", round(cast(float64) 0.49999999999999994));
    assert(round(cast(float64) 4503599627370497.0) == 4503599627370497.0, "an odd integer above 2^52 rounds to itself: %", round(cast(float64) 4503599627370497.0));
    assert(round(cast(float64) -4503599627370497.0) == -4503599627370497.0);
    assert(is_negative_zero(round(cast(float64) -0.3)), "round(-0.3) is -0");
    assert(round(INF) == INF && isnan(round(NAN)) && is_negative_zero(round(NEG_ZERO)));
    assert(floor(cast(float32) -1.5) == -2 && ceil(cast(float32) 1.25) == 2 && round(cast(float32) 1.5) == 2 && round(cast(float32) 0.49999997) == 0);
    assert(frac(cast(float64) 3.75) == 0.75 && frac(cast(float64) -3.75) == 0.25 && frac(cast(float32) 2.5) == 0.5);

    // fmod_cycling: the result takes the sign of the divisor.
    assert(fmod_cycling(cast(float64) 7.5, 2) == 1.5 && fmod_cycling(cast(float64) -7.5, 2) == 0.5);
    assert(fmod_cycling(cast(float64) 7.5, -2) == -0.5 && fmod_cycling(cast(float64) -7.5, -2) == -1.5);
    assert(fmod_cycling(cast(float64) 6, 2) == 0 && fmod_cycling(cast(float64) 3, INF) == 3);
    assert(isinf(fmod_cycling(cast(float64) 5, 0)) && isnan(fmod_cycling(INF, 2)) && isnan(fmod_cycling(cast(float64) 1, NAN)));
    assert(near(fmod_cycling(cast(float64) 1e10, PI64), 1e10 - floor(1e10 / PI64) * PI64, 1e-5));
    assert(fmod_cycling(cast(float32) 5.5, 2) == 1.5);

    // frexp / ldexp, including subnormals and the extremes.
    mantissa, power := frexp(cast(float64) 8);
    assert(mantissa == 0.5 && power == 4);
    mantissa, power = frexp(cast(float64) -0.375);
    assert(mantissa == -0.75 && power == -1);
    mantissa, power = frexp(float64_from_bits(1));
    assert(mantissa == 0.5 && power == -1073, "frexp of the smallest subnormal: % %", mantissa, power);
    mantissa, power = frexp(INF);
    assert(mantissa == INF && power == 0);
    assert(ldexp(0.5, 4) == 8 && ldexp(0.5, -1073) == float64_from_bits(1) && ldexp(1, 1023) == 8.98846567431158e307);
    assert(isinf(ldexp(1, 1024)) && ldexp(1, -1080) == 0 && ldexp(-3, 2) == -12 && is_negative_zero(ldexp(NEG_ZERO, 5)));
    for i: -60..60 {
        x := 1.0 + cast(float64) i * 0.0371;
        m, p := frexp(x);
        assert(abs(m) >= 0.5 && abs(m) < 1 && ldexp(m, p) == x, "frexp/ldexp round trip for %", x);
    }

    // sqrt.
    assert(sqrt(cast(float64) 144) == 12 && sqrt(cast(float64) 0.25) == 0.5 && sqrt(cast(float64) 1e-300) == 1e-150);
    assert(near(sqrt(cast(float64) 2), 1.4142135623730951, 2e-16) && near(sqrt(cast(float64) 1e300), 1e150, 2e-16));
    assert(isnan(sqrt(cast(float64) -4)) && sqrt(INF) == INF && is_negative_zero(sqrt(NEG_ZERO)) && isnan(sqrt(NAN)));
    assert(near(sqrt(float64_from_bits(1)), 2.2227587494850775e-162, 1e-15), "sqrt of a subnormal");
    assert(sqrt(cast(float32) 9) == 3 && near32(sqrt(cast(float32) 2), 1.4142135623730951));
    // sqrt is correctly rounded: bit-exact against a reference over normal and subnormal inputs.
    for SQRT_BITS {
        root := sqrt(float64_from_bits(it[0]));
        assert(float64_bits(root) == it[1], "sqrt of bits % gave bits %, expected %", it[0], float64_bits(root), it[1]);
    }
    for i: 1..1000 {
        n := cast(float64) i;
        assert(sqrt(n * n) == n, "sqrt(%^2)", n);
    }

    // exp and log.
    assert(exp(cast(float64) 0) == 1 && near(exp(cast(float64) -1), 0.36787944117144233) && near(exp(cast(float64) 10), 22026.465794806718));
    assert(near(exp(cast(float64) 700), 1.0142320547350045e304, 1e-12) && near(exp(cast(float64) -700), 9.85967654375977e-305, 1e-12));
    assert(exp(-INF) == 0 && exp(INF) == INF && isnan(exp(NAN)));
    assert(log(cast(float64) 1) == 0 && near(log(cast(float64) 10), 2.302585092994046) && near(log(cast(float64) 1e-300), -690.7755278982137));
    assert(near(log(FLOAT64_MAX), 709.782712893384) && near(log(float64_from_bits(1)), -744.4400719213812));
    assert(log(INF) == INF && log(cast(float64) 0) == -INF && isnan(log(cast(float64) -1)) && isnan(log(NAN)));
    assert(near32(exp(cast(float32) 1), 2.718281828459045) && near32(log(cast(float32) 2), 0.6931471805599453));

    // pow, including the special cases.
    assert(pow(cast(float64) 2, 10) == 1024 && near(pow(cast(float64) 2, 0.5), 1.4142135623730951) && near(pow(cast(float64) 10, -3), 0.001));
    assert(near(pow(cast(float64) -2, -3), -0.125) && near(pow(cast(float64) 2, 1023), 8.98846567431158e307, 1e-12));
    assert(pow(NAN, 0) == 1 && pow(cast(float64) 1, NAN) == 1 && isnan(pow(cast(float64) 2, NAN)) && isnan(pow(cast(float64) -8, 1.0 / 3.0)));
    assert(pow(cast(float64) 0, -1) == INF && pow(NEG_ZERO, -1) == -INF && pow(NEG_ZERO, -2) == INF && is_negative_zero(pow(NEG_ZERO, 3)));
    assert(pow(cast(float64) 0.5, INF) == 0 && pow(cast(float64) 2, INF) == INF && pow(cast(float64) 2, -INF) == 0 && pow(cast(float64) -1, INF) == 1);
    assert(pow(INF, -1) == 0 && pow(-INF, 3) == -INF && pow(-INF, 2) == INF);
    assert(near32(pow(cast(float32) 3, 4), 81) && near32(pow(cast(float32) 2, -1), 0.5));
    assert(pow(cast(float64) 3, 4) == 81 && pow(cast(float64) 10, 15) == 1e15 && pow(cast(float64) 0.5, -3) == 8 && pow(cast(float32) 2, 20) == 1048576);
    assert(pow(cast(float64) 10, -320) > 0 && pow(cast(float64) 10, -320) < 1.1e-320 && isinf(pow(cast(float64) 10, 400)) && pow(cast(float64) 10, -400) == 0);
    // Whole exponents give the nearest double (reference values computed exactly).
    for POW_CASES {
        result := pow(float64_from_bits(it.base), cast(float64) it.power);
        assert(float64_bits(result) == it.bits, "pow(%, %) = %, expected %", float64_from_bits(it.base), it.power, result, float64_from_bits(it.bits));
    }

    // Trigonometry.
    assert(near(sin(cast(float64) 1), 0.8414709848078965) && near(cos(cast(float64) 1), 0.5403023058681398));
    assert(near(tan(cast(float64) 1), 1.5574077246549023) && near(cot(cast(float64) 1), 0.6420926159343306));
    assert(near(sin(cast(float64) 100), -0.5063656411097588, 1e-12) && near(cos(cast(float64) 100), 0.8623188722876839, 1e-12));
    assert(near(sin(cast(float64) -3), -0.1411200080598672, 1e-12) && near(cos(cast(float64) -3), -0.9899924966004454, 1e-12));
    assert(abs(sin(PI64)) < 1e-15 && near(cos(PI64), -1) && abs(cos(PI64 / 2)) < 1e-15 && near(sin(PI64 / 2), 1));
    assert(isnan(sin(INF)) && isnan(cos(-INF)) && isnan(tan(NAN)) && is_negative_zero(sin(NEG_ZERO)) && cos(NEG_ZERO) == 1);
    assert(sin_or_cos(cast(float64) 1, true) == sin(cast(float64) 1) && sin_or_cos(cast(float64) 1, false) == cos(cast(float64) 1));
    assert(near32(sin(cast(float32) 0.5), 0.479425538604203) && near32(cos(cast(float32) 0.5), 0.8775825618903728));
    assert(near32(tan(cast(float32) 0.5), 0.5463024898437905) && near32(cot(cast(float32) 0.5), 1.830487721712452));
    for i: -50..50 {
        x := cast(float64) i * 0.731;
        s := sin(x);
        c := cos(x);
        assert(near(s * s + c * c, 1, 1e-15), "sin^2 + cos^2 at %", x);
        assert(abs(sin(-x) + s) <= 1e-15 && abs(cos(-x) - c) <= 1e-15, "symmetry at %", x);
    }

    assert(near(atan(cast(float64) 1), PI64 / 4) && near(atan(cast(float64) -0.5), -0.4636476090008061) && near(atan(cast(float64) 1e10), 1.5707963266948965));
    assert(near(atan(INF), PI64 / 2) && near(atan(-INF), -PI64 / 2) && isnan(atan(NAN)) && is_negative_zero(atan(NEG_ZERO)));
    assert(near(atan2(cast(float64) 1, 1), PI64 / 4) && near(atan2(cast(float64) -1, -1), -3 * PI64 / 4) && near(atan2(cast(float64) 1, -1), 3 * PI64 / 4));
    assert(near(atan2(cast(float64) 1, 0), PI64 / 2) && near(atan2(cast(float64) -1, 0), -PI64 / 2));
    assert(atan2(cast(float64) 0, -1) == PI64 && atan2(NEG_ZERO, -1) == -PI64 && atan2(cast(float64) 0, 1) == 0 && is_negative_zero(atan2(NEG_ZERO, 1)));
    assert(atan2(cast(float64) 0, NEG_ZERO) == PI64 && atan2(cast(float64) 0, 0) == 0);
    assert(near(atan2(INF, INF), PI64 / 4) && near(atan2(-INF, -INF), -3 * PI64 / 4) && isnan(atan2(NAN, 1)));
    assert(near32(atan2(cast(float32) 1, cast(float32) 1), PI64 / 4) && near32(atan(cast(float32) 2), 1.1071487177940904));
    assert(near(asin(cast(float64) 1), PI64 / 2) && near(asin(cast(float64) -0.5), -PI64 / 6) && near(acos(cast(float64) 1) + 1, 1) && near(acos(cast(float64) 0), PI64 / 2));
    assert(isnan(acos(cast(float64) 1.0001)) && isnan(asin(cast(float64) -2)));
    assert(near32(asin(cast(float32) 0.5), PI64 / 6) && near32(acos(cast(float32) 0.5), PI64 / 3));
    for i: -20..20 {
        x := cast(float64) i * 0.05;
        assert(near(sin(asin(x)), x, 1e-14) || abs(x) < 1e-15, "sin(asin(%))", x);
        assert(near(cos(acos(x)), x, 1e-13) || abs(x) < 1e-15, "cos(acos(%))", x);
        assert(near(tan(atan(x * 20)), x * 20, 1e-13) || x == 0, "tan(atan(%))", x * 20);
    }

    // Half floats.
    assert(half_to_float(0x3c00) == 1 && half_to_float(0xc000) == -2 && half_to_float(0x7bff) == 65504);
    assert(half_to_float(0x0001) == 0.000000059604644775390625 && half_to_float(0x03ff) == 0.00006097555160522461);
    assert(half_to_float(0x7c00) == FLOAT32_INFINITY && isnan(half_to_float(0x7e00)) && float32_bits(half_to_float(0x8000)) == 0x8000_0000);
    assert(half_to_float(0x3555) == 0.333251953125);

    // Generic helpers.
    assert(square(cast(float) 3) == 9 && square(-4) == 16);
    assert(saturate(cast(float) 1.5) == 1 && saturate(cast(float) -0.5) == 0 && saturate(cast(float64) 0.25) == 0.25 && saturate(cast(float64) -3) == 0);
    assert(min(3, 1, 2) == 1 && max(3, 1, 2, 9, -5) == 9 && min(2.5, -1.5) == -1.5 && max(cast(u8) 7, cast(u8) 200) == 200);
    assert(clamp(5, 0, 3) == 3 && clamp(-5, 0, 3) == 0 && clamp(2, 0, 3) == 2);
    value := 12.5;
    Clamp(*value, 0, 10);
    assert(value == 10);
    count := -3;
    Clamp(*count, 0, 100);
    assert(count == 0);
    a, b := 1, 2;
    Swap(*a, *b);
    assert(a == 2 && b == 1 && Min(4, 3) == 3 && Max(4, 3) == 4);
    assert(lerp(cast(float) 2, cast(float) 6, 0.25) == 3 && lerp(cast(float64) -1, 1, 0.5) == 0 && lerp(10, 20, 0.5) == 15);
    assert(lerp(cast(u8) 0, cast(u8) 200, 0.25) == 50);

    print("ok\n");
}

// [input bits, correctly rounded square root bits]
SQRT_BITS :: [2] u64.[
    .[0x00097b753ceb3ffe, 0x1ff8a276709d4509],
    .[0x000216368b529b4b, 0x1fe71cf0d221f066],
    .[0x000ea7b55eb561a5, 0x1ffea0270c23108e],
    .[0x000795b99a9a80fe, 0x1ff6085236bed874],
    .[0x00094b2ba02f34a7, 0x1ff8636c3b801eff],
    .[0x0009b08910c67fda, 0x1ff8e7096167f72b],
    .[0x000e8a85035efa26, 0x1ffe81981a7200f9],
    .[0x000781f9d6645faa, 0x1ff5eb90743dfacc],
    .[0x46901c7642650644, 0x43400e34d2a30284],
    .[0x189b12133bfd1d33, 0x2c44cfd00a8ae6cc],
    .[0x5bdb81c3fee29476, 0x4de4fa931b67161f],
    .[0x454ea1da78633074, 0x429f4f03cd387ee9],
    .[0x466a506bd6225675, 0x432d04a50e17fb16],
    .[0x32e54e4179f248b0, 0x396a1c678bf51b50],
    .[0x6e45f8f0a399f82a, 0x571a84305b4e8835],
    .[0x1dbf9ec3268ecc45, 0x2ed67e232d2c5b75],
    .[0x13785ca2a2863a7f, 0x29b3be3b513687ee],
    .[0x7691c6dade383784, 0x5b40dd710c15d10c],
    .[0x31f9724885ef3430, 0x38f42d82d3f084ea],
    .[0x02005409bdc2ae99, 0x20f6dbbdac83bbaa],
    .[0x638c6d1fabe19f58, 0x51be2905e8ec9eb7],
    .[0x1477379210645d51, 0x2a33460d348c7d75],
    .[0x7a9f4391c21b6092, 0x5d465d9c63ad1af3],
    .[0x058a1c6997524d6a, 0x22bce7e9a20bfe6c],
    .[0x63e98bec4d1fe09f, 0x51ec977f76f7a7a1],
    .[0x697ac22107f062ce, 0x54b4b0fe0dfdd5aa],
    .[0x228cbca6dd933160, 0x313e53191b563b54],
    .[0x4c30c08b79061596, 0x46105f2ab6ad61bd],
    .[0x75d7b127b804d820, 0x5ae3784345d1f981],
    .[0x31ad2877e0f9e038, 0x38ce8bc71ae2e9d3],
    .[0x64f0c038b6d13089, 0x52705f0245e95583],
    .[0x36b5cd6debcd1f5e, 0x3b52ad5facb646d8],
    .[0x5d433b59651c5253, 0x4e98cec10789ea7a],
    .[0x49e82d02cd085b71, 0x44ebd06b75b73941],
    .[0x7b776c8571d2af72, 0x5db35bfb39922fc7],
    .[0x113bcc4fef829c88, 0x289516ed5d558cff],
    .[0x2ed95921e0fd67dd, 0x3764238797b6a13b],
    .[0x04a7eeef18f2c41c, 0x224bac9ec439d9f1],
    .[0x3f6856fa22cedafb, 0x3fabe88517047cc0],
    .[0x2115875f378c74dc, 0x30828f457767573f],
    .[0x56157271f729b4c8, 0x4b02863e1127d063],
    .[0x63c55fa16fa84dca, 0x51da270af445a013],
    .[0x6d88082da06c05a1, 0x56bbbb3342f3cadf],
    .[0x35f831c54d100d8f, 0x3af3acd5336e0fb2],
    .[0x6abf60d281daad10, 0x555668109ae173d7],
    .[0x498993bd62c82185, 0x44bc9bdef0904646],
    .[0x446d7d6c59d54505, 0x422eb826b9c34556],
    .[0x343b722d95c76ab4, 0x3a14f4a0926a81df],
];

Pow_Case :: struct {
    base: u64;
    power: s64;
    bits: u64;
}
// [base bits, whole exponent, bits of the nearest double to the exact power]
POW_CASES :: Pow_Case.[
    .{ 0x4024000000000000, -3, 0x3f50624dd2f1a9fc },
    .{ 0x3fee03ef7c25d185, 7, 0x3fe470f2588abd49 },
    .{ 0x402c000000000000, -22, 0x3ab2df495ccaa571 },
    .{ 0x3ff0fda92a5b2ccb, 21, 0x400c41435176f786 },
    .{ 0x3ff17cbac5619631, 34, 0x4034883077dd9b73 },
    .{ 0x3ff064dde867efa3, -52, 0x3fd2101a297c0682 },
    .{ 0x3ff136f08b4bf998, -57, 0x3f8fa00826f6e617 },
    .{ 0x402f2b30e0f54454, 6, 0x416b5340c41daf36 },
    .{ 0x3fefffa0da67573f, -50, 0x3ff0094d6cf5a2ef },
    .{ 0x3fef67442b32b852, 59, 0x3fd5155d21d10c8b },
    .{ 0x4030d1fe7d082f18, 5, 0x41348b4afb47f467 },
    .{ 0x4017685726722f1e, -9, 0x3e80ad8a69bc5ced },
    .{ 0x4001f6564cf26f83, 48, 0x43701f71b8c08d9e },
    .{ 0x402b757bbfa0b1ab, 7, 0x4195ec78de798f08 },
    .{ 0x401e497069412b95, 12, 0x422088f350925529 },
    .{ 0x402c000000000000, -49, 0x3445b32515f5c18b },
    .{ 0x40192c1af7425cb1, -43, 0x38cd9cca37a4e8d7 },
    .{ 0x4008000000000000, 42, 0x4417b9f95b8dc87d },
    .{ 0x4028000000000000, 5, 0x410e600000000000 },
    .{ 0x402a000000000000, 22, 0x45054111135e81fb },
    .{ 0x401ef25726e8c4e4, -60, 0x34ddcd5505753605 },
    .{ 0x4015e02c9399a1a1, -34, 0x3ab93be76a7d1574 },
    .{ 0x3ff05392d441b956, -48, 0x3fd845e0f46a302c },
    .{ 0x4010000000000000, -59, 0x3890000000000000 },
    .{ 0x401c000000000000, -51, 0x36fc57b30d024b28 },
    .{ 0x3ff94045fffc95f9, -59, 0x3d81e6823ed42ad4 },
    .{ 0x4028218f92425840, 15, 0x434db2d8adfa51a1 },
    .{ 0x4026000000000000, -37, 0x37f002ecf83a043d },
    .{ 0x3ff14e94aa1762ef, -29, 0x3fba42f196c8dd2f },
    .{ 0x3ff0c422ef0d3510, -35, 0x3fc8e6ec096f46a7 },
    .{ 0x3ff13917003168e0, -50, 0x3f99c11a72055e1c },
    .{ 0x3fed7f7689a18669, -28, 0x40238c00bff7773a },
    .{ 0x402a000000000000, 45, 0x4a56f09d69183da3 },
    .{ 0x40321b22bcb8a137, 58, 0x4f14599c49d90cf1 },
    .{ 0x4008000000000000, 11, 0x41059fd800000000 },
    .{ 0x4026000000000000, -33, 0x38cc9dba95037064 },
    .{ 0x40325affebe6a38a, -26, 0x391cceed5a982746 },
    .{ 0x4020932498f7a68a, -24, 0x3b5b6f84d143f2d2 },
    .{ 0x401dc15c5afbc5bb, -14, 0x3d662540b42d9d35 },
    .{ 0x3ff1899ec621e40c, 33, 0x4034a7d80b0ca12d },
    .{ 0x3ff16e102474c585, -56, 0x3f80f660c0c4cf75 },
    .{ 0x3ff1500bc957ce1e, -21, 0x3fc870533eb960e6 },
    .{ 0x4033425734f55c05, 1, 0x4033425734f55c05 },
    .{ 0x3ff704d9fa12e5d2, -51, 0x3e42ddc6809c50e3 },
    .{ 0x3ff0c227a79e2739, 34, 0x401350bb8eafa225 },
    .{ 0x402c000000000000, -15, 0x3c5da4e7e1af48bd },
    .{ 0x4022000000000000, 60, 0x4bd2526a6da66fc5 },
    .{ 0x402fa9c5bf2026c3, 19, 0x44aa2bffd55a2396 },
];
