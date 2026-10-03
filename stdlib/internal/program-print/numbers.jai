// Canonical spelling preserves lexer precision metadata, not just rounded output.
hex_bits :: (builder: *Basic.String_Builder, bits: u64, count: int) {
    digits :: "0123456789abcdef";
    Basic.append(builder, "0h");
    for i: 0..count - 1 Basic.append(builder, digits[(bits >> ((count - 1 - i) * 4)) & 15]);
}

floating :: (builder: *Basic.String_Builder, literal: *C.Code_Literal) -> bool {
    value := literal._float64;

    if literal.value_flags & .HEX {
        bits64 := literal._u64;

        if literal.value_flags & .MINUS_SIGN {
            Basic.append(builder, "-");
            value = -value;
            bits64 ^= 0x8000_0000_0000_0000;
        }

        if literal.value_flags & .DEFAULTS_TO_FLOAT64 hex_bits(builder, bits64, 16);
        else {
            narrow := cast,no_check(float32) value;
            wide := cast(float64) narrow;

            if (cast(*u64) *wide).* != (cast(*u64) *value).* return false;

            bits := (cast(*u32) *narrow).*;

            hex_bits(builder, cast(u64) bits, 8);
        }

        return true;
    }

    format := Basic.formatFloat(value, trailing_width = 16, mode = .SCIENTIFIC, zero_removal = .NO);
    text := Basic.sprint("%", format);

    defer Basic.free(text);
    // Nonfinite decimal values come from overflow, rather than a fabricated result.
    if value != value return false;

    if value != 0 && value == value * 2 {
        if value < 0 Basic.append(builder, "-");
        if literal.value_flags & .REQUIRES_FLOAT64 Basic.append(builder, "1.000000001e999");
        else Basic.append(builder, "1.0e999");
        return true;
    }

    coefficient: [64] u8;
    count, fraction, exponent: int;
    after_point, negative_exponent: bool;
    i: int;

    if text.count && text[0] == #char "-" {
        Basic.append(builder, "-");
        i = 1;
    }

    while i < text.count && text[i] != #char "e" && text[i] != #char "E" {
        c := text[i];

        i += 1;

        if c == #char "." {
            after_point = true;
            continue;
        }

        if c < #char "0" || c > #char "9" || count == coefficient.count return false;

        coefficient[count] = c;
        count += 1;
        if after_point fraction += 1;
    }

    if i < text.count {
        i += 1;
        if i < text.count && (text[i] == #char "-" || text[i] == #char "+") {
            negative_exponent = text[i] == #char "-";
            i += 1;
        }

        while i < text.count {
            c := text[i];
            if c < #char "0" || c > #char "9" return false;
            exponent = exponent * 10 + cast(int) (c - #char "0");
            i += 1;
        }
    }

    if negative_exponent exponent = -exponent;

    exponent -= fraction;

    while count > 1 && coefficient[count - 1] == #char "0" {
        count -= 1;
        exponent += 1;
    }

    wanted: int;
    if literal.value_flags & .REQUIRES_FLOAT64 wanted = 9;
    else if literal.value_flags & .DEFAULTS_TO_FLOAT64 wanted = 8;

    if value == 0 && wanted > 0 {
        // Underflow provides the same zero bits with the requested precision flags.
        Basic.append(builder, "0.");
        for j: 1..wanted - 1 Basic.append(builder, "0");

        Basic.append(builder, "1e-999");

        return true;
    }

    if wanted == 0 {
        for j: 0..count - 1 Basic.append(builder, coefficient[j]);
        Basic.append(builder, ".0");
    } else {
        if count <= wanted {
            Basic.append(builder, "0.");
            for j: 1..wanted - count Basic.append(builder, "0");
            for j: 0..count - 1 Basic.append(builder, coefficient[j]);
        } else {
            for j: 0..count - 1 {
                if j == count - wanted Basic.append(builder, ".");
                Basic.append(builder, coefficient[j]);
            }
        }
    }

    Basic.print_to_builder(builder, "e%", exponent + wanted);

    return true;
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
