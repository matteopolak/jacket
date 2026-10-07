// The transcendental functions of C's math library: sin, cos, tan, exp, exp2, log, log2, log10,
// pow, atan, atan2, asin, acos, and their float32 forms. jaic emits calls to sin and cos for
// `#intrinsic` procedures of those names; the rest are here for programs that declare them
// `#foreign` from libm, which WASI builds do not link.
//
// Each works in float64 pairs (a value and the rounding error under it, about 106 bits) and
// rounds once at the end, so results are within an ulp; sin, cos and tan reduce their argument
// with 2/pi to 1344 bits, so huge arguments are exact too. The float32 forms compute in float64.

#scope_module

// A float64 pair: head + tail, |tail| at most half an ulp of head.
Pair :: struct {
    head: float64;
    tail: float64;
}

ONE_PAIR :: Pair.{ 1, 0 };

// a + b exactly, for any a and b.
two_sum :: (a: float64, b: float64) -> Pair #no_context {
    sum := a + b;
    b_part := sum - a;
    error := (a - (sum - b_part)) + (b - b_part);
    return .{ sum, error };
}

// a + b exactly, for |a| >= |b|.
quick_two_sum :: (a: float64, b: float64) -> Pair #no_context {
    sum := a + b;
    return .{ sum, b - (sum - a) };
}

// a * b exactly (for |a|, |b| below 2^996): each factor is split into 26-bit halves, whose
// products are exact.
two_product :: (a: float64, b: float64) -> Pair #no_context {
    SPLITTER :: 134217729.0; // 2^27 + 1
    product := a * b;
    a_spread := SPLITTER * a;
    a_high := a_spread - (a_spread - a);
    a_low := a - a_high;
    b_spread := SPLITTER * b;
    b_high := b_spread - (b_spread - b);
    b_low := b - b_high;
    error := ((a_high * b_high - product) + a_high * b_low + a_low * b_high) + a_low * b_low;
    return .{ product, error };
}

pair_round :: (a: Pair) -> float64 #no_context {
    return a.head + a.tail;
}

pair_negate :: (a: Pair) -> Pair #no_context {
    return .{ -a.head, -a.tail };
}

pair_add :: (a: Pair, b: Pair) -> Pair #no_context {
    heads := two_sum(a.head, b.head);
    tails := two_sum(a.tail, b.tail);
    sum := quick_two_sum(heads.head, heads.tail + tails.head);
    return quick_two_sum(sum.head, sum.tail + tails.tail);
}

pair_multiply :: (a: Pair, b: Pair) -> Pair #no_context {
    product := two_product(a.head, b.head);
    return quick_two_sum(product.head, product.tail + (a.head * b.tail + a.tail * b.head));
}

pair_multiply_float :: (a: Pair, b: float64) -> Pair #no_context {
    product := two_product(a.head, b);
    return quick_two_sum(product.head, product.tail + a.tail * b);
}

pair_divide :: (a: Pair, b: float64) -> Pair #no_context {
    first := a.head / b;
    product := two_product(first, b);
    second := (((a.head - product.head) - product.tail) + a.tail) / b;
    return quick_two_sum(first, second);
}

// Long division: each step takes the next float64 of the quotient from the remainder.
pair_divide_pair :: (a: Pair, b: Pair) -> Pair #no_context {
    first := a.head / b.head;
    remainder := pair_add(a, pair_negate(pair_multiply_float(b, first)));
    second := remainder.head / b.head;
    remainder = pair_add(remainder, pair_negate(pair_multiply_float(b, second)));
    third := remainder.head / b.head;
    return pair_add(quick_two_sum(first, second), .{ third, 0 });
}

// One Newton step from the float64 root: s + (a - s^2) / 2s.
pair_square_root :: (a: Pair) -> Pair #no_context {
    root := sqrt_instruction(a.head);
    if root == 0 return .{ 0, 0 };
    left := pair_add(a, pair_negate(two_product(root, root)));
    return quick_two_sum(root, left.head / (2 * root));
}

// ---------------------------------------------------------------------------------------------
// sin, cos and tan. x = n * pi/2 + r with |r| <= pi/4: n comes from x * 2/pi formed exactly enough
// with the bits of 2/pi around x's exponent, r as a float64 pair (head and tail).

// 2/pi = 0.a2f9836e 4e441529... in hexadecimal, 32 bits a word.
TWO_OVER_PI :: u32.[
    0xa2f9836e, 0x4e441529, 0xfc2757d1, 0xf534ddc0, 0xdb629599, 0x3c439041, 0xfe5163ab,
    0xdebbc561, 0xb7246e3a, 0x424dd2e0, 0x06492eea, 0x09d1921c, 0xfe1deb1c, 0xb129a73e,
    0xe88235f5, 0x2ebb4484, 0xe99c7026, 0xb45f7e41, 0x3991d639, 0x835339f4, 0x9c845f8b,
    0xbdf9283b, 0x1ff897ff, 0xde05980f, 0xef2f118b, 0x5a0a6d1f, 0x6d367ecf, 0x27cb09b7,
    0x4f463f66, 0x9e5fea2d, 0x7527bac7, 0xebe5f17b, 0x3d0739f7, 0x8a5292ea, 0x6bfb5fb1,
    0x1f8d5d08, 0x56033046, 0xfc7b6bab, 0xf0cfbc20, 0x9af4361d, 0xa9e39161, 0x5ee61b08
];

QUARTER_PI :: 0.7853981633974483;

WINDOW_WORDS :: 8; // words of 2/pi multiplied: 256 bits, past every x's 53 and a margin
PRODUCT_WORDS :: WINDOW_WORDS + 2;

Reduced :: struct {
    quadrant: u64;
    head: float64;
    tail: float64;
}

// 64 bits of the big-endian word array `words` starting at bit `position` from its end.
product_bits :: (words: *[PRODUCT_WORDS] u32, position: s64) -> u64 #no_context {
    result: u64;
    for k: 0..63 {
        at := position + k;
        if at >= PRODUCT_WORDS * 32 break;
        word := words.*[PRODUCT_WORDS - 1 - at / 32];
        bit := cast(u64) ((word >> cast(u32) (at % 32)) & 1);
        result |= bit << cast(u64) k;
    }
    return result;
}

// Head and tail of the float64 pair nearest the 128-bit fraction high:low / 2^128.
pair_from_fraction :: (high: u64, low: u64) -> head: float64, tail: float64 #no_context {
    h := high;
    l := low;
    shift := 0;
    while h == 0 && shift < 128 {
        h = l;
        l = 0;
        shift += 64;
    }
    if h == 0 return 0, 0;
    while (h & SIGN_BIT) == 0 {
        h = (h << 1) | (l >> 63);
        l <<= 1;
        shift += 1;
    }
    // h holds the leading 64 bits: the top 53 make the head, the next 53 the tail.
    head_bits := h >> 11;
    tail_bits := ((h & 0x7ff) << 42) | (l >> 22);
    // The fraction is h:l / 2^(128 + shift).
    head := compose_float(false, head_bits, -53 - shift);
    tail := compose_float(false, tail_bits, -106 - shift);
    return head, tail;
}

// x = quadrant * pi/2 + (head + tail), for finite x with |x| > pi/4.
reduce :: (x: float64) -> Reduced #no_context {
    m, e := split_float(x);
    // The words before `first` only add multiples of 8 to x * 2/pi, which leave the quadrant
    // and the fraction unchanged.
    first := 0;
    if e >= 35 first = (e - 35) / 32 + 1;
    window: [WINDOW_WORDS] u32;
    for j: 0..WINDOW_WORDS - 1 window[j] = TWO_OVER_PI[first + j];
    // m * window, as PRODUCT_WORDS big-endian words.
    product: [PRODUCT_WORDS] u32;
    m_words: [2] u32;
    m_words[0] = cast,no_check(u32) (m >> 32);
    m_words[1] = cast,no_check(u32) m;
    for < i: 0..1 {
        carry: u64;
        for < j: 0..WINDOW_WORDS - 1 {
            at := i + j + 1;
            sum := cast(u64) m_words[i] * cast(u64) window[j] + cast(u64) product[at] + carry;
            product[at] = cast,no_check(u32) sum;
            carry = sum >> 32;
        }
        product[i] = cast,no_check(u32) (cast(u64) product[i] + carry);
    }
    // The product is m * 2/pi * 2^(32 * (first + WINDOW_WORDS)); x * 2/pi puts the binary point
    // `point` bits from the product's end.
    point := 32 * (first + WINDOW_WORDS) - e;
    result: Reduced;
    result.quadrant = product_bits(*product, point) & 3;
    high := product_bits(*product, point - 64);
    low := product_bits(*product, point - 128);
    negative := false;
    if (high & SIGN_BIT) != 0 {
        // Past one half: count the next quadrant and take the fraction minus one.
        result.quadrant = (result.quadrant + 1) & 3;
        low = ~low + 1;
        high = ~high + (ifx low == 0 then cast(u64) 1 else 0);
        negative = true;
    }
    f_head, f_tail := pair_from_fraction(high, low);
    // (f_head + f_tail) * pi/2.
    angle := pair_multiply(.{ f_head, f_tail }, HALF_PI);
    result.head = angle.head;
    result.tail = angle.tail;
    if negative {
        result.head = -result.head;
        result.tail = -result.tail;
    }
    return result;
}

// sin(head + tail) for |head| <= pi/4, tail tiny next to head, as a pair good to about 56 bits.
sine_pair :: (head: float64, tail: float64) -> Pair #no_context {
    z := head * head;
    series := -0.16666666666666666 + z * (0.008333333333333333 + z * (-0.0001984126984126984 + z * (2.7557319223985893e-06 + z * (-2.505210838544172e-08 + z * (1.6059043836821613e-10 + z * (-7.647163731819816e-13 + z * (2.8114572543455206e-15 + z * (-8.22063524662433e-18))))))));
    return quick_two_sum(head, head * z * series + tail * (1 - 0.5 * z));
}

sine_kernel :: (head: float64, tail: float64) -> float64 #no_context {
    return pair_round(sine_pair(head, tail));
}

// cos(head + tail) for |head| <= pi/4, as a pair good to about 56 bits.
cosine_pair :: (head: float64, tail: float64) -> Pair #no_context {
    square := two_product(head, head);
    z := square.head;
    series := 0.041666666666666664 + z * (-0.001388888888888889 + z * (2.48015873015873e-05 + z * (-2.755731922398589e-07 + z * (2.08767569878681e-09 + z * (-1.1470745597729725e-11 + z * (4.779477332387385e-14 + z * (-1.5619206968586225e-16 + z * 4.110317623312165e-19)))))));
    half_z := 0.5 * z;
    w := 1 - half_z;
    // 1 - z/2 loses the bits of z/2 below w's last place (and z those of head^2); add them back
    // with the rest.
    return quick_two_sum(w, ((1 - w) - half_z) + (z * z * series - 0.5 * square.tail - head * tail));
}

cosine_kernel :: (head: float64, tail: float64) -> float64 #no_context {
    return pair_round(cosine_pair(head, tail));
}

#program_export "sin"
wasi_sin :: (x: float64) -> float64 #c_call {
    if !is_finite(x) return x - x; // NaN
    magnitude := ifx x < 0 then -x else x;
    if magnitude <= QUARTER_PI return sine_kernel(x, 0);
    r := reduce(magnitude);
    value: float64;
    if r.quadrant == {
        case 0; value = sine_kernel(r.head, r.tail);
        case 1; value = cosine_kernel(r.head, r.tail);
        case 2; value = -sine_kernel(r.head, r.tail);
        case; value = -cosine_kernel(r.head, r.tail);
    }
    return ifx x < 0 then -value else value;
}

#program_export "cos"
wasi_cos :: (x: float64) -> float64 #c_call {
    if !is_finite(x) return x - x;
    magnitude := ifx x < 0 then -x else x;
    if magnitude <= QUARTER_PI return cosine_kernel(magnitude, 0);
    r := reduce(magnitude);
    if r.quadrant == {
        case 0; return cosine_kernel(r.head, r.tail);
        case 1; return -sine_kernel(r.head, r.tail);
        case 2; return -cosine_kernel(r.head, r.tail);
    }
    return sine_kernel(r.head, r.tail);
}

#program_export "sinf"
wasi_sinf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_sin(x);
}

#program_export "cosf"
wasi_cosf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_cos(x);
}

#program_export "tan"
wasi_tan :: (x: float64) -> float64 #c_call {
    if !is_finite(x) return x - x;
    magnitude := ifx x < 0 then -x else x;
    r: Reduced;
    if magnitude <= QUARTER_PI r.head = magnitude;
    else r = reduce(magnitude);
    sine := sine_pair(r.head, r.tail);
    cosine := cosine_pair(r.head, r.tail);
    value: float64;
    if (r.quadrant & 1) == 0 value = pair_round(pair_divide_pair(sine, cosine));
    else value = -pair_round(pair_divide_pair(cosine, sine));
    return ifx x < 0 then -value else value;
}

#program_export "tanf"
wasi_tanf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_tan(x);
}

// ---------------------------------------------------------------------------------------------
// exp, log and pow, through float64 pairs: each result is worked out to about 100 bits and
// rounded once at the end, so it is within an ulp (subnormal results may round twice).

LN2 :: Pair.{ 0.6931471805599453, 2.3190468138462996e-17 };
LOG2_E :: Pair.{ 1.4426950408889634, 2.0355273740931033e-17 }; // 1 / ln 2
LOG10_E :: Pair.{ 0.4342944819032518, 1.098319650216765e-17 }; // 1 / ln 10
EXP_OVERFLOW :: 709.782712893384; // above: beyond the largest float64
EXP_UNDERFLOW :: -745.1332191019412; // below: under half the smallest subnormal

// value * 2^exponent for any exponent, through steps that stay in range.
scale :: (value: float64, exponent: s64) -> float64 #no_context {
    x := value;
    e := exponent;
    while e > 1023 {
        x *= from_bits(0x7fe0_0000_0000_0000); // 2^1023
        e -= 1023;
    }
    while e < -1022 {
        x *= from_bits(0x0010_0000_0000_0000); // 2^-1022
        e += 1022;
    }
    return x * from_bits((cast(u64) (e + 1023)) << 52);
}

// e^r for |r| <= 0.35, by its Taylor series.
exp_series :: (r: Pair) -> Pair #no_context {
    sum := pair_add(ONE_PAIR, r);
    term := r;
    for n: 2..40 {
        term = pair_divide(pair_multiply(term, r), cast(float64) n);
        sum = pair_add(sum, term);
        magnitude := ifx term.head < 0 then -term.head else term.head;
        if magnitude < sum.head * 7.703719777548943e-34 break;
    }
    return sum;
}

// e^z = 2^k * e^r, r = z - k ln 2.
exp_pair :: (z: Pair) -> float64 #no_context {
    k := nearest_integer(z.head * LOG2_E.head);
    r := pair_add(z, pair_negate(pair_multiply_float(LN2, k)));
    return scale(pair_round(exp_series(r)), cast(s64) k);
}

// ln x for finite x > 0: x = m * 2^k with m in [sqrt(1/2), sqrt(2)), and ln m = 2 atanh(s),
// s = (m - 1) / (m + 1).
log_pair :: (x: float64) -> Pair #no_context {
    m, e := split_float(x);
    while m < IMPLICIT_BIT {
        m <<= 1;
        e -= 1;
    }
    k := e + 52;
    mantissa := from_bits((m & FRACTION_BITS) | 0x3ff0_0000_0000_0000); // in [1, 2)
    if mantissa > 1.4142135623730951 {
        mantissa *= 0.5;
        k += 1;
    }
    f := mantissa - 1; // exact
    s := pair_divide_pair(.{ f, 0 }, two_sum(2, f));
    s2 := pair_multiply(s, s);
    sum := s;
    power := s;
    for n: 1..40 {
        power = pair_multiply(power, s2);
        term := pair_divide(power, cast(float64) (2 * n + 1));
        sum = pair_add(sum, term);
        magnitude := ifx term.head < 0 then -term.head else term.head;
        total := ifx sum.head < 0 then -sum.head else sum.head;
        if magnitude < total * 7.703719777548943e-34 break;
    }
    return pair_add(pair_multiply_float(LN2, cast(float64) k), pair_multiply_float(sum, 2));
}

#program_export "exp"
wasi_exp :: (x: float64) -> float64 #c_call {
    if x != x return x;
    if x > EXP_OVERFLOW return from_bits(0x7ff0_0000_0000_0000);
    if x < EXP_UNDERFLOW return 0;
    return exp_pair(.{ x, 0 });
}

#program_export "expf"
wasi_expf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_exp(x);
}

#program_export "exp2"
wasi_exp2 :: (x: float64) -> float64 #c_call {
    if x != x return x;
    if x >= 1024 return from_bits(0x7ff0_0000_0000_0000);
    if x < -1075 return 0;
    whole := truncate(x);
    if whole == x return scale(1, cast(s64) whole);
    return exp_pair(pair_multiply_float(LN2, x));
}

#program_export "exp2f"
wasi_exp2f :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_exp2(x);
}

// The special cases every log shares; `done` false means x is finite and positive.
log_special :: (x: float64) -> result: float64, done: bool #no_context {
    if x != x return x, true;
    if x == 0 return from_bits(0xfff0_0000_0000_0000), true; // -inf
    if x < 0 return (x - x) / (x - x), true; // NaN
    if !is_finite(x) return x, true;
    return 0, false;
}

#program_export "log"
wasi_log :: (x: float64) -> float64 #c_call {
    special, done := log_special(x);
    if done return special;
    return pair_round(log_pair(x));
}

#program_export "logf"
wasi_logf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_log(x);
}

#program_export "log2"
wasi_log2 :: (x: float64) -> float64 #c_call {
    special, done := log_special(x);
    if done return special;
    return pair_round(pair_multiply(log_pair(x), LOG2_E));
}

#program_export "log2f"
wasi_log2f :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_log2(x);
}

#program_export "log10"
wasi_log10 :: (x: float64) -> float64 #c_call {
    special, done := log_special(x);
    if done return special;
    return pair_round(pair_multiply(log_pair(x), LOG10_E));
}

#program_export "log10f"
wasi_log10f :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_log10(x);
}

// Whether a float64 is an odd integer (every float64 from 2^53 up is even).
is_odd_integer :: (y: float64) -> bool #no_context {
    magnitude := ifx y < 0 then -y else y;
    if magnitude >= 9007199254740992.0 || truncate(y) != y return false;
    return truncate(magnitude * 0.5) * 2 != magnitude;
}

// The C99 special cases (Annex F.10.4.4), then x^y = e^(y ln x).
#program_export "pow"
wasi_pow :: (x: float64, y: float64) -> float64 #c_call {
    INFINITY :: 0x7ff0_0000_0000_0000;
    if y == 0 || x == 1 return 1;
    if x != x || y != y return x + y;
    y_odd := is_odd_integer(y);
    magnitude := ifx x < 0 then -x else x;
    if !is_finite(y) {
        if magnitude == 1 return 1;
        grows := (magnitude > 1) == (y > 0);
        return ifx grows then from_bits(INFINITY) else 0;
    }
    if x == 0 || !is_finite(x) {
        // x^y is 0 or infinite, negative for a negative x and an odd y.
        large := (x == 0) == (y < 0);
        result := from_bits(ifx large then INFINITY else 0);
        if y_odd && (bits_of(x) & SIGN_BIT) != 0 result = -result;
        return result;
    }
    negative := false;
    if x < 0 {
        if truncate(y) != y return (x - x) / (x - x); // a negative number to a fractional power
        negative = y_odd;
    }
    result: float64;
    y_magnitude := ifx y < 0 then -y else y;
    if magnitude == 1 {
        result = 1;
    } else if y_magnitude >= 18446744073709551616.0 {
        // |y ln x| is far past every float64 exponent, for any x other than 1.
        grows := (magnitude > 1) == (y > 0);
        result = from_bits(ifx grows then INFINITY else 0);
    } else {
        exponent := pair_multiply_float(log_pair(magnitude), y);
        if exponent.head > EXP_OVERFLOW result = from_bits(INFINITY);
        else if exponent.head < EXP_UNDERFLOW result = 0;
        else result = exp_pair(exponent);
    }
    return ifx negative then -result else result;
}

#program_export "powf"
wasi_powf :: (x: float32, y: float32) -> float32 #c_call {
    return cast(float32) wasi_pow(x, y);
}

// ---------------------------------------------------------------------------------------------
// atan and the angles made from it. atan(t) for 0 <= t <= 1 halves the angle three times
// (atan t = 2 atan(t / (1 + sqrt(1 + t^2)))) and sums the series t - t^3/3 + ... on what is
// left, |t| < 0.1.

PI :: Pair.{ 3.141592653589793, 1.2246467991473532e-16 };
HALF_PI :: Pair.{ 1.5707963267948966, 6.123233995736766e-17 };

// atan(t) for 0 <= t <= 1.
atan_pair :: (t: Pair) -> Pair #no_context {
    u := t;
    for 1..3 {
        root := pair_square_root(pair_add(ONE_PAIR, pair_multiply(u, u)));
        u = pair_divide_pair(u, pair_add(ONE_PAIR, root));
    }
    u2 := pair_multiply(u, u);
    sum := u;
    power := u;
    for n: 1..40 {
        power = pair_negate(pair_multiply(power, u2));
        term := pair_divide(power, cast(float64) (2 * n + 1));
        sum = pair_add(sum, term);
        magnitude := ifx term.head < 0 then -term.head else term.head;
        if magnitude < sum.head * 7.703719777548943e-34 break;
    }
    return pair_multiply_float(sum, 8);
}

// The angle of the point (x, y) for x, y >= 0, not both zero.
first_quadrant_angle :: (y: Pair, x: Pair) -> Pair #no_context {
    if y.head <= x.head return atan_pair(pair_divide_pair(y, x));
    return pair_add(HALF_PI, pair_negate(atan_pair(pair_divide_pair(x, y))));
}

#program_export "atan"
wasi_atan :: (x: float64) -> float64 #c_call {
    if x != x return x;
    magnitude := ifx x < 0 then -x else x;
    angle: float64;
    if magnitude < 7.450580596923828e-09 return x; // x^3/3 is below half an ulp of x
    if magnitude > 1152921504606846976.0 angle = HALF_PI.head; // pi/2 - 1/x rounds to pi/2
    else angle = pair_round(first_quadrant_angle(.{ magnitude, 0 }, ONE_PAIR));
    return ifx x < 0 then -angle else angle;
}

#program_export "atanf"
wasi_atanf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_atan(x);
}

// The C99 special cases (Annex F.10.1.4), then the angle of (|x|, |y|) moved to x's side.
#program_export "atan2"
wasi_atan2 :: (y: float64, x: float64) -> float64 #c_call {
    if x != x || y != y return x + y;
    y_negative := (bits_of(y) & SIGN_BIT) != 0;
    x_negative := (bits_of(x) & SIGN_BIT) != 0;
    ay := ifx y_negative then -y else y;
    ax := ifx x_negative then -x else x;
    angle: Pair;
    if y == 0 {
        angle = ifx x_negative then PI else .{ 0, 0 };
    } else if !is_finite(ay) {
        if !is_finite(ax) angle = ifx x_negative then pair_multiply_float(PI, 0.75) else pair_multiply_float(PI, 0.25);
        else angle = HALF_PI;
    } else if !is_finite(ax) || x == 0 {
        if x == 0 angle = HALF_PI;
        else angle = ifx x_negative then PI else .{ 0, 0 };
    } else {
        // Far apart, atan2 is the quotient (or pi/2 less it) to well past an ulp; close, both are
        // brought near 1, exactly, so the pair arithmetic neither overflows nor underflows.
        if ay < ax * 8.673617379884035e-19 { // 2^-60
            angle = .{ ay / ax, 0 };
        } else if ax < ay * 8.673617379884035e-19 {
            angle = pair_add(HALF_PI, .{ -(ax / ay), 0 });
        } else {
            larger := ifx ay > ax then ay else ax;
            exponent := 0;
            if larger > 3.273390607896142e+150 exponent = -600; // 2^500
            else if larger < 3.054936363499605e-151 exponent = 600; // 2^-500
            ay = scale(ay, exponent);
            ax = scale(ax, exponent);
            angle = first_quadrant_angle(.{ ay, 0 }, .{ ax, 0 });
        }
        if x_negative angle = pair_add(PI, pair_negate(angle));
    }
    result := pair_round(angle);
    return ifx y_negative then -result else result;
}

#program_export "atan2f"
wasi_atan2f :: (y: float32, x: float32) -> float32 #c_call {
    return cast(float32) wasi_atan2(y, x);
}

// sqrt(1 - x^2) as (1 - x)(1 + x), for |x| <= 1.
cosine_from_sine :: (x: float64) -> Pair #no_context {
    return pair_square_root(pair_multiply(two_sum(1, -x), two_sum(1, x)));
}

#program_export "asin"
wasi_asin :: (x: float64) -> float64 #c_call {
    if x != x return x;
    magnitude := ifx x < 0 then -x else x;
    if magnitude > 1 return (x - x) / (x - x);
    if magnitude < 7.450580596923828e-09 return x; // x^3/6 is below half an ulp of x
    angle := pair_round(first_quadrant_angle(.{ magnitude, 0 }, cosine_from_sine(magnitude)));
    return ifx x < 0 then -angle else angle;
}

#program_export "asinf"
wasi_asinf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_asin(x);
}

#program_export "acos"
wasi_acos :: (x: float64) -> float64 #c_call {
    if x != x return x;
    magnitude := ifx x < 0 then -x else x;
    if magnitude > 1 return (x - x) / (x - x);
    angle := first_quadrant_angle(cosine_from_sine(magnitude), .{ magnitude, 0 });
    if x < 0 angle = pair_add(PI, pair_negate(angle));
    return pair_round(angle);
}

#program_export "acosf"
wasi_acosf :: (x: float32) -> float32 #c_call {
    return cast(float32) wasi_acos(x);
}
