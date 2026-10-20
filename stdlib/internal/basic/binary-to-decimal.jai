// Fast binary float to decimal text. The result is the same text `portable_float` builds from the
// exact decimal expansion: the digits are rounded from an exact integer quotient and remainder, so
// they are only produced when 64- and 128-bit arithmetic can hold the whole computation.
#scope_module
FLOAT_TEXT_STORAGE :: 48;

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
    scratch: [20] u8;
    count: s64;
    rest := value;

    while rest != 0 || count < minimum {
        scratch[19 - count] = cast(u8) (48 + rest % 10);
        rest /= 10;
        count += 1;
    }

    for i: 0..count - 1 digits.*[i] = scratch[20 - count + i];

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
        at = 2;

        for i: 1..count - 1 {
            storage.*[at] = digits[i];
            at += 1;
        }
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
