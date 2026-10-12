// Numbers for Json: reading a validated token as a float (Basic does the correctly rounded
// conversion) and shortest round-trip printing of floats. Internal; the public pieces are in
// module.jai.

#scope_module

// A token that already passed the JSON grammar check, rounded to the nearest float64 (float32).
// `overflow` is set when it is beyond the largest finite value; the exponent is not limited, so
// 1e999999 overflows and 1e-999999 is zero.
float64_from_token :: (token: string) -> float64, bool {
    decimal: Decimal_Float = ---;

    scan_decimal_float(token, *decimal, limit_exponent = false);

    return decimal_float_to_float64(*decimal);
}

float32_from_token :: (token: string) -> float32, bool {
    decimal: Decimal_Float = ---;

    scan_decimal_float(token, *decimal, limit_exponent = false);

    return decimal_float_to_float32(*decimal);
}

// ---- printing ----

// Appends digits (no leading or trailing zeros, count >= 1) times 10^(point - count) in the
// shortest of JSON's spellings: plain for magnitudes 1e-6 up to 1e21, with a ".0" when it would
// otherwise read as an integer, and `1.5e-9` / `1e21` outside that.
append_decimal_float :: (builder: *String_Builder, negative: bool, digits: *u8, count: int, exponent10: int) {
    if negative append(builder, #char "-");

    // exponent10 is the exponent of the first digit: the value is d.ddd * 10^exponent10.
    if exponent10 < -6 || exponent10 >= 21 {
        append(builder, digits[0] + #char "0");

        if count > 1 {
            append(builder, #char ".");
            for 1..count - 1 append(builder, digits[it] + #char "0");
        }

        append(builder, #char "e");

        value := exponent10;

        if value < 0 {
            append(builder, #char "-");
            value = -value;
        }

        text: [4] u8;
        length := 0;

        if value == 0 {
            text[0] = #char "0";
            length = 1;
        } else {
            while value {
                text[length] = cast,no_check(u8) (value % 10) + #char "0";
                length += 1;
                value /= 10;
            }
        }

        i := length - 1;

        while i >= 0 {
            append(builder, text[i]);
            i -= 1;
        }

        return;
    }

    if exponent10 < 0 {
        append(builder, "0.");
        for 1..(-exponent10 - 1) append(builder, #char "0");
        for 0..count - 1 append(builder, digits[it] + #char "0");

        return;
    }

    whole := exponent10 + 1;

    for 0..whole - 1 append(builder, ifx it < count then digits[it] + #char "0" else cast(u8) #char "0");

    if count > whole {
        append(builder, #char ".");
        for whole..count - 1 append(builder, digits[it] + #char "0");
    } else {
        append(builder, ".0");
    }
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

    digits := __float_decimal_digits(magnitude, -1);

    append_decimal_float(builder, negative, digits.digits.data, digits.count, digits.exponent);
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
