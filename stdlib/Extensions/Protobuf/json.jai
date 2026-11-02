// The proto3 JSON mapping of generated messages, read and written with Extensions/Json.
// The code for a message is generated from its Jai type when a program first uses it, so a
// schema pays nothing for JSON until then.

// How to_json writes and from_json reads.
Json_Options :: struct {
    // Print implicit-presence fields that hold their default, and empty repeated fields and maps.
    emit_defaults := false;
    // Keys are the .proto field names instead of the lowerCamelCase JSON names.
    proto_names := false;
    // Enums print as numbers instead of names.
    enum_numbers := false;
    // Indent with two spaces.
    pretty := false;
    // Skip unknown keys, and drop a field, element or map entry holding an unknown enum name.
    ignore_unknown_fields := false;
    // Nesting of JSON objects and arrays.
    max_depth: s64 = 100;
    // The types a google.protobuf.Any may hold: a schema module's `proto_registry`, or several
    // joined into one array.
    registry: [] Json_Type;
}

// One message type of an Any registry: its full .proto name and its JSON codec.
Json_Type :: struct {
    name: string;
    run: Json_Any_Proc;
}

// Writes the fields of `bytes` (with `out`) or reads them into `result` (with `input`).
Json_Any_Proc :: #type (out: *Json_Out, input: *Json_In, bytes: [] u8, result: *[] u8) -> bool;

// The JSON codec of one message type, for Any. Generated schemas list one per message in
// `proto_registry`.
Json_Any :: struct(T: Type) {
    run :: (out: *Json_Out, input: *Json_In, bytes: [] u8, result: *[] u8) -> bool {
        if out {
            value, error := decode(T, bytes);

            defer T.proto_free(*value);
            defer json_free_error(*error);

            if error.kind != .NONE return json_write_fail(out, .BAD_VALUE, "an Any holds bytes that don't decode as its type");

            first := false;

            return json_write_any_body(out, *value, *first);
        }

        value: T;
        saved := input.pr;

        input.pr = Reader.{ bytes = saved.bytes, pos = saved.pos, storage = *value.proto_storage };
        ok := json_read_any_body(input, *value);

        input.pr = saved;

        if ok {
            encoded, error := encode(value);

            if error.kind != .NONE {
                json_free_error(*error);
                ok = Json.json_fail(*input.r, .INVALID_VALUE, "an Any's message doesn't encode");
            } else result.* = encoded;
        }

        T.proto_free(*value);

        return ok;
    }
}

// Which special JSON form a well-known type has. Generated well-known messages say theirs.
Well_Known :: enum u8 {
    NONE;
    TIMESTAMP;
    DURATION;
    FIELD_MASK;
    STRUCT;
    VALUE;
    LIST_VALUE;
    ANY;
    EMPTY;
    WRAPPER;
}

// The state of a running to_json, for Any codecs.
Json_Out :: struct {
    w: Json.Json_Writer;
    options: Json_Options;
    error: Error;
}

// The state of a running from_json, for Any codecs.
Json_In :: struct {
    r: Json.Json_Reader;
    pr: Reader;
    options: Json_Options;
    skipped: bool;
}

// The proto3 JSON of a generated message, allocated with context.allocator. Fails with
// `.BAD_VALUE` for a value JSON can't express (a Timestamp out of range, a NaN in a Value) and
// `.UNKNOWN_TYPE` for an Any whose type isn't in `options.registry`.
to_json :: (value: $T, options := Json_Options.{}) -> string, Error {
    builder: String_Builder;
    error := write_json(*builder, value, options);

    if error.kind != .NONE {
        free_buffers(*builder);
        return "", error;
    }

    return builder_to_string(*builder), .{};
}

// Appends the proto3 JSON of a message to a builder. On failure part of it may be written.
write_json :: (builder: *String_Builder, value: $T, options := Json_Options.{}) -> Error {
    o: Json_Out;

    o.options = options;
    o.w.builder = builder;
    o.w.pretty = options.pretty;

    v := value;

    json_write_message(*o, *v);

    return o.error;
}

// Decodes a message from its proto3 JSON. A failure releases everything and returns a zero
// value; the error's `message` says what was wrong and `path` where.
from_json :: ($T: Type, text: string, options := Json_Options.{}) -> T, Error {
    value: T;
    error := merge_json(*value, text, options);
    if error.kind != .NONE T.proto_free(*value);
    return value, error;
}

// Merges proto3 JSON into a message: present scalars replace, repeated fields append, messages
// merge and map keys replace. On failure the merged prefix stays owned by value.
merge_json :: (value: *$T, text: string, options := Json_Options.{}) -> Error {
    i: Json_In;

    i.options = options;
    Json.json_reader_init(*i.r, text, .{ max_depth = options.max_depth });
    defer Json.json_reader_free(*i.r);

    i.pr = Reader.{ bytes = .{ text.count, text.data }, storage = *value.proto_storage };

    ok := json_read_message(*i, value);

    if ok {
        Json.json_peek(*i.r);
        if i.r.pos < text.count ok = Json.json_fail(*i.r, .SYNTAX, "unexpected data after the value");
    }

    if ok return .{};

    e := Json.json_reader_error(*i.r);
    kind: Error_Kind = .BAD_JSON;

    if e.kind == .OUT_OF_RANGE || e.kind == .INVALID_VALUE kind = .BAD_VALUE;
    else if e.kind == .UNKNOWN_FIELD kind = .UNKNOWN_FIELD;
    else if e.kind == .DUPLICATE_FIELD kind = .DUPLICATE_FIELD;
    else if e.kind == .UNKNOWN_VARIANT kind = .UNKNOWN_TYPE;
    else if e.kind == .LIMIT kind = .TOO_DEEP;

    return .{ kind = kind, offset = e.offset, path = e.path, message = e.message };
}

#scope_module

Json :: #import "Extensions/Json";

JSON_FLOAT_MAX :: 1.7976931348623157e308;

Json_Flags :: enum_flags u8 {
    CLOSED;
    NULL;
    FORCE;
}

// ---- writing ----

json_free_error :: (error: *Error) {
    Basic.free(error.path);
    Basic.free(error.message);
    error.* = .{};
}

json_write_fail :: (o: *Json_Out, kind: Error_Kind, message: string) -> bool {
    if o.error.kind == .NONE o.error = .{ kind = kind, message = copy_string(message) };
    return false;
}

// Adds a field name to a failed write's path, innermost first.
json_write_path :: (o: *Json_Out, name: string, index: s64 = -1) -> bool {
    prefix := name;
    if index >= 0 prefix = tprint("%[%]", name, index);

    old := o.error.path;
    if old.count o.error.path = sprint("%.%", prefix, old);
    else o.error.path = copy_string(prefix);

    Basic.free(old);

    return false;
}

json_key :: inline (o: *Json_Out, first: *bool, key: string, name: string) {
    Json.json_key(*o.w, first, ifx o.options.proto_names then name else key);
}

json_write_message :: (o: *Json_Out, v: *$T) -> bool {
    KIND :: #run json_well_known_of(type_info(T));

    #if KIND != .NONE && KIND != .EMPTY return json_write_special(o, v, KIND);
    else {
        Json.json_begin_object(*o.w);
        first := true;
        if !json_write_fields(o, v, *first) return false;

        Json.json_end_object(*o.w, first);

        return true;
    }
}

json_write_fields :: (o: *Json_Out, v: *$T, first: *bool) -> bool {
    #insert -> string {
        return json_writer_code(T);
    };
}

// One field: skipped when absent (or default, for implicit presence), else its key and value.
json_put :: inline (o: *Json_Out, first: *bool, value: *$T, key: string, name: string, $flags: Json_Flags, names: [] string) -> bool {
    info :: #run type_info(T);

    #if #run json_is_optional(type_info(T)) {
        if !value.has return true;
        json_key(o, first, key, name);
        if !json_value(o, *value.value, flags, names) return json_write_path(o, slice(key, 1, key.count - 2));
    } else #if info.type == .POINTER {
        if !value.* return true;
        json_key(o, first, key, name);
        if !json_write_message(o, value.*) return json_write_path(o, slice(key, 1, key.count - 2));
    } else #if info.type == .ARRAY && (cast(*Type_Info_Array) info).array_type == .RESIZABLE {
        if !value.count && !o.options.emit_defaults return true;

        json_key(o, first, key, name);
        Json.json_begin_array(*o.w);
        inner := true;

        for *value.* {
            Json.json_item(*o.w, *inner);
            if !json_value(o, it, flags, names) return json_write_path(o, slice(key, 1, key.count - 2), it_index);
        }

        Json.json_end_array(*o.w, inner);
    } else {
        #if !(flags & .FORCE) if !o.options.emit_defaults && json_is_default(value.*) return true;
        json_key(o, first, key, name);
        if !json_value(o, value, flags, names) return json_write_path(o, slice(key, 1, key.count - 2));
    }

    return true;
}

// A finite float, shortest round-trip, with integral values printed as integers (`1`, `-0`).
json_write_number :: (w: *Json.Json_Writer, x: $T) {
    if x > -1.0e15 && x < 1.0e15 && x == cast(T) cast(s64) x {
        if x == 0 && (cast(*u8) *x)[size_of(T) - 1] & 0x80 Json.json_write_raw(w, "-0");
        else Json.json_write_int(w, cast,no_check(s64) x);
    } else #if T == float32 Json.json_write_float32(w, x);
    else Json.json_write_float64(w, x);
}

json_is_default :: inline (value: $T) -> bool {
    #if T == float32 return (cast(*u32) *value).* == 0;
    else #if T == float64 return (cast(*u64) *value).* == 0;
    else #if T == string || T == [] u8 return value.count == 0;
    else #if T == bool return !value;
    else return cast,no_check(s64) value == 0;
}

// A map field: a JSON object with stringified keys.
json_put_map :: (o: *Json_Out, first: *bool, map: *$M, key: string, name: string, $flags: Json_Flags, names: [] string) -> bool {
    if !map.count && !o.options.emit_defaults return true;
    json_key(o, first, key, name);
    if !json_write_map_body(o, map, flags, names) return json_write_path(o, slice(key, 1, key.count - 2));
    return true;
}

json_write_map_body :: (o: *Json_Out, map: *$M, $flags: Json_Flags, names: [] string) -> bool {
    Json.json_begin_object(*o.w);
    inner := true;
    buffer: [24] u8 = ---;

    #if #run json_map_mode(type_info(M)) == .ARRAY {
        for *map.* {
            Json.json_key_dynamic(*o.w, *inner, json_key_text(*buffer, it.key));
            if !json_value(o, *it.value, flags, names) return false;
        }
    } else for item, k: map.* {
        Json.json_key_dynamic(*o.w, *inner, json_key_text(*buffer, k));
        copy := item;
        if !json_value(o, *copy, flags, names) return false;
    }

    Json.json_end_object(*o.w, inner);

    return true;
}

json_key_text :: (buffer: *[24] u8, key: $K) -> string {
    #if K == string return key;
    else #if K == bool return ifx key then "true" else "false";
    else {
        negative := key < 0;
        magnitude: u64;

        #if K == s32 || K == s64 {
            if negative magnitude = ~(cast,no_check(u64) cast(s64) key) + 1;
            else magnitude = cast(u64) key;
        } else magnitude = cast(u64) key;

        pos := buffer.count;

        while true {
            pos -= 1;
            buffer.*[pos] = cast(u8) (magnitude % 10) + #char "0";
            magnitude /= 10;
            if !magnitude break;
        }

        if negative {
            pos -= 1;
            buffer.*[pos] = #char "-";
        }

        return .{ buffer.count - pos, buffer.data + pos };
    }
}

// A value of a field, repeated element or map value.
json_value :: inline (o: *Json_Out, value: *$T, $flags: Json_Flags, names: [] string) -> bool {
    w := *o.w;
    info :: #run type_info(T);

    #if T == bool Json.json_write_bool(w, value.*);
    else #if T == s32 Json.json_write_int(w, value.*);
    else #if T == u32 Json.json_write_uint(w, value.*);
    else #if T == s64 {
        Json.json_write_raw(w, "\"");
        Json.json_write_int(w, value.*);
        Json.json_write_raw(w, "\"");
    } else #if T == u64 {
        Json.json_write_raw(w, "\"");
        Json.json_write_uint(w, value.*);
        Json.json_write_raw(w, "\"");
    } else #if T == float32 || T == float64 {
        x := cast(float64) value.*;

        if x != x Json.json_write_raw(w, "\"NaN\"");
        else if x > JSON_FLOAT_MAX Json.json_write_raw(w, "\"Infinity\"");
        else if x < -JSON_FLOAT_MAX Json.json_write_raw(w, "\"-Infinity\"");
        else json_write_number(w, value.*);
    } else #if T == string Json.json_write_string(w, value.*);
    else #if T == [] u8 json_write_base64(w, value.*);
    else #if info.type == .ENUM json_write_enum(o, value.*, flags, names);
    else #if info.type == .POINTER {
        if !value.* {
            Json.json_write_raw(w, "{}");
            return true;
        }

        return json_write_message(o, value.*);
    } else return json_write_message(o, value);

    return true;
}

json_write_enum :: (o: *Json_Out, value: $T, $flags: Json_Flags, names: [] string) {
    number := cast(s64) value;

    #if flags & .NULL if number == 0 {
        Json.json_write_null(*o.w);
        return;
    }

    if !o.options.enum_numbers {
        info := type_info(T);

        for info.values if it == number {
            Json.json_write_string(*o.w, ifx names.count then names[it_index] else info.names[it_index]);
            return;
        }
    }

    Json.json_write_int(*o.w, number);
}

BASE64 :: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// Standard padded base64 in quotes, written in stack-sized pieces.
json_write_base64 :: (w: *Json.Json_Writer, bytes: [] u8) {
    buffer: [256] u8 = ---;
    text := buffer.data;
    pos := 0;

    text[pos] = #char "\"";
    pos += 1;

    i := 0;

    while i + 3 <= bytes.count {
        if pos > buffer.count - 8 {
            Json.json_write_raw(w, .{ pos, text });
            pos = 0;
        }

        n := ((cast(u32) bytes[i]) << 16) | ((cast(u32) bytes[i + 1]) << 8) | bytes[i + 2];

        text[pos] = BASE64[n >> 18];
        text[pos + 1] = BASE64[(n >> 12) & 63];
        text[pos + 2] = BASE64[(n >> 6) & 63];
        text[pos + 3] = BASE64[n & 63];
        pos += 4;
        i += 3;
    }

    left := bytes.count - i;

    if left {
        n := (cast(u32) bytes[i]) << 16;
        if left == 2 n |= (cast(u32) bytes[i + 1]) << 8;

        text[pos] = BASE64[n >> 18];
        text[pos + 1] = BASE64[(n >> 12) & 63];
        text[pos + 2] = ifx left == 2 then BASE64[(n >> 6) & 63] else #char "=";
        text[pos + 3] = #char "=";
        pos += 4;
    }

    text[pos] = #char "\"";

    Json.json_write_raw(w, .{ pos + 1, text });
}

json_write_any_body :: (o: *Json_Out, v: *$T, first: *bool) -> bool {
    KIND :: #run json_well_known_of(type_info(T));

    #if KIND != .NONE && KIND != .EMPTY {
        Json.json_key(*o.w, first, "\"value\"");
        return json_write_special(o, v, KIND);
    } else return json_write_fields(o, v, first);
}

json_write_special :: (o: *Json_Out, v: *$T, $kind: Well_Known) -> bool {
    w := *o.w;

    #if kind == .WRAPPER return json_value(o, *v.value, 0, .[]);
    else #if kind == .TIMESTAMP {
        text, ok := json_timestamp_text(v.seconds, v.nanos);
        if !ok return json_write_fail(o, .BAD_VALUE, "a Timestamp is out of range");
        Json.json_write_string(w, text);
    } else #if kind == .DURATION {
        text, ok := json_duration_text(v.seconds, v.nanos);
        if !ok return json_write_fail(o, .BAD_VALUE, "a Duration is out of range");
        Json.json_write_string(w, text);
    } else #if kind == .FIELD_MASK {
        b: String_Builder;
        b.allocator = temp;

        for v.paths {
            if it_index append(*b, ",");

            upper := false;

            for c: it {
                if c >= #char "A" && c <= #char "Z" return json_write_fail(o, .BAD_VALUE, "a FieldMask path has an uppercase letter");

                if c == #char "_" {
                    if upper return json_write_fail(o, .BAD_VALUE, "a FieldMask path has a doubled underscore");
                    upper = true;
                    continue;
                }

                if upper {
                    if c < #char "a" || c > #char "z" return json_write_fail(o, .BAD_VALUE, "a FieldMask path has an underscore before a non-letter");
                    append(*b, c - 32);
                    upper = false;
                } else append(*b, c);
            }

            if upper return json_write_fail(o, .BAD_VALUE, "a FieldMask path ends in an underscore");
        }

        Json.json_write_string(w, builder_to_string(*b,, temp));
    } else #if kind == .STRUCT return json_write_map_body(o, *v.fields, 0, .[]);
    else #if kind == .LIST_VALUE {
        Json.json_begin_array(w);
        inner := true;

        for *v.values {
            Json.json_item(w, *inner);
            if !json_write_message(o, it) return json_write_path(o, "values", it_index);
        }

        Json.json_end_array(w, inner);
    } else #if kind == .VALUE {
        if v.kind.which == {
            case .NULL_VALUE;
                Json.json_write_null(w);
            case .NUMBER_VALUE;
                x := v.kind.number_value;
                if x != x || x > JSON_FLOAT_MAX || x < -JSON_FLOAT_MAX return json_write_fail(o, .BAD_VALUE, "a Value holds NaN or an infinity");

                json_write_number(w, x);
            case .STRING_VALUE;
                Json.json_write_string(w, v.kind.string_value);
            case .BOOL_VALUE;
                Json.json_write_bool(w, v.kind.bool_value);
            case .STRUCT_VALUE;
                return json_value(o, *v.kind.struct_value, 0, .[]);
            case .LIST_VALUE;
                return json_value(o, *v.kind.list_value, 0, .[]);
            case;
                return json_write_fail(o, .BAD_VALUE, "a Value has no kind set");
        }
    } else #if kind == .ANY {
        if !v.type_url.count && !v.value.count {
            Json.json_write_raw(w, "{}");
            return true;
        }

        type := json_find_type(*o.options, v.type_url);
        if !type return json_write_fail(o, .UNKNOWN_TYPE, tprint("an Any holds `%`, which isn't in the registry", v.type_url));

        Json.json_begin_object(w);
        first := true;

        Json.json_key(w, *first, "\"@type\"");
        Json.json_write_string(w, v.type_url);
        if !type.run(o, null, v.value, null) return false;

        Json.json_end_object(w, false);
    }

    return true;
}

json_find_type :: (options: *Json_Options, url: string) -> *Json_Type {
    name := url;
    slash := find_index_from_right(url, #char "/");

    if slash >= 0 name = slice(url, slash + 1, url.count - slash - 1);

    for *options.registry if it.name == name return it;

    return null;
}

// ---- reading ----

json_read_message :: (i: *Json_In, v: *$T, $in_any := false) -> bool {
    KIND :: #run json_well_known_of(type_info(T));

    #if KIND != .NONE && KIND != .EMPTY return json_read_special(i, v, KIND);
    else {
        #insert -> string {
            return json_reader_code(T, in_any);
        };
    }
}

json_unknown :: (i: *Json_In, key: string) -> bool {
    if i.options.ignore_unknown_fields return Json.json_skip_value(*i.r);

    Json.json_unknown_field(*i.r, key);

    return false;
}

json_seen :: inline (i: *Json_In, seen: *u64, bit: s64, key: string) -> bool {
    mask := (cast(u64) 1) << cast(u64) (bit & 63);

    if seen[bit >> 6] & mask {
        Json.json_duplicate_field(*i.r, key);
        return false;
    }

    seen[bit >> 6] |= mask;

    return true;
}

// Whether `null` is a value of T rather than absence: google.protobuf.Value, and NullValue.
json_takes_null :: ($T: Type, $flags: Json_Flags) -> bool {
    #if flags & .NULL return true;
    else return #run json_well_known_of(type_info(T)) == .VALUE;
}

// A singular, optional or repeated field.
json_get :: (i: *Json_In, value: *$T, $flags: Json_Flags, names: [] string) -> bool {
    r := *i.r;
    info :: #run type_info(T);

    #if #run json_is_optional(type_info(T)) {
        V :: type_of(value.value);
        if !json_takes_null(V, flags) && Json.json_next_is_null(r) return Json.json_read_null(r);

        item: V;
        if !json_read_item(i, *item, flags, names) return false;

        if !i.skipped {
            if value.has release_scalar(*i.pr, *value.value, json_scalar_kind(V));
            value.value = item;
            value.has = true;
        }
    } else #if info.type == .ARRAY && (cast(*Type_Info_Array) info).array_type == .RESIZABLE {
        if Json.json_next_is_null(r) return Json.json_read_null(r);
        return json_read_array(i, value, flags, names);
    } else #if info.type == .POINTER {
        if !json_takes_null(T, flags) && Json.json_next_is_null(r) return Json.json_read_null(r);
        return json_read_item(i, value, flags, names);
    } else {
        if !json_takes_null(T, flags) && Json.json_next_is_null(r) return Json.json_read_null(r);

        item: T;
        if !json_read_item(i, *item, flags, names) return false;

        if !i.skipped {
            release_scalar(*i.pr, value, json_scalar_kind(T));
            value.* = item;
        }
    }

    i.skipped = false;

    return true;
}

json_scalar_kind :: ($T: Type) -> Scalar {
    #if T == string return .STRING;
    else #if T == [] u8 return .BYTES;
    else return .INT32;
}

// A member of a oneof; `null` leaves the oneof alone, two members are a duplicate.
json_get_case :: (i: *Json_In, variant: *$V, member: *$M, $index: s64, seen: *u64, bit: s64, key: string, $flags: Json_Flags, names: [] string) -> bool {
    r := *i.r;

    if !json_takes_null(M, flags) && Json.json_next_is_null(r) return Json.json_read_null(r);
    if !json_seen(i, seen, bit, key) return false;

    item: M;

    #if type_info(M).type == .POINTER if cast(s64) variant.which == index item = member.*;

    if !json_read_item(i, *item, flags, names) {
        #if type_info(M).type == .POINTER if cast(s64) variant.which != index && item {
            release_field(*i.pr, *item, .MESSAGE);
        }

        return false;
    }

    if i.skipped {
        i.skipped = false;
        return true;
    }

    #if type_info(M).type == .POINTER {
        if cast(s64) variant.which == index {
            member.* = item;
            return true;
        }
    }

    release_oneof(*i.pr, variant);
    variant.which = cast(type_of(variant.which)) index;
    member.* = item;

    return true;
}

json_read_array :: (i: *Json_In, values: *[..] $E, $flags: Json_Flags, names: [] string) -> bool {
    r := *i.r;
    if !Json.json_begin_array(r) return false;

    first := true;

    while true {
        status := Json.json_array_next(r, *first);
        if status == .END return true;
        if status == .ERROR return false;

        if !json_takes_null(E, flags) && Json.json_next_is_null(r) {
            Json.json_fail(r, .TYPE, "null is not allowed in a repeated field");
            Json.json_path_index(r, values.count);
            return false;
        }

        item: E;

        if !json_read_item(i, *item, flags, names) {
            #if type_info(E).type == .STRUCT release_entry(*i.pr, *item);
            Json.json_path_index(r, values.count);
            return false;
        }

        if i.skipped {
            i.skipped = false;
            continue;
        }

        append_decoded(*i.pr, values, item);
    }

    return true;
}

Json_Map_Mode :: enum u8 {
    HASH_MAP;
    TABLE;
    ARRAY;
    CUSTOM;
}

// A map field, whose entries are E (the message's Entry_Codec) and whose owner is O.
json_get_map :: (i: *Json_In, map: *$M, $E: Type, $O: Type, $flags: Json_Flags, names: [] string) -> bool {
    r := *i.r;
    if Json.json_next_is_null(r) return Json.json_read_null(r);
    if !Json.json_begin_object(r) return false;

    MODE :: #run json_map_mode(type_info(M));
    first := true;

    while true {
        text, status := Json.json_object_next(r, *first);
        if status == .END return true;
        if status == .ERROR return false;

        k: E.K;
        if !json_map_key(i, text, r.key_start, *k) return false;

        if !json_takes_null(E.V, flags) && Json.json_next_is_null(r) return Json.json_fail(r, .TYPE, "null is not allowed as a map value");

        v: E.V = #run entry_default(E.V);

        if !json_read_item(i, *v, flags, names) {
            #if type_info(E.V).type == .POINTER if v release_field(*i.pr, *v, .MESSAGE);
            return false;
        }

        if i.skipped {
            i.skipped = false;
            continue;
        }

        #if E.value_kind == .MESSAGE if !v v = new_message(*i.pr, type_of(v.*));

        #if MODE == .ARRAY array_add(map, .{ k, v });
        else #if MODE == .CUSTOM O.proto_map_insert(map, k, v);
        else {
            #if MODE == .HASH_MAP p := proto_maps.find(map, k);
            else p := proto_tables.table_find_pointer(map, k);

            if p {
                release_field(*i.pr, p, E.value_kind);
                p.* = v;
            } else {
                #if MODE == .HASH_MAP proto_maps.insert(map, k, v);
                else proto_tables.table_set(map, k, v);
            }
        }
    }

    return true;
}

json_map_key :: (i: *Json_In, text: string, start: s64, out: *$K) -> bool {
    r := *i.r;

    #if K == string {
        out.* = json_keep(i, text, start);
        return true;
    } else #if K == bool {
        if text == "true" out.* = true;
        else if text == "false" out.* = false;
        else return Json.json_fail_at(r, .INVALID_VALUE, "a bool map key must be `true` or `false`", start);
        return true;
    } else {
        if !json_integer_into(text, out) return Json.json_fail_at(r, .INVALID_VALUE, tprint("map key `%` isn't a %", text, K), start);
        return true;
    }
}

// Copies a string read at `start` into the decode storage.
json_keep :: (i: *Json_In, text: string, start: s64) -> string {
    if !text.count return "";

    i.pr.pos = start;
    bytes := payload_bytes(*i.pr, text.count);

    memcpy(bytes.data, text.data, text.count);

    return .{ bytes.count, bytes.data };
}

// One value of a field, element or map value. Sets i.skipped for an ignored unknown enum name.
json_read_item :: (i: *Json_In, out: *$T, $flags: Json_Flags, names: [] string) -> bool {
    r := *i.r;
    info :: #run type_info(T);

    #if T == bool {
        value, ok := Json.json_read_bool(r);
        out.* = value;
        return ok;
    } else #if T == s32 || T == s64 || T == u32 || T == u64 {
        return json_read_integer(i, out);
    } else #if T == float32 || T == float64 {
        return json_read_float(i, out);
    } else #if T == string {
        Json.json_peek(r);
        start := r.pos;
        text, ok := Json.json_read_name(r);

        if !ok return false;

        out.* = json_keep(i, text, start);

        return true;
    } else #if T == [] u8 {
        Json.json_peek(r);
        start := r.pos;
        text, ok := Json.json_read_name(r);

        if !ok return false;

        i.pr.pos = start;
        bytes, valid := json_base64(*i.pr, text);
        if !valid return Json.json_fail_at(r, .INVALID_VALUE, "invalid base64", start);

        out.* = bytes;

        return true;
    } else #if info.type == .ENUM {
        return json_read_enum(i, out, flags, names);
    } else #if info.type == .POINTER {
        if !out.* out.* = new_message(*i.pr, type_of(out.*.*));
        return json_read_message(i, out.*);
    } else return json_read_message(i, out);
}

json_read_integer :: (i: *Json_In, out: *$T) -> bool {
    r := *i.r;
    c := Json.json_peek(r);
    start := r.pos;
    text: string;
    ok: bool;

    if c == #char "\"" text, ok = Json.json_read_name(r);
    else if c == #char "-" || (c >= #char "0" && c <= #char "9") text, ok = Json.json_read_number(r);
    else return Json.json_fail(r, .TYPE, "expected an integer");

    if !ok return false;
    if !json_integer_into(text, out) return Json.json_fail_at(r, .INVALID_VALUE, tprint("`%` isn't a %", text, T), start);

    return true;
}

// Parses an integer written in any JSON number form (`1e3`, `100.0`), exactly and in range.
json_integer_into :: (text: string, out: *$T) -> bool {
    negative, magnitude, ok := json_integer(text);
    if !ok return false;

    #if T == s32 || T == s64 {
        LOW :: #run -> s64 {
            if T == s32 return -2147483648;
            return -9223372036854775807 - 1;
        };

        HIGH :: #run -> u64 {
            if T == s32 return 2147483647;
            return 9223372036854775807;
        };

        if negative {
            if magnitude > cast(u64) (-(LOW + 1)) + 1 return false;
            out.* = cast,no_check(T) (~magnitude + 1);
        } else {
            if magnitude > HIGH return false;
            out.* = cast,no_check(T) magnitude;
        }
    } else {
        if negative && magnitude != 0 return false;
        #if T == u32 if magnitude > 4294967295 return false;
        out.* = cast,no_check(T) magnitude;
    }

    return true;
}

// The sign and magnitude of a JSON number that is an integer; false when it's not a number,
// has a fraction, or is past u64.
json_integer :: (text: string) -> negative: bool, magnitude: u64, ok: bool {
    i := 0;
    negative := false;

    if i < text.count && text[i] == #char "-" {
        negative = true;
        i += 1;
    }

    int_start := i;

    while i < text.count && json_digit(text[i]) i += 1;

    int_end := i;

    if int_end == int_start return false, 0, false;
    if text[int_start] == #char "0" && int_end - int_start > 1 return false, 0, false;

    frac_start := i;
    frac_end := i;

    if i < text.count && text[i] == #char "." {
        i += 1;
        frac_start = i;
        while i < text.count && json_digit(text[i]) i += 1;

        frac_end = i;
        if frac_end == frac_start return false, 0, false;
    }

    exponent: s64;

    if i < text.count && (text[i] == #char "e" || text[i] == #char "E") {
        i += 1;
        sign: s64 = 1;

        if i < text.count && (text[i] == #char "+" || text[i] == #char "-") {
            if text[i] == #char "-" sign = -1;
            i += 1;
        }

        digits := i;

        while i < text.count && json_digit(text[i]) {
            if exponent < 100000 exponent = exponent * 10 + text[i] - #char "0";
            i += 1;
        }

        if i == digits return false, 0, false;

        exponent *= sign;
    }

    if i != text.count return false, 0, false;

    // The digits of the integer and fraction parts, times 10^scale.
    digit_at :: (k: s64) -> u8 #expand {
        if k < int_end - int_start return text[int_start + k];
        return text[frac_start + k - (int_end - int_start)];
    }

    count := (int_end - int_start) + (frac_end - frac_start);
    scale := exponent - (frac_end - frac_start);
    first := 0;

    while count > 0 && digit_at(count - 1) == #char "0" {
        count -= 1;
        scale += 1;
    }

    while first < count && digit_at(first) == #char "0" first += 1;

    if first == count return negative, 0, true;
    if scale < 0 return false, 0, false;
    if count - first + scale > 20 return false, 0, false;

    magnitude: u64;

    for k: first..count - 1 {
        d := cast(u64) (digit_at(k) - #char "0");
        if magnitude > (0xffff_ffff_ffff_ffff - d) / 10 return false, 0, false;
        magnitude = magnitude * 10 + d;
    }

    for 1..scale {
        if magnitude > 0xffff_ffff_ffff_ffff / 10 return false, 0, false;
        magnitude *= 10;
    }

    return negative, magnitude, true;
}

json_digit :: inline (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
}

json_read_float :: (i: *Json_In, out: *$T) -> bool {
    r := *i.r;
    c := Json.json_peek(r);

    if c != #char "\"" {
        #if T == float32 return Json.json_read_float32(r, out);
        else return Json.json_read_float64(r, out);
    }

    start := r.pos;
    text, ok := Json.json_read_name(r);

    if !ok return false;

    if text == "NaN" {
        out.* = cast(T) float_default(false, true);
        return true;
    }

    if text == "Infinity" || text == "-Infinity" {
        out.* = cast(T) float_default(text[0] == #char "-", false);
        return true;
    }

    if !text.count || (text[0] != #char "-" && !json_digit(text[0])) return Json.json_fail_at(r, .INVALID_VALUE, tprint("`%` isn't a number", text), start);

    sub: Json.Json_Reader;

    Json.json_reader_init(*sub, text);
    defer Json.json_reader_free(*sub);

    #if T == float32 ok = Json.json_read_float32(*sub, out);
    else ok = Json.json_read_float64(*sub, out);

    if sub.error.message.data Basic.free(sub.error.message);
    if !ok || sub.pos != text.count return Json.json_fail_at(r, .INVALID_VALUE, tprint("`%` isn't a number", text), start);

    return true;
}

json_read_enum :: (i: *Json_In, out: *$T, $flags: Json_Flags, names: [] string) -> bool {
    r := *i.r;
    c := Json.json_peek(r);
    start := r.pos;
    info := type_info(T);

    #if flags & .NULL if c == #char "n" {
        out.* = cast(T) 0;
        return Json.json_read_null(r);
    }

    if c == #char "\"" {
        text, ok := Json.json_read_name(r);
        if !ok return false;

        list := ifx names.count then names else info.names;

        for list if it == text {
            out.* = cast(T) info.values[it_index];
            return true;
        }

        if i.options.ignore_unknown_fields {
            i.skipped = true;
            return true;
        }

        return Json.json_fail_at(r, .INVALID_VALUE, tprint("unknown enum value `%`", text), start);
    }

    if c != #char "-" && !json_digit(c) return Json.json_fail(r, .TYPE, "expected an enum name or number");

    number: s32;
    if !json_read_integer(i, *number) return false;

    #if flags & .CLOSED {
        known := false;
        for info.values if it == number known = true;

        if !known {
            if i.options.ignore_unknown_fields {
                i.skipped = true;
                return true;
            }

            return Json.json_fail_at(r, .INVALID_VALUE, tprint("unknown enum value %", number), start);
        }
    }

    out.* = cast(T) number;

    return true;
}

// Decodes standard or URL-safe base64, padded or not, into decode storage.
json_base64 :: (pr: *Reader, text: string) -> [] u8, bool {
    n := text.count;
    while n && text[n - 1] == #char "=" n -= 1;

    if text.count - n > 2 || n % 4 == 1 return .{}, false;
    if text.count != n && text.count % 4 != 0 return .{}, false;

    count := n / 4 * 3;
    if n % 4 count += n % 4 - 1;

    bytes := payload_bytes(pr, count);
    bits: u32;
    held := 0;
    pos := 0;

    for k: 0..n - 1 {
        c := text[k];
        v: u32;

        if c >= #char "A" && c <= #char "Z" v = c - #char "A";
        else if c >= #char "a" && c <= #char "z" v = c - #char "a" + 26;
        else if json_digit(c) v = c - #char "0" + 52;
        else if c == #char "+" || c == #char "-" v = 62;
        else if c == #char "/" || c == #char "_" v = 63;
        else return .{}, false;

        bits = (bits << 6) | v;
        held += 6;

        if held >= 8 {
            held -= 8;
            bytes[pos] = cast(u8) ((bits >> cast(u32) held) & 255);
            pos += 1;
        }
    }

    return bytes, true;
}

json_read_any_body :: (i: *Json_In, v: *$T) -> bool {
    KIND :: #run json_well_known_of(type_info(T));

    #if KIND != .NONE && KIND != .EMPTY {
        r := *i.r;
        if !Json.json_begin_object(r) return false;

        first := true;
        seen := false;

        while true {
            key, status := Json.json_object_next(r, *first);
            if status == .END return true;
            if status == .ERROR return false;

            if key == "@type" {
                if !Json.json_skip_value(r) return false;
            } else if key == "value" {
                if seen {
                    Json.json_duplicate_field(r, key);
                    return false;
                }

                seen = true;
                if !json_read_special(i, v, KIND) return false;
            } else if !json_unknown(i, key) return false;
        }

        return true;
    } else return json_read_message(i, v, true);
}

json_read_special :: (i: *Json_In, v: *$T, $kind: Well_Known) -> bool {
    r := *i.r;

    #if kind == .WRAPPER return json_get(i, *v.value, 0, .[]);
    else #if kind == .TIMESTAMP || kind == .DURATION {
        Json.json_peek(r);
        start := r.pos;
        text, ok := Json.json_read_name(r);

        if !ok return false;

        seconds: s64;
        nanos: s32;

        #if kind == .TIMESTAMP ok = json_parse_timestamp(text, *seconds, *nanos);
        else ok = json_parse_duration(text, *seconds, *nanos);

        if !ok return Json.json_fail_at(r, .INVALID_VALUE, tprint("`%` isn't a valid %", text, ifx kind == .TIMESTAMP then "Timestamp" else "Duration"), start);

        v.seconds = seconds;
        v.nanos = nanos;

        return true;
    } else #if kind == .FIELD_MASK {
        Json.json_peek(r);
        start := r.pos;
        text, ok := Json.json_read_name(r);

        if !ok return false;
        if !text.count return true;

        rest := text;

        while true {
            comma := find_index_from_left(rest, #char ",");
            part := rest;

            if comma >= 0 part.count = comma;

            b: String_Builder;

            for c: part {
                if c == #char "_" return Json.json_fail_at(r, .INVALID_VALUE, "a FieldMask path in JSON has no underscores", start);

                if c >= #char "A" && c <= #char "Z" {
                    append(*b, #char "_");
                    append(*b, c + 32);
                } else append(*b, c);
            }

            append_decoded(*i.pr, *v.paths, builder_to_string(*b));

            if comma < 0 break;

            rest = slice(rest, comma + 1, rest.count - comma - 1);
        }

        return true;
    } else #if kind == .STRUCT {
        return json_get_map(i, *v.fields, T.proto_entry_1, T, 0, .[]);
    } else #if kind == .LIST_VALUE {
        if Json.json_next_is_null(r) return Json.json_fail(r, .TYPE, "expected an array");
        return json_read_array(i, *v.values, 0, .[]);
    } else #if kind == .VALUE {
        c := Json.json_peek(r);
        Kind :: type_of(v.kind);

        if c == #char "n" {
            if !Json.json_read_null(r) return false;
            release_oneof(*i.pr, *v.kind);
            v.kind.which = .NULL_VALUE;
        } else if c == #char "t" || c == #char "f" {
            value, ok := Json.json_read_bool(r);
            if !ok return false;

            release_oneof(*i.pr, *v.kind);
            v.kind.which = .BOOL_VALUE;
            v.kind.bool_value = value;
        } else if c == #char "\"" {
            start := r.pos;
            text, ok := Json.json_read_name(r);

            if !ok return false;

            release_oneof(*i.pr, *v.kind);
            v.kind.which = .STRING_VALUE;
            v.kind.string_value = json_keep(i, text, start);
        } else if c == #char "{" {
            if v.kind.which != .STRUCT_VALUE {
                release_oneof(*i.pr, *v.kind);
                v.kind.which = .STRUCT_VALUE;
            }

            return json_read_item(i, *v.kind.struct_value, 0, .[]);
        } else if c == #char "[" {
            if v.kind.which != .LIST_VALUE {
                release_oneof(*i.pr, *v.kind);
                v.kind.which = .LIST_VALUE;
            }

            return json_read_item(i, *v.kind.list_value, 0, .[]);
        } else {
            x: float64;
            if !Json.json_read_float64(r, *x) return false;

            release_oneof(*i.pr, *v.kind);
            v.kind.which = .NUMBER_VALUE;
            v.kind.number_value = x;
        }

        return true;
    } else #if kind == .ANY {
        return json_read_any(i, v);
    }
}

// An Any: finds "@type" wherever it is, then reads the rest as that type.
json_read_any :: (i: *Json_In, v: *$T) -> bool {
    r := *i.r;

    Json.json_peek(r);

    start := r.pos;
    depth := r.depth;
    url := "";
    url_start := -1;
    members := 0;

    if !Json.json_begin_object(r) return false;

    first := true;

    while true {
        key, status := Json.json_object_next(r, *first);
        if status == .END break;
        if status == .ERROR return false;

        members += 1;

        if key == "@type" {
            if url_start >= 0 {
                Json.json_duplicate_field(r, key);
                return false;
            }

            Json.json_peek(r);
            url_start = r.pos;
            text, ok := Json.json_read_name(r);
            if !ok return false;

            url = copy_string(text,, temp);
        } else if !Json.json_skip_value(r) return false;
    }

    if url_start < 0 {
        if members return Json.json_fail_at(r, .INVALID_VALUE, "an Any with fields needs `@type`", start);
        return true;
    }

    type := json_find_type(*i.options, url);
    if !type return Json.json_fail_at(r, .UNKNOWN_VARIANT, tprint("`%` isn't in the registry", url), url_start);

    end := r.pos;

    r.pos = start;
    r.depth = depth;

    bytes: [] u8;
    if !type.run(null, i, .{}, *bytes) return false;

    release_scalar(*i.pr, *v.type_url, .STRING);
    release_scalar(*i.pr, *v.value, .BYTES);
    v.type_url = json_keep(i, url, url_start);
    v.value = bytes;

    assert(r.pos == end);

    return true;
}

// ---- Timestamp and Duration ----

MIN_TIMESTAMP :: -62135596800;
MAX_TIMESTAMP :: 253402300799;
MAX_DURATION :: 315576000000;

json_days_to_civil :: (days: s64) -> s64, s64, s64 {
    z := days + 719468;
    era := (ifx z >= 0 then z else z - 146096) / 146097;
    doe := z - era * 146097;
    yoe := (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    doy := doe - (365 * yoe + yoe / 4 - yoe / 100);
    mp := (5 * doy + 2) / 153;
    d := doy - (153 * mp + 2) / 5 + 1;
    m := ifx mp < 10 then mp + 3 else mp - 9;
    y := yoe + era * 400;

    if m <= 2 y += 1;

    return y, m, d;
}

json_civil_to_days :: (year: s64, m: s64, d: s64) -> s64 {
    y := year;
    if m <= 2 y -= 1;

    era := (ifx y >= 0 then y else y - 399) / 400;
    yoe := y - era * 400;
    doy := (153 * (ifx m > 2 then m - 3 else m + 9) + 2) / 5 + d - 1;
    doe := yoe * 365 + yoe / 4 - yoe / 100 + doy;

    return era * 146097 + doe - 719468;
}

json_put_digits :: (b: *String_Builder, value: s64, width: s64) {
    digits: [20] u8;
    v := value;

    for < k: 0..width - 1 {
        digits[k] = cast(u8) (v % 10) + #char "0";
        v /= 10;
    }

    append(b, string.{ width, digits.data });
}

// ".000", ".000000" or ".000000000" by how many digits the nanoseconds need; none for zero.
json_put_nanos :: (b: *String_Builder, nanos: s64) {
    if !nanos return;

    append(b, ".");

    if nanos % 1000000 == 0 json_put_digits(b, nanos / 1000000, 3);
    else if nanos % 1000 == 0 json_put_digits(b, nanos / 1000, 6);
    else json_put_digits(b, nanos, 9);
}

json_timestamp_text :: (seconds: s64, nanos: s32) -> string, bool {
    if seconds < MIN_TIMESTAMP || seconds > MAX_TIMESTAMP || nanos < 0 || nanos > 999999999 return "", false;

    days := seconds / 86400;
    rest := seconds % 86400;

    if rest < 0 {
        days -= 1;
        rest += 86400;
    }

    y, m, d := json_days_to_civil(days);
    b: String_Builder;
    b.allocator = temp;

    json_put_digits(*b, y, 4);
    append(*b, "-");
    json_put_digits(*b, m, 2);
    append(*b, "-");
    json_put_digits(*b, d, 2);
    append(*b, "T");
    json_put_digits(*b, rest / 3600, 2);
    append(*b, ":");
    json_put_digits(*b, rest / 60 % 60, 2);
    append(*b, ":");
    json_put_digits(*b, rest % 60, 2);
    json_put_nanos(*b, nanos);
    append(*b, "Z");

    return builder_to_string(*b,, temp), true;
}

json_duration_text :: (seconds: s64, nanos: s32) -> string, bool {
    if seconds < -MAX_DURATION || seconds > MAX_DURATION || nanos < -999999999 || nanos > 999999999 return "", false;
    if (seconds > 0 && nanos < 0) || (seconds < 0 && nanos > 0) return "", false;

    b: String_Builder;
    b.allocator = temp;

    if seconds < 0 || nanos < 0 append(*b, "-");

    print(*b, "%", ifx seconds < 0 then -seconds else seconds);
    json_put_nanos(*b, ifx nanos < 0 then -cast(s64) nanos else nanos);
    append(*b, "s");

    return builder_to_string(*b,, temp), true;
}

// Reads `count` digits at `pos` as a number; -1 when any isn't a digit.
json_number_at :: (text: string, pos: s64, count: s64) -> s64 {
    if pos + count > text.count return -1;

    value: s64;

    for k: pos..pos + count - 1 {
        if !json_digit(text[k]) return -1;
        value = value * 10 + text[k] - #char "0";
    }

    return value;
}

// Up to nine fraction digits after a `.` at `pos`, as nanoseconds; returns the next position.
json_fraction :: (text: string, pos: s64) -> s64, s64, bool {
    if pos >= text.count || text[pos] != #char "." return 0, pos, true;

    k := pos + 1;
    nanos: s64;
    digits := 0;

    while k < text.count && json_digit(text[k]) {
        if digits == 9 return 0, k, false;
        nanos = nanos * 10 + text[k] - #char "0";
        digits += 1;
        k += 1;
    }

    if !digits return 0, k, false;
    for digits..8 nanos *= 10;

    return nanos, k, true;
}

json_parse_timestamp :: (text: string, seconds: *s64, nanos: *s32) -> bool {
    if text.count < 20 return false;

    year := json_number_at(text, 0, 4);
    month := json_number_at(text, 5, 2);
    day := json_number_at(text, 8, 2);
    hour := json_number_at(text, 11, 2);
    minute := json_number_at(text, 14, 2);
    second := json_number_at(text, 17, 2);

    if year < 0 || month < 1 || month > 12 || day < 1 || hour < 0 || hour > 23 || minute < 0 || minute > 59 || second < 0 || second > 59 return false;
    if text[4] != #char "-" || text[7] != #char "-" || text[10] != #char "T" || text[13] != #char ":" || text[16] != #char ":" return false;

    month_days := s64.[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    limit := month_days[month - 1];

    if month == 2 && year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) limit = 29;
    if day > limit return false;

    fraction, pos, ok := json_fraction(text, 19);
    if !ok || pos >= text.count return false;

    offset: s64;

    if text[pos] == #char "Z" {
        if pos + 1 != text.count return false;
    } else if text[pos] == #char "+" || text[pos] == #char "-" {
        if pos + 6 != text.count || text[pos + 3] != #char ":" return false;

        h := json_number_at(text, pos + 1, 2);
        m := json_number_at(text, pos + 4, 2);

        if h < 0 || h > 23 || m < 0 || m > 59 return false;

        offset = h * 3600 + m * 60;
        if text[pos] == #char "-" offset = -offset;
    } else return false;

    total := json_civil_to_days(year, month, day) * 86400 + hour * 3600 + minute * 60 + second - offset;
    if total < MIN_TIMESTAMP || total > MAX_TIMESTAMP return false;

    seconds.* = total;
    nanos.* = cast(s32) fraction;

    return true;
}

json_parse_duration :: (text: string, seconds: *s64, nanos: *s32) -> bool {
    pos := 0;
    negative := false;

    if pos < text.count && text[pos] == #char "-" {
        negative = true;
        pos += 1;
    }

    start := pos;
    whole: s64;

    while pos < text.count && json_digit(text[pos]) {
        if whole > MAX_DURATION return false;
        whole = whole * 10 + text[pos] - #char "0";
        pos += 1;
    }

    if pos == start || whole > MAX_DURATION return false;

    fraction, next, ok := json_fraction(text, pos);
    if !ok || next + 1 != text.count || text[next] != #char "s" return false;

    seconds.* = ifx negative then -whole else whole;
    nanos.* = cast(s32) (ifx negative then -fraction else fraction);

    return true;
}

// ---- compile time: the code for one message ----

json_is_optional :: (info: *Type_Info) -> bool {
    return info.type == .STRUCT && (cast(*Type_Info_Struct) info).polymorph_source_struct == type_info(Optional);
}

// A oneof: an Extensions/Variant, known by its `which` tag and `case_infos` constant.
json_is_variant :: (info: *Type_Info) -> bool {
    if info.type != .STRUCT return false;

    infos := false;
    which := false;

    for (cast(*Type_Info_Struct) info).members {
        if it.name == "case_infos" && it.flags & .CONSTANT infos = true;
        if it.name == "which" which = true;
    }

    return infos && which;
}

json_map_mode :: (info: *Type_Info) -> Json_Map_Mode {
    if info.type == .ARRAY return .ARRAY;

    source := (cast(*Type_Info_Struct) info).polymorph_source_struct;
    if source == type_info(proto_maps.Map) return .HASH_MAP;
    if source == type_info(proto_tables.Table) return .TABLE;

    return .CUSTOM;
}

json_well_known_of :: (type: *Type_Info) -> Well_Known {
    info := type;
    if info.type == .POINTER info = (cast(*Type_Info_Pointer) info).pointer_to;
    if info.type != .STRUCT return .NONE;

    s := cast(*Type_Info_Struct) info;

    for s.members {
        if it.name != "proto_well_known" || !(it.flags & .CONSTANT) || it.offset_into_constant_storage < 0 continue;
        return (cast(*Well_Known) (s.constant_storage.data + it.offset_into_constant_storage)).*;
    }

    return .NONE;
}

Json_Plan :: struct {
    access: string;
    json: string;
    proto: string;
    flags: string;
    names: string;
    is_map: bool;
    entry: string;
    oneof: string;
    index: s64;
    at: s64 = -1;
}

// lowerCamelCase: underscores dropped, the letter after one capitalized.
json_camel :: (name: string) -> string {
    b: String_Builder;
    upper := false;

    for name {
        if it == #char "_" {
            upper = true;
            continue;
        }

        c := it;
        if upper && c >= #char "a" && c <= #char "z" c -= 32;

        upper = false;
        append(*b, c);
    }

    return builder_to_string(*b);
}

// A Jai string literal of the JSON string literal of text.
json_jai_literal :: (text: string, quoted: bool) -> string {
    b: String_Builder;

    append(*b, "\"");
    if quoted append(*b, "\\\"");

    for text {
        if it == #char "\"" {
            if quoted append(*b, "\\\\\\\"");
            else append(*b, "\\\"");
        } else if it == #char "\\" {
            if quoted append(*b, "\\\\\\\\");
            else append(*b, "\\\\");
        } else if it < 32 || it >= 127 print(*b, "\\x%", formatInt(it, base = 16, minimum_digits = 2));
        else append(*b, it);
    }

    if quoted append(*b, "\\\"");

    append(*b, "\"");

    return builder_to_string(*b);
}

json_plan_member :: (plan: *[..] Json_Plan, member: *Type_Info_Struct_Member, access: string) {
    p: Json_Plan;
    p.access = access;
    p.proto = member.name;
    p.flags = "0";
    p.names = ".[]";

    rename := "";
    flags: [..] string;

    for member.notes {
        if begins_with(it, "proto_name=") p.proto = slice(it, 11, it.count - 11);
        else if begins_with(it, "proto_at_") p.at = string_to_int(slice(it, 9, it.count - 9));
        else if it == "proto_closed" array_add(*flags, ".CLOSED");
        else if it == "proto_null" array_add(*flags, ".NULL");
        else if begins_with(it, "proto_enum=") {
            b: String_Builder;

            append(*b, "string.[");

            for part, index: split(slice(it, 11, it.count - 11), ",") {
                if index append(*b, ", ");
                print(*b, "\"%\"", part);
            }

            append(*b, "]");
            p.names = builder_to_string(*b);
        } else if begins_with(it, "json(rename=") && ends_with(it, ")") {
            decoded, error := Json.from_json(string, slice(it, 12, it.count - 13));
            if error.kind == .NONE rename = decoded;
        }
    }

    if flags.count p.flags = tprint("Json_Flags%", join(..flags, " | ",, temp));

    p.json = ifx rename.data then rename else json_camel(p.proto);

    array_add(plan, p);
}

json_plan :: (T: Type) -> [..] Json_Plan, s64 {
    plan: [..] Json_Plan;
    info := cast(*Type_Info_Struct) type_info(T);
    entries: [..] string;
    oneofs := 0;

    for info.members if it.flags & .CONSTANT && begins_with(it.name, "proto_entry_") array_add(*entries, it.name);

    maps := 0;

    for *member: info.members {
        if member.flags & .CONSTANT || member.name == "_unknown" || member.name == "proto_storage" continue;

        type := member.type;

        if json_is_variant(type) {
            variant := cast(*Type_Info_Struct) type;
            types: [] Type;

            for variant.specified_parameters if it.name == "types" types = (cast(*[] Type) (variant.constant_storage.data + it.offset_into_constant_storage)).*;

            cases := cast(*Type_Info_Struct) types[0];
            index := 0;

            for *c: cases.members {
                if c.flags & .CONSTANT continue;

                if c.type.type != .VOID {
                    json_plan_member(*plan, c, tprint("%.%", member.name, c.name));
                    plan[plan.count - 1].oneof = member.name;
                    plan[plan.count - 1].index = index;
                }

                index += 1;
            }

            oneofs += 1;
            continue;
        }

        json_plan_member(*plan, member, member.name);

        is_entry_array := type.type == .ARRAY && (cast(*Type_Info_Array) type).element_type.type == .STRUCT && (cast(*Type_Info_Struct) (cast(*Type_Info_Array) type).element_type).polymorph_source_struct == type_info(Map_Entry);

        if (type.type == .STRUCT && !json_is_optional(type)) || is_entry_array {
            plan[plan.count - 1].is_map = true;
            plan[plan.count - 1].entry = entries[maps];
            maps += 1;
        }
    }

    // Oneof members marked with a declaration index move before later plain fields; the rest
    // keep the struct's order.
    placed: [..] Json_Plan;
    next := 0;

    for slot: 0..plan.count - 1 {
        found := false;

        for plan if it.at == slot {
            array_add(*placed, it);
            found = true;
            break;
        }

        if found continue;

        while next < plan.count && plan[next].at >= 0 next += 1;
        if next >= plan.count break;

        array_add(*placed, plan[next]);
        next += 1;
    }

    return placed, oneofs;
}

json_writer_code :: (T: Type) -> string {
    plan := json_plan(T);
    b: String_Builder;

    for plan {
        if it.oneof.count {
            forced := ifx it.flags == "0" then "Json_Flags.FORCE" else tprint("% | .FORCE", it.flags);
            print(*b, "if cast(s64) v.%.which == % && !json_put(o, first, *v.%, %, %, %, %) return false;\n", it.oneof, it.index, it.access, json_jai_literal(it.json, true), json_jai_literal(it.proto, true), forced, it.names);
        } else if it.is_map {
            print(*b, "if !json_put_map(o, first, *v.%, %, %, %, %) return false;\n", it.access, json_jai_literal(it.json, true), json_jai_literal(it.proto, true), it.flags, it.names);
        } else {
            print(*b, "if !json_put(o, first, *v.%, %, %, %, %) return false;\n", it.access, json_jai_literal(it.json, true), json_jai_literal(it.proto, true), it.flags, it.names);
        }
    }

    append(*b, "return true;\n");

    return builder_to_string(*b);
}

json_reader_code :: (T: Type, in_any: bool) -> string {
    plan, oneof_count := json_plan(T);
    b: String_Builder;
    words := (plan.count + oneof_count + 63) / 64;

    if !words words = 1;

    print(*b, "r := *i.r;\nif !Json.json_begin_object(r) return false;\nseen: [%] u64;\nfirst := true;\n", words);
    append(*b, "while true {\nkey, status := Json.json_object_next(r, *first);\nif status == .END return true;\nif status == .ERROR return false;\nfield := -1;\n");
    if in_any append(*b, "if key == \"@type\" { if !Json.json_skip_value(r) return false; continue; }\n");

    // Match by length, then by text.
    lengths: [..] s64;

    for plan {
        if !array_find(lengths, it.json.count) array_add(*lengths, it.json.count);
        if !array_find(lengths, it.proto.count) array_add(*lengths, it.proto.count);
    }

    if plan.count {
        append(*b, "if key.count == {\n");

        for length: lengths {
            print(*b, "case %;\n", length);
            word := "if";

            for plan {
                if it.json.count == length {
                    print(*b, "% key == % field = %;\n", word, json_jai_literal(it.json, false), it_index);
                    word = "else if";
                }

                if it.proto.count == length && it.proto != it.json {
                    print(*b, "% key == % field = %;\n", word, json_jai_literal(it.proto, false), it_index);
                    word = "else if";
                }
            }
        }

        append(*b, "}\n");
    }

    append(*b, "if field < 0 { if !json_unknown(i, key) return false; continue; }\nok := true;\nif field == {\n");

    oneofs: [..] string;

    for plan {
        print(*b, "case %;\n", it_index);

        if it.oneof.count {
            bit := array_find_index(oneofs, it.oneof);

            if bit < 0 {
                array_add(*oneofs, it.oneof);
                bit = oneofs.count - 1;
            }

            print(*b, "ok = json_get_case(i, *v.%, *v.%, %, seen.data, %, key, %, %);\n", it.oneof, it.access, it.index, plan.count + bit, it.flags, it.names);
        } else {
            print(*b, "if !json_seen(i, seen.data, %, key) return false;\n", it_index);
            if it.is_map print(*b, "ok = json_get_map(i, *v.%, T.%, T, %, %);\n", it.access, it.entry, it.flags, it.names);
            else print(*b, "ok = json_get(i, *v.%, %, %);\n", it.access, it.flags, it.names);
        }

        print(*b, "if !ok { Json.json_path_key(r, %); return false; }\n", json_jai_literal(it.json, false));
    }

    append(*b, "}\n}\nreturn true;\n");

    return builder_to_string(*b);
}

array_find_index :: (array: [..] string, value: string) -> s64 {
    for array if it == value return it_index;
    return -1;
}
