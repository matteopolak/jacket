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
        converted = cast(float64) (cast(*Formatter_Extended_Float) item.value_pointer).*;
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

    if format.mode == .SHORTEST {
        limit := ifx wide then 17 else 9;
        for digits: 1..limit {
            candidate := portable_float(value, 103, digits);
            matches := candidate_matches(candidate, value, wide);

            // Normal powers of two have a smaller interval below than above.
            // The nearest decimal can miss below while its ceiling still fits.
            if !matches && fraction == 0 && exponent > 1 {
                free(candidate);
                candidate = portable_float(value, 103, digits, round_up = true);
                matches = candidate_matches(candidate, value, wide);
            }

            if matches {
                raw = candidate;
                break;
            }

            free(candidate);
        }

        assert(raw.data != null, "shortest float conversion failed to round-trip");
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

candidate_matches :: (text: string, value: float64, wide: bool) -> bool {
    if wide {
        parsed, ok, rest := string_to_float64(text);
        return ok && rest.count == 0 && parsed == value;
    }

    parsed, ok, rest := string_to_float(text);

    return ok && rest.count == 0 && parsed == cast(float32) value;
}
