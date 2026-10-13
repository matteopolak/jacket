// The run-time half of encoding: a writer over a String_Builder and the small procedures the
// generated encoders call. Internal except where module.jai exports it for `with=` handlers.

#scope_module

HEX_DIGITS :: "0123456789abcdef";

// How many bytes of `text` starting at `at` form one well-formed UTF-8 sequence (RFC 3629: no
// overlong forms, no surrogates, nothing above U+10FFFF), or 0.
utf8_sequence_length :: (text: string, at: int) -> int #no_abc {
    first := text[at];

    if first < 0x80 return 1;
    if first < 0xc2 return 0;

    left := text.count - at;

    if first < 0xe0 {
        if left >= 2 && (text[at + 1] & 0xc0) == 0x80 return 2;

        return 0;
    }

    if first < 0xf0 {
        if left < 3 return 0;

        second := text[at + 1];
        low: u8 = 0x80;
        high: u8 = 0xbf;

        if first == 0xe0 low = 0xa0;
        if first == 0xed high = 0x9f;
        if second < low || second > high return 0;
        if (text[at + 2] & 0xc0) != 0x80 return 0;

        return 3;
    }

    if first < 0xf5 {
        if left < 4 return 0;

        second := text[at + 1];
        low: u8 = 0x80;
        high: u8 = 0xbf;

        if first == 0xf0 low = 0x90;
        if first == 0xf4 high = 0x8f;
        if second < low || second > high return 0;
        if (text[at + 2] & 0xc0) != 0x80 || (text[at + 3] & 0xc0) != 0x80 return 0;

        return 4;
    }

    return 0;
}

#scope_module

// Output without per-piece checks. `builder_reserve` returns a pointer to `n` writable bytes at
// the end of the builder (null when the builder cannot give that many in one piece, in which case
// the caller uses plain `append`); the caller writes at most `n` of them and then calls
// `builder_commit` with the number it wrote. This replaces the generic append's initialisation
// test, copy loop and segment loop for the short pieces a JSON document is made of.
builder_reserve :: inline (builder: *String_Builder, n: s64) -> *u8 {
    if builder.initialized {
        segment := get_current_buffer(builder);

        if segment.allocated - segment.count >= n return get_buffer_data(segment) + segment.count;
    }

    return builder_reserve_slow(builder, n);
}

builder_reserve_slow :: (builder: *String_Builder, n: s64) -> *u8 {
    available := ensure_contiguous_space_and_return_available_range(builder, n);

    if available.count < n return null;

    return available.data;
}

builder_commit :: inline (builder: *String_Builder, n: s64) {
    get_current_buffer(builder).count += n;
}

// Copies `n` bytes (not overlapping): runs up to 32 bytes with overlapping word moves, longer ones with memcpy.
copy_bytes :: inline (destination: *u8, source: *u8, n: s64) {
    if n >= 8 {
        if n <= 16 {
            head := (cast(*u64) source).*;
            tail := (cast(*u64) (source + n - 8)).*;

            (cast(*u64) destination).* = head;
            (cast(*u64) (destination + n - 8)).* = tail;
        } else if n <= 32 {
            head0 := (cast(*u64) source).*;
            head1 := (cast(*u64) (source + 8)).*;
            tail0 := (cast(*u64) (source + n - 16)).*;
            tail1 := (cast(*u64) (source + n - 8)).*;

            (cast(*u64) destination).* = head0;
            (cast(*u64) (destination + 8)).* = head1;
            (cast(*u64) (destination + n - 16)).* = tail0;
            (cast(*u64) (destination + n - 8)).* = tail1;
        } else {
            memcpy(destination, source, n);
        }
    } else if n >= 4 {
        head := (cast(*u32) source).*;
        tail := (cast(*u32) (source + n - 4)).*;

        (cast(*u32) destination).* = head;
        (cast(*u32) (destination + n - 4)).* = tail;
    } else if n >= 2 {
        head := (cast(*u16) source).*;
        tail := (cast(*u16) (source + n - 2)).*;

        (cast(*u16) destination).* = head;
        (cast(*u16) (destination + n - 2)).* = tail;
    } else if n == 1 {
        destination.* = source.*;
    }
}

// Appends a short piece of text, as `append` would.
put :: inline (builder: *String_Builder, text: string) {
    out := builder_reserve(builder, text.count);

    if out {
        copy_bytes(out, text.data, text.count);
        builder_commit(builder, text.count);
    } else {
        append(builder, text);
    }
}

put :: inline (builder: *String_Builder, byte: u8) {
    out := builder_reserve(builder, 1);

    if out {
        out.* = byte;
        builder_commit(builder, 1);
    } else {
        append(builder, byte);
    }
}

#scope_export

// Appends `text` as a JSON string with its quotes. Control characters, quotes and backslashes
// are escaped; bytes that are not valid UTF-8 become U+FFFD so the output is always valid JSON.
append_json_string :: (builder: *String_Builder, text: string) #no_abc {
    // The opening quote is written when the first piece is (a string that needs no escaping,
    // the common case, is then one reservation: quote, bytes, quote).
    opened := false;
    start := 0;
    i := 0;
    scalar_until := 0;

    while i < text.count {
        if simd_enabled() && i >= scalar_until {
            // Sixteen bytes at a time while none needs attention: below 0x20, a quote, a
            // backslash, or 0x80 and up (which may or may not be valid UTF-8). After a byte
            // of 0x80 or more the next sixteen bytes are handled one by one, since text with
            // many such bytes does not repay the vector tests.
            while i + 16 <= text.count {
                chunk := load(u8x16, text.data + i);
                special := to_bits(lt(chunk, 0x20) | eq(chunk, #char "\"") | eq(chunk, #char "\\"));
                high := to_bits(ge(chunk, 0x80));

                if (special | high) == 0 {
                    i += 16;

                    continue;
                }

                i += trailing_zeros(special | high);

                if high != 0 && trailing_zeros(high) < trailing_zeros(special) scalar_until = i + 16;

                break;
            }

            if i >= text.count break;
        }

        c := text[i];
        replacement := false;
        step := 1;

        if c >= 0x20 && c != #char "\"" && c != #char "\\" {
            if c < 0x80 {
                i += 1;

                continue;
            }

            step = utf8_sequence_length(text, i);

            if step > 0 {
                i += step;

                continue;
            }

            step = 1;
            replacement = true;
        }

        if !opened {
            put(builder, #char "\"");
            opened = true;
        }

        if i > start append(builder, slice(text, start, i - start));

        if replacement {
            append(builder, cast(u8) 0xef);
            append(builder, cast(u8) 0xbf);
            append(builder, cast(u8) 0xbd);
        } else if c == #char "\"" {
            append(builder, "\\\"");
        } else if c == #char "\\" {
            append(builder, "\\\\");
        } else if c == #char "\n" {
            append(builder, "\\n");
        } else if c == #char "\r" {
            append(builder, "\\r");
        } else if c == #char "\t" {
            append(builder, "\\t");
        } else if c == 8 {
            append(builder, "\\b");
        } else if c == 12 {
            append(builder, "\\f");
        } else {
            append(builder, "\\u00");
            append(builder, HEX_DIGITS[c >> 4]);
            append(builder, HEX_DIGITS[c & 15]);
        }

        i += step;
        start = i;
    }

    if !opened {
        // Nothing needed escaping.
        out := builder_reserve(builder, text.count + 2);

        if out {
            out[0] = #char "\"";
            copy_bytes(out + 1, text.data, text.count);
            out[text.count + 1] = #char "\"";
            builder_commit(builder, text.count + 2);

            return;
        }

        put(builder, #char "\"");
    }

    if text.count > start append(builder, slice(text, start, text.count - start));

    put(builder, #char "\"");
}

#scope_module

#scope_export

Json_Writer :: struct {
    builder: *String_Builder;
    pretty: bool;
    depth: int;
}

#scope_module

write_newline :: (w: *Json_Writer) {
    put(w.builder, #char "\n");

    for 1..w.depth put(w.builder, "  ");
}

DIGIT_PAIRS :: "00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899";

// The decimal digits of `value`, ending just before `end`; returns where they start.
write_digits_backward :: inline (end: *u8, value: u64) -> *u8 {
    at := end;
    rest := value;

    // Two digits per step. DIGIT_PAIRS holds 200 bytes and `rest % 100` is below 100, so
    // `2 * pair + 1` is at most 199.
    while rest >= 100 {
        pair := cast(s64) (rest % 100);

        rest /= 100;
        at -= 2;
        at[0] = DIGIT_PAIRS.data[2 * pair];
        at[1] = DIGIT_PAIRS.data[2 * pair + 1];
    }

    if rest >= 10 {
        at -= 2;
        at[0] = DIGIT_PAIRS.data[2 * cast(s64) rest];
        at[1] = DIGIT_PAIRS.data[2 * cast(s64) rest + 1];
    } else {
        at -= 1;
        at[0] = cast,no_check(u8) rest + #char "0";
    }

    return at;
}

// How many decimal digits `value` has.
count_digits :: inline (value: u64) -> s64 {
    digits := 1;
    rest := value;

    while rest >= 10000 {
        rest /= 10000;
        digits += 4;
    }

    if rest >= 1000 return digits + 3;
    if rest >= 100 return digits + 2;
    if rest >= 10 return digits + 1;

    return digits;
}

#scope_export

json_write_null :: inline (w: *Json_Writer) {
    put(w.builder, "null");
}

json_write_bool :: inline (w: *Json_Writer, value: bool) {
    put(w.builder, ifx value then "true" else "false");
}

json_write_uint :: (w: *Json_Writer, value: u64) {
    out := builder_reserve(w.builder, 20);

    if !out {
        buffer: [20] u8 = ---;
        start := write_digits_backward(buffer.data + 20, value);

        append(w.builder, start, buffer.data + 20 - start);

        return;
    }

    digits := count_digits(value);

    write_digits_backward(out + digits, value);
    builder_commit(w.builder, digits);
}

json_write_int :: (w: *Json_Writer, value: s64) {
    if value >= 0 {
        json_write_uint(w, cast(u64) value);

        return;
    }

    magnitude := (~(cast,no_check(u64) value)) + 1;
    out := builder_reserve(w.builder, 21);

    if !out {
        put(w.builder, #char "-");
        json_write_uint(w, magnitude);

        return;
    }

    digits := count_digits(magnitude);

    out.* = #char "-";
    write_digits_backward(out + 1 + digits, magnitude);
    builder_commit(w.builder, digits + 1);
}

json_write_float64 :: (w: *Json_Writer, value: float64) {
    append_float64(w.builder, value);
}

json_write_float32 :: (w: *Json_Writer, value: float32) {
    append_float32(w.builder, value);
}

json_write_string :: (w: *Json_Writer, value: string) {
    append_json_string(w.builder, value);
}

// Appends text that is already JSON, as is.
json_write_raw :: (w: *Json_Writer, text: string) {
    put(w.builder, text);
}

// `{`. Pair it with json_end_object and put a json_key before each member.
json_begin_object :: inline (w: *Json_Writer) {
    put(w.builder, #char "{");
    w.depth += 1;
}

json_end_object :: (w: *Json_Writer, empty: bool) {
    w.depth -= 1;

    if w.pretty && !empty write_newline(w);

    put(w.builder, #char "}");
}

json_begin_array :: inline (w: *Json_Writer) {
    put(w.builder, #char "[");
    w.depth += 1;
}

json_end_array :: (w: *Json_Writer, empty: bool) {
    w.depth -= 1;

    if w.pretty && !empty write_newline(w);

    put(w.builder, #char "]");
}

// Starts a member: the separator, then the key. `quoted_key` is the key with its quotes and
// escapes already applied (the generated code does that at compile time).
json_key :: (w: *Json_Writer, first: *bool, quoted_key: string) {
    if !first.* put(w.builder, #char ",");

    first.* = false;

    if w.pretty write_newline(w);

    put(w.builder, quoted_key);
    put(w.builder, ifx w.pretty then ": " else ":");
}

// json_key with the compact forms prebuilt at compile time: `first_form` is `"key":` and
// `next_form` is `,"key":`, so compact output costs one append.
json_key_fixed :: inline (w: *Json_Writer, first: *bool, quoted_key: string, first_form: string, next_form: string) {
    if w.pretty {
        json_key(w, first, quoted_key);

        return;
    }

    put(w.builder, ifx first.* then first_form else next_form);

    first.* = false;
}

// A member whose key is only known at run time (a Hash_Table's).
json_key_dynamic :: (w: *Json_Writer, first: *bool, key: string) {
    if !first.* put(w.builder, #char ",");

    first.* = false;

    if w.pretty write_newline(w);

    append_json_string(w.builder, key);
    put(w.builder, ifx w.pretty then ": " else ":");
}

// Starts an array element.
json_item :: inline (w: *Json_Writer, first: *bool) {
    if !first.* put(w.builder, #char ",");

    first.* = false;

    if w.pretty write_newline(w);
}

json_write_value :: (w: *Json_Writer, value: *Json_Value) {
    if value.kind == {
        case .NULL;
            json_write_null(w);
        case .BOOL;
            json_write_bool(w, value.boolean);
        case .NUMBER;
            put(w.builder, value.text);
        case .STRING;
            json_write_string(w, value.text);
        case .ARRAY;
            json_begin_array(w);

            first := true;

            for *item: value.items {
                json_item(w, *first);
                json_write_value(w, item);
            }

            json_end_array(w, first);
        case .OBJECT;
            json_begin_object(w);

            first := true;

            for *member: value.members {
                json_key_dynamic(w, *first, member.key);
                json_write_value(w, *member.value);
            }

            json_end_object(w, first);
    }
}
