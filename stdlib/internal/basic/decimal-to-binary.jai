// Fast decimal to binary float conversion. Both paths here are exact: each either produces the
// correctly rounded result or reports that it cannot, and the caller then takes the big-integer path.
#scope_module
Decimal_Scan :: enum u8 {
    DONE;
    NO_DIGITS;
    NEEDS_EXACT;
}

// Reads the decimal digits and exponent at `cursor` and rounds them to T's bits (without the sign).
decimal_float_bits :: (text: string, cursor: s64, $T: Type) -> bits: u64, status: Decimal_Scan, rest: string {
    at := cursor;
    significand: u64;
    kept: s64;
    digits: s64;
    fraction_kept: s64;
    whole_dropped: s64;
    point := false;
    inexact := false;

    while at < text.count {
        byte := text[at];

        if byte == #char "." && !point {
            point = true;
            at += 1;
            continue;
        }

        if byte < #char "0" || byte > #char "9" break;

        digit := cast(u64) (byte - 48);

        // Nineteen digits always fit in 64 bits; leading zeros do not count towards them.
        if kept < 19 {
            significand = significand * 10 + digit;
            if significand != 0 kept += 1;
            if point fraction_kept += 1;
        } else {
            if digit != 0 inexact = true;
            if !point whole_dropped += 1;
        }

        digits += 1;
        at += 1;
    }

    if digits == 0 return 0, .NO_DIGITS, text;

    exponent: s64;

    if at < text.count && (text[at] == #char "e" || text[at] == #char "E") {
        end := at + 1;
        minus := false;

        if end < text.count && (text[end] == #char "+" || text[end] == #char "-") {
            minus = text[end] == #char "-";
            end += 1;
        }

        start := end;

        while end < text.count {
            if text[end] < #char "0" || text[end] > #char "9" break;

            digit := cast(s64) (text[end] - 48);

            // Past this size the result is zero or infinite whatever the digits, which the exact
            // path decides without arithmetic on the exponent.
            if exponent >= 1 << 40 inexact = true;
            else exponent = exponent * 10 + digit;

            end += 1;
        }

        if end > start {
            at = end;
            if minus exponent = -exponent;
        } else exponent = 0;
    }

    if inexact return 0, .NEEDS_EXACT, text;

    rest := advance(text, at);
    if significand == 0 return 0, .DONE, rest;

    power := exponent - fraction_kept + whole_dropped;

    bits, ok := small_decimal_bits(significand, power, T);

    if ok return bits, .DONE, rest;

    bits, ok = nearest_binary_bits(significand, power, T);
    if ok return bits, .DONE, rest;

    return 0, .NEEDS_EXACT, text;
}

#scope_file
// Every power of ten up to 10^22 is exact in a float64, and up to 10^10 in a float32.
TEN_POWERS_64 :: float64.[1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13,
                          1e14, 1e15, 1e16, 1e17, 1e18, 1e19, 1e20, 1e21, 1e22];

TEN_POWERS_32 :: float32.[1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10];

// When the significand and the power of ten are both exact in T, one multiplication or division
// rounds their exact product once, which is the correctly rounded result.
small_decimal_bits :: (significand: u64, power: s64, $T: Type) -> bits: u64, ok: bool {
    #if T == float64 {
        if significand > 1 << 53 || power < -22 || power > 22 return 0, false;

        value := cast(float64) significand;
        if power < 0 value /= TEN_POWERS_64[-power];
        else value *= TEN_POWERS_64[power];

        return (cast(*u64) *value).*, true;
    } else {
        if significand > 1 << 24 || power < -10 || power > 10 return 0, false;

        value := cast(float32) significand;
        if power < 0 value /= TEN_POWERS_32[-power];
        else value *= TEN_POWERS_32[power];

        return cast(u64) (cast(*u32) *value).*, true;
    }
}

// The float nearest to significand * 10^power, from a 128-bit approximation of the power of five
// (the Eisel-Lemire method). The table entries round down, so the product falls short of the exact
// value by less than two units in its second word; `ok` is false when that shortfall could carry
// into the kept bits, or could hide that the exact value lies on a rounding boundary.
nearest_binary_bits :: (significand: u64, power: s64, $T: Type) -> bits: u64, ok: bool {
    #if T == float64 {
        stored :: 52;
        bias :: 1023;
        infinite :: 2047;
    } else {
        stored :: 23;
        bias :: 127;
        infinite :: 255;
    }

    if power < FIVE_POWER_LOWEST return 0, true;
    if power > FIVE_POWER_HIGHEST return cast(u64) infinite << stored, true;

    normalize := leading_zero_bits(significand);
    scaled := significand << normalize;
    index := 2 * (power - FIVE_POWER_LOWEST);

    upper, middle := multiply_64bits_with_overflow(scaled, FIVE_POWER_WORDS[index]);
    carry, lowest := multiply_64bits_with_overflow(scaled, FIVE_POWER_WORDS[index + 1]);

    middle += carry;
    if middle < carry upper += 1;

    // Both factors have their top bit set, so the product's top bit is bit 63 or 62 of `upper`.
    top := cast(s64) (upper >> 63);
    // Keep the leading one, the stored bits and one rounding bit.
    shift := top + 64 - stored - 3;
    below_mask := ((cast(u64) 1) << shift) - 1;
    below := upper & below_mask;

    if below == below_mask && middle == 0xffff_ffff_ffff_ffff return 0, false;

    // 5^0 through 5^55 fit in 128 bits, so their entries and the product are exact; any other
    // product that passed the check above lies strictly inside a rounding interval.
    exact := power >= 0 && power <= 55;
    sticky := !exact || below != 0 || middle != 0 || lowest != 0;
    kept := upper >> shift;

    // floor(power * log2(10)) is (power * 217706) >> 16 across the table's range.
    exponent := ((power * 217706) >> 16) + 63 + top - normalize + bias;

    if exponent <= 0 {
        // Below the normal range the unit stays at the smallest exponent, so more bits round away.
        extra := 1 - exponent;
        if extra >= 64 return 0, true;
        if (kept & ((cast(u64) 1 << extra) - 1)) != 0 sticky = true;

        kept >>= extra;
        round := kept & 1;

        kept >>= 1;
        if round != 0 && (sticky || (kept & 1) != 0) kept += 1;

        // A carry into the implicit bit position reads as the smallest normal number.
        return kept, true;
    }

    round := kept & 1;

    kept >>= 1;
    if round != 0 && (sticky || (kept & 1) != 0) kept += 1;

    if kept == (cast(u64) 1) << (stored + 1) {
        kept >>= 1;
        exponent += 1;
    }

    if exponent >= infinite return cast(u64) infinite << stored, true;

    return ((cast(u64) exponent) << stored) | (kept & (((cast(u64) 1) << stored) - 1)), true;
}

#scope_module
leading_zero_bits :: (value: u64) -> s64 {
    count: s64;
    rest := value;

    if rest >> 32 == 0 {
        count += 32;
        rest <<= 32;
    }

    if rest >> 48 == 0 {
        count += 16;
        rest <<= 16;
    }

    if rest >> 56 == 0 {
        count += 8;
        rest <<= 8;
    }

    if rest >> 60 == 0 {
        count += 4;
        rest <<= 4;
    }

    if rest >> 62 == 0 {
        count += 2;
        rest <<= 2;
    }

    if rest >> 63 == 0 count += 1;

    return count;
}
