is_hex_digit :: (c: u32) -> bool {
    return (c >= #char "0" && c <= #char "9") || (c >= #char "a" && c <= #char "f") || (c >= #char "A" && c <= #char "F");
}

is_valid_int_literal_suffix :: (suffix: string) -> bool {
    if suffix.count > 0 && (suffix[0] == #char "u" || suffix[0] == #char "U") suffix = slice(suffix, 1, suffix.count - 1);
    return suffix == "" || suffix == "z" || suffix == "Z" || suffix == "l" || suffix == "L" || suffix == "ll" || suffix == "LL";
}

is_valid_float_literal_suffix :: (suffix: string) -> bool {
    return suffix == "" || suffix == "f" || suffix == "F";
}

convert_float_literal :: (c_value: string) -> value: string, success: bool {
    index: s64;
    while index < c_value.count && literal_digit(c_value[index]) index += 1;

    digits := index;
    dot := false;
    trailing_dot := false;

    if index < c_value.count && c_value[index] == #char "." {
        dot = true;
        index += 1;
        start := index;

        while index < c_value.count && literal_digit(c_value[index]) index += 1;

        trailing_dot = index == start;
        digits += index - start;
    }

    if digits == 0 return "", false;

    exponent_start := index;
    exponent := false;

    if index < c_value.count && (c_value[index] == #char "e" || c_value[index] == #char "E") {
        exponent = true;
        index += 1;
        if index < c_value.count && (c_value[index] == #char "+" || c_value[index] == #char "-") index += 1;

        start := index;

        while index < c_value.count && literal_digit(c_value[index]) index += 1;
        if index == start return "", false;
    }

    if (!dot && !exponent) || !is_valid_float_literal_suffix(slice(c_value, index, c_value.count - index)) return "", false;

    mantissa := slice(c_value, 0, exponent_start);
    if !dot return tprint("%.0%", mantissa, slice(c_value, exponent_start, index - exponent_start)), true;
    if trailing_dot return tprint("%0%", mantissa, slice(c_value, exponent_start, index - exponent_start)), true;

    return slice(c_value, 0, index), true;
}

convert_literal_value :: (c_value: string) -> value: string, success: bool, was_string := false {
    if !c_value return "", false, false;

    if c_value[0] == #char "\"" {
        if c_value.count < 2 || c_value[c_value.count - 1] != #char "\"" return "", false, false;
        for index: 1..c_value.count - 2 if c_value[index] == #char "\"" return "", false, false;
        return c_value, true, true;
    }

    if c_value.count >= 2 && c_value[0] == #char "0" && (c_value[1] == #char "x" || c_value[1] == #char "X") {
        index: s64 = 2;
        while index < c_value.count && is_hex_digit(c_value[index]) index += 1;
        if index == 2 || !is_valid_int_literal_suffix(slice(c_value, index, c_value.count - index)) return "", false, false;
        return tprint("0x%", slice(c_value, 2, index - 2)), true, false;
    }

    for c: c_value if c == #char "." || c == #char "e" || c == #char "E" {
        text, ok := convert_float_literal(c_value);
        return text, ok, false;
    }

    if c_value[0] == #char "0" {
        index: s64 = 1;
        base: u64 = 8;
        binary := c_value.count >= 2 && (c_value[1] == #char "b" || c_value[1] == #char "B");

        if binary {
            index = 2;
            base = 2;
        }

        start := index;
        number: u64;

        while index < c_value.count && literal_digit(c_value[index]) {
            digit := cast(u64) (c_value[index] - #char "0");
            if digit >= base || number > (0xffffffffffffffff - digit) / base return "", false, false;
            number = number * base + digit;
            index += 1;
        }

        if (binary && index == start) || !is_valid_int_literal_suffix(slice(c_value, index, c_value.count - index)) return "", false, false;
        if binary return tprint("0b%", slice(c_value, start, index - start)), true, false;

        return tprint("0x%", formatInt(number, base = 16)), true, false;
    }

    index: s64;
    while index < c_value.count && literal_digit(c_value[index]) index += 1;
    if index == 0 || !is_valid_int_literal_suffix(slice(c_value, index, c_value.count - index)) return "", false, false;

    return slice(c_value, 0, index), true, false;
}

#scope_file
#import "Basic";
#import "String";

literal_digit :: (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
}
