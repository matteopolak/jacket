// Parameter encoders written for the type of the value. The code for a `bool`, a `[] s64` or a
// `[] Nullable(string)` is made when something uses it, so a value costs its own conversion and
// no walk of type information. `param_of` sends one value as a query argument, `put_scalar` writes
// one element's binary form, and `array_param_of` sends a whole array; `copy.jai` uses the last two
// for the members of a COPY row. A value that arrives as `Any` is handed to these by
// `encode_param` (codecs.jai) when its type is one of PLAIN_TYPES, and goes through the slower
// walkers there otherwise. The typed query macros know each argument's type from the call and
// use `argument_param`, which skips that test.
#scope_module

// The types an `Any` argument is sent to a typed encoder for. Each name is written into the
// dispatch code by `plain_cases`, so the list is the only place to add one.
PLAIN_TYPES :: string.["s64", "string", "s32", "bool", "float64", "s16", "s8", "u8", "u16", "u32", "u64", "float32", "Uuid", "Timestamp", "Local_Timestamp", "Date", "Time", "Interval", "Numeric", "Json_Raw"];

// The OID of the binary form `put_scalar` writes for a `T`, or 0 when it writes none.
scalar_oid :: ($T: Type) -> Oid {
    #if T == bool {
        return .BOOL;
    } else #if type_info(T).type == .INTEGER {
        SIGNED :: #run (cast(*Type_Info_Integer) type_info(T)).signed;

        #if size_of(T) <= 1 || (size_of(T) == 2 && SIGNED) return .INT2;
        else #if size_of(T) <= 2 || (size_of(T) == 4 && SIGNED) return .INT4;
        else return .INT8;
    } else #if type_info(T).type == .FLOAT {
        #if size_of(T) == 4 return .FLOAT4;
        else return .FLOAT8;
    } else #if type_info(T).type == .STRING {
        return .TEXT;
    } else #if T == Uuid {
        return .UUID;
    } else #if T == Timestamp {
        return .TIMESTAMPTZ;
    } else #if T == Local_Timestamp {
        return .TIMESTAMP;
    } else #if T == Date {
        return .DATE;
    } else #if T == Time {
        return .TIME;
    } else #if T == Interval {
        return .INTERVAL;
    } else #if T == Numeric {
        return .NUMERIC;
    } else #if #run is_json_value(T) {
        return .JSONB;
    } else {
        return cast(Oid) 0;
    }
}

// Appends the binary form of one non-null `value` (its OID is `scalar_oid(T)`). Returns why it
// cannot be sent, or an empty string.
put_scalar :: (out: *[..] u8, value: *$T) -> string {
    #if T == bool {
        put_u8(out, xx value.*);
    } else #if type_info(T).type == .INTEGER {
        SIGNED :: #run (cast(*Type_Info_Integer) type_info(T)).signed;
        v := cast,no_check(s64) value.*;

        #if size_of(T) <= 1 || (size_of(T) == 2 && SIGNED) {
            put_u16(out, cast,no_check(u16) v);
        } else #if size_of(T) <= 2 || (size_of(T) == 4 && SIGNED) {
            put_u32(out, cast,no_check(u32) v);
        } else {
            #if size_of(T) == 8 && !SIGNED {
                if value.* > xx S64_MAX return "a u64 parameter above S64_MAX does not fit in int8";
            }

            put_u64(out, cast,no_check(u64) v);
        }
    } else #if type_info(T).type == .FLOAT {
        #if size_of(T) == 4 put_u32(out, (cast(*u32) value).*);
        else put_u64(out, (cast(*u64) value).*);
    } else #if type_info(T).type == .STRING {
        if contains_zero(value.*) return "a string parameter holds a zero byte, which Postgres text cannot store";

        put_bytes(out, value.*);
    } else #if T == Uuid {
        put_bytes(out, cast([] u8) value.bytes);
    } else #if T == Timestamp || T == Local_Timestamp || T == Time {
        put_u64(out, cast,no_check(u64) (cast(*s64) value).*);
    } else #if T == Date {
        put_u32(out, cast,no_check(u32) (cast(*s32) value).*);
    } else #if T == Interval {
        put_u64(out, cast,no_check(u64) value.microseconds);
        put_u32(out, cast,no_check(u32) value.days);
        put_u32(out, cast,no_check(u32) value.months);
    } else #if T == Numeric {
        // The binary form of a numeric is not its text, so the text is converted.
        bytes, why := numeric_binary(value.text);
        if why return why;

        put_bytes(out, bytes);
    } else #if #run is_json_value(T) {
        text, why := json_text_of(value);
        if why return why;

        put_u8(out, 1);
        put_bytes(out, text);
    } else {
        return tprint("Postgres has no encoding for parameters of type %", T);
    }

    return "";
}

// The document text of a `Json(T)` or a `Json_Raw`, written by the `write` the `Json` carries.
json_text_of :: (value: *$T) -> string, string {
    #if T == Json_Raw {
        return value.text, "";
    } else {
        if !value.write return "", "a Json value with no `write`: make it with Pg.json(value) or a declaration, not zeroed memory";

        builder: String_Builder;
        builder.allocator = temp;
        value.write(*value.value, *builder);

        return builder_to_string(*builder,, temp), "";
    }
}

// An array of bytes, which is a bytea.
is_bytes :: ($T: Type) -> bool {
    info := type_info(T);

    return info.type == .ARRAY && (cast(*Type_Info_Array) info).element_type == type_info(u8);
}

// An array of enums.
is_enum_array :: ($T: Type) -> bool {
    info := type_info(T);

    return info.type == .ARRAY && (cast(*Type_Info_Array) info).element_type.type == .ENUM;
}

// A `u64`, which text carries exactly and the signed int8 would not.
is_unsigned_64 :: ($T: Type) -> bool {
    info := type_info(T);

    return info.type == .INTEGER && info.runtime_size == 8 && !(cast(*Type_Info_Integer) info).signed;
}

// A `json` or `jsonb` document: a `Json(T)` or a `Json_Raw`.
is_json_value :: ($T: Type) -> bool {
    return is_json(T) || T == Json_Raw;
}

// Picks the format and OID for one plain argument, as `encode_param` does for an `Any`: numbers
// (but `u64` and floats), strings and bools go as text with no OID, so the server infers the type
// from the query; types whose meaning text would lose go as binary with their OID. Returns why
// the value cannot be sent, or an empty string.
//
// - value: the argument.
param_of :: (value: *$T) -> Param, string {
    #if #run is_json_value(T) {
        // No type OID: the server reads the text as json or jsonb, whichever the statement wants.
        text, why := json_text_of(value);
        return .{ data = text }, why;
    } else #if type_info(T).type == .STRING {
        if contains_zero(value.*) return .{}, "a string parameter holds a zero byte, which Postgres text cannot store";

        return .{ data = value.* }, "";
    } else #if #run is_unsigned_64(T) {
        return .{ data = tprint("%", value.*) }, "";
    } else #if type_info(T).type == .INTEGER {
        return .{ data = tprint("%", cast,no_check(s64) value.*) }, "";
    } else #if T == bool {
        return .{ data = ifx value.* then "t" else "f" }, "";
    } else #if T == Numeric {
        // Numerics go as text: the server parses them exactly.
        return .{ oid = xx Oid.NUMERIC, data = value.text }, "";
    } else {
        oid := scalar_oid(T);
        if !oid return .{}, tprint("Postgres has no encoding for parameters of type %", T);

        out: [..] u8;
        out.allocator = temp;

        why := put_scalar(*out, value);
        if why return .{}, why;

        return .{ oid = xx oid, format = .BINARY, data = cast(string) out }, "";
    }
}

// Whether an array of `E` holds arrays that are not byte strings: more than one dimension.
is_nested_array :: ($E: Type) -> bool {
    #if #run is_nullable(E) {
        return is_nested_array(type_of((cast(*E) null).value));
    } else {
        info := type_info(E);

        return info.type == .ARRAY && (cast(*Type_Info_Array) info).element_type != type_info(u8);
    }
}

// The OID of an array's elements: that of `put_scalar`, or bytea for byte strings.
array_element_oid :: ($E: Type) -> Oid {
    #if #run is_nullable(E) {
        return array_element_oid(type_of((cast(*E) null).value));
    } else #if type_info(E).type == .ARRAY {
        return .BYTEA;
    } else {
        return scalar_oid(E);
    }
}

// A one-dimensional binary array: dimensions, has-nulls, element OID, length, lower bound, then
// each element as a length and its binary form.
//
// - items: the elements, which may be `Nullable`.
array_param_of :: (items: [] $E) -> Param, string {
    #if #run is_nested_array(E) {
        return .{}, "multi-dimensional array parameters are not supported";
    } else {
        element_oid := array_element_oid(E);
        if !element_oid return .{}, tprint("Postgres has no encoding for parameters of type %", E);

        array_oid := array_oid_of(cast(u32) element_oid);
        if !array_oid return .{}, tprint("Postgres has no array type for % elements here", type_name(cast(u32) element_oid));

        elements: [..] u8;
        elements.allocator = temp;
        has_nulls := false;

        for *items {
            why := put_array_item(*elements, it, *has_nulls);
            if why return .{}, why;
        }

        out: [..] u8;
        out.allocator = temp;
        put_u32(*out, 1);
        put_u32(*out, xx has_nulls);
        put_u32(*out, cast(u32) element_oid);
        put_u32(*out, cast(u32) items.count);
        put_u32(*out, 1);
        put_bytes(*out, cast([] u8) elements);

        return .{ oid = array_oid, format = .BINARY, data = cast(string) out }, "";
    }
}

// One element of an array: its length (or -1 for NULL), then its binary form.
put_array_item :: (out: *[..] u8, item: *$E, has_nulls: *bool) -> string {
    #if #run is_nullable(E) {
        if item.null {
            has_nulls.* = true;
            put_u32(out, 0xffff_ffff);

            return "";
        }

        return put_array_item(out, *item.value, has_nulls);
    } else #if type_info(E).type == .ARRAY {
        view: [] u8 = item.*;

        put_u32(out, cast(u32) view.count);
        put_bytes(out, view);

        return "";
    } else {
        // The length is not known until the element is written, so it is filled in after.
        start := out.count;

        put_u32(out, 0);

        why := put_scalar(out, item);
        if why return why;

        patch_u32(out, start, cast(u32) (out.count - start - 4));

        return "";
    }
}

// Overwrites the four bytes at `at` with `value`, big-endian.
patch_u32 :: (out: *[..] u8, at: s64, value: u32) {
    bytes := out.data + at;

    (bytes + 0).* = cast,trunc(u8) (value >> 24);
    (bytes + 1).* = cast,trunc(u8) (value >> 16);
    (bytes + 2).* = cast,trunc(u8) (value >> 8);
    (bytes + 3).* = cast,trunc(u8) value;
}

// Whether `info` is one of PLAIN_TYPES.
is_plain_type :: (info: *Type_Info) -> bool {
    #insert -> string {
        builder: String_Builder;
        builder.allocator = temp;

        for name: PLAIN_TYPES print_to_builder(*builder, "if info == type_info(%) return true;\n", name);

        return builder_to_string(*builder);
    };

    return false;
}

// Whether `info` is a `Nullable` of one of PLAIN_TYPES.
is_plain_nullable :: (info: *Type_Info) -> bool {
    return is_nullable_info(info) && is_plain_type(nullable_value_type(info));
}

// Whether `info` is an array that `array_param_of` sends: not bytes, and of PLAIN_TYPES or of
// `Nullable` of one.
is_plain_array :: (info: *Type_Info) -> bool {
    if info.type != .ARRAY return false;

    element := (cast(*Type_Info_Array) info).element_type;

    if element == type_info(u8) return false;

    return is_plain_type(element) || is_plain_nullable(element);
}

// Whether `argument_param` has an encoder written for arguments of this type.
has_typed_encoder :: (info: *Type_Info) -> bool {
    return is_plain_type(info) || is_plain_nullable(info) || is_plain_array(info);
}

array_element_type :: (T: Type) -> Type {
    return cast(Type) (cast(*Type_Info_Array) type_info(T)).element_type;
}

// The query parameter for an argument whose type `T` the call site knows. Plain types, arrays of
// them and `Nullable`s of them go straight to their encoder with no test of the type; the value is
// read through the `Any` the call built. Anything else, or an `Any` of another type than `T`, is
// sent as `encode_param` would.
//
// - value: the argument, boxed.
// - T: the argument's static type.
argument_param :: (value: Any, $T: Type) -> Param, string {
    #if #run has_typed_encoder(type_info(T)) {
        if value.type == type_info(T) {
            #if #run is_plain_type(type_info(T)) {
                return param_of(cast(*T) value.value_pointer);
            } else #if #run is_plain_nullable(type_info(T)) {
                item := cast(*T) value.value_pointer;

                if item.null return .{ null = true }, "";

                return param_of(*item.value);
            } else {
                E :: #run array_element_type(T);
                view := any_array_view(value);

                return array_param_of(([] E).{ view.count / size_of(E), cast(*E) view.data });
            }
        }
    }

    return encode_param(value);
}

// The source of `any_param`: for each of PLAIN_TYPES, a test of the argument's type and a call of
// `param_of` made for that type.
param_cases :: () -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for name: PLAIN_TYPES {
        print_to_builder(*builder, "if info == type_info(%) { param, why := param_of(cast(*%) value.value_pointer); return param, why, true; }\n", name, name);
    }

    return builder_to_string(*builder);
}

// The source of `any_put_scalar`: the same tests, each writing the binary form for its type.
scalar_cases :: () -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for name: PLAIN_TYPES {
        print_to_builder(*builder, "if info == type_info(%) { why := put_scalar(out, cast(*%) value.value_pointer); return scalar_oid(%), why, true; }\n", name, name, name);
    }

    return builder_to_string(*builder);
}

// The source of `any_array_param`: an array whose elements are one of PLAIN_TYPES, or a
// `Nullable` of one, goes to the encoder written for that element type.
array_cases :: () -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for name: PLAIN_TYPES {
        if name == "u8" continue; // an array of bytes is a bytea

        print_to_builder(*builder, "if element == type_info(%) { param, why := array_param_of(([] %).{ count, cast(*%) view.data }); return param, why, true; }\n", name, name, name);
        print_to_builder(*builder, "if element == type_info(Nullable(%)) { param, why := array_param_of(([] Nullable(%)).{ count, cast(*Nullable(%)) view.data }); return param, why, true; }\n", name, name, name);
    }

    return builder_to_string(*builder);
}
