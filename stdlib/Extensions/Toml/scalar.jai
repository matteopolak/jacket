#scope_module
// Validate once, including bytes inside comments and keys. Never normalize ill-formed input.
// Returns the offending byte and a static message; successful checks need no reader state.
toml_utf8_error :: (text: string) -> int, string {
    i := 0;
    while i < text.count {
        first := text[i];
        if first < 128 {
            i += 1;
            continue;
        }

        count := 0;
        scalar: u32;
        minimum: u32;

        if first >= 0xc2 && first <= 0xdf {
            count = 2;
            scalar = first & 31;
            minimum = 0x80;
        } else if first >= 0xe0 && first <= 0xef {
            count = 3;
            scalar = first & 15;
            minimum = 0x800;
        } else if first >= 0xf0 && first <= 0xf4 {
            count = 4;
            scalar = first & 7;
            minimum = 0x10000;
        } else return i, "invalid UTF-8";

        if count > text.count - i return i, "truncated UTF-8";

        for j: 1..count - 1 {
            byte := text[i + j];
            if (byte & 0xc0) != 0x80 return i + j, "invalid UTF-8 continuation";
            scalar = (scalar << 6) | (byte & 63);
        }

        if scalar < minimum || scalar > 0x10ffff || (scalar >= 0xd800 && scalar <= 0xdfff) return i, "invalid Unicode scalar";

        i += count;
    }

    return -1, "";
}

toml_validate_utf8 :: (r: *Toml_Reader) -> bool {
    at, message := toml_utf8_error(r.text);
    if at < 0 return true;
    return toml_fail(r, .ENCODING, message, at);
}

toml_hex :: (c: u8) -> int {
    if c >= 48 && c <= 57 return c - 48;
    if c >= 65 && c <= 70 return c - 65 + 10;
    if c >= 97 && c <= 102 return c - 97 + 10;
    return -1;
}

toml_append_scalar :: (builder: *String_Builder, scalar: u32) {
    bytes: [4] u8;
    count := 1;

    if scalar < 0x80 bytes[0] = cast(u8) scalar;
    else if scalar < 0x800 {
        count = 2;
        bytes[0] = cast(u8) (0xc0 | (scalar >> 6));
        bytes[1] = cast(u8) (0x80 | (scalar & 63));
    } else if scalar < 0x10000 {
        count = 3;
        bytes[0] = cast(u8) (0xe0 | (scalar >> 12));
        bytes[1] = cast(u8) (0x80 | ((scalar >> 6) & 63));
        bytes[2] = cast(u8) (0x80 | (scalar & 63));
    } else {
        count = 4;
        bytes[0] = cast(u8) (0xf0 | (scalar >> 18));
        bytes[1] = cast(u8) (0x80 | ((scalar >> 12) & 63));
        bytes[2] = cast(u8) (0x80 | ((scalar >> 6) & 63));
        bytes[3] = cast(u8) (0x80 | (scalar & 63));
    }

    append(builder, string.{ count, bytes.data });
}

// Stop at syntax, a control byte, or the first byte beyond the decoded budget. The input
// bounds prove each pointer load; valid ordinary UTF-8 needs no per-byte transformation.
// Example: end := toml_plain_string_end(text, start, 34, remaining);
toml_plain_string_end :: inline (text: string, start: int, quote: u8, remaining: int) -> int {
    cursor := start;
    while cursor < text.count {
        byte := text.data[cursor];
        if byte == quote || (byte == 92 && quote == 34) || byte == 10 || byte == 13 || (byte < 32 && byte != 9) || byte == 127 break;
        cursor += 1;
        if cursor - start > remaining break;
    }

    return cursor;
}

toml_string :: (r: *Toml_Reader, out: *string, key: bool = false) -> bool {
    quote := toml_peek(r);

    r.pos += 1;
    multiline := toml_peek(r) == quote && toml_peek(r, 1) == quote;
    if multiline {
        if key return toml_fail(r, .SYNTAX, "multiline keys are forbidden");
        r.pos += 2;
        toml_newline(r);
    }

    plain_start := r.pos;
    plain_end := toml_plain_string_end(r.text, plain_start, quote, r.options.max_string_bytes);
    count := plain_end - plain_start;

    r.pos = plain_end;
    if count > r.options.max_string_bytes return toml_fail(r, .LIMIT, "decoded string byte limit exceeded");

    plain := toml_finish_plain_string(r, out, quote, multiline, plain_start, count, key);
    if plain != 0 return plain > 0;

    // Only transformed strings need builder staging and its possible extra heap buffer.
    builder: String_Builder;

    defer free_buffers(*builder);
    if count append(*builder, slice(r.text, plain_start, count));

    while r.pos < r.text.count {
        c := toml_peek(r);

        if c == quote {
            run := 1;
            while toml_peek(r, run) == quote run += 1;

            if !multiline || run >= 3 {
                if multiline && run > 5 return toml_fail(r, .SYNTAX, "too many closing quotes");

                extra := ifx multiline then run - 3 else 0;
                if extra > r.options.max_string_bytes - count {
                    r.pos += r.options.max_string_bytes - count + 1;
                    return toml_fail(r, .LIMIT, "decoded string byte limit exceeded");
                }

                if extra append(*builder, slice(r.text, r.pos, extra));

                count += extra;

                r.pos += ifx multiline then run else 1;
                if builder.failed return toml_fail(r, .LIMIT, "string builder allocation failed");

                length := builder_string_length(*builder);
                if length > r.options.max_string_bytes return toml_fail(r, .LIMIT, "decoded string byte limit exceeded");
                if !toml_admit(*r.doc, length) return toml_fail(r, .LIMIT, "retained byte limit exceeded");

                out.* = builder_to_string(*builder);
                r.doc.retained_bytes += length;

                return true;
            }

            // One or two quotes are literal content inside a multiline string.
            if run > r.options.max_string_bytes - count {
                r.pos += r.options.max_string_bytes - count + 1;
                return toml_fail(r, .LIMIT, "decoded string byte limit exceeded");
            }

            append(*builder, slice(r.text, r.pos, run));
            r.pos += run;
            count += run;
            continue;
        }

        if c == 92 && quote == 34 {
            written := toml_string_escape(r, *builder, multiline, r.options.max_string_bytes - count);
            if written < 0 return false;
            count += written;
            continue;
        } else if c == 10 || c == 13 {
            if !multiline return toml_fail(r, .SYNTAX, "newline in single-line string");
            if !toml_newline(r) return false;
            if count >= r.options.max_string_bytes return toml_fail(r, .LIMIT, "decoded string byte limit exceeded");
            append(*builder, "\n");
        } else {
            if (c < 32 && c != 9) || c == 127 return toml_fail(r, .SYNTAX, "control character in string");

            written := toml_string_span(r, *builder, quote, r.options.max_string_bytes - count);
            if written < 0 return false;

            count += written;
            continue;
        }

        count += 1;
        if count > r.options.max_string_bytes return toml_fail(r, .LIMIT, "decoded string byte limit exceeded");
    }

    return toml_fail(r, .SYNTAX, "unclosed string");
}

// Finish a contiguous string without staging; zero requests transformed decoding.
toml_finish_plain_string :: (r: *Toml_Reader, out: *string, quote: u8, multiline: bool, plain_start: int, count: int, key: bool) -> int {
    if toml_peek(r) == quote {
        run := 1;
        if multiline while toml_peek(r, run) == quote run += 1;

        if !multiline || run >= 3 {
            if multiline && run > 5 {
                toml_fail(r, .SYNTAX, "too many closing quotes");
                return -1;
            }

            extra := ifx multiline then run - 3 else 0;
            if extra > r.options.max_string_bytes - count {
                r.pos += r.options.max_string_bytes - count + 1;
                toml_fail(r, .LIMIT, "decoded string byte limit exceeded");
                return -1;
            }

            length := count + extra;

            r.pos += ifx multiline then run else 1;

            if key && length > 0 {
                out.* = slice(r.doc.source, plain_start, length);
                return 1;
            }

            if !toml_admit(*r.doc, length) {
                toml_fail(r, .LIMIT, "retained byte limit exceeded");
                return -1;
            }

            // Values and empty keys keep independent payloads; plain nonempty keys use owned source.
            out.* = copy_string(slice(r.text, plain_start, length));
            r.doc.retained_bytes += length;

            return 1;
        }
    }

    return 0;
}

// Append one unchanged span after admitting its decoded byte count.
toml_string_span :: (r: *Toml_Reader, builder: *String_Builder, quote: u8, remaining: int) -> int {
    start := r.pos;
    cursor := toml_plain_string_end(r.text, start, quote, remaining);

    r.pos = cursor;

    if cursor - start > remaining {
        toml_fail(r, .LIMIT, "decoded string byte limit exceeded");
        return -1;
    }

    // Input UTF-8 was validated before grammar parsing; ASCII delimiters cannot
    // split a scalar. No over-budget run is staged in the builder.
    append(builder, slice(r.text, start, cursor - start));

    return cursor - start;
}

// Decode one basic-string escape. Zero means a trimmed multiline continuation;
// a positive result is the UTF-8 width already appended, and -1 preserves the reader error.
toml_string_escape :: (r: *Toml_Reader, builder: *String_Builder, multiline: bool, remaining: int) -> int {
    r.pos += 1;
    c := toml_peek(r);
    if multiline && (c == 32 || c == 9 || c == 10 || c == 13) {
        toml_space(r);

        if !toml_newline(r) {
            toml_fail(r, .SYNTAX, "continuation requires newline");
            return -1;
        }

        while true {
            toml_space(r);
            if !toml_newline(r) break;
        }

        return 0;
    }

    r.pos += 1;
    simple := c == 98 || c == 116 || c == 110 || c == 102 || c == 114 || c == 101 || c == 34 || c == 92;
    if simple && remaining < 1 {
        toml_fail(r, .LIMIT, "decoded string byte limit exceeded");
        return -1;
    }

    if c == 98 append(builder, "\b");
    else if c == 116 append(builder, "\t");
    else if c == 110 append(builder, "\n");
    else if c == 102 append(builder, "\f");
    else if c == 114 append(builder, "\r");
    else if c == 101 toml_append_scalar(builder, 27);
    else if c == 34 append(builder, "\"");
    else if c == 92 append(builder, "\\");
    else if c == 120 || c == 117 || c == 85 {
        digits := 2;
        if c == 117 digits = 4;
        if c == 85 digits = 8;

        scalar: u32;

        for 1..digits {
            digit := toml_hex(toml_peek(r));
            if digit < 0 {
                toml_fail(r, .SYNTAX, "expected hexadecimal escape digit");
                return -1;
            }

            scalar = (scalar << 4) | cast(u32) digit;
            r.pos += 1;
        }

        if scalar > 0x10ffff || (scalar >= 0xd800 && scalar <= 0xdfff) {
            toml_fail(r, .ENCODING, "escape is not a Unicode scalar");
            return -1;
        }

        written := ifx scalar < 0x80 then 1 else ifx scalar < 0x800 then 2 else ifx scalar < 0x10000 then 3 else 4;
        if written > remaining {
            toml_fail(r, .LIMIT, "decoded string byte limit exceeded");
            return -1;
        }

        toml_append_scalar(builder, scalar);

        return written;
    } else {
        toml_fail(r, .SYNTAX, "unknown string escape");
        return -1;
    }

    return 1;
}

toml_digit :: (c: u8) -> bool {
    return c >= 48 && c <= 57;
}

toml_digits :: (token: string, pos: *int, base: int) -> bool {
    start := pos.*;
    previous_digit := false;

    while pos.* < token.count {
        c := token[pos.*];
        if c == 95 {
            if !previous_digit || pos.* + 1 >= token.count return false;
            next := toml_hex(token[pos.* + 1]);
            if next < 0 || next >= base return false;
            previous_digit = false;
        } else {
            digit := toml_hex(c);
            if digit < 0 || digit >= base break;
            previous_digit = true;
        }

        pos.* += 1;
    }

    return pos.* > start && previous_digit;
}

toml_scalar :: (r: *Toml_Reader, token: string, start: int) -> *Toml_Value {
    if !token.count {
        toml_fail(r, .SYNTAX, "expected a value", start);
        return null;
    }

    if token == "true" || token == "false" {
        node := toml_node(r, .BOOL);
        if node node.boolean = token == "true";
        return node;
    }

    if (token.count >= 10 && token[4] == 45 && token[7] == 45) || (token.count >= 5 && token[2] == 58) return toml_temporal(r, token, start);

    if token.count > r.options.max_number_bytes {
        toml_fail(r, .LIMIT, "number token byte limit exceeded", start);
        return null;
    }

    negative := token[0] == 45;
    signed := negative || token[0] == 43;
    pos := ifx signed then 1 else 0;
    word := slice(token, pos, token.count - pos);

    if word == "inf" || word == "nan" {
        node := toml_node(r, .FLOAT);
        if !node return null;

        bits: u64 = ifx word == "inf" then 0x7ff0000000000000 else 0x7ff8000000000000;
        if negative bits |= 0x8000000000000000;

        node.real = cast,force(float64) bits;

        return node;
    }

    base := 10;

    if !signed && token.count > 2 && token[0] == 48 {
        if token[1] == 120 base = 16;
        if token[1] == 111 base = 8;
        if token[1] == 98 base = 2;
        if base != 10 pos = 2;
    }

    first_digit := pos;

    if !toml_digits(token, *pos, base) {
        toml_fail(r, .SYNTAX, "invalid number", start);
        return null;
    }

    if base == 10 && token[first_digit] == 48 && pos - first_digit > 1 {
        toml_fail(r, .SYNTAX, "leading zero in decimal number", start);
        return null;
    }

    is_float := false;

    if base == 10 && pos < token.count && token[pos] == 46 {
        is_float = true;
        pos += 1;
        if !toml_digits(token, *pos, 10) {
            toml_fail(r, .SYNTAX, "fraction needs digits", start);
            return null;
        }
    }

    if base == 10 && pos < token.count && (token[pos] == 101 || token[pos] == 69) {
        is_float = true;
        pos += 1;
        if pos < token.count && (token[pos] == 43 || token[pos] == 45) {
            pos += 1;
        }

        if !toml_digits(token, *pos, 10) {
            toml_fail(r, .SYNTAX, "exponent needs digits", start);
            return null;
        }
    }

    if pos != token.count {
        toml_fail(r, .SYNTAX, "invalid number suffix", start + pos);
        return null;
    }

    if is_float return toml_float(r, token, start);

    magnitude: u64;
    limit: u64 = ifx negative then 0x8000000000000000 else 0x7fffffffffffffff;

    for i: first_digit..token.count - 1 {
        if token[i] == 95 continue;

        digit := cast(u64) toml_hex(token[i]);
        if magnitude > (limit - digit) / cast(u64) base {
            toml_fail(r, .RANGE, "integer out of signed 64-bit range", start);
            return null;
        }

        magnitude = magnitude * cast(u64) base + digit;
    }

    node := toml_node(r, .INTEGER);
    if node node.integer = ifx negative then cast,no_check(s64) (~magnitude + 1) else cast(s64) magnitude;

    return node;
}

// Grammar is checked before conversion. Plain floats use the retained source directly;
// only valid underscore spellings need a normalized temporary string.
toml_float :: (r: *Toml_Reader, token: string, start: int) -> *Toml_Value {
    if !contains(token, "_") return toml_clean_float(r, token, start);

    normalized: String_Builder;

    defer free_buffers(*normalized);
    begin := 0;

    for token {
        if it != 95 continue;
        append(*normalized, slice(token, begin, it_index - begin));
        begin = it_index + 1;
    }

    append(*normalized, slice(token, begin, token.count - begin));

    if normalized.failed {
        toml_fail(r, .LIMIT, "number normalization allocation failed", start);
        return null;
    }

    clean := builder_to_string(*normalized);

    defer free(clean);

    return toml_clean_float(r, clean, start);
}

toml_clean_float :: (r: *Toml_Reader, clean: string, start: int) -> *Toml_Value {
    value, ok, remainder := string_to_float64(clean);
    if !ok || remainder.count || (((cast,force(u64) value) >> 52) & 0x7ff) == 0x7ff {
        toml_fail(r, .RANGE, "finite float overflow", start);
        return null;
    }

    node := toml_node(r, .FLOAT);
    if node node.real = value;

    return node;
}

toml_fixed_digits :: (token: string, pos: *int, count: int) -> int {
    if count > token.count - pos.* return -1;

    value := 0;

    for 1..count {
        c := token[pos.*];
        if !toml_digit(c) return -1;
        value = value * 10 + c - 48;
        pos.* += 1;
    }

    return value;
}

toml_take :: (token: string, pos: *int, c: u8) -> bool {
    if pos.* >= token.count || token[pos.*] != c return false;
    pos.* += 1;
    return true;
}

toml_temporal :: (r: *Toml_Reader, token: string, start: int) -> *Toml_Value {
    pos := 0;
    date := token.count >= 10 && token[4] == 45;
    record: Toml_Temporal;
    kind := Toml_Kind.LOCAL_TIME;

    if date {
        kind = .LOCAL_DATE;
        record.year = toml_fixed_digits(token, *pos, 4);

        if !toml_take(token, *pos, 45) {
            toml_fail(r, .SYNTAX, "invalid date", start);
            return null;
        }

        record.month = toml_fixed_digits(token, *pos, 2);

        if !toml_take(token, *pos, 45) {
            toml_fail(r, .SYNTAX, "invalid date", start);
            return null;
        }

        record.day = toml_fixed_digits(token, *pos, 2);
        if !toml_valid_date(record) {
            toml_fail(r, .RANGE, "invalid civil date", start);
            return null;
        }

        if pos < token.count {
            if token[pos] != 84 && token[pos] != 116 && token[pos] != 32 {
                toml_fail(r, .SYNTAX, "expected datetime separator", start + pos);
                return null;
            }

            pos += 1;
            kind = .LOCAL_DATETIME;
        }
    }

    if !date || kind == .LOCAL_DATETIME {
        record.hour = toml_fixed_digits(token, *pos, 2);

        if !toml_take(token, *pos, 58) {
            toml_fail(r, .SYNTAX, "invalid time", start + pos);
            return null;
        }

        record.minute = toml_fixed_digits(token, *pos, 2);
        has_seconds := toml_take(token, *pos, 58);
        if has_seconds record.second = toml_fixed_digits(token, *pos, 2);

        if !toml_valid_time(record) {
            toml_fail(r, .RANGE, "invalid civil time", start + pos);
            return null;
        }

        fraction_start := -1;

        if toml_take(token, *pos, 46) {
            if !has_seconds {
                toml_fail(r, .SYNTAX, "fraction requires seconds", start + pos - 1);
                return null;
            }

            fraction_start = pos;
            while pos < token.count && toml_digit(token[pos]) pos += 1;

            if fraction_start == pos {
                toml_fail(r, .SYNTAX, "fraction needs digits", start + pos);
                return null;
            }
        }

        fraction_end := pos;

        if date && pos < token.count {
            kind = .OFFSET_DATETIME;

            if !toml_take(token, *pos, 90) && !toml_take(token, *pos, 122) {
                negative := toml_take(token, *pos, 45);
                if !negative && !toml_take(token, *pos, 43) {
                    toml_fail(r, .SYNTAX, "invalid UTC offset", start + pos);
                    return null;
                }

                hours := toml_fixed_digits(token, *pos, 2);

                if !toml_take(token, *pos, 58) {
                    toml_fail(r, .SYNTAX, "invalid UTC offset", start + pos);
                    return null;
                }

                minutes := toml_fixed_digits(token, *pos, 2);
                if hours < 0 || hours > 23 || minutes < 0 || minutes > 59 {
                    toml_fail(r, .RANGE, "invalid UTC offset", start + pos);
                    return null;
                }

                record.offset_minutes = (hours * 60 + minutes) * ifx negative then -1 else 1;
                record.negative_zero_offset = negative && !record.offset_minutes;
            }
        }

        if pos != token.count {
            toml_fail(r, .SYNTAX, "invalid temporal suffix", start + pos);
            return null;
        }

        if fraction_start >= 0 {
            count := fraction_end - fraction_start;
            if count > r.options.max_string_bytes {
                toml_fail(r, .LIMIT, "decoded fraction byte limit exceeded");
                return null;
            }

            if !toml_admit(*r.doc, count) {
                toml_fail(r, .LIMIT, "retained byte limit exceeded");
                return null;
            }

            record.fraction = copy_string(slice(token, fraction_start, fraction_end - fraction_start));
            r.doc.retained_bytes += count;
        }
    }

    node := toml_node(r, kind);
    if !node {
        if record.fraction.data free(record.fraction);
        r.doc.retained_bytes -= record.fraction.count;
        return null;
    }

    node.temporal = record;

    return node;
}

toml_valid_date :: (record: Toml_Temporal) -> bool {
    days := 31;
    if record.month == 4 || record.month == 6 || record.month == 9 || record.month == 11 days = 30;

    if record.month == 2 {
        days = 28;
        if record.year % 4 == 0 && (record.year % 100 != 0 || record.year % 400 == 0) days = 29;
    }

    return record.year >= 0 && record.year <= 9999 && record.month >= 1 && record.month <= 12 && record.day >= 1 && record.day <= days;
}

toml_valid_time :: (record: Toml_Temporal) -> bool {
    return record.hour >= 0 && record.hour <= 23 && record.minute >= 0 && record.minute <= 59 && record.second >= 0 && record.second <= 60;
}
