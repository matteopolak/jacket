// Numbers for Json: reading a validated token as a float (Basic does the correctly rounded
// conversion) and shortest round-trip printing of floats. Internal; the public pieces are in
// module.jai.

#scope_module

// A token that already passed the JSON grammar check, rounded to the nearest float64 (float32).
// `overflow` is set when it is beyond the largest finite value; the exponent is not limited, so
// 1e999999 overflows and 1e-999999 is zero.
//
// Most tokens (at most 19 significant digits and a small exponent) are converted directly: exactly
// by one multiplication or division when the digits and the power of ten are exactly
// representable, otherwise by the Eisel-Lemire algorithm (Lemire, "Number Parsing at a Gigabyte per
// Second", as in fast_float) with a table of 128-bit powers of five. Both give the correctly
// rounded result by construction, and the algorithm says when it cannot decide; everything else,
// and every infinity, goes to Basic's exact conversion (`scan_decimal_float`).
float64_from_token :: (token: string) -> float64, bool {
    ok, negative, w, q := split_decimal(token);

    if ok {
        sign: u64 = ifx negative then 0x8000_0000_0000_0000 else 0;

        if w == 0 return cast,force(float64) sign, false;

        if w <= (cast(u64) 1) << 53 && q >= -22 && q <= 22 {
            value := cast(float64) w;
            value = ifx q >= 0 then value * EXACT_POWERS_OF_TEN[q] else value / EXACT_POWERS_OF_TEN[-q];

            return ifx negative then -value else value, false;
        }

        bits, decided := eisel_lemire(w, q, 52, -1023, 0x7ff, -4, 23);

        if decided return cast,force(float64) (bits | sign), false;
    }

    decimal: Decimal_Float = ---;

    scan_decimal_float(token, *decimal, limit_exponent = false);

    return decimal_float_to_float64(*decimal);
}

float32_from_token :: (token: string) -> float32, bool {
    ok, negative, w, q := split_decimal(token);

    if ok {
        sign: u32 = ifx negative then 0x8000_0000 else 0;

        if w == 0 return cast,force(float32) sign, false;

        if w <= (cast(u64) 1) << 24 && q >= -10 && q <= 10 {
            value := cast(float32) w;
            value = ifx q >= 0 then value * cast(float32) EXACT_POWERS_OF_TEN[q] else value / cast(float32) EXACT_POWERS_OF_TEN[-q];

            return ifx negative then -value else value, false;
        }

        bits, decided := eisel_lemire(w, q, 23, -127, 0xff, -17, 10);

        if decided return cast,force(float32) (cast,no_check(u32) bits | sign), false;
    }

    decimal: Decimal_Float = ---;

    scan_decimal_float(token, *decimal, limit_exponent = false);

    return decimal_float_to_float32(*decimal);
}

EXACT_POWERS_OF_TEN :: float64.[1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16, 1e17, 1e18, 1e19, 1e20, 1e21, 1e22];

// A validated token as sign, significant digits `w` (leading zeros skipped) and a power of ten
// `q`: the value is w * 10^q. False when it has more than 19 significant digits, an exponent
// beyond five digits, or anything the grammar check would not have let through.
split_decimal :: (token: string) -> bool, bool, u64, s64 #no_abc #no_aoc {
    data := token.data;
    count := token.count;
    i := 0;
    negative := false;

    if count > 0 && data[0] == #char "-" {
        negative = true;
        i = 1;
    }

    w: u64;
    digits := 0;
    q: s64;

    while i < count {
        d := cast(u64) data[i] - 48;

        if d >= 10 break;

        if w != 0 || d != 0 {
            digits += 1;

            if digits > 19 return false, false, 0, 0;

            w = w * 10 + d;
        }

        i += 1;
    }

    if i < count && data[i] == #char "." {
        i += 1;

        while i < count {
            d := cast(u64) data[i] - 48;

            if d >= 10 break;

            if w != 0 || d != 0 {
                digits += 1;

                if digits > 19 return false, false, 0, 0;

                w = w * 10 + d;
            }

            q -= 1;
            i += 1;
        }
    }

    if i < count && (data[i] == #char "e" || data[i] == #char "E") {
        i += 1;

        exponent_negative := false;

        if i < count && data[i] == #char "-" {
            exponent_negative = true;
            i += 1;
        } else if i < count && data[i] == #char "+" {
            i += 1;
        }

        e: s64;

        while i < count {
            d := cast(u64) data[i] - 48;

            if d >= 10 break;

            e = e * 10 + cast(s64) d;

            if e >= 100000 return false, false, 0, 0;

            i += 1;
        }

        q += ifx exponent_negative then -e else e;
    }

    if i != count return false, false, 0, 0;

    return true, negative, w, q;
}

// The high and low words of the 128-bit product.
multiply_wide :: inline (a: u64, b: u64) -> u64, u64 #no_aoc {
    mask :: 0xffff_ffff;
    p00 := (a & mask) * (b & mask);
    p01 := (a & mask) * (b >> 32);
    p10 := (a >> 32) * (b & mask);
    p11 := (a >> 32) * (b >> 32);
    middle := (p00 >> 32) + (p01 & mask) + (p10 & mask);

    return p11 + (p01 >> 32) + (p10 >> 32) + (middle >> 32), (middle << 32) | (p00 & mask);
}

count_leading_zeros :: inline (x: u64) -> s64 #no_aoc {
    #if OS == .WASM {
        n: s64;
        v := x;

        if v >> 32 == 0 {
            n += 32;
            v <<= 32;
        }

        if v >> 48 == 0 {
            n += 16;
            v <<= 16;
        }

        if v >> 56 == 0 {
            n += 8;
            v <<= 8;
        }

        if v >> 60 == 0 {
            n += 4;
            v <<= 4;
        }

        if v >> 62 == 0 {
            n += 2;
            v <<= 2;
        }

        if v >> 63 == 0 {
            n += 1;
        }

        return n;
    } else {
        return cast(s64) leading_zeros(x);
    }
}

// w * 10^q (w nonzero) rounded to a binary float with `mantissa_bits` explicit mantissa bits whose
// exponent field is `infinite` for infinities and whose smallest normal exponent is
// `min_exponent`. Returns the bits below the sign, and false when the answer is not certain (the
// product is too close to a rounding boundary for the 128-bit table, or the result would be an
// infinity). `even_low..even_high` is the range of q where a result exactly halfway between two
// floats can occur, where ties go to the even one.
eisel_lemire :: (w_in: u64, q: s64, mantissa_bits: s64, min_exponent: s64, infinite: s64, even_low: s64, even_high: s64) -> u64, bool #no_abc #no_aoc {
    if q < -342 || q > 308 return 0, false;

    lz := count_leading_zeros(w_in);
    w := w_in << cast(u64) lz;
    index := 2 * (q + 342);
    first_high, first_low := multiply_wide(w, POWER_OF_FIVE_128[index]);
    precision_mask: u64 = 0xffff_ffff_ffff_ffff >> cast(u64) (mantissa_bits + 3);

    if (first_high & precision_mask) == precision_mask {
        second_high, _ := multiply_wide(w, POWER_OF_FIVE_128[index + 1]);

        first_low += second_high;

        if second_high > first_low first_high += 1;
    }

    // The table is exact enough for every 19-digit w; this guard, from the first published form
    // of the algorithm, only makes the rare all-ones case ask the exact conversion.
    if first_low == 0xffff_ffff_ffff_ffff && (q < -27 || q > 55) return 0, false;

    upper := cast(s64) (first_high >> 63);
    shift := cast(u64) (upper + 64 - mantissa_bits - 3);
    mantissa := first_high >> shift;

    // floor(log2(10^q)) as ((152170 + 65536) * q) >> 16, with the bias and the shifts applied.
    power2 := (((152170 + 65536) * q) >> 16) + 63 + upper - lz - min_exponent;

    if power2 <= 0 {
        // Subnormal (or zero): shift out the bits below the smallest exponent and round half up;
        // an exact tie cannot occur here.
        if -power2 + 1 >= 64 return 0, true;

        mantissa >>= cast(u64) (-power2 + 1);
        mantissa += mantissa & 1;
        mantissa >>= 1;

        // Rounding can carry it up to the smallest normal number.
        power2 = ifx mantissa < (cast(u64) 1) << cast(u64) mantissa_bits then 0 else 1;

        return mantissa | (cast(u64) power2 << cast(u64) mantissa_bits), true;
    }

    // A result exactly halfway between two floats rounds to even instead of up.
    if first_low <= 1 && q >= even_low && q <= even_high && (mantissa & 3) == 1 {
        if (mantissa << shift) == first_high mantissa &= 0xffff_ffff_ffff_fffe;
    }

    mantissa += mantissa & 1;
    mantissa >>= 1;

    if mantissa >= (cast(u64) 2) << cast(u64) mantissa_bits {
        mantissa = (cast(u64) 1) << cast(u64) mantissa_bits;
        power2 += 1;
    }

    mantissa &= ~((cast(u64) 1) << cast(u64) mantissa_bits);

    if power2 >= infinite return 0, false;

    return mantissa | (cast(u64) power2 << cast(u64) mantissa_bits), true;
}

// ---- printing ----

// Appends digits (no leading or trailing zeros, count >= 1) times 10^(point - count) in the
// shortest of JSON's spellings: plain for magnitudes 1e-6 up to 1e21, with a ".0" when it would
// otherwise read as an integer, and `1.5e-9` / `1e21` outside that. The text is put together in a
// local buffer and appended once.
append_decimal_float :: (builder: *String_Builder, negative: bool, digits: *u8, count: int, exponent10: int) {
    ascii: [32] u8 = ---;

    for 0..count - 1 ascii[it] = digits[it] + #char "0";

    append_decimal_ascii(builder, negative, ascii.data, count, exponent10);
}

// The same for digits that are already characters. The text is built in a window reserved in the
// builder (at most 32 bytes: 17 digits, a sign, a point, up to five zeros and an exponent), or in a
// local buffer when the builder cannot give one.
append_decimal_ascii :: (builder: *String_Builder, negative: bool, digits: *u8, count: int, exponent10: int) {
    local: [64] u8 = ---;
    window := builder_reserve(builder, 32);
    out := window;

    if !out out = local.data;

    n := 0;

    if negative {
        out[n] = #char "-";
        n += 1;
    }

    // exponent10 is the exponent of the first digit: the value is d.ddd * 10^exponent10.
    if exponent10 < -6 || exponent10 >= 21 {
        out[n] = digits[0];
        n += 1;

        if count > 1 {
            out[n] = #char ".";
            copy_bytes(out + n + 1, digits + 1, count - 1);
            n += count;
        }

        out[n] = #char "e";
        n += 1;

        value := exponent10;

        if value < 0 {
            out[n] = #char "-";
            n += 1;
            value = -value;
        }

        if value >= 100 {
            out[n] = cast,no_check(u8) (value / 100) + #char "0";
            n += 1;
        }

        if value >= 10 {
            out[n] = cast,no_check(u8) ((value / 10) % 10) + #char "0";
            n += 1;
        }

        out[n] = cast,no_check(u8) (value % 10) + #char "0";
        n += 1;
    } else if exponent10 < 0 {
        out[n] = #char "0";
        out[n + 1] = #char ".";
        n += 2;

        for 1..(-exponent10 - 1) {
            out[n] = #char "0";
            n += 1;
        }

        copy_bytes(out + n, digits, count);
        n += count;
    } else {
        whole := exponent10 + 1;

        if count > whole {
            copy_bytes(out + n, digits, whole);
            n += whole;
            out[n] = #char ".";
            copy_bytes(out + n + 1, digits + whole, count - whole);
            n += 1 + count - whole;
        } else {
            copy_bytes(out + n, digits, count);
            n += count;

            for count..whole - 1 {
                out[n] = #char "0";
                n += 1;
            }

            out[n] = #char ".";
            out[n + 1] = #char "0";
            n += 2;
        }
    }

    if window builder_commit(builder, n);
    else append(builder, out, n);
}

// The shortest decimal that reads back as the same float64, and of those the closest to it:
// `digits * 10^exponent` with no trailing zero in `digits`. `magnitude` is the bits without the
// sign, of a finite normal number (a subnormal can need a single digit, and this gives at least two). This is Schubfach (Giulietti, "The Schubfach way to render
// doubles"), following the reference implementation, with the 126-bit powers of ten of pow10.jai.
// Basic's `__float_decimal_digits` gives the same digits; tests/numbers.jai checks that they agree.
shortest_decimal :: (magnitude: u64) -> u64, s64 #no_abc #no_aoc {
    biased := cast(s64) (magnitude >> 52);
    mq := 1075 - biased;
    c := ((cast(u64) 1) << 52) | (magnitude & 0x000f_ffff_ffff_ffff);

    // An integer below 2^53 is its own shortest form (once the trailing zeros are gone).
    if mq > 0 && mq < 53 {
        whole := c >> cast(u64) mq;

        if (whole << cast(u64) mq) == c return strip_zeros(whole, 0);
    }

    q := -mq;

    odd := c & 1;
    cb := c << 2;
    cbr := cb + 2;
    cbl: u64;
    k: s64;

    if c != ((cast(u64) 1) << 52) || q == -1074 {
        cbl = cb - 2;
        k = (q * 78913) >> 18;
    } else {
        cbl = cb - 1;
        k = (q * 631305 - 261663) >> 21;
    }

    h := cast(u64) (q + (((0 - k) * 108853) >> 15) + 2);
    index := 2 * (k + 324);
    g1 := POWER_OF_TEN_126[index];
    g0 := POWER_OF_TEN_126[index + 1];

    vb := round_to_odd(g1, g0, cb << h);
    vbl := round_to_odd(g1, g0, cbl << h);
    vbr := round_to_odd(g1, g0, cbr << h);

    s := vb >> 2;

    if s >= 100 {
        // One digit fewer: the multiples of ten around s, if exactly one is inside the interval.
        quotient, _ := multiply_wide(s, 115_292_150_460_684_698 << 4);
        sp10 := 10 * quotient;
        tp10 := sp10 + 10;
        upper_in := vbl + odd <= sp10 << 2;
        lower_in := (tp10 << 2) + odd <= vbr;

        if upper_in != lower_in return strip_zeros(ifx upper_in then sp10 else tp10, k);
    }

    t10 := s + 1;
    s_in := vbl + odd <= s << 2;
    t_in := (t10 << 2) + odd <= vbr;

    if s_in != t_in return strip_zeros(ifx s_in then s else t10, k);

    // Both are inside: the closer one, the even one when equally close.
    difference := cast(s64) vb - cast(s64) ((s + t10) << 1);

    return strip_zeros(ifx difference < 0 || (difference == 0 && (s & 1) == 0) then s else t10, k);
}

round_to_odd :: inline (g1: u64, g0: u64, cp: u64) -> u64 #no_aoc {
    x1, _ := multiply_wide(g0, cp);
    y1, y0 := multiply_wide(g1, cp);
    z := (y0 >> 1) + x1;
    result := y1 + (z >> 63);

    return result | ((((z & 0x7fff_ffff_ffff_ffff) + 0x7fff_ffff_ffff_ffff) >> 63));
}

strip_zeros :: inline (value: u64, exponent: s64) -> u64, s64 {
    v := value;
    e := exponent;

    while v % 10 == 0 {
        v /= 10;
        e += 1;
    }

    return v, e;
}

// The fewest digits that read back as the same float64; NaN and infinities have no JSON
// spelling and are written as null.
append_float64 :: (builder: *String_Builder, value: float64) {
    bits := cast,force(u64) value;
    magnitude := bits & 0x7fff_ffff_ffff_ffff;

    if (magnitude >> 52) == 0x7ff {
        append(builder, "null");

        return;
    }

    negative := (bits >> 63) != 0;

    if magnitude == 0 {
        append(builder, ifx negative then "-0.0" else "0.0");

        return;
    }

    // Subnormals are rare, and Schubfach never gives a single digit for the smallest of them
    // (4.9e-323 where 5e-323 reads back the same), so they keep the exact generator.
    if (magnitude >> 52) == 0 {
        digits := __float_decimal_digits(magnitude, -1);

        append_decimal_float(builder, negative, digits.digits.data, digits.count, digits.exponent);

        return;
    }

    value_digits, exponent := shortest_decimal(magnitude);

    // The digits, most significant first, as characters.
    text: [20] u8 = ---;
    start := write_digits_backward(text.data + 20, value_digits);
    count := text.data + 20 - start;

    append_decimal_ascii(builder, negative, start, count, exponent + count - 1);
}

append_float32 :: (builder: *String_Builder, value: float32) {
    bits := cast,force(u32) value;
    magnitude := bits & 0x7fff_ffff;

    if (magnitude >> 23) == 0xff {
        append(builder, "null");

        return;
    }

    negative := (bits >> 31) != 0;

    if magnitude == 0 {
        append(builder, ifx negative then "-0.0" else "0.0");

        return;
    }

    wide := cast(float64) value;
    wide_bits := (cast,force(u64) wide) & 0x7fff_ffff_ffff_ffff;

    for precision: 1..9 {
        digits := __float_decimal_digits(wide_bits, precision);

        count := digits.count;

        while count > 1 && digits.digits[count - 1] == 0 count -= 1;

        candidate: Decimal_Float = ---;
        candidate.negative = false;
        candidate.truncated = false;
        candidate.count = count;
        candidate.exponent = digits.exponent - count + 1;

        for 0..count - 1 candidate.digits[it] = digits.digits[it];

        back, overflow := decimal_float_to_float32(*candidate);

        if !overflow && (cast,force(u32) back) == (magnitude) {
            append_decimal_float(builder, negative, digits.digits.data, count, digits.exponent);

            return;
        }
    }

    digits := __float_decimal_digits(wide_bits, 9);

    append_decimal_float(builder, negative, digits.digits.data, digits.count, digits.exponent);
}
