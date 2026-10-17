print_float :: (builder: *String_Builder, format: FormatFloat) {
    emit_float(builder, format);
}

stb_print_float :: (builder: *String_Builder, format: FormatFloat) -> int {
    before := builder_string_length(builder);
    emit_float(builder, format);
    return builder_string_length(builder) - before;
}

#scope_file
// The compiler selects x87 or binary128 for this type on wide targets.
// Identity, rather than storage size, determines whether this conversion applies.
Formatter_Extended_Float :: #jaic_type long_double;

emit_float :: (builder: *String_Builder, format: FormatFloat) {
    item := format.value;
    if !item.type || !item.value_pointer {
        append(builder, "(null Any)");
        return;
    }

    item.type = formatter_base_type(item.type);
    assert(item.type.type == .FLOAT, "FormatFloat requires a float value");
    converted: float64;

    if item.type.runtime_size != 4 && item.type.runtime_size != 8 {
        assert(item.type == type_info(Formatter_Extended_Float), "unsupported reflected float representation");
        converted = extended_to_float64(item.value_pointer);
        item = Any.{ type_info(float64), *converted };
    }

    bits := get_u64_from_void_pointer(item.value_pointer, item.type.runtime_size);
    wide := item.type.runtime_size == 8;
    negative := ifx wide then (bits >> 63) != 0 else (bits >> 31) != 0;
    exponent := ifx wide then (bits >> 52) & 0x7ff else (bits >> 23) & 0xff;
    fraction := ifx wide then bits & 0x000fffffffffffff else bits & 0x7fffff;
    prefix: u8;

    if negative prefix = 45;
    else if format.positive_number_prefix == .PLUS prefix = 43;
    else if format.positive_number_prefix == .SPACE prefix = 32;

    if exponent == (ifx wide then 0x7ff else 0xff) {
        text := ifx fraction != 0 then "NaN" else "Inf";
        count := text.count;

        if prefix != 0 count += 1;

        formatter_repeat(builder, 32, max(cast(s64) 0, format.width - count));
        if prefix != 0 append(builder, prefix);

        append(builder, text);

        return;
    }

    value: float64;
    if wide value = (cast(*float64) item.value_pointer).*;
    else value = cast(float64) (cast(*float32) item.value_pointer).*;
    if negative value = -value;

    precision := format.trailing_width;
    if precision < 0 precision = 6;

    conversion: u8 = 102;
    if format.mode == .SCIENTIFIC conversion = 101;
    else if format.mode == .SHORTEST conversion = 103;

    raw: string;
    integer_limit: float64 = 9007199254740992.0;

    power_of_two := fraction == 0 && exponent > 1;

    if format.mode == .SHORTEST {
        raw = shortest_text(value, wide, power_of_two);
    } else if format.mode == .DECIMAL && format.trailing_width < 0 && value >= integer_limit {
        raw = padded_integer(value);
    } else raw = portable_float(value, conversion, precision);

    defer free(raw);
    text := raw;
    point: s64 = -1;
    exponent_at := text.count;

    for byte, index: text {
        if byte == 46 point = index;

        if byte == 101 || byte == 69 {
            exponent_at = index;
            break;
        }
    }

    fraction_end := exponent_at;
    removed: s64;
    zero_mode := format.zero_removal;

    if format.mode == .SHORTEST zero_mode = .YES;

    if point >= 0 && zero_mode != .NO {
        minimum := point + 1;
        if zero_mode == .ONE_ZERO_AFTER_DECIMAL minimum += 1;
        while fraction_end > minimum && text[fraction_end - 1] == 48 fraction_end -= 1;
        if fraction_end == point + 1 fraction_end = point;

        removed = exponent_at - fraction_end;
    }

    trailing_spaces := ifx format.trailing_width >= 0 && format.mode != .SHORTEST then removed else 0;
    integer_end := ifx point >= 0 then point else exponent_at;
    separators: s64;

    if format.thousand_separators && integer_end > 0 separators = (integer_end - 1) / 3;

    count := fraction_end + (text.count - exponent_at) + separators + trailing_spaces;
    if prefix != 0 count += 1;

    formatter_repeat(builder, 32, max(cast(s64) 0, format.width - count));
    if prefix != 0 append(builder, prefix);

    for i: 0..fraction_end - 1 {
        if i > 0 && i < integer_end && format.thousand_separators && (integer_end - i) % 3 == 0 append(builder, format.thousand_separator_character);
        byte := text[i];
        if byte == 46 byte = format.decimal_separator_character;
        append(builder, byte);
    }

    for i: exponent_at..text.count - 1 {
        byte := text[i];
        if i == exponent_at byte = format.exponent_character;
        append(builder, byte);
    }

    formatter_repeat(builder, 32, trailing_spaces);
}

// The fewest significant digits that read back as `value`, as printed by the `g` conversion.
shortest_text :: (value: float64, wide: bool, power_of_two: bool) -> string {
    raw: string;
    limit := ifx wide then 17 else 9;
    low := 1;
    high := limit;
    matched_digits := 0;

    // A digit count that round-trips keeps doing so with more digits, so the smallest one is
    // found by bisection instead of trying every count from one.
    while low < high {
        middle := (low + high) / 2;
        candidate, matches := shortest_candidate(value, middle, wide, power_of_two);

        if matches {
            free(raw);
            raw = candidate;
            matched_digits = middle;
            high = middle;
        } else {
            free(candidate);
            low = middle + 1;
        }
    }

    if matched_digits != low {
        free(raw);

        candidate, matches := shortest_candidate(value, low, wide, power_of_two);

        if matches raw = candidate;
        else free(candidate);
    }

    assert(raw.data != null, "shortest float conversion failed to round-trip");

    return raw;
}

// A value too large to hold fractional digits keeps its first 18 significant digits and the rest
// are zeros, so the digits past that precision are not the exact integer the bits encode.
padded_integer :: (value: float64) -> string {
    scientific := portable_float(value, 101, 17);

    defer free(scientific);

    builder: String_Builder;

    defer free_buffers(*builder);

    digits: s64;
    exponent_value: s64;
    seen_exponent := false;
    negative_exponent := false;

    for byte: scientific {
        if seen_exponent {
            if byte == 45 negative_exponent = true;
            else if byte >= 48 && byte <= 57 exponent_value = exponent_value * 10 + cast(s64) (byte - 48);
        } else if byte == 101 || byte == 69 {
            seen_exponent = true;
        } else if byte >= 48 && byte <= 57 {
            append(*builder, byte);
            digits += 1;
        }
    }

    if negative_exponent exponent_value = -exponent_value;

    formatter_repeat(*builder, 48, max(cast(s64) 0, exponent_value + 1 - digits));

    return builder_to_string(*builder);
}

// The text of `value` with `digits` significant digits, and whether it reads back as `value`.
// Normal powers of two have a smaller interval below than above: the nearest decimal can miss
// below while its ceiling still fits.
shortest_candidate :: (value: float64, digits: s64, wide: bool, power_of_two: bool) -> string, bool {
    candidate := portable_float(value, 103, digits);
    matches := candidate_matches(candidate, value, wide);

    if !matches && power_of_two {
        free(candidate);
        candidate = portable_float(value, 103, digits, round_up = true);
        matches = candidate_matches(candidate, value, wide);
    }

    return candidate, matches;
}

candidate_matches :: (text: string, value: float64, wide: bool) -> bool {
    if wide {
        parsed, ok, rest := string_to_float64(text);
        return ok && rest.count == 0 && parsed == value;
    }

    parsed, ok, rest := string_to_float(text);

    return ok && rest.count == 0 && parsed == cast(float32) value;
}

// Reads a long_double in memory without the target's runtime library: a wasm module that has no
// support for 128-bit floats must still be able to format values. x87 extended precision is used
// on x86-64 and IEEE binary128 elsewhere; both reduce to a sign, a 15-bit exponent and a 64-bit
// significand, and the single u64 to float64 conversion rounds it.
extended_to_float64 :: (pointer: *void) -> float64 {
    words := cast(*u64) pointer;
    low := words[0];
    high := words[1];

    negative: bool;
    exponent: s64;
    significand: u64;
    has_payload: bool;

    #if CPU == .X64 {
        negative = (high & 0x8000) != 0;
        exponent = cast(s64) (high & 0x7fff);
        significand = low;
        has_payload = (low << 1) != 0;
    } else {
        fraction_high := high & 0xffff_ffff_ffff;

        negative = (high >> 63) != 0;
        exponent = cast(s64) ((high >> 48) & 0x7fff);
        significand = (fraction_high << 15) | (low >> 49);
        // Bits below the 64-bit significand only decide ties, so they collapse into its last bit.
        if (low & ((cast(u64) 1 << 49) - 1)) != 0 significand |= 1;
        if exponent != 0 significand |= cast(u64) 1 << 63;

        has_payload = fraction_high != 0 || low != 0;
    }

    sign_bit: u64 = ifx negative then (cast(u64) 1 << 63) else 0;

    if exponent == 0x7fff {
        bits := sign_bit | 0x7ff0_0000_0000_0000;
        if has_payload bits |= 0x0008_0000_0000_0000;

        return (cast(*float64) *bits).*;
    }

    if significand == 0 return (cast(*float64) *sign_bit).*;

    if exponent == 0 exponent = 1;

    scale := exponent - 16383 - 63;

    // Outside float64's range after the 64-bit significand: overflow or underflow to zero.
    if scale > 1023 + 64 {
        bits := sign_bit | 0x7ff0_0000_0000_0000;

        return (cast(*float64) *bits).*;
    }

    if scale < -1074 - 64 return (cast(*float64) *sign_bit).*;

    result := cast(float64) significand;

    while scale != 0 {
        step := max(-1000, min(1000, scale));
        factor_bits := cast(u64) (step + 1023) << 52;

        result *= (cast(*float64) *factor_bits).*;
        scale -= step;
    }

    return ifx negative then -result else result;
}
