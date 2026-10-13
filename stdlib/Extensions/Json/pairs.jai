// Typed values from text that is not JSON: the key/value pairs of a URL query or a form, and the
// single strings of a path segment or a header. Each value is read by the type of the field its
// key names, with the same notes as JSON (`rename`, `alias`, `default`, `deny_unknown_fields`...),
// by turning the pairs into the JSON object that says the same thing and decoding that. So every
// rule and every error message (`missing field `q``, `expected an integer`, the field path) is
// the one `from_json` has.

// One `key=value` of a query or a form, already percent-decoded.
Text_Pair :: struct {
    key: string;
    value: string;
}

// How a field's text becomes a JSON value, decided from its type at compile time.
Pair_Kind :: enum u8 {
    UNKNOWN; // no field has this key: ignored
    UNKNOWN_DENIED; // no field has this key, and the struct has `deny_unknown_fields`
    TEXT; // a JSON string: strings and enums by name
    NUMBER; // numbers, and enums with `as_number`
    BOOL;
    TEXT_LIST; // repeated keys make an array
    NUMBER_LIST;
    BOOL_LIST;
}

// Decodes a `T` from `pairs`. A key that repeats gives an array for an array field, and is a
// "duplicate field" error for any other. Keys no field has are ignored unless `T` is marked
// `deny_unknown_fields`. Everything allocated (strings, arrays) comes from context.allocator and
// is released by `free_json`, as for `from_json`.
//
//     Search :: struct @json(default) { q: string; limit := 20; tags: [..] string; }
//     search, error := from_pairs(Search, pairs);
from_pairs :: ($T: Type, pairs: [] Text_Pair, options := Json_Options.{}) -> T, Json_Error {
    builder: String_Builder;

    append(*builder, "{");

    used := NewArray(pairs.count, bool,, temp);
    first := true;

    for pair, index: pairs {
        if used[index] continue;

        kind := pair_kind(T, pair.key);

        if kind == .UNKNOWN continue;
        if kind == .UNKNOWN_DENIED kind = .TEXT;

        if !first append(*builder, ",");

        first = false;

        append_json_string(*builder, pair.key);
        append(*builder, ":");

        if kind == .TEXT_LIST || kind == .NUMBER_LIST || kind == .BOOL_LIST {
            item_kind := ifx kind == .TEXT_LIST then Pair_Kind.TEXT else ifx kind == .NUMBER_LIST then Pair_Kind.NUMBER else Pair_Kind.BOOL;

            append(*builder, "[");

            separated := false;

            for later, j: pairs {
                if j < index || later.key != pair.key continue;

                used[j] = true;

                if separated append(*builder, ",");

                separated = true;

                append_pair_value(*builder, item_kind, later.value);
            }

            append(*builder, "]");
        } else {
            append_pair_value(*builder, kind, pair.value);
        }
    }

    append(*builder, "}");

    text := builder_to_string(*builder,, temp);

    return from_json(T, text, options);
}

// Decodes a `T` from one piece of text: an integer, float, bool, string or enum. `from_text(int,
// "42")` is 42; `from_text(Color, "red")` the member `red`; a string is taken as it is.
from_text :: ($T: Type, text: string, options := Json_Options.{}) -> T, Json_Error {
    kind :: #run text_kind(type_info(T));

    builder: String_Builder;

    append_pair_value(*builder, kind, text);

    literal := builder_to_string(*builder,, temp);

    return from_json(T, literal, options);
}

// The member of `T` that `key` names, from the notes (`rename`, `alias`...).
pair_kind :: ($T: Type, key: string) -> Pair_Kind {
    #insert -> string {
        return generate_pair_kinds(T);
    };
}

#scope_module

append_pair_value :: (builder: *String_Builder, kind: Pair_Kind, text: string) {
    if kind == .NUMBER && looks_like_number(text) {
        append(builder, text);
    } else if kind == .BOOL && (text == "true" || text == "false") {
        append(builder, text);
    } else {
        append_json_string(builder, text);
    }
}

// The grammar of a JSON number (RFC 8259): `-`, an integer part without leading zeros, then an
// optional fraction and exponent. Anything else goes on as a string, which the decoder rejects
// with its "expected an integer" error.
looks_like_number :: (text: string) -> bool {
    i := 0;
    count := text.count;

    if i < count && text[i] == #char "-" i += 1;
    if i >= count return false;

    if text[i] == #char "0" {
        i += 1;
    } else if text[i] >= #char "1" && text[i] <= #char "9" {
        while i < count && text[i] >= #char "0" && text[i] <= #char "9" i += 1;
    } else {
        return false;
    }

    if i < count && text[i] == #char "." {
        i += 1;
        digits := i;

        while i < count && text[i] >= #char "0" && text[i] <= #char "9" i += 1;

        if i == digits return false;
    }

    if i < count && (text[i] == #char "e" || text[i] == #char "E") {
        i += 1;

        if i < count && (text[i] == #char "+" || text[i] == #char "-") i += 1;

        digits := i;

        while i < count && text[i] >= #char "0" && text[i] <= #char "9" i += 1;

        if i == digits return false;
    }

    return i == count;
}

// How text for a value of this type is written as JSON, or an error for types text cannot hold.
scalar_kind :: (g: *Gen, info: *Type_Info, style: Enum_Style) -> Pair_Kind {
    if info.type == {
        case .INTEGER;
            if info.runtime_size > 8 {
                fail(g, "128-bit integers are not supported");

                return .TEXT;
            }

            return .NUMBER;
        case .FLOAT; return .NUMBER;
        case .BOOL; return .BOOL;
        case .STRING; return .TEXT;
        case .ENUM; return ifx style.as_number then Pair_Kind.NUMBER else Pair_Kind.TEXT;
    }

    return .UNKNOWN;
}

field_kind :: (g: *Gen, info: *Type_Info, style: Enum_Style) -> Pair_Kind {
    kind := scalar_kind(g, info, style);

    if kind != .UNKNOWN return kind;

    if info.type == .POINTER {
        pointee := (cast(*Type_Info_Pointer) info).pointer_to;

        if pointee.type != .VOID && pointee.type != .POINTER && pointee.type != .ARRAY {
            kind = scalar_kind(g, pointee, style);

            if kind != .UNKNOWN return kind;
        }
    }

    if info.type == .ARRAY {
        element := scalar_kind(g, (cast(*Type_Info_Array) info).element_type, style);

        if element == .TEXT return .TEXT_LIST;
        if element == .NUMBER return .NUMBER_LIST;
        if element == .BOOL return .BOOL_LIST;
    }

    fail(g, tprint("% cannot be read from text pairs (a query or a form holds numbers, bools, strings, enums and arrays of those)", type_label(info)));

    return .TEXT;
}

text_kind :: (info: *Type_Info) -> Pair_Kind {
    g: Gen;

    g.type_name = tprint("%", info.type);

    kind := scalar_kind(*g, info, .{});

    if kind == .UNKNOWN {
        compiler_report(tprint("Json: from_text reads integers, floats, bools, strings and enums, not %", type_label(info)));
    }

    return kind;
}

generate_pair_kinds :: (T: Type) -> string {
    g: Gen;
    info := type_info(T);

    g.type_name = tprint("%", T);

    if info.type != .STRUCT || is_json_value(info) || is_table(info) {
        fail(*g, "only a struct can be read from text pairs");

        return "return .UNKNOWN;";
    }

    plan := analyze(T);

    g.type_name = plan.type_name;

    if !plan.ok return "return .UNKNOWN;";

    if plan.transparent {
        fail(*g, "a `transparent` struct cannot be read from text pairs");

        return "return .UNKNOWN;";
    }

    for *leaf: plan.leaves {
        if leaf.skip_deserializing continue;

        g.field = leaf.member;

        kind := field_kind(*g, leaf.info, leaf.style);

        names: [..] string;

        array_add(*names, leaf.key);
        for leaf.aliases array_add(*names, it);

        for name: names print(*g.out, "if key == % return .%;\n", jai_quote(name), kind);
    }

    if plan.deny_unknown print(*g.out, "return .UNKNOWN_DENIED;\n");
    else print(*g.out, "return .UNKNOWN;\n");

    if !g.ok return "return .UNKNOWN;";

    return builder_to_string(*g.out,, temp);
}
