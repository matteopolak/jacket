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

#scope_export

// Appends `text` as a JSON string with its quotes. Control characters, quotes and backslashes
// are escaped; bytes that are not valid UTF-8 become U+FFFD so the output is always valid JSON.
append_json_string :: (builder: *String_Builder, text: string) #no_abc {
    append(builder, #char "\"");

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

    if text.count > start append(builder, slice(text, start, text.count - start));

    append(builder, #char "\"");
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
    append(w.builder, #char "\n");

    for 1..w.depth append(w.builder, "  ");
}

#scope_export

json_write_null :: (w: *Json_Writer) {
    append(w.builder, "null");
}

json_write_bool :: (w: *Json_Writer, value: bool) {
    append(w.builder, ifx value then "true" else "false");
}

json_write_uint :: (w: *Json_Writer, value: u64) {
    buffer: [20] u8;
    at := 20;
    rest := value;

    if rest == 0 {
        at -= 1;
        buffer[at] = #char "0";
    }

    while rest {
        at -= 1;
        buffer[at] = cast,no_check(u8) (rest % 10) + #char "0";
        rest /= 10;
    }

    append(w.builder, *buffer[at], 20 - at);
}

json_write_int :: (w: *Json_Writer, value: s64) {
    if value >= 0 {
        json_write_uint(w, cast(u64) value);

        return;
    }

    append(w.builder, #char "-");
    json_write_uint(w, (~(cast,no_check(u64) value)) + 1);
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
    append(w.builder, text);
}

// `{`. Pair it with json_end_object and put a json_key before each member.
json_begin_object :: (w: *Json_Writer) {
    append(w.builder, #char "{");
    w.depth += 1;
}

json_end_object :: (w: *Json_Writer, empty: bool) {
    w.depth -= 1;

    if w.pretty && !empty write_newline(w);

    append(w.builder, #char "}");
}

json_begin_array :: (w: *Json_Writer) {
    append(w.builder, #char "[");
    w.depth += 1;
}

json_end_array :: (w: *Json_Writer, empty: bool) {
    w.depth -= 1;

    if w.pretty && !empty write_newline(w);

    append(w.builder, #char "]");
}

// Starts a member: the separator, then the key. `quoted_key` is the key with its quotes and
// escapes already applied (the generated code does that at compile time).
json_key :: (w: *Json_Writer, first: *bool, quoted_key: string) {
    if !first.* append(w.builder, #char ",");

    first.* = false;

    if w.pretty write_newline(w);

    append(w.builder, quoted_key);
    append(w.builder, ifx w.pretty then ": " else ":");
}

// json_key with the compact forms prebuilt at compile time: `first_form` is `"key":` and
// `next_form` is `,"key":`, so compact output costs one append.
json_key_fixed :: (w: *Json_Writer, first: *bool, quoted_key: string, first_form: string, next_form: string) {
    if w.pretty {
        json_key(w, first, quoted_key);

        return;
    }

    append(w.builder, ifx first.* then first_form else next_form);

    first.* = false;
}

// A member whose key is only known at run time (a Hash_Table's).
json_key_dynamic :: (w: *Json_Writer, first: *bool, key: string) {
    if !first.* append(w.builder, #char ",");

    first.* = false;

    if w.pretty write_newline(w);

    append_json_string(w.builder, key);
    append(w.builder, ifx w.pretty then ": " else ":");
}

// Starts an array element.
json_item :: (w: *Json_Writer, first: *bool) {
    if !first.* append(w.builder, #char ",");

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
            append(w.builder, value.text);
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
