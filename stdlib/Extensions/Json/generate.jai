// Compile-time half of Json: reads a type's members and their @json notes, rejects bad
// declarations with a message naming the field, and writes the Jai source of the encoder and
// decoder for it. json_encode and json_decode `#insert` that source, so every type gets
// straight-line code with its field names, keys and types baked in.
//
// Everything here runs inside the compiler. Nothing here is called at run time.

#scope_module

// ---- renaming ----

Rename_Rule :: enum u8 {
    NONE;
    LOWERCASE;
    UPPERCASE;
    CAMEL;
    PASCAL;
    SNAKE;
    SCREAMING_SNAKE;
    KEBAB;
    SCREAMING_KEBAB;
}

parse_rule :: (text: string) -> Rename_Rule, bool {
    if text == "lowercase" return .LOWERCASE, true;
    if text == "UPPERCASE" return .UPPERCASE, true;
    if text == "camelCase" return .CAMEL, true;
    if text == "PascalCase" return .PASCAL, true;
    if text == "snake_case" return .SNAKE, true;
    if text == "SCREAMING_SNAKE_CASE" return .SCREAMING_SNAKE, true;
    if text == "kebab-case" return .KEBAB, true;
    if text == "SCREAMING-KEBAB-CASE" return .SCREAMING_KEBAB, true;

    return .NONE, false;
}

is_lower_byte :: (c: u8) -> bool {
    return c >= #char "a" && c <= #char "z";
}

is_upper_byte :: (c: u8) -> bool {
    return c >= #char "A" && c <= #char "Z";
}

lower_byte :: (c: u8) -> u8 {
    return ifx is_upper_byte(c) then c + 32 else c;
}

upper_byte :: (c: u8) -> u8 {
    return ifx is_lower_byte(c) then c - 32 else c;
}

// Splits an identifier into words: at `_` and `-`, at a lowercase letter followed by an
// uppercase one, and before the last capital of an acronym (`HTTPServer` is `HTTP`, `Server`).
split_words :: (name: string) -> [..] string {
    words: [..] string;
    start := -1;

    for i: 0..name.count {
        c: u8 = 0;
        if i < name.count c = name[i];

        boundary := i == name.count || c == #char "_" || c == #char "-";

        if !boundary && start >= 0 && i > 0 {
            before := name[i - 1];

            if is_upper_byte(c) && (is_lower_byte(before) || (before >= #char "0" && before <= #char "9")) boundary = true;
            else if is_upper_byte(c) && is_upper_byte(before) && i + 1 < name.count && is_lower_byte(name[i + 1]) boundary = true;
        }

        if boundary && start >= 0 {
            array_add(*words, slice(name, start, i - start));
            start = -1;
        }

        if !(i == name.count || c == #char "_" || c == #char "-") && start < 0 start = i;
    }

    return words;
}

apply_rule :: (name: string, rule: Rename_Rule) -> string {
    if rule == .NONE return name;

    builder: String_Builder;

    if rule == .LOWERCASE || rule == .UPPERCASE {
        for name append(*builder, ifx rule == .LOWERCASE then lower_byte(it) else upper_byte(it));

        return builder_to_string(*builder,, temp);
    }

    words := split_words(name);

    for word, word_index: words {
        if word_index > 0 {
            if rule == .SNAKE || rule == .SCREAMING_SNAKE append(*builder, #char "_");
            else if rule == .KEBAB || rule == .SCREAMING_KEBAB append(*builder, #char "-");
        }

        for c, i: word {
            upper := false;

            if rule == .SCREAMING_SNAKE || rule == .SCREAMING_KEBAB upper = true;
            else if i == 0 && (rule == .PASCAL || (rule == .CAMEL && word_index > 0)) upper = true;

            append(*builder, ifx upper then upper_byte(c) else lower_byte(c));
        }
    }

    return builder_to_string(*builder,, temp);
}

// ---- analysis ----

// What a field says about the enums inside its type (an enum has no notes of its own).
Enum_Style :: struct {
    rule: Rename_Rule;
    as_number: bool;
}

Leaf :: struct {
    member: string; // the Jai name
    path: string; // from the struct being coded, through flattened structs: `address.zip`
    owner: string; // the path of the struct that declares it, "" for the struct being coded
    info: *Type_Info;
    key: string;
    aliases: [..] string;
    skip_serializing: bool;
    skip_deserializing: bool;
    skip_if: string;
    optional: bool;
    with: string;
    style: Enum_Style;
}

Plan :: struct {
    type_name: string;
    ok := true;
    transparent: bool;
    deny_unknown: bool;
    leaves: [..] Leaf;
}

complain :: (type_name: string, field: string, message: string) {
    where := ifx field.count then tprint("%.%", type_name, field) else type_name;
    compiler_report(tprint("Json: %: %", where, message));
}

// The options inside a member's or struct's `@json(...)` notes, as `name` or `name=value`.
Option :: struct {
    name: string;
    value: string;
    has_value: bool;
}

read_options :: (notes: [] string) -> [..] Option {
    options: [..] Option;

    for notes {
        note := it;

        if !begins_with(note, "json(") || !ends_with(note, ")") continue;

        rest := slice(note, 5, note.count - 6);

        while rest.count {
            found, piece, after := split_from_left(rest, #char ",");

            if !found {
                piece = rest;
                after = "";
            }

            option: Option;
            equals := find_index_from_left(piece, #char "=");

            if equals >= 0 {
                option.name = slice(piece, 0, equals);
                option.value = slice(piece, equals + 1, piece.count - equals - 1);
                option.has_value = true;
            } else {
                option.name = piece;
            }

            if option.name.count array_add(*options, option);

            rest = after;
        }
    }

    return options;
}

is_json_value :: (info: *Type_Info) -> bool {
    return info == type_info(Json_Value);
}

is_table :: (info: *Type_Info) -> bool {
    if info.type != .STRUCT return false;

    struct_info := cast(*Type_Info_Struct) info;

    if !begins_with(struct_info.name, "Table") return false;

    for struct_info.members if it.name == "entries" return true;

    return false;
}

// The key and value types of a Hash_Table, read from its entries.
table_entry :: (info: *Type_Info) -> key: *Type_Info, value: *Type_Info {
    struct_info := cast(*Type_Info_Struct) info;
    key: *Type_Info;
    value: *Type_Info;

    for struct_info.members {
        if it.name != "entries" continue;

        entries := cast(*Type_Info_Array) it.type;
        entry := cast(*Type_Info_Struct) entries.element_type;

        for entry.members {
            if it.name == "key" key = it.type;
            if it.name == "value" value = it.type;
        }
    }

    return key, value;
}

type_label :: (info: *Type_Info) -> string {
    if info.type == {
        case .INTEGER; return "an integer";
        case .FLOAT; return "a float";
        case .BOOL; return "a bool";
        case .STRING; return "a string";
        case .POINTER; return "a pointer";
        case .PROCEDURE; return "a procedure";
        case .STRUCT; return tprint("the struct `%`", (cast(*Type_Info_Struct) info).name);
        case .ENUM; return tprint("the enum `%`", (cast(*Type_Info_Enum) info).name);
        case .ARRAY; return "an array";
        case .ANY; return "`Any`";
        case; return "that type";
    }
}

// Reads the `@json(...)` options of one member. Reports bad ones and says whether all were fine.
read_leaf :: (plan: *Plan, leaf: *Leaf, member: *Type_Info_Struct_Member, struct_rule: Rename_Rule, optional_from_struct: bool, flatten: *bool) -> bool {
    ok := true;
    renamed := false;

    leaf.member = member.name;
    leaf.info = member.type;
    leaf.optional = optional_from_struct;

    for option: read_options(member.notes) {
        name := option.name;

        if name == "rename" {
            if !option.has_value || !option.value.count {
                complain(plan.type_name, member.name, "`rename=` needs a name, as in `@json(rename=userId)`");
                ok = false;
            } else {
                leaf.key = copy_string(option.value,, temp);
                renamed = true;
            }
        } else if name == "alias" {
            if !option.has_value || !option.value.count {
                complain(plan.type_name, member.name, "`alias=` needs a name, as in `@json(alias=userId)`");
                ok = false;
            } else {
                array_add(*leaf.aliases, copy_string(option.value,, temp));
            }
        } else if name == "rename_all" {
            rule, valid := parse_rule(option.value);

            if !valid {
                complain(plan.type_name, member.name, tprint("unknown rename rule `%`. The rules are lowercase, UPPERCASE, camelCase, PascalCase, snake_case, SCREAMING_SNAKE_CASE, kebab-case and SCREAMING-KEBAB-CASE", option.value));
                ok = false;
            } else {
                leaf.style.rule = rule;
            }
        } else if name == "skip" {
            leaf.skip_serializing = true;
            leaf.skip_deserializing = true;
        } else if name == "skip_serializing" {
            leaf.skip_serializing = true;
        } else if name == "skip_deserializing" {
            leaf.skip_deserializing = true;
        } else if name == "skip_serializing_if" {
            if option.value != "empty" && option.value != "zero" && option.value != "null" {
                complain(plan.type_name, member.name, "`skip_serializing_if=` takes `empty`, `zero` or `null`");
                ok = false;
            } else {
                leaf.skip_if = option.value;
            }
        } else if name == "default" {
            leaf.optional = true;
        } else if name == "flatten" {
            flatten.* = true;
        } else if name == "with" {
            if !option.has_value || !option.value.count {
                complain(plan.type_name, member.name, "`with=` needs a name; the struct must declare `<name>_encode` and `<name>_decode`");
                ok = false;
            } else {
                leaf.with = copy_string(option.value,, temp);
            }
        } else if name == "as_number" {
            leaf.style.as_number = true;
        } else {
            complain(plan.type_name, member.name, tprint("unknown option `%` in @json. The options are rename=, alias=, rename_all=, skip, skip_serializing, skip_deserializing, skip_serializing_if=, default, flatten, with= and as_number", name));
            ok = false;
        }
    }

    if !renamed leaf.key = apply_rule(member.name, struct_rule);

    return ok;
}

// Adds the members of `info` to the plan; a flattened struct's members are added in place.
collect_leaves :: (plan: *Plan, info: *Type_Info_Struct, path_prefix: string, struct_rule: Rename_Rule, optional: bool, top: bool) {
    if info.textual_flags & .UNION {
        complain(plan.type_name, "", tprint("%: unions cannot be encoded", info.name));
        plan.ok = false;

        return;
    }

    deny := false;
    default_all := false;
    rule := Rename_Rule.NONE;
    transparent := false;

    for option: read_options(info.notes) {
        name := option.name;

        if name == "rename_all" {
            parsed, valid := parse_rule(option.value);

            if !valid {
                complain(plan.type_name, "", tprint("unknown rename rule `%`", option.value));
                plan.ok = false;
            } else {
                rule = parsed;
            }
        } else if name == "default" {
            default_all = true;
        } else if name == "deny_unknown_fields" {
            deny = true;
        } else if name == "transparent" {
            transparent = true;
        } else {
            complain(plan.type_name, "", tprint("unknown option `%` in @json on a struct. The options are rename_all=, default, deny_unknown_fields and transparent", name));
            plan.ok = false;
        }
    }

    if top {
        plan.deny_unknown = deny;
        plan.transparent = transparent;
    }

    for *member: info.members {
        if member.flags & .CONSTANT continue;

        leaf: Leaf;
        flatten := false;

        if !read_leaf(plan, *leaf, member, rule, default_all || optional, *flatten) plan.ok = false;

        path := ifx path_prefix.count then tprint("%.%", path_prefix, member.name) else member.name;

        if flatten {
            if leaf.skip_serializing && leaf.skip_deserializing continue;

            if member.type.type != .STRUCT || is_json_value(member.type) || is_table(member.type) {
                complain(plan.type_name, member.name, tprint("`flatten` needs a struct field, not %", type_label(member.type)));
                plan.ok = false;

                continue;
            }

            collect_leaves(plan, cast(*Type_Info_Struct) member.type, path, Rename_Rule.NONE, leaf.optional, false);

            continue;
        }

        leaf.path = copy_string(path,, temp);
        leaf.owner = copy_string(path_prefix,, temp);

        array_add(*plan.leaves, leaf);
    }

    if top && transparent {
        active := 0;
        for plan.leaves if !it.skip_serializing || !it.skip_deserializing active += 1;

        if active != 1 || plan.leaves.count != 1 {
            complain(plan.type_name, "", "`transparent` needs a struct with exactly one field");
            plan.ok = false;
        }
    }
}

analyze :: (T: Type) -> Plan {
    plan: Plan;
    info := cast(*Type_Info_Struct) type_info(T);

    plan.type_name = info.name;

    collect_leaves(*plan, info, "", Rename_Rule.NONE, false, true);

    // Two members must not claim the same key.
    for i: 0..plan.leaves.count - 1 {
        first := plan.leaves[i];

        if first.skip_deserializing continue;

        for j: i + 1..plan.leaves.count - 1 {
            second := plan.leaves[j];

            if second.skip_deserializing continue;

            names: [..] string;

            array_add(*names, first.key);
            for first.aliases array_add(*names, it);

            for name: names {
                clash := second.key == name;
                for second.aliases if it == name clash = true;

                if clash {
                    complain(plan.type_name, second.member, tprint("the key `%` is also used by `%`", name, first.member));
                    plan.ok = false;
                }
            }
        }
    }

    return plan;
}

// ---- source text helpers ----

Gen :: struct {
    out: String_Builder;
    uid: int;
    ok := true;
    type_name: string;
    field: string;
}

fail :: (g: *Gen, message: string) {
    g.ok = false;
    complain(g.type_name, g.field, message);
}

next_id :: (g: *Gen) -> int {
    g.uid += 1;

    return g.uid;
}

jai_quote :: (text: string) -> string {
    builder: String_Builder;

    append(*builder, "\"");

    for text {
        if it == #char "\"" append(*builder, "\\\"");
        else if it == #char "\\" append(*builder, "\\\\");
        else if it == #char "\n" append(*builder, "\\n");
        else if it == #char "\r" append(*builder, "\\r");
        else if it == #char "\t" append(*builder, "\\t");
        else append(*builder, it);
    }

    append(*builder, "\"");

    return builder_to_string(*builder,, temp);
}

// The text a key or name has once it is a JSON string, as Jai source.
json_literal :: (text: string) -> string {
    builder: String_Builder;
    append_json_string(*builder, text);

    return jai_quote(builder_to_string(*builder,, temp));
}

// `*x`, except that `p.*` is just `p`.
address_of :: (expression: string) -> string {
    if ends_with(expression, ".*") return slice(expression, 0, expression.count - 2);

    return tprint("*%", expression);
}

owner_type :: (base: string, owner: string) -> string {
    if !owner.count return "T";

    return tprint("type_of(%.%)", base, owner);
}

// ---- encoding ----

signed_limits :: (size: s64) -> string, string {
    if size == 1 return "-128", "127";
    if size == 2 return "-32768", "32767";
    if size == 4 return "-2147483648", "2147483647";

    return "(-9223372036854775807 - 1)", "9223372036854775807";
}

unsigned_limit :: (size: s64) -> string {
    if size == 1 return "255";
    if size == 2 return "65535";
    if size == 4 return "4294967295";

    return "0xffff_ffff_ffff_ffff";
}

emit_encode :: (g: *Gen, value: string, info: *Type_Info, style: Enum_Style) {
    out := *g.out;

    if info.type == {
        case .INTEGER;
            integer := cast(*Type_Info_Integer) info;

            if info.runtime_size > 8 {
                fail(g, "128-bit integers are not supported");

                return;
            }

            if integer.signed print(out, "json_write_int(w, cast(s64) %);\n", value);
            else print(out, "json_write_uint(w, cast(u64) %);\n", value);
        case .FLOAT;
            if info.runtime_size == 4 print(out, "json_write_float32(w, %);\n", value);
            else if info.runtime_size == 8 print(out, "json_write_float64(w, %);\n", value);
            else fail(g, "only float32 and float64 are supported");
        case .BOOL;
            print(out, "json_write_bool(w, %);\n", value);
        case .STRING;
            print(out, "json_write_string(w, %);\n", value);
        case .ENUM;
            emit_encode_enum(g, value, cast(*Type_Info_Enum) info, style);
        case .POINTER;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;

            if pointee.type == .VOID {
                fail(g, "a `*void` cannot be encoded");

                return;
            }

            id := next_id(g);

            print(out, "{\npointer_% := %;\nif !pointer_% json_write_null(w);\nelse {\n", id, value, id);
            emit_encode(g, tprint("pointer_%.*", id), pointee, style);
            print(out, "}\n}\n");
        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            id := next_id(g);

            print(out, "{\njson_begin_array(w);\nfirst_% := true;\nfor index_%: 0..%.count - 1 {\njson_item(w, *first_%);\n", id, id, value, id);
            emit_encode(g, tprint("%[index_%]", value, id), array.element_type, style);
            print(out, "}\njson_end_array(w, first_%);\n}\n", id);
        case .STRUCT;
            if is_json_value(info) {
                print(out, "json_write_value(w, %);\n", address_of(value));
            } else if is_table(info) {
                key, element := table_entry(info);

                if key.type != .STRING {
                    fail(g, "a Hash_Table must have string keys to be encoded as an object");

                    return;
                }

                id := next_id(g);

                print(out, "{\njson_begin_object(w);\nfirst_% := true;\nfor entry_%, key_%: % {\njson_key_dynamic(w, *first_%, key_%);\n", id, id, id, value, id, id);
                emit_encode(g, tprint("entry_%", id), element, style);
                print(out, "}\njson_end_object(w, first_%);\n}\n", id);
            } else {
                print(out, "json_encode(w, %);\n", address_of(value));
            }
        case;
            fail(g, tprint("cannot encode %", type_label(info)));
    }
}

emit_encode_enum :: (g: *Gen, value: string, info: *Type_Info_Enum, style: Enum_Style) {
    out := *g.out;
    signed := info.internal_type.signed;
    flags := (info.enum_type_flags & .FLAGS) != 0;

    if style.as_number || flags {
        if signed print(out, "json_write_int(w, cast(s64) %);\n", value);
        else print(out, "json_write_uint(w, cast(u64) %);\n", value);

        return;
    }

    id := next_id(g);

    print(out, "{\nnumber_% := cast(s64) %;\nif number_% == {\n", id, value, id);

    seen: [..] s64;

    for name: info.names {
        number := info.values[it_index];
        duplicate := false;

        for seen if it == number duplicate = true;

        if duplicate continue;

        array_add(*seen, number);

        print(out, "case %;\njson_write_raw(w, %);\n", number, json_literal(apply_rule(name, style.rule)));
    }

    print(out, "case;\njson_write_int(w, number_%);\n}\n}\n", id);
}

skip_condition :: (g: *Gen, leaf: *Leaf, value: string) -> string {
    info := leaf.info;

    if leaf.skip_if == "empty" {
        if info.type == .STRING return tprint("%.count == 0", value);
        if info.type == .ARRAY return tprint("%.count == 0", value);
        if is_table(info) return tprint("%.count == 0", value);

        fail(g, "`skip_serializing_if=empty` needs a string, an array or a Hash_Table");

        return "false";
    }

    if leaf.skip_if == "zero" {
        if info.type == .INTEGER || info.type == .FLOAT return tprint("% == 0", value);
        if info.type == .BOOL return tprint("!%", value);
        if info.type == .ENUM return tprint("cast(s64) % == 0", value);

        fail(g, "`skip_serializing_if=zero` needs a number, a bool or an enum");

        return "false";
    }

    if info.type == .POINTER return tprint("% == null", value);
    if is_json_value(info) return tprint("%.kind == .NULL", value);

    fail(g, "`skip_serializing_if=null` needs a pointer or a Json_Value");

    return "false";
}

// The condition that `key` equals `text`: a length test, then whole words compared against
// constants built here, so no bytes are walked at run time. (Little-endian loads, which every
// supported target uses.)
key_match :: (text: string) -> string {
    builder: String_Builder;

    print(*builder, "key.count == %", text.count);

    at := 0;

    while at < text.count {
        step := 1;

        if text.count - at >= 8 step = 8;
        else if text.count - at >= 4 step = 4;
        else if text.count - at >= 2 step = 2;

        word: u64 = 0;

        for 0..step - 1 word |= (cast(u64) text[at + it]) << (8 * it);

        print(*builder, " && json_load%(key.data + %) == %", step * 8, at, word);

        at += step;
    }

    return builder_to_string(*builder,, temp);
}

generate_encoder :: (T: Type) -> string {
    g: Gen;
    info := type_info(T);

    if info.type != .STRUCT || is_json_value(info) || is_table(info) {
        g.type_name = tprint("%", T);
        emit_encode(*g, "value.*", info, .{});

        if !g.ok return "";

        return builder_to_string(*g.out,, temp);
    }

    plan := analyze(T);

    g.type_name = plan.type_name;

    if !plan.ok return "";

    if plan.transparent {
        leaf := *plan.leaves[0];

        g.field = leaf.member;
        emit_leaf_encode(*g, leaf);

        if !g.ok return "";

        return builder_to_string(*g.out,, temp);
    }

    print(*g.out, "json_begin_object(w);\nfirst := true;\n");

    for *leaf: plan.leaves {
        if leaf.skip_serializing continue;

        g.field = leaf.member;
        value := tprint("value.%", leaf.path);

        if leaf.skip_if.count print(*g.out, "if !(%) {\n", skip_condition(*g, leaf, value));
        else print(*g.out, "{\n");

        // The separator, quotes and colon of compact output are one constant each.
        quoted: String_Builder;

        append_json_string(*quoted, leaf.key);
        quoted_key := builder_to_string(*quoted,, temp);

        print(*g.out, "json_key_fixed(w, *first, %, %, %);\n", jai_quote(quoted_key), jai_quote(tprint("%:", quoted_key)), jai_quote(tprint(",%:", quoted_key)));
        emit_leaf_encode(*g, leaf);
        print(*g.out, "}\n");
    }

    print(*g.out, "json_end_object(w, first);\n");

    if !g.ok return "";

    return builder_to_string(*g.out,, temp);
}

emit_leaf_encode :: (g: *Gen, leaf: *Leaf) {
    value := tprint("value.%", leaf.path);

    if leaf.with.count {
        print(*g.out, "%.%_encode(w, %);\n", owner_type("value", leaf.owner), leaf.with, address_of(value));

        return;
    }

    emit_encode(g, value, leaf.info, leaf.style);
}

// ---- freeing ----
//
// The code that releases what a decoder allocated: strings, arrays, pointers, tables and nested
// structs. It serves two callers. A decoder that fails frees what it had stored so far (it knows
// those values are its own, so it frees against zero). The public `free_json` frees a finished
// value and compares each field with the type's default value first, so a field that kept a
// default such as `name := "guest"` is left alone.

// Whether a value of this type can own memory.
owns_memory :: (info: *Type_Info) -> bool {
    if info.type == {
        case .STRING; return true;
        case .POINTER; return true;
        case .STRUCT; return true;
        case .ARRAY;
            array := cast(*Type_Info_Array) info;

            if array.array_type == .FIXED return owns_memory(array.element_type);

            return true;
    }

    return false;
}

// Releases the elements of `target` (a resizable array or view), one `for` loop.
emit_free_elements :: (g: *Gen, out: *String_Builder, target: string, element: *Type_Info) {
    if !owns_memory(element) return;

    id := next_id(g);

    print(out, "for *item_%: % {\n", id, target);
    emit_free(g, out, tprint("item_%.*", id), element, "");
    print(out, "}\n");
}

// Writes the statements that free what `target` owns and reset it. `default` is an expression
// for the value a field has when it was not decoded, or "" for the zero value.
emit_free :: (g: *Gen, out: *String_Builder, target: string, info: *Type_Info, default: string) {
    if info.type == {
        case .STRING;
            if default.count print(out, "if %.count && %.data != (%).data { free(%); }\n% = %;\n", target, target, default, target, target, default);
            else print(out, "if %.count { free(%); }\n% = \"\";\n", target, target, target);
        case .POINTER;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;

            if pointee.type == .VOID return;

            if default.count print(out, "if % != (%) {\n", target, default);
            else print(out, "if % != null {\n", target);

            emit_free(g, out, tprint("%.*", target), pointee, "");
            print(out, "free(%);\n}\n% = %;\n", target, target, ifx default.count then default else "null");
        case .ARRAY;
            array := cast(*Type_Info_Array) info;

            if array.array_type == .FIXED {
                emit_free_elements(g, out, target, array.element_type);

                return;
            }

            if default.count print(out, "if %.data != (%).data {\n", target, default);
            else print(out, "if %.data {\n", target);

            emit_free_elements(g, out, target, array.element_type);

            if array.array_type == .VIEW {
                print(out, "free(%.data);\n", target);

                if default.count print(out, "% = %;\n", target, default);
                else print(out, "%.data = null;\n%.count = 0;\n", target, target);
            } else if default.count {
                print(out, "array_free(%);\n% = %;\n", target, target, default);
            } else {
                print(out, "array_reset(%);\n", address_of(target));
            }

            print(out, "}\n");
        case .STRUCT;
            if is_json_value(info) {
                print(out, "free_json(%);\n", address_of(target));
            } else if is_table(info) {
                _, element := table_entry(info);
                id := next_id(g);

                print(out, "for *entry_%, key_%: % {\nif key_%.count { free(key_%); }\n", id, id, target, id, id);
                emit_free(g, out, tprint("entry_%.*", id), element, "");
                print(out, "}\ndeinit(%);\n", address_of(target));
            } else if default.count {
                print(out, "free_json_fields(%, %);\n", address_of(target), address_of(default));
            } else {
                id := next_id(g);

                print(out, "{\ndefault_%: type_of(%);\nfree_json_fields(%, *default_%);\n}\n", id, target, address_of(target), id);
            }
    }
}

// The body of free_json_fields for T: `value` is what to free, `defaults` a default T.
generate_freer :: (T: Type) -> string {
    g: Gen;
    info := type_info(T);

    if info.type != .STRUCT || is_json_value(info) || is_table(info) {
        g.type_name = tprint("%", T);
        emit_free(*g, *g.out, "value.*", info, "defaults.*");

        return builder_to_string(*g.out,, temp);
    }

    plan := analyze(T);

    g.type_name = plan.type_name;

    if !plan.ok return "";

    // Fields with a `with=` handler belong to the handler, which allocated them.
    for *leaf: plan.leaves {
        if leaf.skip_deserializing || leaf.with.count || !owns_memory(leaf.info) continue;

        emit_free(*g, *g.out, tprint("value.%", leaf.path), leaf.info, tprint("defaults.%", leaf.path));
    }

    return builder_to_string(*g.out,, temp);
}

// ---- decoding ----
//
// A decoder that fails frees everything it allocated before it reports the failure, so a failed
// json_decode leaves nothing behind (the value it was reading into is then unspecified).

emit_decode :: (g: *Gen, target: string, info: *Type_Info, style: Enum_Style, on_fail: string) {
    out := *g.out;

    if info.type == {
        case .INTEGER;
            integer := cast(*Type_Info_Integer) info;
            size := info.runtime_size;

            if size > 8 {
                fail(g, "128-bit integers are not supported");

                return;
            }

            id := next_id(g);

            if integer.signed {
                low, high := signed_limits(size);

                print(out, "{\nnumber_%: s64;\nif !json_read_signed(r, *number_%, %, %, \"%\") { % }\n% = cast,no_check(type_of(%)) number_%;\n}\n", id, id, low, high, type_label_text(info), on_fail, target, target, id);
            } else {
                print(out, "{\nnumber_%: u64;\nif !json_read_unsigned(r, *number_%, %, \"%\") { % }\n% = cast,no_check(type_of(%)) number_%;\n}\n", id, id, unsigned_limit(size), type_label_text(info), on_fail, target, target, id);
            }
        case .FLOAT;
            if info.runtime_size == 4 print(out, "if !json_read_float32(r, %) { % }\n", address_of(target), on_fail);
            else if info.runtime_size == 8 print(out, "if !json_read_float64(r, %) { % }\n", address_of(target), on_fail);
            else fail(g, "only float32 and float64 are supported");
        case .BOOL;
            id := next_id(g);

            print(out, "{\nflag_%, ok_% := json_read_bool(r);\nif !ok_% { % }\n% = flag_%;\n}\n", id, id, id, on_fail, target, id);
        case .STRING;
            id := next_id(g);

            print(out, "{\ntext_%, ok_% := json_read_string(r);\nif !ok_% { % }\n% = text_%;\n}\n", id, id, id, on_fail, target, id);
        case .ENUM;
            emit_decode_enum(g, target, cast(*Type_Info_Enum) info, style, on_fail);
        case .POINTER;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;

            if pointee.type == .VOID {
                fail(g, "a `*void` cannot be decoded");

                return;
            }

            print(out, "if json_next_is_null(r) {\nif !json_read_null(r) { % }\n% = null;\n} else {\n% = New(type_of(%.*));\n", on_fail, target, target, target);
            emit_decode(g, tprint("%.*", target), pointee, style, tprint("free(%); % = null; %", target, target, on_fail));
            print(out, "}\n");
        case .ARRAY;
            emit_decode_array(g, target, cast(*Type_Info_Array) info, style, on_fail);
        case .STRUCT;
            if is_json_value(info) {
                print(out, "if !json_read_value(r, %) { free_json(%); % }\n", address_of(target), address_of(target), on_fail);
            } else if is_table(info) {
                emit_decode_table(g, target, info, style, on_fail);
            } else {
                print(out, "if !json_decode(r, %) { % }\n", address_of(target), on_fail);
            }
        case;
            fail(g, tprint("cannot decode %", type_label(info)));
    }
}

type_label_text :: (info: *Type_Info) -> string {
    if info.type == .INTEGER {
        integer := cast(*Type_Info_Integer) info;

        return tprint("%int%", ifx integer.signed then "s" else "u", info.runtime_size * 8);
    }

    return "that type";
}

emit_decode_enum :: (g: *Gen, target: string, info: *Type_Info_Enum, style: Enum_Style, on_fail: string) {
    out := *g.out;
    signed := info.internal_type.signed;
    flags := (info.enum_type_flags & .FLAGS) != 0;
    size := info.internal_type.runtime_size;
    id := next_id(g);

    if style.as_number || flags {
        if signed {
            low, high := signed_limits(size);

            print(out, "{\nnumber_%: s64;\nif !json_read_signed(r, *number_%, %, %, \"%\") { % }\n", id, id, low, high, info.name, on_fail);
        } else {
            print(out, "{\nnumber_%: u64;\nif !json_read_unsigned(r, *number_%, %, \"%\") { % }\n", id, id, unsigned_limit(size), info.name, on_fail);
        }

        if !flags {
            print(out, "if ");

            for value: info.values {
                if it_index print(out, " && ");
                print(out, "number_% != %", id, value);
            }

            if !info.values.count print(out, "true");

            print(out, " { json_fail(r, .INVALID_VALUE, tprint(\"% is not a value of `%`\", number_%)); % }\n", "%", info.name, id, on_fail);
        }

        print(out, "% = cast,no_check(type_of(%)) number_%;\n}\n", target, target, id);

        return;
    }

    print(out, "{\njson_peek(r);\nstart_% := r.pos;\nname_%, ok_% := json_read_name(r);\nif !ok_% { % }\n", id, id, id, id, on_fail);

    for name: info.names {
        print(out, "%if name_% == % { % = cast,no_check(type_of(%)) %; }\n", ifx it_index then "else " else "", id, jai_quote(apply_rule(name, style.rule)), target, target, info.values[it_index]);
        if it_index == info.names.count - 1 print(out, "else ");
    }

    choices: String_Builder;

    for name: info.names {
        if it_index append(*choices, ", ");
        append(*choices, apply_rule(name, style.rule));
    }

    print(out, "{\njson_fail_at(r, .INVALID_VALUE, tprint(\"unknown variant `%`, expected one of: %\", name_%), start_%);\n% }\n}\n", "%", builder_to_string(*choices,, temp), id, id, on_fail);
}

emit_decode_array :: (g: *Gen, target: string, array: *Type_Info_Array, style: Enum_Style, on_fail: string) {
    out := *g.out;
    id := next_id(g);

    // What a failure inside the array releases: the elements read so far and their storage.
    cleanup: String_Builder;

    if array.array_type == .FIXED {
        emit_free_elements(g, *cleanup, target, array.element_type);
    } else if array.array_type == .VIEW {
        emit_free_elements(g, *cleanup, tprint("list_%", id), array.element_type);
        print(*cleanup, "array_reset(*list_%);\n", id);
    } else {
        emit_free_elements(g, *cleanup, target, array.element_type);
        print(*cleanup, "array_reset(%);\n", address_of(target));
    }

    fail := tprint("% %", builder_to_string(*cleanup,, temp), on_fail);
    element_fail := tprint("json_path_index(r, index_%); %", id, fail);

    if array.array_type == .VIEW print(out, "{\nlist_%: [..] type_of(%[0]);\n", id, target);
    else print(out, "{\n");

    print(out, "first_% := true;\nindex_% := 0;\nif !json_begin_array(r) { % }\n", id, id, fail);

    if array.array_type == .FIXED {
        print(out, "while true {\nstatus_% := json_array_next(r, *first_%);\nif status_% == .ERROR { % }\nif status_% == .END break;\n", id, id, id, fail, id);
        print(out, "if index_% >= % { json_fail(r, .TYPE, \"too many elements: expected %\"); % }\n", id, array.array_count, array.array_count, fail);
        emit_decode(g, tprint("%[index_%]", target, id), array.element_type, style, element_fail);
        print(out, "index_% += 1;\n}\n", id);
        print(out, "if index_% != % { json_fail(r, .TYPE, \"too few elements: expected %\"); % }\n}\n", id, array.array_count, array.array_count, fail);

        return;
    }

    list := target;

    if array.array_type == .VIEW {
        list = tprint("list_%", id);
    } else {
        print(out, "%.count = 0;\n", target);
    }

    print(out, "while true {\nstatus_% := json_array_next(r, *first_%);\nif status_% == .ERROR { % }\nif status_% == .END break;\n", id, id, id, fail, id);
    print(out, "item_%: type_of(%[0]);\n", id, target);
    emit_decode(g, tprint("item_%", id), array.element_type, style, element_fail);
    print(out, "array_add(*%, item_%);\nindex_% += 1;\n}\n", list, id, id);

    if array.array_type == .VIEW print(out, "%.data = list_%.data;\n%.count = list_%.count;\n", target, id, target, id);

    print(out, "}\n");
}

emit_decode_table :: (g: *Gen, target: string, info: *Type_Info, style: Enum_Style, on_fail: string) {
    out := *g.out;
    key, element := table_entry(info);

    if key.type != .STRING {
        fail(g, "a Hash_Table must have string keys to be decoded from an object");

        return;
    }

    id := next_id(g);

    // What a failure releases: the entries stored so far and the table's storage.
    cleanup: String_Builder;

    emit_free(g, *cleanup, target, info, "");

    fail := tprint("% %", builder_to_string(*cleanup,, temp), on_fail);
    element_fail := tprint("json_path_key(r, copy_string(key_%,, temp)); %", id, fail);

    print(out, "{\nfirst_% := true;\nif !json_begin_object(r) { % }\n", id, on_fail);
    print(out, "while true {\nkey_%, status_% := json_object_next(r, *first_%);\nif status_% == .ERROR { % }\nif status_% == .END break;\n", id, id, id, id, fail, id);
    print(out, "item_%: type_of(%.entries[0].value);\n", id, target);
    emit_decode(g, tprint("item_%", id), element, style, element_fail);

    // A repeated key replaces the earlier value, which is released.
    print(out, "existing_% := table_find_pointer(%, key_%);\nif existing_% {\n", id, address_of(target), id, id);
    emit_free(g, out, tprint("existing_%.*", id), element, "");
    print(out, "existing_%.* = item_%;\n} else table_add(%, copy_string(key_%), item_%);\n}\n}\n", id, id, address_of(target), id, id);
}

generate_decoder :: (T: Type) -> string {
    g: Gen;
    info := type_info(T);

    if info.type != .STRUCT || is_json_value(info) || is_table(info) {
        g.type_name = tprint("%", T);
        emit_decode(*g, "out.*", info, .{}, "return false;");
        print(*g.out, "return true;\n");

        if !g.ok return "return false;";

        return builder_to_string(*g.out,, temp);
    }

    plan := analyze(T);

    g.type_name = plan.type_name;

    if !plan.ok return "return false;";

    if plan.transparent {
        leaf := *plan.leaves[0];

        g.field = leaf.member;
        emit_leaf_decode(*g, leaf, "return false;");
        print(*g.out, "return true;\n");

        if !g.ok return "return false;";

        return builder_to_string(*g.out,, temp);
    }

    out := *g.out;
    count := plan.leaves.count;

    print(out, "if !json_begin_object(r) return false;\n");

    // seen[i]: 0 not read, 1 being read, 2 read. A failure releases the fields that were read.
    cleanup: String_Builder;

    for *leaf: plan.leaves {
        if leaf.skip_deserializing || leaf.with.count || !owns_memory(leaf.info) continue;

        print(*cleanup, "if seen[%] == 2 {\n", it_index);
        emit_free(*g, *cleanup, tprint("out.%", leaf.path), leaf.info, "");
        print(*cleanup, "}\n");
    }

    cleaned := builder_to_string(*cleanup,, temp);

    if count print(out, "seen: [%] u8;\n", count);
    if cleaned.count print(out, "done := false;\ndefer if !done {\n%}\n", cleaned);

    print(out, "first := true;\nwhile true {\nkey, status := json_object_next(r, *first);\nif status == .ERROR return false;\nif status == .END break;\n");

    chain := false;

    for *leaf: plan.leaves {
        if leaf.skip_deserializing continue;

        g.field = leaf.member;

        print(out, "%if % ", ifx chain then "else " else "", key_match(leaf.key));
        for alias: leaf.aliases print(out, "|| (%) ", key_match(alias));

        print(out, "{\n");
        print(out, "if seen[%] != 0 { json_duplicate_field(r, key); return false; }\nseen[%] = 1;\n", it_index, it_index);

        emit_leaf_decode(*g, leaf, tprint("{ json_path_key(r, %); return false; }", jai_quote(leaf.key)));

        print(out, "seen[%] = 2;\n}\n", it_index);
        chain = true;
    }

    unknown := ifx plan.deny_unknown then "{ json_unknown_field(r, key); return false; }" else "if !json_skip_value(r) return false;";

    if chain print(out, "else { % }\n", unknown);
    else print(out, "%\n", unknown);

    print(out, "}\n");

    for *leaf: plan.leaves {
        if leaf.skip_deserializing || leaf.optional continue;

        print(out, "if seen[%] == 0 { json_missing_field(r, %); return false; }\n", it_index, jai_quote(leaf.key));
    }

    if cleaned.count print(out, "done = true;\n");

    print(out, "return true;\n");

    if !g.ok return "return false;";

    return builder_to_string(*g.out,, temp);
}

emit_leaf_decode :: (g: *Gen, leaf: *Leaf, on_fail: string) {
    target := tprint("out.%", leaf.path);

    if leaf.with.count {
        print(*g.out, "if !%.%_decode(r, %) { % }\n", owner_type("out", leaf.owner), leaf.with, address_of(target), on_fail);

        return;
    }

    emit_decode(g, target, leaf.info, leaf.style, on_fail);
}
