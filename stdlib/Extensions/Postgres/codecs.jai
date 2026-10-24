// Converting between Postgres values (text or binary, by type OID) and Jai values.

// The type OIDs this client knows by number. Other types (enums, domains, composites) get their
// OIDs when they are created, so they are only known by name.
Oid :: enum u32 {
    BOOL :: 16;
    BYTEA :: 17;
    CHAR :: 18;
    NAME :: 19;
    INT8 :: 20;
    INT2 :: 21;
    INT4 :: 23;
    TEXT :: 25;
    OID :: 26;
    JSON :: 114;
    JSON_ARRAY :: 199;
    FLOAT4 :: 700;
    FLOAT8 :: 701;
    UNKNOWN :: 705;
    BPCHAR :: 1042;
    VARCHAR :: 1043;
    DATE :: 1082;
    TIME :: 1083;
    TIMESTAMP :: 1114;
    TIMESTAMPTZ :: 1184;
    INTERVAL :: 1186;
    NUMERIC :: 1700;
    UUID :: 2950;
    JSONB :: 3802;

    BOOL_ARRAY :: 1000;
    BYTEA_ARRAY :: 1001;
    CHAR_ARRAY :: 1002;
    NAME_ARRAY :: 1003;
    INT2_ARRAY :: 1005;
    INT4_ARRAY :: 1007;
    TEXT_ARRAY :: 1009;
    BPCHAR_ARRAY :: 1014;
    VARCHAR_ARRAY :: 1015;
    INT8_ARRAY :: 1016;
    OID_ARRAY :: 1028;
    FLOAT4_ARRAY :: 1021;
    FLOAT8_ARRAY :: 1022;
    TIMESTAMP_ARRAY :: 1115;
    DATE_ARRAY :: 1182;
    TIME_ARRAY :: 1183;
    TIMESTAMPTZ_ARRAY :: 1185;
    INTERVAL_ARRAY :: 1187;
    NUMERIC_ARRAY :: 1231;
    UUID_ARRAY :: 2951;
    JSONB_ARRAY :: 3807;
}

Format :: enum u16 {
    TEXT :: 0;
    BINARY :: 1;
}

#scope_module

// Decodes one non-null value into `out`. Returns "" on success, otherwise why it failed.
decode :: (bytes: [] u8, format: Format, oid: u32, out: *$T, json: *Json_Layers = null) -> string {
    text := cast(string) bytes;
    binary := format == .BINARY;

    #if T == bool {
        if binary {
            if bytes.count != 1 return "a binary bool is one byte";
            out.* = bytes[0] != 0;
        } else {
            if text == "t" out.* = true;
            else if text == "f" out.* = false;
            else return tprint("\"%\" is not a bool", text);
        }

        return "";
    } else #if type_info(T).type == .ENUM {
        // A Postgres enum is its label in both formats; a generated enum spells it with
        // `enum_member_name`.
        member := enum_member_name(text);
        info := cast(*Type_Info_Enum) type_info(T);

        for info.names if it == member {
            out.* = cast,no_check(T) info.values[it_index];
            return "";
        }

        return tprint("\"%\" is not a label of %", text, T);
    } else #if type_info(T).type == .INTEGER {
        value: s64;

        if binary {
            if bytes.count == 2 value = cast,no_check(s16) read_u16(bytes.data);
            else if bytes.count == 4 && oid == xx Oid.OID value = read_u32(bytes.data);
            else if bytes.count == 4 value = cast,no_check(s32) read_u32(bytes.data);
            else if bytes.count == 8 value = cast,no_check(s64) read_u64(bytes.data);
            else return tprint("a binary % is not an integer", type_name(oid));
        } else {
            valid: bool;
            rest: string;
            value, valid, rest = string_to_int(text, 10, s64);
            if !valid || rest return tprint("\"%\" is not an integer", text);
        }

        info := cast(*Type_Info_Integer) type_info(T);
        bits := size_of(T) * 8;

        if bits < 64 {
            low := ifx info.signed then -(1 << (bits - 1)) else 0;
            high := ifx info.signed then (1 << (bits - 1)) - 1 else (1 << bits) - 1;
            if value < low || value > high return tprint("% does not fit in %", value, T);
        } else if !info.signed && value < 0 {
            return tprint("% does not fit in %", value, T);
        }

        out.* = cast,no_check(T) value;

        return "";
    } else #if type_info(T).type == .FLOAT {
        value: float64;

        if binary {
            if bytes.count == 4 {
                bits := read_u32(bytes.data);
                value = (cast(*float32) *bits).*;
            } else if bytes.count == 8 {
                bits := read_u64(bytes.data);
                value = (cast(*float64) *bits).*;
            } else {
                return tprint("a binary % is not a float", type_name(oid));
            }
        } else {
            if text == "NaN" value = FLOAT64_NAN;
            else if text == "Infinity" value = FLOAT64_INFINITY;
            else if text == "-Infinity" value = -FLOAT64_INFINITY;
            else {
                valid: bool;
                rest: string;
                value, valid, rest = string_to_float64(text);
                if !valid || rest return tprint("\"%\" is not a number", text);
            }
        }

        out.* = cast(T) value;

        return "";
    } else #if #run is_json(T) {
        document, why := json_document(bytes, format, oid);
        if why return why;

        #if type_of(out.value) == Json_Raw {
            out.value.text = document;
            return "";
        }

        layers: Json_Layers;
        if json layers = json.*;

        value, error := Json_Module.from_json(type_of(out.value), document, module_json_options(layers, T.OPTIONS));

        if error.kind != .NONE {
            why := tprint("at %: %", Json_Module.json_path_text(error), error.message);
            Json_Module.free_json(*error);
            return why;
        }

        out.value = value;

        return "";
    } else #if T == Json_Raw {
        document, why := json_document(bytes, format, oid);
        out.text = document;

        return why;
    } else #if T == string {
        if binary {
            if oid == xx Oid.JSONB {
                document, why := json_document(bytes, format, oid);
                if why return why;
                text = document;
            } else if !is_text_like(oid) {
                return tprint("binary % is not text; read it as [] u8, or use text results", type_name(oid));
            }
        }

        // The text form of the "char" byte 0 is an empty string.
        if binary && oid == xx Oid.CHAR && bytes.count == 1 && bytes[0] == 0 text = "";

        out.* = text;

        return "";
    } else #if T == [] u8 {
        if !binary && oid == xx Oid.BYTEA {
            decoded, why := decode_bytea_text(text);
            if why return why;
            out.* = decoded;
        } else {
            out.* = bytes;
        }

        return "";
    } else #if T == Uuid {
        if binary {
            if bytes.count != 16 return "a binary uuid is 16 bytes";
            memcpy(out.bytes.data, bytes.data, 16);
        } else {
            valid: bool;
            out.*, valid = parse_uuid(text);
            if !valid return tprint("\"%\" is not a uuid", text);
        }

        return "";
    } else #if T == Timestamp || T == Local_Timestamp {
        if binary {
            if bytes.count != 8 return "a binary timestamp is 8 bytes";
            out.microseconds = cast,no_check(s64) read_u64(bytes.data);
            return "";
        }

        valid: bool;
        out.microseconds, valid = parse_timestamp_text(text, T == Timestamp);
        if !valid return tprint("\"%\" is not a timestamp", text);

        return "";
    } else #if T == Date {
        if binary {
            if bytes.count != 4 return "a binary date is 4 bytes";
            out.days = cast,no_check(s32) read_u32(bytes.data);
            return "";
        }

        days, valid := parse_date_text(text);
        if !valid return tprint("\"%\" is not a date", text);

        out.days = days;

        return "";
    } else #if T == Time {
        if binary {
            if bytes.count != 8 return "a binary time is 8 bytes";
            out.microseconds = cast,no_check(s64) read_u64(bytes.data);
            return "";
        }

        r := text;
        valid := parse_clock(*r, *out.microseconds);

        if !valid || r return tprint("\"%\" is not a time", text);

        return "";
    } else #if T == Interval {
        if binary {
            if bytes.count != 16 return "a binary interval is 16 bytes";

            out.microseconds = cast,no_check(s64) read_u64(bytes.data);
            out.days = cast,no_check(s32) read_u32(bytes.data + 8);
            out.months = cast,no_check(s32) read_u32(bytes.data + size_of(s64) + size_of(s32));

            return "";
        }

        // Servers from 17 have interval infinities: every field at its limit.
        if text == "infinity" {
            out.* = .{ S32_MAX, S32_MAX, S64_MAX };
            return "";
        }

        if text == "-infinity" {
            out.* = .{ S32_MIN, S32_MIN, S64_MIN };
            return "";
        }

        if !parse_interval_text(text, out) return tprint("\"%\" is not an interval in IntervalStyle postgres", text);

        return "";
    } else #if T == Numeric {
        if binary return decode_numeric_binary(bytes, out);
        out.text = text;
        return "";
    } else #if type_info(T).type == .ARRAY {
        if binary return decode_array_binary(bytes, out, json);
        return decode_array_text(text, oid, out, json);
    } else {
        #assert false "Postgres cannot decode into this type";
        return "";
    }
}

is_text_like :: (oid: u32) -> bool {
    o := cast(Oid) oid;
    return o == .TEXT || o == .VARCHAR || o == .BPCHAR || o == .NAME || o == .CHAR || o == .JSON || o == .UNKNOWN;
}

// Array type OIDs and their element type OIDs, for the types this client knows.
ARRAY_TYPES :: struct {
    array: Oid;
    element: Oid;
} .[
.{ .BOOL_ARRAY, .BOOL }, .{ .BYTEA_ARRAY, .BYTEA }, .{ .INT2_ARRAY, .INT2 }, .{ .INT4_ARRAY, .INT4 },
.{ .INT8_ARRAY, .INT8 }, .{ .TEXT_ARRAY, .TEXT }, .{ .VARCHAR_ARRAY, .VARCHAR },
.{ .FLOAT4_ARRAY, .FLOAT4 }, .{ .FLOAT8_ARRAY, .FLOAT8 }, .{ .TIMESTAMP_ARRAY, .TIMESTAMP },
.{ .TIMESTAMPTZ_ARRAY, .TIMESTAMPTZ }, .{ .DATE_ARRAY, .DATE }, .{ .NUMERIC_ARRAY, .NUMERIC },
.{ .UUID_ARRAY, .UUID }, .{ .JSON_ARRAY, .JSON }, .{ .JSONB_ARRAY, .JSONB },
.{ .CHAR_ARRAY, .CHAR }, .{ .NAME_ARRAY, .NAME }, .{ .BPCHAR_ARRAY, .BPCHAR }, .{ .OID_ARRAY, .OID },
.{ .TIME_ARRAY, .TIME }, .{ .INTERVAL_ARRAY, .INTERVAL },
];

element_oid :: (array_oid: u32) -> u32 {
    for ARRAY_TYPES if it.array == xx array_oid return xx it.element;
    return 0;
}

array_oid_of :: (element: u32) -> u32 {
    for ARRAY_TYPES if it.element == xx element return xx it.array;
    return 0;
}

type_name :: (oid: u32) -> string {
    names :: string.[
        "bool", "bytea", "char", "name", "int8", "int2", "int4", "text", "oid", "json", "float4",
        "float8", "unknown", "bpchar", "varchar", "date", "time", "timestamp", "timestamptz",
        "interval", "numeric", "uuid", "jsonb",
    ];
    oids :: u32.[16, 17, 18, 19, 20, 21, 23, 25, 26, 114, 700, 701, 705, 1042, 1043, 1082, 1083, 1114, 1184, 1186, 1700, 2950, 3802];

    for oids if it == oid return names[it_index];
    if element_oid(oid) return tprint("%[]", type_name(element_oid(oid)));

    return tprint("type %", oid);
}

decode_array_binary :: (bytes: [] u8, out: *[] $Element, json: *Json_Layers) -> string {
    r := Cursor.{ data = bytes };
    dimensions := get_s32(*r);

    get_s32(*r); // has nulls
    element := get_u32(*r);
    count: s64;

    if dimensions == 1 {
        count = get_s32(*r);
        get_s32(*r); // lower bound
    } else if dimensions > 1 {
        return "multi-dimensional arrays are not supported";
    }

    if r.bad || count < 0 return "a malformed binary array";

    // Each element has at least its 4-byte length: a count the data cannot hold is never allocated.
    if count > (r.data.count - r.at) / 4 return "a truncated binary array";

    result := NewArray(count, Element);
    failed := true;

    defer if failed array_free(result);

    for *result {
        length := get_s32(*r);

        if r.bad return "a truncated binary array";

        if length < 0 {
            #if #run is_nullable(Element) {
                it.null = true;
                continue;
            } else {
                return tprint("element % is NULL; read the array as [] Nullable(%)", it_index + 1, Element);
            }
        }

        value := get_bytes(*r, length);

        if r.bad return "a truncated binary array";

        #if #run is_nullable(Element) why := decode(value, .BINARY, element, *it.value, json);
        else why := decode(value, .BINARY, element, it, json);
        if why return tprint("element %: %", it_index + 1, why);
    }

    failed = false;
    out.* = result;

    return "";
}

decode_array_text :: (text: string, oid: u32, out: *[] $Element, json: *Json_Layers) -> string {
    element := element_oid(oid);
    r := text;

    // Arrays with lower bounds other than 1 start with "[l:u]=".
    if begins_with(r, "[") {
        equals := find_index_from_left(r, #char "=");
        if equals < 0 return "a malformed array";
        r = advance(r, equals + 1);
    }

    if !begins_with(r, "{") || !ends_with(r, "}") return tprint("\"%\" is not an array", text);

    r = slice(r, 1, r.count - 2);

    items: [..] Element;

    while r {
        quoted := false;
        value: string;

        if r[0] == #char "{" return "multi-dimensional arrays are not supported";

        if r[0] == #char "\"" {
            quoted = true;
            i := 1;
            needs_copy := false;

            while i < r.count && r[i] != #char "\"" {
                if r[i] == #char "\\" {
                    needs_copy = true;
                    i += 1;
                }

                i += 1;
            }

            if i >= r.count return "an unterminated quoted array element";

            value = slice(r, 1, i - 1);
            if needs_copy value = unescape_array_element(value);

            r = advance(r, i + 1);
        } else {
            comma := find_index_from_left(r, #char ",");
            value = ifx comma < 0 then r else slice(r, 0, comma);
            r = advance(r, value.count);
            value = trim(value);
        }

        if begins_with(r, ",") r = advance(r, 1);

        item: Element;

        if !quoted && to_lower_copy(value,, temp) == "null" {
            #if #run is_nullable(Element) {
                item.null = true;
            } else {
                return tprint("element % is NULL; read the array as [] Nullable(%)", items.count + 1, Element);
            }
        } else {
            #if #run is_nullable(Element) why := decode(cast([] u8) value, .TEXT, element, *item.value, json);
            else why := decode(cast([] u8) value, .TEXT, element, *item, json);
            if why return tprint("element %: %", items.count + 1, why);
        }

        array_add(*items, item);
    }

    out.* = items;

    return "";
}

unescape_array_element :: (text: string) -> string {
    out := alloc_string(text.count);
    count := 0;
    i := 0;

    while i < text.count {
        if text[i] == #char "\\" && i + 1 < text.count i += 1;
        out[count] = text[i];
        count += 1;
        i += 1;
    }

    out.count = count;

    return out;
}

decode_bytea_text :: (text: string) -> [] u8, string {
    if !begins_with(text, "\\x") return .[], "bytea text is not in hex format (set bytea_output = 'hex')";

    digits := advance(text, 2);
    if digits.count % 2 return .[], "odd-length bytea hex";

    out := NewArray(digits.count / 2, u8, initialized = false);
    for 0..out.count - 1 {
        high := hex_value(digits[it * 2]);
        low := hex_value(digits[it * 2 + 1]);
        if high < 0 || low < 0 return .[], "bad bytea hex";
        out[it] = cast(u8) (high * 16 + low);
    }

    return out, "";
}

NUMERIC_BASE :: 10_000;
MICROSECOND_DIGITS :: 6;

// Binary numeric: digit count, weight (of the first digit, in base 10000), sign, display scale,
// then the base-10000 digits.
decode_numeric_binary :: (bytes: [] u8, out: *Numeric) -> string {
    r := Cursor.{ data = bytes };
    count := cast(s64) get_u16(*r);
    weight := cast(s64) get_s16(*r);
    sign := get_u16(*r);
    scale := cast(s64) get_u16(*r);

    if r.bad return "a truncated numeric";

    if sign == 0xc000 {
        out.text = "NaN";
        return "";
    }

    if sign == 0xd000 {
        out.text = "Infinity";
        return "";
    }

    if sign == 0xf000 {
        out.text = "-Infinity";
        return "";
    }

    digits := NewArray(count, s64,, temp);
    for *digits {
        it.* = get_u16(*r);
        if it.* >= NUMERIC_BASE r.bad = true;
    }

    if r.bad return "a malformed numeric";

    builder: String_Builder;
    builder.allocator = temp;
    if sign == 0x4000 && count > 0 append(*builder, "-");

    if weight < 0 {
        append(*builder, "0");
    } else {
        for i: 0..weight {
            d := ifx i < count then digits[i] else 0;
            if i == 0 print_to_builder(*builder, "%", d);
            else print_to_builder(*builder, "%", formatInt(d, minimum_digits = 4));
        }
    }

    if scale > 0 {
        append(*builder, ".");
        written := 0;
        j := 1;

        while written < scale {
            index := weight + j;
            d := ifx index >= 0 && index < count then digits[index] else 0;
            group := tprint("%", formatInt(d, minimum_digits = 4));
            take := min(4, scale - written);

            append(*builder, slice(group, 0, take));
            written += take;
            j += 1;
        }
    }

    out.text = builder_to_string(*builder);

    return "";
}

// The binary form of a numeric given as text: "-12.50", "0", "NaN", "Infinity", "-Infinity".
// Digits are base 10000, so both parts are padded to whole groups of four decimal digits.
numeric_binary :: (text: string) -> string, string {
    out: [..] u8;
    out.allocator = temp;

    if text == "NaN" || text == "Infinity" || text == "-Infinity" {
        sign: u16 = 0xc000;
        if text == "Infinity" sign = 0xd000;
        else if text == "-Infinity" sign = 0xf000;

        put_u16(*out, 0);
        put_u16(*out, 0);
        put_u16(*out, sign);
        put_u16(*out, 0);

        return cast(string) out, "";
    }

    r := text;
    negative := begins_with(r, "-");

    if negative || begins_with(r, "+") r = advance(r, 1);

    dot := find_index_from_left(r, #char ".");
    whole := ifx dot < 0 then r else slice(r, 0, dot);
    fraction := ifx dot < 0 then "" else advance(r, dot + 1);

    if !whole && !fraction return "", tprint("\"%\" is not a numeric", text);

    for whole if it < #char "0" || it > #char "9" return "", tprint("\"%\" is not a numeric", text);
    for fraction if it < #char "0" || it > #char "9" return "", tprint("\"%\" is not a numeric", text);

    // Every group of four digits, the whole part right-aligned and the fraction left-aligned.
    groups: [..] s64;
    groups.allocator = temp;
    lead := (4 - whole.count % 4) % 4;
    weight := (whole.count + lead) / 4 - 1;

    for 0..(whole.count + lead) / 4 - 1 {
        value := 0;

        for k: 0..3 {
            at := it * 4 + k - lead;
            value = value * 10 + ifx at >= 0 then whole[at] - #char "0" else 0;
        }

        array_add(*groups, value);
    }

    for 0..(fraction.count + 3) / 4 - 1 {
        value := 0;

        for k: 0..3 {
            at := it * 4 + k;
            value = value * 10 + ifx at < fraction.count then fraction[at] - #char "0" else 0;
        }

        array_add(*groups, value);
    }

    first := 0;

    while first < groups.count && groups[first] == 0 {
        first += 1;
        weight -= 1;
    }

    last := groups.count;
    while last > first && groups[last - 1] == 0 last -= 1;

    count := last - first;
    if !count weight = 0;

    put_u16(*out, cast(u16) count);
    put_u16(*out, cast,no_check(u16) weight);
    put_u16(*out, ifx negative && count > 0 then cast(u16) 0x4000 else 0);
    put_u16(*out, cast(u16) fraction.count);

    for first..last - 1 put_u16(*out, cast(u16) groups[it]);

    return cast(string) out, "";
}

// "2024-01-31" or "2024-01-31 BC", "infinity", "-infinity".
parse_date_text :: (text: string) -> s32, bool {
    if text == "infinity" return DATE_INFINITY, true;
    if text == "-infinity" return DATE_MINUS_INFINITY, true;

    r := text;
    year, month, day: s64;

    if !parse_ymd(*r, *year, *month, *day) return 0, false;

    if r == " BC" {
        year = 1 - year;
        r = "";
    }

    if r return 0, false;

    return date_from_civil(year, month, day).days, true;
}

// "2024-01-31 13:45:00.123456+05:30", with an offset only for timestamptz; " BC" may follow.
parse_timestamp_text :: (text: string, zoned: bool) -> s64, bool {
    if text == "infinity" return TIMESTAMP_INFINITY, true;
    if text == "-infinity" return TIMESTAMP_MINUS_INFINITY, true;

    r := text;
    year, month, day: s64;

    if !parse_ymd(*r, *year, *month, *day) return 0, false;
    if !r || (r[0] != #char " " && r[0] != #char "T") return 0, false;

    r = advance(r, 1);
    clock: s64;
    if !parse_clock(*r, *clock) return 0, false;

    offset := 0;

    if zoned && r && (r[0] == #char "+" || r[0] == #char "-") {
        negative := r[0] == #char "-";

        r = advance(r, 1);
        hours, minutes, seconds: s64;
        if !parse_digits(*r, 2, *hours) return 0, false;

        if begins_with(r, ":") {
            r = advance(r, 1);
            if !parse_digits(*r, 2, *minutes) return 0, false;

            if begins_with(r, ":") {
                r = advance(r, 1);
                if !parse_digits(*r, 2, *seconds) return 0, false;
            }
        }

        offset = ((hours * 60 + minutes) * 60 + seconds) * 1_000_000;
        if negative offset = -offset;
    }

    if r == " BC" {
        year = 1 - year;
        r = "";
    }

    if r return 0, false;

    days := cast(s64) date_from_civil(year, month, day).days;

    return days * MICROSECONDS_PER_DAY + clock - offset, true;
}

parse_ymd :: (r: *string, year: *s64, month: *s64, day: *s64) -> bool {
    dash := find_index_from_left(r.*, #char "-");
    if dash < 4 return false;

    valid: bool;
    rest: string;
    year.*, valid, rest = string_to_int(slice(r.*, 0, dash), 10, s64);
    if !valid || rest return false;

    r.* = advance(r.*, dash + 1);
    if !parse_digits(r, 2, month) || !begins_with(r.*, "-") return false;

    r.* = advance(r.*, 1);

    return parse_digits(r, 2, day);
}

// "HH:MM:SS" with an optional fraction, into microseconds. Hours may have more than two digits.
parse_clock :: (r: *string, out: *s64) -> bool {
    colon := find_index_from_left(r.*, #char ":");
    if colon < 1 return false;

    hours, valid, rest := string_to_int(slice(r.*, 0, colon), 10, s64);
    if !valid || rest || hours < 0 return false;

    r.* = advance(r.*, colon + 1);
    minutes, seconds: s64;
    if !parse_digits(r, 2, *minutes) || !begins_with(r.*, ":") return false;

    r.* = advance(r.*, 1);
    if !parse_digits(r, 2, *seconds) return false;

    fraction := 0;

    if begins_with(r.*, ".") {
        r.* = advance(r.*, 1);
        scale := 100_000;
        digits := 0;

        while r.count && r.*[0] >= #char "0" && r.*[0] <= #char "9" {
            if digits < MICROSECOND_DIGITS fraction += (r.*[0] - #char "0") * scale;
            scale /= 10;
            digits += 1;
            r.* = advance(r.*, 1);
        }

        if !digits return false;
    }

    out.* = ((hours * 60 + minutes) * 60 + seconds) * 1_000_000 + fraction;

    return true;
}

parse_digits :: (r: *string, count: s64, out: *s64) -> bool {
    if r.count < count return false;

    value := 0;

    for 0..count - 1 {
        c := r.*[it];
        if c < #char "0" || c > #char "9" return false;
        value = value * 10 + c - #char "0";
    }

    out.* = value;
    r.* = advance(r.*, count);

    return true;
}

// IntervalStyle postgres: "1 year 2 mons -3 days +04:05:06.5", "00:00:00", "-1 days".
parse_interval_text :: (text: string, out: *Interval) -> bool {
    out.* = .{};
    r := text;
    while r {
        r = trim_left(r);
        if !r break;

        // A clock part: optional sign, then HH:MM:SS.
        word_end := find_index_from_left(r, #char " ");
        word := ifx word_end < 0 then r else slice(r, 0, word_end);

        if contains(word, ":") {
            negative := begins_with(word, "-");
            clock_text := word;

            if begins_with(clock_text, "-") || begins_with(clock_text, "+") clock_text = advance(clock_text, 1);

            clock: s64;
            if !parse_clock(*clock_text, *clock) || clock_text return false;

            out.microseconds += ifx negative then -clock else clock;
            r = advance(r, word.count);
            continue;
        }

        value, valid, rest := string_to_int(word, 10, s64);
        if !valid || rest return false;

        r = trim_left(advance(r, word.count));
        unit_end := find_index_from_left(r, #char " ");
        unit := ifx unit_end < 0 then r else slice(r, 0, unit_end);

        r = advance(r, unit.count);
        if unit == "year" || unit == "years" out.months += cast(s32) (value * MONTHS_PER_YEAR);
        else if unit == "mon" || unit == "mons" out.months += cast(s32) value;
        else if unit == "day" || unit == "days" out.days += cast(s32) value;
        else return false;
    }

    return true;
}

// One parameter, ready for Parse (its OID) and Bind (its format and bytes).
Param :: struct {
    oid: u32;
    format: Format;
    null: bool;
    data: string; // in temporary storage
}

// Picks a format and OID for one argument. Plain numbers, strings and bools go as text with OID
// 0, so the server infers the type from the query; types whose meaning would be lost as text
// (bytea, floats, uuid, dates and times, arrays) go as binary with their OID.
encode_param :: (value: Any) -> Param, string {
    info := value.type;
    data := value.value_pointer;

    if is_json_info(info) || info == type_info(Json_Raw) {
        // No type OID: the server reads the text as json or jsonb, whichever the statement wants.
        text, why := json_parameter_text(value);
        return .{ data = text }, why;
    }

    if is_nullable_info(info) {
        s := cast(*Type_Info_Struct) info;
        null_member, inner_member: *Type_Info_Struct_Member;

        for *s.members {
            if it.name == "null" null_member = it;
            if it.name == "value" inner_member = it;
        }

        if (cast(*bool) (data + null_member.offset_in_bytes)).* return .{ null = true }, "";

        return encode_param(Any.{ inner_member.type, data + inner_member.offset_in_bytes });
    }

    if info.type == .ARRAY {
        array := cast(*Type_Info_Array) info;
        if array.element_type == type_info(u8) {
            return .{ oid = xx Oid.BYTEA, format = .BINARY, data = cast(string) any_array_view(value) }, "";
        }

        if array.element_type.type == .ENUM return encode_enum_array(value);

        return encode_array(value);
    }

    if info.type == .STRING {
        text := (cast(*string) data).*;
        if contains_zero(text) return .{}, "a string parameter holds a zero byte, which Postgres text cannot store";
        return .{ data = text }, "";
    }

    if info.type == .INTEGER && info.runtime_size == 8 && !(cast(*Type_Info_Integer) info).signed {
        return .{ data = tprint("%", (cast(*u64) data).*) }, "";
    }

    if info.type == .ENUM {
        // Sent as text with no type, so the server reads it as the enum the statement wants.
        label, found := enum_value_label(value);
        if !found return .{}, tprint("% holds a value that is none of its members", type_to_string(info,, temp));

        return .{ data = label }, "";
    }

    if info.type == .INTEGER {
        return .{ data = tprint("%", any_integer(value)) }, "";
    }
    if info.type == .BOOL {
        return .{ data = ifx (cast(*bool) data).* then "t" else "f" }, "";
    }

    param, why := encode_binary(value);

    return param, why;
}

// The binary form and OID of a non-null scalar.
encode_binary :: (value: Any) -> Param, string {
    info := value.type;
    data := value.value_pointer;
    out: [..] u8;
    out.allocator = temp;
    oid: Oid;

    if info.type == .BOOL {
        oid = .BOOL;
        put_u8(*out, xx (cast(*bool) data).*);
    } else if info.type == .INTEGER {
        v := any_integer(value);

        if info.runtime_size <= 1 {
            oid = .INT2;
            put_u16(*out, cast,no_check(u16) v);
        } else if info.runtime_size == 2 && (cast(*Type_Info_Integer) info).signed {
            oid = .INT2;
            put_u16(*out, cast,no_check(u16) v);
        } else if info.runtime_size <= 2 || (info.runtime_size == 4 && (cast(*Type_Info_Integer) info).signed) {
            oid = .INT4;
            put_u32(*out, cast,no_check(u32) v);
        } else {
            if info.runtime_size == 8 && !(cast(*Type_Info_Integer) info).signed && (cast(*u64) data).* > xx S64_MAX {
                return .{}, "a u64 parameter above S64_MAX does not fit in int8";
            }

            oid = .INT8;
            put_u64(*out, cast,no_check(u64) v);
        }
    } else if info.type == .FLOAT {
        if info.runtime_size == 4 {
            oid = .FLOAT4;
            put_u32(*out, (cast(*u32) data).*);
        } else {
            oid = .FLOAT8;
            put_u64(*out, (cast(*u64) data).*);
        }
    } else if info.type == .STRING {
        text := (cast(*string) data).*;
        if contains_zero(text) return .{}, "a string parameter holds a zero byte, which Postgres text cannot store";
        oid = .TEXT;
        put_bytes(*out, text);
    } else if info == type_info(Uuid) {
        oid = .UUID;
        put_bytes(*out, cast([] u8) (cast(*Uuid) data).bytes);
    } else if info == type_info(Timestamp) || info == type_info(Local_Timestamp) {
        oid = ifx info == type_info(Timestamp) then Oid.TIMESTAMPTZ else .TIMESTAMP;
        put_u64(*out, cast,no_check(u64) (cast(*s64) data).*);
    } else if info == type_info(Date) {
        oid = .DATE;
        put_u32(*out, cast,no_check(u32) (cast(*s32) data).*);
    } else if info == type_info(Time) {
        oid = .TIME;
        put_u64(*out, cast,no_check(u64) (cast(*s64) data).*);
    } else if info == type_info(Interval) {
        oid = .INTERVAL;
        interval := cast(*Interval) data;

        put_u64(*out, cast,no_check(u64) interval.microseconds);
        put_u32(*out, cast,no_check(u32) interval.days);
        put_u32(*out, cast,no_check(u32) interval.months);
    } else if is_json_info(info) || info == type_info(Json_Raw) {
        text, why := json_parameter_text(value);
        if why return .{}, why;

        oid = .JSONB;
        put_u8(*out, 1);
        put_bytes(*out, text);
    } else if info == type_info(Numeric) {
        // Numerics go as text: the server parses them exactly.
        return .{ oid = xx Oid.NUMERIC, data = (cast(*Numeric) data).text }, "";
    } else {
        return .{}, tprint("Postgres has no encoding for parameters of type %", type_to_string(info,, temp));
    }

    return .{ oid = xx oid, format = .BINARY, data = cast(string) out }, "";
}

// A one-dimensional binary array: dimensions, has-nulls, element OID, length, lower bound, then
// each element as a length and its binary form.
encode_array :: (value: Any) -> Param, string {
    array := cast(*Type_Info_Array) value.type;
    view := any_array_view(value);
    count := view.count / array.element_type.runtime_size;
    element_info := array.element_type;
    nullable := is_nullable_info(element_info);
    inner_info := element_info;
    null_offset, inner_offset: s64;

    if nullable {
        for (cast(*Type_Info_Struct) element_info).members {
            if it.name == "null" null_offset = it.offset_in_bytes;

            if it.name == "value" {
                inner_offset = it.offset_in_bytes;
                inner_info = it.type;
            }
        }
    }

    if inner_info.type == .ARRAY && (cast(*Type_Info_Array) inner_info).element_type != type_info(u8) {
        return .{}, "multi-dimensional array parameters are not supported";
    }

    elements: [..] u8;
    elements.allocator = temp;
    element_oid: u32;
    has_nulls := false;

    for 0..count - 1 {
        at := view.data + it * element_info.runtime_size;
        if nullable && (cast(*bool) (at + null_offset)).* {
            has_nulls = true;
            put_u32(*elements, 0xffff_ffff);
            continue;
        }

        param: Param;
        why: string;

        if inner_info.type == .ARRAY {
            param = .{ oid = xx Oid.BYTEA, format = .BINARY, data = cast(string) any_array_view(Any.{ inner_info, at + inner_offset }) };
        } else if inner_info == type_info(Numeric) {
            // An array element is binary, so the numeric's text is turned into binary form.
            param.data, why = numeric_binary((cast(*Numeric) (at + inner_offset)).text);
            if why return .{}, why;

            param.oid = xx Oid.NUMERIC;
        } else {
            param, why = encode_binary(Any.{ inner_info, at + inner_offset });
            if why return .{}, why;
        }

        element_oid = param.oid;
        put_u32(*elements, cast(u32) param.data.count);
        put_bytes(*elements, param.data);
    }

    if !element_oid {
        // An empty or all-null array still needs an element type.
        probe: [16] u8;

        if inner_info.type == .ARRAY {
            element_oid = xx Oid.BYTEA;
        } else {
            param, why := encode_binary(Any.{ inner_info, probe.data });
            if why return .{}, why;
            element_oid = param.oid;
        }
    }

    array_oid := array_oid_of(element_oid);
    if !array_oid return .{}, tprint("Postgres has no array type for % elements here", type_name(element_oid));

    out: [..] u8;
    out.allocator = temp;
    put_u32(*out, 1);
    put_u32(*out, xx has_nulls);
    put_u32(*out, element_oid);
    put_u32(*out, cast(u32) count);
    put_u32(*out, 1);
    put_bytes(*out, cast([] u8) elements);

    return .{ oid = array_oid, format = .BINARY, data = cast(string) out }, "";
}

is_nullable :: ($T: Type) -> bool {
    return is_nullable_info(type_info(T));
}

is_nullable_info :: (info: *Type_Info) -> bool {
    if info.type != .STRUCT return false;
    return (cast(*Type_Info_Struct) info).polymorph_source_struct == type_info(Nullable);
}

is_json :: ($T: Type) -> bool {
    return is_json_info(type_info(T));
}

is_json_info :: (info: *Type_Info) -> bool {
    if info.type != .STRUCT return false;
    return (cast(*Type_Info_Struct) info).polymorph_source_struct == type_info(Json);
}

// The document text of a `json` or `jsonb` value (or a text one holding JSON). Binary jsonb
// starts with a version byte, and 1 is the only version there is.
json_document :: (bytes: [] u8, format: Format, oid: u32) -> string, string {
    text := cast(string) bytes;

    if oid == xx Oid.JSONB && format == .BINARY {
        if !bytes.count return "", "an empty binary jsonb";
        if bytes[0] != 1 return "", tprint("jsonb version % is not supported", bytes[0]);
        return advance(text, 1), "";
    }

    if oid == xx Oid.JSONB || is_text_like(oid) return text, "";

    return "", tprint("% is not json", type_name(oid));
}

// The text of a `Json(T)` passed as `Any`, written by its `write` member.
json_parameter_text :: (value: Any) -> string, string {
    if value.type == type_info(Json_Raw) return (cast(*Json_Raw) value.value_pointer).text, "";

    s := cast(*Type_Info_Struct) value.type;
    value_offset, write_offset: s64 = -1;

    for s.members {
        if it.name == "value" value_offset = it.offset_in_bytes;
        if it.name == "write" write_offset = it.offset_in_bytes;
    }

    Writer :: #type (value: *void, builder: *String_Builder);
    write := (cast(*Writer) (value.value_pointer + write_offset)).*;

    if !write return "", "a Json value with no `write`: make it with Pg.json(value) or a declaration, not zeroed memory";

    builder: String_Builder;
    builder.allocator = temp;
    write(value.value_pointer + value_offset, *builder);

    return builder_to_string(*builder,, temp), "";
}

any_array_view :: (value: Any) -> [] u8 {
    array := cast(*Type_Info_Array) value.type;
    element := array.element_type.runtime_size;

    if array.array_type == .FIXED {
        return .{ array.array_count * element, value.value_pointer };
    }

    view := cast(*[] u8) value.value_pointer;

    return .{ view.count * element, view.data };
}

// An array of Jai enums as an array literal in text form, with no type, so the server reads it as
// the enum array the statement wants (binary arrays need the enum's OID, which only it knows).
encode_enum_array :: (value: Any) -> Param, string {
    array := cast(*Type_Info_Array) value.type;
    view := any_array_view(value);
    size := array.element_type.runtime_size;

    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "{");

    for 0..view.count / size - 1 {
        label, found := enum_value_label(Any.{ array.element_type, view.data + it * size });
        if !found return .{}, tprint("element % holds a value that is none of its enum's members", it + 1);

        if it append(*builder, ",");

        append(*builder, "\"");

        for label {
            if it == #char "\"" || it == #char "\\" append(*builder, "\\");
            print_to_builder(*builder, "%", slice(label, it_index, 1));
        }

        append(*builder, "\"");
    }

    append(*builder, "}");

    return .{ data = builder_to_string(*builder) }, "";
}

// The label of a Jai enum value, as a Postgres enum spells it.
enum_value_label :: (value: Any) -> string, bool {
    info := cast(*Type_Info_Enum) value.type;
    number := any_integer(value);

    for info.values if it == number return enum_label(info.names[it_index]), true;

    return "", false;
}

any_integer :: (value: Any) -> s64 {
    info := value.type;
    if info.type == .ENUM info = (cast(*Type_Info_Enum) info).internal_type;

    integer := cast(*Type_Info_Integer) info;
    data := value.value_pointer;

    if info.runtime_size == {
        case 1; return ifx integer.signed then (cast(*s8) data).* else (cast(*u8) data).*;
        case 2; return ifx integer.signed then (cast(*s16) data).* else (cast(*u16) data).*;
        case 4; return ifx integer.signed then (cast(*s32) data).* else (cast(*u32) data).*;
    }

    return (cast(*s64) data).*;
}
