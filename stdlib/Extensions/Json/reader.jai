// The run-time half of decoding: a reader over the text and the small procedures the
// generated decoders call. The ones a `with=` handler may need are exported.

#scope_module

Path_Segment :: struct {
    key: string;
    index: int;
    is_key: bool;
}

#scope_export

// Reads one JSON document. Every reading procedure returns false after recording an error in
// `error`; the first error wins, and callers must stop at once and return false themselves.
Json_Reader :: struct {
    text: string;
    pos: int;
    depth: int;
    max_depth: int;
    error: Json_Error;
    key_start: int; // where the key last returned by json_object_next begins
    path: [..] Path_Segment;
    scratch: [..] u8;
    values: [..] Json_Value; // children of the arrays being read into a Json_Value, innermost last
    members: [..] Json_Member; // the same for objects
    index: [] u32; // stage 1's structural index, when `indexed` (see index.jai)
    ip: int; // the first index entry that may lie at or after `pos`
    indexed: bool;
}

Object_Status :: enum u8 {
    ERROR;
    KEY;
    END;
}

Array_Status :: enum u8 {
    ERROR;
    ITEM;
    END;
}

#scope_module

reader_init :: (r: *Json_Reader, text: string, options: Json_Options) {
    r.* = .{};
    r.text = text;
    r.max_depth = options.max_depth;
}

reader_free :: (r: *Json_Reader) {
    array_free(r.path);
    array_free(r.scratch);
    array_free(r.values);
    array_free(r.members);
}

#scope_export

// Records an error at the current position (when there is none yet) and returns false.
json_fail :: (r: *Json_Reader, kind: Json_Error_Kind, message: string) -> bool {
    return json_fail_at(r, kind, message, r.pos);
}

json_fail_at :: (r: *Json_Reader, kind: Json_Error_Kind, message: string, offset: int) -> bool {
    if r.error.kind == .NONE {
        r.error.kind = kind;
        r.error.message = copy_string(message);
        r.error.offset = offset;
    }

    return false;
}

// Say the value being read is at `key` / `index` of its parent. The generated decoders call
// these while an error travels up, so the error's path reads `address.tags[2]`.
json_path_key :: (r: *Json_Reader, key: string) {
    segment: Path_Segment;
    segment.is_key = true;
    segment.key = key;
    array_add(*r.path, segment);
}

json_path_index :: (r: *Json_Reader, index: int) {
    segment: Path_Segment;
    segment.index = index;
    array_add(*r.path, segment);
}

#scope_module

// Most values are not preceded by whitespace in compact documents, and by one byte or a short
// run in pretty-printed ones: the test for "nothing to skip" is inline, the loop is not.
skip_whitespace :: inline (r: *Json_Reader) {
    if r.pos < r.text.count && r.text.data[r.pos] > 0x20 return;

    skip_whitespace_run(r);
}

skip_whitespace_run :: (r: *Json_Reader) #no_abc {
    data := r.text.data;
    count := r.text.count;
    i := r.pos;

    while i < count {
        c := data[i];

        if c != #char " " && c != #char "\n" && c != #char "\t" && c != #char "\r" break;

        i += 1;
    }

    r.pos = i;
}

describe_byte :: (c: u8) -> string {
    if c >= 0x20 && c < 0x7f return tprint("`%`", string.{ 1, *c });

    return tprint("byte 0x%", FormatInt.{ value = c, base = 16, minimum_digits = 2 });
}

// "a string", "an object"...: what kind of value starts with `c`, or "" if none does.
value_name :: (c: u8) -> string {
    if c == {
        case #char "{"; return "an object";
        case #char "["; return "an array";
        case #char "\""; return "a string";
        case #char "t"; return "a boolean";
        case #char "f"; return "a boolean";
        case #char "n"; return "null";
    }

    if c == #char "-" || (c >= #char "0" && c <= #char "9") return "a number";

    return "";
}

// A syntax error: the next byte is not what the grammar allows.
unexpected :: (r: *Json_Reader, expected: string) -> bool {
    if r.pos >= r.text.count return json_fail(r, .SYNTAX, tprint("unexpected end of input, expected %", expected));

    return json_fail(r, .SYNTAX, tprint("unexpected %, expected %", describe_byte(r.text[r.pos]), expected));
}

// The next value is not of the type being read: `expected` is "an object", "an integer"...
wrong_type :: (r: *Json_Reader, expected: string) -> bool {
    c: u8;
    if r.pos < r.text.count c = r.text[r.pos];

    found := ifx r.pos < r.text.count then value_name(c) else "";

    if !found.count return unexpected(r, expected);

    return json_fail(r, .TYPE, tprint("invalid type: expected %, found %", expected, found));
}

#scope_export

// The next significant byte (0 at the end of the input).
json_peek :: inline (r: *Json_Reader) -> u8 {
    skip_whitespace(r);

    if r.pos >= r.text.count return 0;

    return r.text.data[r.pos];
}

#scope_module

enter_nesting :: (r: *Json_Reader) -> bool {
    r.depth += 1;

    if r.depth > r.max_depth return json_fail(r, .LIMIT, tprint("nesting is deeper than the limit of %", r.max_depth));

    return true;
}

#scope_export

json_duplicate_field :: (r: *Json_Reader, key: string) {
    json_fail_at(r, .DUPLICATE_FIELD, tprint("duplicate field `%`", key), r.key_start);
}

json_unknown_field :: (r: *Json_Reader, key: string) {
    json_fail_at(r, .UNKNOWN_FIELD, tprint("unknown field `%`", key), r.key_start);
}

json_missing_field :: (r: *Json_Reader, key: string) {
    json_fail_at(r, .MISSING_FIELD, tprint("missing field `%`", key), r.pos - 1);
}

json_begin_object :: (r: *Json_Reader) -> bool {
    if json_peek(r) != #char "{" return wrong_type(r, "an object");

    r.pos += 1;

    return enter_nesting(r);
}

// Reads up to the next member of the object: its key (valid until the next read) and the colon
// after it, or the closing brace. `first` is true before the first call.
json_object_next :: (r: *Json_Reader, first: *bool) -> string, Object_Status {
    c := json_peek(r);

    if first.* {
        first.* = false;

        if c == #char "}" {
            r.pos += 1;
            r.depth -= 1;

            return "", .END;
        }
    } else if c == #char "}" {
        r.pos += 1;
        r.depth -= 1;

        return "", .END;
    } else if c == #char "," {
        r.pos += 1;
        c = json_peek(r);
    } else {
        unexpected(r, "`,` or `}`");

        return "", .ERROR;
    }

    if c != #char "\"" {
        unexpected(r, "a string");

        return "", .ERROR;
    }

    r.key_start = r.pos;

    key, ok := json_read_name(r);

    if !ok return "", .ERROR;

    if json_peek(r) != #char ":" {
        unexpected(r, "`:`");

        return "", .ERROR;
    }

    r.pos += 1;
    skip_whitespace(r);

    return key, .KEY;
}

json_begin_array :: (r: *Json_Reader) -> bool {
    if json_peek(r) != #char "[" return wrong_type(r, "an array");

    r.pos += 1;

    return enter_nesting(r);
}

json_array_next :: (r: *Json_Reader, first: *bool) -> Array_Status {
    c := json_peek(r);

    if first.* {
        first.* = false;

        if c == #char "]" {
            r.pos += 1;
            r.depth -= 1;

            return .END;
        }
    } else if c == #char "]" {
        r.pos += 1;
        r.depth -= 1;

        return .END;
    } else if c == #char "," {
        r.pos += 1;
        json_peek(r);
    } else {
        unexpected(r, "`,` or `]`");

        return .ERROR;
    }

    return .ITEM;
}

#scope_module

hex_value :: (c: u8) -> int {
    if c >= #char "0" && c <= #char "9" return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;

    return -1;
}

// Four hex digits at `at`, or -1.
hex4 :: (text: string, at: int) -> int #no_abc {
    if at + 4 > text.count return -1;

    value := 0;

    for 0..3 {
        digit := hex_value(text[at + it]);

        if digit < 0 return -1;

        value = value * 16 + digit;
    }

    return value;
}

// Checks one string (the cursor is on the opening quote) and moves past it. Returns the raw
// text between the quotes and whether it has escapes.
scan_string :: (r: *Json_Reader) -> string, bool, bool #no_abc {
    text := r.text;

    if r.indexed {
        // An opening quote followed by its closing quote: no escapes, and stage 1 has already
        // checked the contents. A string with escapes is checked below.
        ip := r.ip;
        count := r.index.count;

        while ip < count && cast(int) (r.index[ip] & POSITION_MASK) < r.pos ip += 1;

        r.ip = ip;

        if ip + 1 < count && r.index[ip] == cast(u32) r.pos {
            next := r.index[ip + 1];

            if (next & CLOSE_FLAG) != 0 && (next & BACKSLASH_FLAG) == 0 {
                close := cast(int) (next & POSITION_MASK);
                start := r.pos + 1;

                r.pos = close + 1;
                r.ip = ip + 2;

                return slice(text, start, close - start), false, true;
            }
        }
    }

    i := r.pos + 1;
    start := i;
    escaped := false;
    scalar_until := 0;

    while true {
        if simd_enabled() && i >= scalar_until && i + 16 <= text.count {
            // Sixteen bytes at a time while all are plain ASCII; then the byte that needs a
            // closer look, and the next sixteen bytes one by one (text with many bytes of 0x80
            // and up does not repay the vector tests).
            chunk := load(u8x16, text.data + i);
            attention := to_bits(lt(chunk, 0x20) | eq(chunk, #char "\"") | eq(chunk, #char "\\") | ge(chunk, 0x80));

            if attention == 0 {
                i += 16;

                continue;
            }

            i += trailing_zeros(attention);

            if text[i] >= 0x80 scalar_until = i + 16;
        }

        if i >= text.count {
            json_fail_at(r, .SYNTAX, "unterminated string", i);

            return "", false, false;
        }

        c := text[i];

        if c == #char "\"" break;

        if c < 0x20 {
            json_fail_at(r, .SYNTAX, "control character in string", i);

            return "", false, false;
        }

        if c == #char "\\" {
            escaped = true;

            if i + 1 >= text.count {
                json_fail_at(r, .SYNTAX, "unterminated string", i + 1);

                return "", false, false;
            }

            e := text[i + 1];

            if e == #char "u" {
                code := hex4(text, i + 2);

                if code < 0 {
                    json_fail_at(r, .SYNTAX, "invalid \\u escape", i);

                    return "", false, false;
                }

                i += 6;

                if code >= 0xd800 && code <= 0xdbff {
                    low := -1;
                    if i + 1 < text.count && text[i] == #char "\\" && text[i + 1] == #char "u" low = hex4(text, i + 2);

                    if low < 0xdc00 || low > 0xdfff {
                        json_fail_at(r, .SYNTAX, "unpaired surrogate in \\u escape", i - 6);

                        return "", false, false;
                    }

                    i += 6;
                } else if code >= 0xdc00 && code <= 0xdfff {
                    json_fail_at(r, .SYNTAX, "unpaired surrogate in \\u escape", i - 6);

                    return "", false, false;
                }

                continue;
            }

            if e != #char "\"" && e != #char "\\" && e != #char "/" && e != #char "b" && e != #char "f" && e != #char "n" && e != #char "r" && e != #char "t" {
                json_fail_at(r, .SYNTAX, "invalid escape in string", i);

                return "", false, false;
            }

            i += 2;

            continue;
        }

        if c >= 0x80 {
            step := utf8_sequence_length(text, i);

            if step == 0 {
                json_fail_at(r, .SYNTAX, "invalid UTF-8 in string", i);

                return "", false, false;
            }

            i += step;

            continue;
        }

        i += 1;
    }

    r.pos = i + 1;

    return slice(text, start, i - start), escaped, true;
}

// The first backslash at or after `from`, or raw.count.
find_backslash :: (raw: string, from: int) -> int {
    i := from;

    // Escapes are often close together: look at the first few bytes before using vectors.
    stop := from + 8;

    if stop > raw.count stop = raw.count;

    while i < stop {
        if raw[i] == #char "\\" return i;

        i += 1;
    }

    if simd_enabled() {
        while i + 16 <= raw.count {
            found := to_bits(eq(load(u8x16, raw.data + i), #char "\\"));

            if found != 0 return i + trailing_zeros(found);

            i += 16;
        }
    }

    while i < raw.count {
        if raw[i] == #char "\\" return i;

        i += 1;
    }

    return raw.count;
}

append_utf8 :: (dest: *u8, code: int) -> int {
    if code < 0x80 {
        dest[0] = cast,no_check(u8) code;

        return 1;
    }

    if code < 0x800 {
        dest[0] = cast,no_check(u8) (0xc0 | (code >> 6));
        dest[1] = cast,no_check(u8) (0x80 | (code & 0x3f));

        return 2;
    }

    if code < 0x10000 {
        dest[0] = cast,no_check(u8) (0xe0 | (code >> 12));
        dest[1] = cast,no_check(u8) (0x80 | ((code >> 6) & 0x3f));
        dest[2] = cast,no_check(u8) (0x80 | (code & 0x3f));

        return 3;
    }

    dest[0] = cast,no_check(u8) (0xf0 | (code >> 18));
    dest[1] = cast,no_check(u8) (0x80 | ((code >> 12) & 0x3f));
    dest[2] = cast,no_check(u8) (0x80 | ((code >> 6) & 0x3f));
    dest[3] = cast,no_check(u8) (0x80 | (code & 0x3f));

    return 4;
}

// Resolves the escapes of a string that scan_string accepted. The result is never longer
// than `raw`.
unescape_into :: (raw: string, dest: *u8) -> int {
    out := 0;
    i := 0;

    while i < raw.count {
        next := find_backslash(raw, i);

        if next > i {
            memcpy(dest + out, raw.data + i, next - i);
            out += next - i;
            i = next;

            if i >= raw.count break;
        }

        c := raw[i];

        if c != #char "\\" {
            dest[out] = c;
            out += 1;
            i += 1;

            continue;
        }

        e := raw[i + 1];

        i += 2;

        if e == #char "u" {
            code := hex4(raw, i);

            i += 4;

            if code >= 0xd800 && code <= 0xdbff {
                low := hex4(raw, i + 2);
                i += 6;
                code = 0x10000 + ((code - 0xd800) << 10) + (low - 0xdc00);
            }

            out += append_utf8(*dest[out], code);

            continue;
        }

        if e == #char "n" dest[out] = #char "\n";
        else if e == #char "t" dest[out] = #char "\t";
        else if e == #char "r" dest[out] = #char "\r";
        else if e == #char "b" dest[out] = 8;
        else if e == #char "f" dest[out] = 12;
        else dest[out] = e; // " \ /

        out += 1;
    }

    return out;
}

#scope_export

// A string value, copied into new memory from context.allocator.
json_read_string :: (r: *Json_Reader) -> string, bool {
    if json_peek(r) != #char "\"" return "", wrong_type(r, "a string");

    raw, escaped, ok := scan_string(r);

    if !ok return "", false;
    if raw.count == 0 return "", true;

    memory := alloc_string(raw.count);

    if !escaped {
        memcpy(memory.data, raw.data, raw.count);

        return memory, true;
    }

    memory.count = unescape_into(raw, memory.data);

    return memory, true;
}

// A string value without copying: a view of the input, or of the reader's scratch space when
// it has escapes. Valid until the next read; copy it to keep it.
json_read_name :: (r: *Json_Reader) -> string, bool {
    if json_peek(r) != #char "\"" return "", wrong_type(r, "a string");

    raw, escaped, ok := scan_string(r);

    if !ok return "", false;
    if !escaped return raw, true;

    array_reserve(*r.scratch, raw.count);
    r.scratch.count = unescape_into(raw, r.scratch.data);

    return string.{ r.scratch.count, r.scratch.data }, true;
}

json_read_bool :: (r: *Json_Reader) -> bool, bool {
    c := json_peek(r);

    if c == #char "t" {
        ok := read_literal(r, "true");

        return true, ok;
    }

    if c == #char "f" {
        ok := read_literal(r, "false");

        return false, ok;
    }

    return false, wrong_type(r, "a boolean");
}

json_next_is_null :: (r: *Json_Reader) -> bool {
    return json_peek(r) == #char "n";
}

json_read_null :: (r: *Json_Reader) -> bool {
    json_peek(r);

    return read_literal(r, "null");
}

#scope_module

// Consumes `word` at the cursor. A mismatch is a syntax error; the return value is true on
// success (so `true` reads as its own value).
read_literal :: (r: *Json_Reader, word: string) -> bool {
    if r.pos + word.count <= r.text.count && slice(r.text, r.pos, word.count) == word {
        r.pos += word.count;

        return true;
    }

    return json_fail(r, .SYNTAX, tprint("invalid literal, expected `%`", word));
}

// Checks the number grammar of RFC 8259 and returns the token's text. Every access is guarded by
// `i < count`, so the bounds checks are off.
scan_number :: (r: *Json_Reader) -> string, bool #no_abc {
    data := r.text.data;
    count := r.text.count;
    start := r.pos;
    i := start;

    if i < count && data[i] == #char "-" i += 1;

    if i >= count {
        json_fail_at(r, .SYNTAX, "invalid number", i);

        return "", false;
    }

    if data[i] == #char "0" {
        i += 1;
    } else if data[i] >= #char "1" && data[i] <= #char "9" {
        i += 1;

        while i < count && data[i] >= #char "0" && data[i] <= #char "9" i += 1;
    } else {
        json_fail_at(r, .SYNTAX, "invalid number", i);

        return "", false;
    }

    if i < count && data[i] == #char "." {
        i += 1;
        digits := i;

        while i < count && data[i] >= #char "0" && data[i] <= #char "9" i += 1;

        if i == digits {
            json_fail_at(r, .SYNTAX, "invalid number: a digit must follow the decimal point", i);

            return "", false;
        }
    }

    if i < count && (data[i] == #char "e" || data[i] == #char "E") {
        i += 1;

        if i < count && (data[i] == #char "+" || data[i] == #char "-") i += 1;

        digits := i;

        while i < count && data[i] >= #char "0" && data[i] <= #char "9" i += 1;

        if i == digits {
            json_fail_at(r, .SYNTAX, "invalid number: a digit must follow the exponent", i);

            return "", false;
        }
    }

    r.pos = i;

    return string.{ i - start, data + start }, true;
}

starts_number :: inline (c: u8) -> bool {
    return c == #char "-" || (c >= #char "0" && c <= #char "9");
}

// An integer token (optional minus, digits only) as a sign and a magnitude; false for anything
// else or a magnitude above u64.
parse_integer_token :: (token: string) -> bool, u64, bool #no_aoc {
    negative := false;
    i := 0;

    if token.count && token[0] == #char "-" {
        negative = true;
        i = 1;
    }

    if i >= token.count return false, 0, false;

    magnitude: u64;

    // Up to 19 digits cannot overflow a u64, so the common case needs no check per digit.
    if token.count - i <= 19 {
        while i < token.count {
            digit := cast(u64) token.data[i] - 48;

            if digit >= 10 return false, 0, false;

            magnitude = magnitude * 10 + digit;
            i += 1;
        }

        return negative, magnitude, true;
    }

    while i < token.count {
        c := token[i];

        if c < #char "0" || c > #char "9" return false, 0, false;

        digit := cast(u64) (c - #char "0");

        if magnitude > (0xffff_ffff_ffff_ffff - digit) / 10 return false, 0, false;

        magnitude = magnitude * 10 + digit;
        i += 1;
    }

    return negative, magnitude, true;
}

#scope_export

// Reads an integer into `out`, which must fit between `low` and `high`. `type_name` appears in
// the message when it does not.
json_read_signed :: (r: *Json_Reader, out: *s64, low: s64, high: s64, type_name: string) -> bool {
    if !starts_number(json_peek(r)) return wrong_type(r, "an integer");

    start := r.pos;
    token, ok := scan_number(r);

    if !ok return false;

    negative, magnitude, integral := parse_integer_token(token);

    if !integral {
        if find_index_from_left(token, #char ".") < 0 && find_index_from_left(token, #char "e") < 0 && find_index_from_left(token, #char "E") < 0 {
            return json_fail_at(r, .OUT_OF_RANGE, tprint("number `%` is out of range for %", token, type_name), start);
        }

        return json_fail_at(r, .TYPE, tprint("invalid type: expected an integer, found the number `%`", token), start);
    }

    if negative {
        if magnitude > 0x8000_0000_0000_0000 return json_fail_at(r, .OUT_OF_RANGE, tprint("number `%` is out of range for %", token, type_name), start);

        value := cast,no_check(s64) (~magnitude + 1);

        if value < low return json_fail_at(r, .OUT_OF_RANGE, tprint("number `%` is out of range for %", token, type_name), start);

        out.* = value;

        return true;
    }

    if magnitude > 0x7fff_ffff_ffff_ffff || cast,no_check(s64) magnitude > high {
        return json_fail_at(r, .OUT_OF_RANGE, tprint("number `%` is out of range for %", token, type_name), start);
    }

    out.* = cast,no_check(s64) magnitude;

    return true;
}

json_read_unsigned :: (r: *Json_Reader, out: *u64, high: u64, type_name: string) -> bool {
    if !starts_number(json_peek(r)) return wrong_type(r, "an integer");

    start := r.pos;
    token, ok := scan_number(r);

    if !ok return false;

    negative, magnitude, integral := parse_integer_token(token);

    if !integral {
        if find_index_from_left(token, #char ".") < 0 && find_index_from_left(token, #char "e") < 0 && find_index_from_left(token, #char "E") < 0 {
            return json_fail_at(r, .OUT_OF_RANGE, tprint("number `%` is out of range for %", token, type_name), start);
        }

        return json_fail_at(r, .TYPE, tprint("invalid type: expected an integer, found the number `%`", token), start);
    }

    if (negative && magnitude != 0) || magnitude > high {
        return json_fail_at(r, .OUT_OF_RANGE, tprint("number `%` is out of range for %", token, type_name), start);
    }

    out.* = magnitude;

    return true;
}

json_read_float64 :: (r: *Json_Reader, out: *float64) -> bool {
    if !starts_number(json_peek(r)) return wrong_type(r, "a number");

    start := r.pos;
    token, ok := scan_number(r);

    if !ok return false;

    value, overflow := float64_from_token(token);

    if overflow return json_fail_at(r, .OUT_OF_RANGE, tprint("number `%` is out of range for float64", token), start);

    out.* = value;

    return true;
}

json_read_float32 :: (r: *Json_Reader, out: *float32) -> bool {
    if !starts_number(json_peek(r)) return wrong_type(r, "a number");

    start := r.pos;
    token, ok := scan_number(r);

    if !ok return false;

    value, overflow := float32_from_token(token);

    if overflow return json_fail_at(r, .OUT_OF_RANGE, tprint("number `%` is out of range for float32", token), start);

    out.* = value;

    return true;
}

// Checks the next value and moves past it without keeping anything.
json_skip_value :: (r: *Json_Reader) -> bool {
    c := json_peek(r);

    if c == {
        case #char "{";
            if !json_begin_object(r) return false;

            first := true;

            while true {
                _, status := json_object_next(r, *first);

                if status == .ERROR return false;
                if status == .END return true;
                if !json_skip_value(r) return false;
            }
        case #char "[";
            if !json_begin_array(r) return false;

            first := true;

            while true {
                status := json_array_next(r, *first);

                if status == .ERROR return false;
                if status == .END return true;
                if !json_skip_value(r) return false;
            }
        case #char "\"";
            _, _, ok := scan_string(r);

            return ok;
        case #char "t";
            return read_literal(r, "true");
        case #char "f";
            return read_literal(r, "false");
        case #char "n";
            return read_literal(r, "null");
    }

    if c == #char "-" || (c >= #char "0" && c <= #char "9") {
        _, ok := scan_number(r);

        return ok;
    }

    return unexpected(r, "a value");
}

#scope_module

// The children collected since `base`, moved into one allocation of exactly their size.
take_values :: (r: *Json_Reader, base: int) -> [..] Json_Value {
    result: [..] Json_Value;
    count := r.values.count - base;

    if count == 0 return result;

    result.data = alloc(count * size_of(Json_Value));
    result.count = count;
    result.allocated = count;
    result.allocator = context.allocator;

    memcpy(result.data, r.values.data + base, count * size_of(Json_Value));
    r.values.count = base;

    return result;
}

take_members :: (r: *Json_Reader, base: int) -> [..] Json_Member {
    result: [..] Json_Member;
    count := r.members.count - base;

    if count == 0 return result;

    result.data = alloc(count * size_of(Json_Member));
    result.count = count;
    result.allocated = count;
    result.allocator = context.allocator;

    memcpy(result.data, r.members.data + base, count * size_of(Json_Member));
    r.members.count = base;

    return result;
}

#scope_export

// Reads any value into a Json_Value. On failure `out` holds what was read so far, and
// free_json releases it.
json_read_value :: (r: *Json_Reader, out: *Json_Value) -> bool {
    c := json_peek(r);

    if c == {
        case #char "{";
            if !json_begin_object(r) return false;

            out.kind = .OBJECT;

            base := r.members.count;
            first := true;
            ok := true;

            while true {
                key, status := json_object_next(r, *first);

                if status == .ERROR {
                    ok = false;

                    break;
                }

                if status == .END break;

                member: Json_Member;
                member.key = copy_string(key);
                ok = json_read_value(r, *member.value);

                array_add(*r.members, member);

                if !ok break;
            }

            out.members = take_members(r, base);

            return ok;
        case #char "[";
            if !json_begin_array(r) return false;

            out.kind = .ARRAY;

            base := r.values.count;
            first := true;
            ok := true;

            while true {
                status := json_array_next(r, *first);

                if status == .ERROR {
                    ok = false;

                    break;
                }

                if status == .END break;

                item: Json_Value;

                ok = json_read_value(r, *item);

                array_add(*r.values, item);

                if !ok break;
            }

            out.items = take_values(r, base);

            return ok;
        case #char "\"";
            text, ok := json_read_string(r);

            out.kind = .STRING;
            out.text = text;

            return ok;
        case #char "t";
            out.kind = .BOOL;
            out.boolean = true;

            return read_literal(r, "true");
        case #char "f";
            out.kind = .BOOL;

            return read_literal(r, "false");
        case #char "n";
            out.kind = .NULL;

            return read_literal(r, "null");
    }

    if c == #char "-" || (c >= #char "0" && c <= #char "9") {
        token, ok := scan_number(r);

        if !ok return false;

        out.kind = .NUMBER;
        out.text = copy_string(token);

        return true;
    }

    return unexpected(r, "a value");
}

#scope_module

// Fills in the line, column and path of a recorded error.
finish_error :: (r: *Json_Reader) -> Json_Error {
    error := r.error;

    line := 1;
    column := 1;
    i := 0;
    limit := error.offset;

    if limit > r.text.count limit = r.text.count;

    while i < limit {
        c := r.text[i];

        if c == #char "\n" {
            line += 1;
            column = 1;
        } else if (c & 0xc0) != 0x80 {
            column += 1;
        }

        i += 1;
    }

    error.line = line;
    error.column = column;

    builder: String_Builder;

    i = r.path.count - 1;

    while i >= 0 {
        segment := r.path[i];

        if segment.is_key {
            if i != r.path.count - 1 append(*builder, #char ".");

            append(*builder, segment.key);
        } else {
            append(*builder, "[");
            print(*builder, "%", segment.index);
            append(*builder, "]");
        }

        i -= 1;
    }

    error.path = builder_to_string(*builder);

    return error;
}

// Unaligned little-endian loads for the generated key matchers.
json_load64 :: inline (p: *u8) -> u64 {
    return (cast(*u64) p).*;
}

json_load32 :: inline (p: *u8) -> u64 {
    return (cast(*u32) p).*;
}

json_load16 :: inline (p: *u8) -> u64 {
    return (cast(*u16) p).*;
}

json_load8 :: inline (p: *u8) -> u64 {
    return p.*;
}
