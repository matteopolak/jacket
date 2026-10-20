// Fast binary float to decimal text. The result is the same text `portable_float` builds from the
// exact decimal expansion: the digits are rounded from an exact integer quotient and remainder, so
// they are only produced when 64- and 128-bit arithmetic can hold the whole computation.
#scope_module
FLOAT_TEXT_STORAGE :: 48;

// "00" through "99", for writing decimal digits two at a time.
DECIMAL_PAIRS :: "00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899";

// The text `portable_float(value, conversion, precision, round_up)` returns, written into `storage`;
// `ok` is false when the value or precision is too large for the integer arithmetic here.
fast_float_text :: (value: float64, conversion: u8, precision: s64, storage: *[FLOAT_TEXT_STORAGE] u8, round_up := false) -> text: string, ok: bool {
    bits := (cast(*u64) *value).*;
    exponent := cast(s64) ((bits >> 52) & 0x7ff);
    mantissa := bits & 0x000f_ffff_ffff_ffff;

    if exponent != 0 mantissa |= 0x0010_0000_0000_0000;

    binary := ifx exponent == 0 then -1074 else exponent - 1075;
    rounding := ifx round_up then Rounding.UP else .NEAREST;
    digits: [20] u8;

    if conversion == 102 {
        if precision < 0 || precision >= FIVE_POWERS.count return "", false;

        scaled, ok := scaled_round(mantissa, binary, precision, rounding);
        if !ok return "", false;

        count := write_digits(*digits, scaled, 1);

        return fixed_text(storage, digits, count, precision), true;
    }

    significant := ifx conversion == 103 then max(precision, 1) else precision + 1;
    if significant < 1 || significant > 19 return "", false;

    scaled: u64;
    power: s64;

    if mantissa != 0 {
        ok: bool;
        scaled, power, ok = leading_digits(mantissa, binary, significant, rounding);
        if !ok return "", false;
    }

    count := write_digits(*digits, scaled, significant);

    if conversion == 103 {
        text: string;
        if power < -4 || power >= significant text = scientific_text(storage, digits, count, power);
        else text = fixed_text(storage, digits, count, significant - power - 1);
        return trimmed_fraction(text), true;
    }

    return scientific_text(storage, digits, count, power), true;
}

#scope_file
Rounding :: enum u8 {
    NEAREST;
    UP;
    DOWN;
}

TEN_POWERS :: u64.[1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
                   10000000000, 100000000000, 1000000000000, 10000000000000, 100000000000000,
                   1000000000000000, 10000000000000000, 100000000000000000, 1000000000000000000,
                   10000000000000000000];

// How the discarded part of a quotient compares with one half.
Remainder :: enum u8 {
    NONE;
    BELOW_HALF;
    HALF;
    ABOVE_HALF;
}

FIVE_POWERS :: u64.[1, 5, 25, 125, 625, 3125, 15625, 78125, 390625, 1953125, 9765625, 48828125,
                    244140625, 1220703125, 6103515625, 30517578125, 152587890625, 762939453125,
                    3814697265625, 19073486328125, 95367431640625, 476837158203125,
                    2384185791015625, 11920928955078125, 59604644775390625, 298023223876953125,
                    1490116119384765625, 7450580596923828125];

// mantissa * 2^binary * 10^decimal rounded to an integer, for mantissa < 2^53; `ok` is false when
// the integer or an intermediate product does not fit in the arithmetic used. 10^decimal is split
// into 5^decimal, which fits in a word up to 5^27, and a power of two folded into `binary`.
scaled_round :: (mantissa: u64, binary: s64, decimal: s64, rounding: Rounding) -> value: u64, ok: bool {
    quotient: u64;
    remainder: Remainder;
    twos := binary + decimal;

    if decimal >= 0 {
        if decimal >= FIVE_POWERS.count return 0, false;

        high, low := multiply_64bits_with_overflow(mantissa, FIVE_POWERS[decimal]);

        if twos >= 0 {
            if high != 0 || twos > 63 || (twos > 0 && low >> (64 - twos) != 0) return 0, false;
            return low << twos, true;
        }

        shift := -twos;

        if shift < 64 {
            if high >> shift != 0 return 0, false;

            quotient = (high << (64 - shift)) | (low >> shift);
            remainder = compare_with_half(0, low & ((cast(u64) 1 << shift) - 1), 0, cast(u64) 1 << (shift - 1));
        } else if shift < 128 {
            part := shift - 64;
            quotient = high >> part;

            if part == 0 remainder = compare_with_half(0, low, 0, cast(u64) 1 << 63);
            else remainder = compare_with_half(high & ((cast(u64) 1 << part) - 1), low, cast(u64) 1 << (part - 1), 0);
        } else {
            // mantissa * 5^27 < 2^116, so the value is far below one half.
            remainder = ifx high == 0 && low == 0 then Remainder.NONE else .BELOW_HALF;
        }
    } else {
        if -decimal >= FIVE_POWERS.count return 0, false;

        divisor := FIVE_POWERS[-decimal];
        numerator := mantissa;

        if twos >= 0 {
            if twos > 63 || (twos > 0 && mantissa >> (64 - twos) != 0) return 0, false;
            numerator = mantissa << twos;
        } else {
            shift := -twos;

            // Past 2^64 the divisor exceeds twice the numerator: the quotient is zero, below half.
            if shift >= 64 || divisor >> (64 - shift) != 0 {
                remainder = ifx mantissa == 0 then Remainder.NONE else .BELOW_HALF;
                return finish_round(0, remainder, rounding);
            }

            divisor <<= shift;
        }

        quotient = numerator / divisor;
        rest := numerator - quotient * divisor;

        if rest == 0 remainder = .NONE;
        else if rest < divisor - rest remainder = .BELOW_HALF;
        else if rest == divisor - rest remainder = .HALF;
        else remainder = .ABOVE_HALF;
    }

    return finish_round(quotient, remainder, rounding);
}

finish_round :: (quotient: u64, remainder: Remainder, rounding: Rounding) -> value: u64, ok: bool {
    up: bool;

    if rounding == .UP up = remainder != .NONE;
    else if rounding == .NEAREST up = remainder == .ABOVE_HALF || (remainder == .HALF && (quotient & 1) != 0);

    if !up return quotient, true;
    if quotient == 0xffff_ffff_ffff_ffff return 0, false;

    return quotient + 1, true;
}

// Compares the 128-bit remainder (high, low) with the 128-bit half (half_high, half_low).
compare_with_half :: (high: u64, low: u64, half_high: u64, half_low: u64) -> Remainder {
    if high == 0 && low == 0 return .NONE;
    if high != half_high return ifx high < half_high then Remainder.BELOW_HALF else .ABOVE_HALF;
    if low != half_low return ifx low < half_low then Remainder.BELOW_HALF else .ABOVE_HALF;

    return .HALF;
}

// The first `significant` digits of mantissa * 2^binary (mantissa != 0) as an integer, and the
// decimal exponent of the first digit.
leading_digits :: (mantissa: u64, binary: s64, significant: s64, rounding: Rounding) -> digits: u64, power: s64, ok: bool {
    top := binary + 63 - leading_zero_bits(mantissa);
    // The value lies in [2^top, 2^(top + 1)), so its decimal exponent is floor(top * log10(2)) or
    // one more; (top * 78913) >> 18 is that floor across the float64 range.
    power := (top * 78913) >> 18;
    limit := TEN_POWERS[significant];

    scaled, ok := scaled_round(mantissa, binary, significant - 1 - power, rounding);

    if !ok return 0, 0, false;

    // A result of exactly `limit` is either a carry out of the last digit or a low estimate.
    low_estimate := scaled > limit;

    if scaled == limit {
        truncated, exact := scaled_round(mantissa, binary, significant - 1 - power, .DOWN);
        if !exact return 0, 0, false;
        low_estimate = truncated >= limit;
    }

    if low_estimate {
        power += 1;
        scaled, ok = scaled_round(mantissa, binary, significant - 1 - power, rounding);
        if !ok return 0, 0, false;
    }

    if scaled == limit {
        scaled = TEN_POWERS[significant - 1];
        power += 1;
    }

    return scaled, power, true;
}

// Writes `value` in decimal into `digits`, padded with leading zeros to at least `minimum` digits.
write_digits :: (digits: *[20] u8, value: u64, minimum: s64) -> count: s64 {
    scratch: [20] u8 = ---;
    at := scratch.count;
    rest := value;

    while rest >= 100 {
        pair := cast(s64) (rest % 100) * 2;

        rest /= 100;
        at -= 2;
        scratch[at] = DECIMAL_PAIRS[pair];
        scratch[at + 1] = DECIMAL_PAIRS[pair + 1];
    }

    if rest >= 10 {
        pair := cast(s64) rest * 2;
        at -= 2;
        scratch[at] = DECIMAL_PAIRS[pair];
        scratch[at + 1] = DECIMAL_PAIRS[pair + 1];
    } else {
        at -= 1;
        scratch[at] = cast(u8) (48 + rest);
    }

    while scratch.count - at < minimum {
        at -= 1;
        scratch[at] = 48;
    }

    count := scratch.count - at;

    memcpy(digits.data, scratch.data + at, count);

    return count;
}

// `digits` with the last `fraction` of them after the decimal point.
fixed_text :: (storage: *[FLOAT_TEXT_STORAGE] u8, digits: [20] u8, count: s64, fraction: s64) -> string {
    at: s64;
    whole := count - fraction;

    if whole <= 0 {
        storage.*[0] = 48;
        storage.*[1] = 46;
        at = 2;

        for 1..-whole {
            storage.*[at] = 48;
            at += 1;
        }

        for i: 0..count - 1 storage.*[at + i] = digits[i];

        at += count;
    } else {
        for i: 0..whole - 1 storage.*[i] = digits[i];

        at = whole;

        if fraction > 0 {
            storage.*[at] = 46;
            at += 1;

            for i: whole..count - 1 {
                storage.*[at] = digits[i];
                at += 1;
            }
        }
    }

    return string.{ at, storage.data };
}

// d.ddde+XX: one digit, the rest after a point, and at least two exponent digits.
scientific_text :: (storage: *[FLOAT_TEXT_STORAGE] u8, digits: [20] u8, count: s64, power: s64) -> string {
    storage.*[0] = digits[0];
    at := 1;

    if count > 1 {
        storage.*[1] = 46;
        memcpy(storage.data + 2, digits.data + 1, count - 1);
        at = count + 1;
    }

    storage.*[at] = 101;
    storage.*[at + 1] = ifx power < 0 then cast(u8) 45 else 43;
    at += 2;

    magnitude := ifx power < 0 then -power else power;
    exponent: [20] u8;
    length := write_digits(*exponent, cast(u64) magnitude, 2);

    for i: 0..length - 1 storage.*[at + i] = exponent[i];

    return string.{ at + length, storage.data };
}

// Drops trailing fraction zeros, and the point when nothing follows it, before any exponent.
trimmed_fraction :: (text: string) -> string {
    point: s64 = -1;
    end := text.count;

    for byte, index: text {
        if byte == 46 point = index;

        if byte == 101 {
            end = index;
            break;
        }
    }

    if point < 0 return text;

    kept := end;
    while kept > point + 1 && text[kept - 1] == 48 kept -= 1;
    if kept == point + 1 kept = point;

    result := text;
    for i: 0..text.count - end - 1 result[kept + i] = text[end + i];

    result.count -= end - kept;

    return result;
}

#scope_module
// Whether the `g` text of `value` with `significant` digits (rounded up with `round_up`) parses back
// to `value` as a float64, or with `wide` false as the float32 `value` came from. The text is the
// exact decimal D * 10^E, which parses back exactly when it lies inside `value`'s rounding interval:
// half a unit either side, a quarter below for a normal power of two, with the ends included when
// the significand is even. `known` is false when the integers do not fit.
digits_read_back :: (value: float64, wide: bool, significant: s64, round_up: bool, power_of_two: bool) -> matches: bool, known: bool {
    bits := (cast(*u64) *value).*;
    exponent := cast(s64) ((bits >> 52) & 0x7ff);
    mantissa := bits & 0x000f_ffff_ffff_ffff;

    if exponent != 0 mantissa |= 0x0010_0000_0000_0000;
    if mantissa == 0 return true, true;

    binary := ifx exponent == 0 then -1074 else exponent - 1075;
    rounding := ifx round_up then Rounding.UP else .NEAREST;

    if significant < 1 || significant > 19 return false, false;

    digits, power, ok := leading_digits(mantissa, binary, significant, rounding);
    if !ok return false, false;

    m, b := own_significand(value, wide);

    return inside_interval(digits, power - significant + 1, m, b, power_of_two);
}

// Whether the `g` text of a positive value reads back as `value`. Used when `digits_read_back`
// cannot decide; the comparison is exact at any magnitude.
text_reads_back :: (text: string, value: float64, wide: bool, power_of_two: bool) -> bool {
    digits: u64;
    decimal: s64;
    seen_point := false;
    at := 0;

    while at < text.count && text[at] != 101 {
        byte := text[at];

        at += 1;

        if byte == 46 {
            seen_point = true;
            continue;
        }

        digits = digits * 10 + cast(u64) (byte - 48);
        if seen_point decimal -= 1;
    }

    if at < text.count {
        negative := text[at + 1] == 45;
        written: s64;

        for i: at + 2..text.count - 1 written = written * 10 + cast(s64) (text[i] - 48);

        decimal += ifx negative then -written else written;
    }

    m, b := own_significand(value, wide);
    if digits == 0 return m == 0;

    matches, known := inside_interval(digits, decimal, m, b, power_of_two);
    if known return matches;

    // Same comparison as `inside_interval`, in big integers.
    lower: u64 = ifx power_of_two then cast(u64) 1 else 2;
    twos := min(decimal, b - 2);
    fives := min(decimal, 0);

    candidate := nat_from(digits);
    top := nat_from(4 * m + 2);
    bottom := nat_from(4 * m - lower);

    defer {
        array_free(candidate.words);
        array_free(top.words);
        array_free(bottom.words);
    }

    nat_multiply_five_power(*candidate, decimal - fives);
    nat_shift(*candidate, decimal - twos);
    nat_multiply_five_power(*top, -fives);
    nat_shift(*top, b - 2 - twos);
    nat_multiply_five_power(*bottom, -fives);
    nat_shift(*bottom, b - 2 - twos);

    above_bottom := nat_compare(candidate, bottom);
    below_top := nat_compare(candidate, top);

    if above_bottom > 0 && below_top < 0 return true;
    if above_bottom == 0 || below_top == 0 return (m & 1) == 0;

    return false;
}

#scope_file
// The significand and binary exponent of `value` in its own type, so the value is m * 2^b.
own_significand :: (value: float64, wide: bool) -> m: u64, b: s64 {
    if wide {
        bits := (cast(*u64) *value).*;
        field := cast(s64) ((bits >> 52) & 0x7ff);
        m := bits & 0x000f_ffff_ffff_ffff;

        if field != 0 m |= 0x0010_0000_0000_0000;

        return m, ifx field == 0 then -1074 else field - 1075;
    }

    single := cast(float32) value;
    narrow := (cast(*u32) *single).*;
    field := cast(s64) ((narrow >> 23) & 0xff);
    m := cast(u64) (narrow & 0x7f_ffff);

    if field != 0 m |= 0x80_0000;

    return m, ifx field == 0 then -149 else field - 150;
}

// Whether digits * 10^decimal lies in the rounding interval of m * 2^b, in 128-bit integers;
// `known` is false when the scaled numbers do not fit.
inside_interval :: (digits: u64, decimal: s64, m: u64, b: s64, power_of_two: bool) -> matches: bool, known: bool {
    // In units of 2^(b - 2): the value is 4m, half a unit is 2, and a quarter unit is 1.
    lower: u64 = ifx power_of_two then cast(u64) 1 else 2;

    // Divide both sides by the smaller power of two and of five they share, leaving integers.
    twos := min(decimal, b - 2);
    fives := min(decimal, 0);

    if decimal - fives >= FIVE_POWERS.count || -fives >= FIVE_POWERS.count return false, false;

    candidate_high, candidate_low := multiply_64bits_with_overflow(digits, FIVE_POWERS[decimal - fives]);
    scale := FIVE_POWERS[-fives];
    value_high, value_low := multiply_64bits_with_overflow(4 * m, scale);
    upper := 2 * scale;
    below := lower * scale;

    candidate_ok := shift_wide(*candidate_high, *candidate_low, decimal - twos);
    value_ok := shift_wide(*value_high, *value_low, b - 2 - twos);

    if !candidate_ok || !value_ok return false, false;

    // The gaps are at most 2 * 5^27 < 2^64 before shifting, and stay below the value after it.
    upper_high: u64;
    upper_low := upper;
    below_high: u64;
    below_low := below;

    if !shift_wide(*upper_high, *upper_low, b - 2 - twos) || !shift_wide(*below_high, *below_low, b - 2 - twos) return false, false;

    // value + upper must not wrap; value - below cannot go negative since 4m >= 4 > 2.
    top_high, top_low := add_wide(value_high, value_low, upper_high, upper_low);
    if top_high < value_high return false, false;

    bottom_high, bottom_low := subtract_wide(value_high, value_low, below_high, below_low);
    even := (m & 1) == 0;

    above_bottom := compare_wide(candidate_high, candidate_low, bottom_high, bottom_low);
    below_top := compare_wide(candidate_high, candidate_low, top_high, top_low);

    if above_bottom > 0 && below_top < 0 return true, true;
    if above_bottom == 0 || below_top == 0 return even, true;

    return false, true;
}

// Shifts the 128-bit (high, low) left by `amount`; false when bits would be lost.
shift_wide :: (high: *u64, low: *u64, amount: s64) -> bool {
    if amount == 0 return true;
    if amount < 0 || amount >= 128 return high.* == 0 && low.* == 0 && amount >= 0;

    if amount >= 64 {
        if high.* != 0 || low.* >> (128 - amount) != 0 return false;
        high.* = low.* << (amount - 64);
        low.* = 0;
        return true;
    }

    if high.* >> (64 - amount) != 0 return false;

    high.* = (high.* << amount) | (low.* >> (64 - amount));
    low.* <<= amount;

    return true;
}

add_wide :: (a_high: u64, a_low: u64, b_high: u64, b_low: u64) -> high: u64, low: u64 {
    low := a_low + b_low;
    carry: u64 = ifx low < a_low then cast(u64) 1 else 0;
    return a_high + b_high + carry, low;
}

subtract_wide :: (a_high: u64, a_low: u64, b_high: u64, b_low: u64) -> high: u64, low: u64 {
    borrow: u64 = ifx a_low < b_low then cast(u64) 1 else 0;
    return a_high - b_high - borrow, a_low - b_low;
}

compare_wide :: (a_high: u64, a_low: u64, b_high: u64, b_low: u64) -> s64 {
    if a_high != b_high return ifx a_high < b_high then -1 else 1;
    if a_low != b_low return ifx a_low < b_low then -1 else 1;
    return 0;
}
