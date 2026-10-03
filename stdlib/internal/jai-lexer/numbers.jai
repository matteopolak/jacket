has_a_big_exponent :: inline (value: float64) -> bool {
    encoded := ((cast,force(u64) value) >> 52) & 0x7ff;
    if encoded == 0 || encoded == 0x7ff return false;
    exponent := (cast(s64) encoded) - 1023;
    return exponent > 127 || exponent < -126;
}

make_hexfloat :: (lexer: *J.Lexer, result: *J.Token) -> *J.Token #no_aoc {
    if look(lexer, 0) == 48 && (look(lexer, 1) == 104 || look(lexer, 1) == 72) {
        eat_character(lexer);
        eat_character(lexer);
    }

    bits: u64;
    count: s64 = 0;

    while true {
        c := look(lexer, 0);
        if c == 95 {
            eat_character(lexer);
            continue;
        }

        digit := hex_value(c);
        if digit < 0 break;

        bits = (bits << 4) | cast(u64) digit;
        count += 1;
        eat_character(lexer);
    }

    result.type = .NUMBER;
    result.value_flags = .NUMBER | .HEX | .FLOAT;
    if count != 4 && count != 8 && count != 16 report_parse_error(lexer, "0h float bit patterns require 4, 8 or 16 digits");

    if count <= 8 result.float64_value = cast(float64) cast,force(float32) cast(u32) bits;
    else {
        result.float64_value = cast,force(float64) bits;
        result.value_flags |= .DEFAULTS_TO_FLOAT64;
    }

    set_end_of_token(lexer, result);

    return result;
}

make_number :: (lexer: *J.Lexer) -> *J.Token #no_aoc {
    result := get_unused_token(lexer);
    result.type = .NUMBER;
    result.value_flags = .NUMBER;
    start := lexer.input_cursor;

    if look(lexer, 0) == 48 && (look(lexer, 1) == 104 || look(lexer, 1) == 72) return make_hexfloat(lexer, result);

    base: u64 = 10;

    if look(lexer, 0) == 48 && (look(lexer, 1) == 120 || look(lexer, 1) == 88) {
        base = 16;
        result.value_flags |= .HEX;
        eat_character(lexer);
        eat_character(lexer);
    } else if look(lexer, 0) == 48 && (look(lexer, 1) == 98 || look(lexer, 1) == 66) {
        base = 2;
        result.value_flags |= .BINARY;
        eat_character(lexer);
        eat_character(lexer);
    }

    if base != 10 {
        count: s64 = 0;
        value: u64;
        overflow := false;

        while true {
            c := look(lexer, 0);
            if c == 95 {
                eat_character(lexer);
                continue;
            }

            digit := hex_value(c);
            if digit < 0 || cast(u64) digit >= base {
                if base == 2 && starts_number(c) report_parse_error(lexer, "invalid binary digit");
                break;
            }

            if value > (0xffffffffffffffff - cast(u64) digit) / base overflow = true;

            value = value * base + cast(u64) digit;
            count += 1;
            eat_character(lexer);
        }

        if count == 0 report_parse_error(lexer, "numeric prefix requires digits");

        if overflow {
            result.value_flags |= .OVERFLOWED;
            report_parse_error(lexer, "integer literal overflow");
        }

        result.integer_value = value;
        set_end_of_token(lexer, result);

        return result;
    }

    floating := false;
    fraction_start: s64 = -1;
    fraction_last_nonzero: s64 = -1;
    fraction_count: s64 = 0;

    while starts_number(look(lexer, 0)) || look(lexer, 0) == 95 eat_character(lexer);

    if look(lexer, 0) == 46 && look(lexer, 1) != 46 {
        floating = true;
        eat_character(lexer);
        fraction_start = lexer.input_cursor;

        while starts_number(look(lexer, 0)) || look(lexer, 0) == 95 {
            if look(lexer, 0) != 95 {
                fraction_count += 1;
                if look(lexer, 0) != 48 fraction_last_nonzero = fraction_count;
            }

            eat_character(lexer);
        }

        if look(lexer, 0) == 46 && look(lexer, 1) != 46 {
            eat_character(lexer);
            report_parse_error(lexer, "number has a second decimal point");
        }
    }

    if look(lexer, 0) == 101 || look(lexer, 0) == 69 {
        floating = true;
        eat_character(lexer);
        if look(lexer, 0) == 43 || look(lexer, 0) == 45 eat_character(lexer);

        count: s64 = 0;

        while starts_number(look(lexer, 0)) || look(lexer, 0) == 95 {
            if look(lexer, 0) != 95 count += 1;
            eat_character(lexer);
        }

        if count == 0 report_parse_error(lexer, "exponent requires decimal digits");
    }

    builder: Basic.String_Builder;

    Basic.init_string_builder(*builder);
    for i: start..lexer.input_cursor - 1 if lexer.input[i] != 95 Basic.append(*builder, lexer.input[i]);

    spelling := Basic.builder_to_string(*builder);

    defer Basic.free(spelling);
    defer Basic.free_buffers(*builder);

    if floating {
        value, success := Basic.string_to_float64(spelling);
        if !success report_parse_error(lexer, "malformed floating literal");

        result.float64_value = value;
        result.value_flags |= .FLOAT;
        precise := fraction_last_nonzero > 8;
        if precise result.value_flags |= .REQUIRES_FLOAT64_DUE_TO_SIGNIFICANT_DIGITS;
        if precise || has_a_big_exponent(value) result.value_flags |= .REQUIRES_FLOAT64;
        else if fraction_last_nonzero >= 8 result.value_flags |= .DEFAULTS_TO_FLOAT64;
    } else {
        value: u64;
        overflow := false;

        for c: spelling {
            digit := cast(u64) (c - 48);
            if value > (0xffffffffffffffff - digit) / 10 overflow = true;
            value = value * 10 + digit;
        }

        result.integer_value = value;

        if overflow {
            result.value_flags |= .OVERFLOWED;
            report_parse_error(lexer, "integer literal overflow");
        }
    }

    set_end_of_token(lexer, result);

    return result;
}

#scope_module
hex_value :: (c: s32) -> s32 {
    if c >= 48 && c <= 57 return c - 48;
    if c >= 65 && c <= 70 return c - 65 + 10;
    if c >= 97 && c <= 102 return c - 97 + 10;
    return -1;
}

#scope_file
J :: #import "Jai_Lexer";
Basic :: #import "Basic";
