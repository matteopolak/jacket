// Compile-time half of typed TOML: reads a struct's members and their `@toml(...)` notes,
// reports bad declarations by field name, and writes the Jai source of the key dispatch, the
// reader, the required-key check, the encoder and the freer for it. `toml_descend` and its
// siblings (module.jai) `#insert` that source, so each type gets straight-line code with its
// keys and types baked in.
//
// Everything here runs inside the compiler. Nothing here is called at run time.
#scope_module

// ---- analysis ----

Leaf :: struct {
    member: string; // the Jai name
    path: string; // from the struct being coded, through flattened structs: `address.zip`
    info: *Type_Info;
    key: string;
    aliases: [..] string;
    skip_serializing: bool;
    skip_deserializing: bool;
    skip_if: string;
    optional: bool;
    unknown: bool; // the `[..] Toml_Unknown` list of ignored keys
    as_number: bool;
    enum_rule := Rename_Rule.SNAKE;
    has_range: bool;
    low: s64;
    high: s64;
    message: string;
}

Plan :: struct {
    type_name: string;
    ok := true;
    deny_unknown: bool;
    leaves: [..] Leaf;
}

complain :: (type_name: string, field: string, message: string) {
    where := ifx field.count then tprint("%.%", type_name, field) else type_name;
    compiler_report(tprint("Toml: %: %", where, message));
}

Option :: struct {
    name: string;
    value: string;
    has_value: bool;
}

trim_spaces :: (text: string) -> string {
    result := text;

    while result.count && result[0] == #char " " result = slice(result, 1, result.count - 1);
    while result.count && result[result.count - 1] == #char " " result.count -= 1;

    return result;
}

// The options inside the `@toml(...)` notes of a member or struct, as `name` or `name=value`.
read_options :: (notes: [] string) -> [..] Option {
    options: [..] Option;

    for notes {
        if !begins_with(it, "toml(") || !ends_with(it, ")") continue;

        rest := slice(it, "toml(".count, it.count - "toml(".count - ")".count);

        while rest.count {
            found, piece, after := split_note_option(rest);

            piece = trim_spaces(piece);
            option: Option;
            equals := find_index_from_left(piece, #char "=");

            if equals >= 0 {
                option.name = trim_spaces(slice(piece, 0, equals));
                value, ok := unquote(trim_spaces(slice(piece, equals + 1, piece.count - equals - 1)));
                if !ok compiler_report("Toml: invalid escape in a quoted @toml option value");

                option.value = value;
                option.has_value = true;
            } else {
                option.name = piece;
            }

            if option.name.count array_add(*options, option);

            rest = ifx found then after else "";
        }
    }

    return options;
}

is_datetime :: (info: *Type_Info) -> bool {
    return info == type_info(Toml_Datetime);
}

is_unknown_list :: (info: *Type_Info) -> bool {
    if info.type != .ARRAY return false;

    array := cast(*Type_Info_Array) info;

    return array.array_type == .RESIZABLE && array.element_type == type_info(Toml_Unknown);
}

// A struct that is a table of its own: the target of `[header]` and inline tables. That is a
// plain struct, a `Table(string, V)` (see `is_map`) and a `Toml_Value`.
is_table_struct :: (info: *Type_Info) -> bool {
    return info.type == .STRUCT && !is_datetime(info);
}

// The untyped tree of values: any TOML value, kept as the document parser builds it.
is_value :: (info: *Type_Info) -> bool {
    return info == type_info(Toml_Value);
}

// A `Table(Key, Value)`: recognized by its name and its `entries`.
is_map :: (info: *Type_Info) -> bool {
    if info.type != .STRUCT return false;

    struct_info := cast(*Type_Info_Struct) info;

    if !begins_with(struct_info.name, "Table") return false;

    for struct_info.members if it.name == "entries" return true;

    return false;
}

has_string_keys :: (info: *Type_Info) -> bool {
    key, _ := map_types(info);

    return key.type == .STRING;
}

// A struct whose members are the table's keys.
is_plain_struct :: (info: *Type_Info) -> bool {
    return is_table_struct(info) && !is_value(info) && !is_map(info);
}

// The key and value types of a `Table`, read from its entries.
map_types :: (info: *Type_Info) -> key: *Type_Info, value: *Type_Info {
    key: *Type_Info;
    value: *Type_Info;

    for (cast(*Type_Info_Struct) info).members {
        if it.name != "entries" continue;

        entry := cast(*Type_Info_Struct) (cast(*Type_Info_Array) it.type).element_type;

        for entry.members {
            if it.name == "key" key = it.type;
            if it.name == "value" value = it.type;
        }
    }

    return key, value;
}

Table_Shape :: enum u8 {
    NONE;
    STRUCT;
    VALUE; // a `Toml_Value`: a table or not, as the data says
    POINTER;
    ARRAY;
    VIEW;
    FIXED;
}

// How a field takes part in table headers: structs, pointers to structs, and arrays of structs.
table_shape :: (info: *Type_Info) -> Table_Shape {
    if is_value(info) return .VALUE;
    if is_table_struct(info) return .STRUCT;

    // Pointers to and arrays of `Toml_Value` are inline values: any element may be a scalar.
    if info.type == .POINTER {
        pointee := (cast(*Type_Info_Pointer) info).pointer_to;

        if is_table_struct(pointee) && !is_value(pointee) return .POINTER;
    }

    if info.type == .ARRAY {
        array := cast(*Type_Info_Array) info;

        if is_table_struct(array.element_type) && !is_value(array.element_type) {
            if array.array_type == .RESIZABLE return .ARRAY;
            if array.array_type == .VIEW return .VIEW;
            if array.array_type == .FIXED return .FIXED;
        }
    }

    return .NONE;
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

parse_range :: (text: string) -> bool, s64, s64 {
    split := find_index_from_left(text, "..");

    if split < 0 return false, 0, 0;

    low, low_ok := parse_int(slice(text, 0, split));
    high, high_ok := parse_int(slice(text, split + 2, text.count - split - 2));

    return low_ok && high_ok && low <= high, low, high;
}

parse_int :: (text: string) -> s64, bool {
    value, ok, remainder := string_to_int(trim_spaces(text));

    return value, ok && !remainder.count;
}

// Reads the options of one member. Reports bad ones and says whether all were fine.
read_leaf :: (plan: *Plan, leaf: *Leaf, member: *Type_Info_Struct_Member, struct_rule: Rename_Rule, optional_from_struct: bool, flatten: *bool) -> bool {
    ok := true;
    renamed := false;

    leaf.member = member.name;
    leaf.info = member.type;
    leaf.optional = optional_from_struct;

    for option: read_options(member.notes) {
        name := option.name;

        if name == "rename" {
            if !option.value.count {
                complain(plan.type_name, member.name, "`rename=` needs a key, as in `@toml(rename=user-id)`");
                ok = false;
            } else {
                leaf.key = copy_string(option.value,, temp);
                renamed = true;
            }
        } else if name == "alias" {
            if !option.value.count {
                complain(plan.type_name, member.name, "`alias=` needs a key, as in `@toml(alias=user-id)`");
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
                leaf.enum_rule = rule;
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
                leaf.skip_if = copy_string(option.value,, temp);
            }
        } else if name == "default" {
            leaf.optional = true;
        } else if name == "flatten" {
            flatten.* = true;
        } else if name == "unknown" {
            leaf.unknown = true;
        } else if name == "as_number" {
            leaf.as_number = true;
        } else if name == "range" {
            valid, low, high := parse_range(option.value);

            if !valid {
                complain(plan.type_name, member.name, "`range=` takes integers, as in `@toml(range=1..16)`");
                ok = false;
            } else {
                leaf.has_range = true;
                leaf.low = low;
                leaf.high = high;
            }
        } else if name == "message" {
            if !option.value.count {
                complain(plan.type_name, member.name, "`message=` needs text, as in `@toml(message=\"must be 1 to 16\")`");
                ok = false;
            } else {
                leaf.message = copy_string(option.value,, temp);
            }
        } else {
            complain(plan.type_name, member.name, tprint("unknown option `%` in @toml. The options are rename=, alias=, rename_all=, skip, skip_serializing, skip_deserializing, skip_serializing_if=, default, flatten, unknown, as_number, range= and message=", name));
            ok = false;
        }
    }

    if leaf.unknown {
        if !is_unknown_list(member.type) {
            complain(plan.type_name, member.name, "`unknown` needs a `[..] Toml_Unknown` field");
            ok = false;
        }

        leaf.skip_deserializing = true;
        leaf.skip_serializing = true;
    }

    if leaf.has_range && member.type.type != .INTEGER {
        complain(plan.type_name, member.name, "`range=` needs an integer field");
        ok = false;
    }

    if !renamed leaf.key = apply_rule(member.name, struct_rule);

    return ok;
}

// Adds the members of `info` to the plan; a flattened struct's members are added in place.
collect_leaves :: (plan: *Plan, info: *Type_Info_Struct, path_prefix: string, struct_rule: Rename_Rule, optional: bool, top: bool) {
    if info.textual_flags & .UNION {
        complain(plan.type_name, "", tprint("%: unions cannot be coded", info.name));
        plan.ok = false;

        return;
    }

    deny := false;
    default_all := false;
    rule := Rename_Rule.NONE;

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
        } else {
            complain(plan.type_name, "", tprint("unknown option `%` in @toml on a struct. The options are rename_all=, default and deny_unknown_fields", name));
            plan.ok = false;
        }
    }

    if top plan.deny_unknown = deny;

    for *member: info.members {
        if member.flags & .CONSTANT continue;

        leaf: Leaf;
        flatten := false;

        if !read_leaf(plan, *leaf, member, rule, default_all || optional, *flatten) plan.ok = false;

        path := ifx path_prefix.count then tprint("%.%", path_prefix, member.name) else member.name;

        if flatten {
            if leaf.skip_serializing && leaf.skip_deserializing continue;

            if !is_plain_struct(member.type) {
                complain(plan.type_name, member.name, tprint("`flatten` needs a struct field, not %", type_label(member.type)));
                plan.ok = false;

                continue;
            }

            collect_leaves(plan, cast(*Type_Info_Struct) member.type, path, Rename_Rule.NONE, leaf.optional, false);

            continue;
        }

        leaf.path = copy_string(path,, temp);
        array_add(*plan.leaves, leaf);
    }
}

analyze_info :: (info: *Type_Info_Struct) -> Plan {
    plan: Plan;

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

analyze :: (T: Type, any_struct := false) -> Plan {
    info := type_info(T);

    if info.type != .STRUCT || (is_datetime(info) && !any_struct) {
        compiler_report(tprint("Toml: % is not a struct; a TOML document is a table", T));
        plan: Plan;
        plan.ok = false;

        return plan;
    }

    return analyze_info(cast(*Type_Info_Struct) info);
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

// `*x`, except that `p.*` is just `p`.
address_of :: (expression: string) -> string {
    if ends_with(expression, ".*") return slice(expression, 0, expression.count - 2);

    return tprint("*%", expression);
}

number_text :: (value: s64) -> string {
    if value == (-9223372036854775807 - 1) return "(-9223372036854775807 - 1)";

    return tprint("%", value);
}

signed_limits :: (size: s64) -> s64, s64 {
    if size == 1 return -128, 127;
    if size == 2 return -32768, 32767;
    if size == 4 return -2147483648, 2147483647;

    return -9223372036854775807 - 1, 9223372036854775807;
}

unsigned_limit :: (size: s64) -> s64 {
    if size == 1 return 255;
    if size == 2 return 65535;
    if size == 4 return 4294967295;

    return 9223372036854775807;
}

// ---- decoding values ----

// Statements reading one value into `target`; a failure runs `return false;`, since the
// caller frees whatever was stored so far.
emit_read :: (g: *Gen, target: string, info: *Type_Info, leaf: *Leaf) {
    out := *g.out;
    message := jai_quote(leaf.message);

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

                if leaf.has_range {
                    low = max(low, leaf.low);
                    high = min(high, leaf.high);
                }

                print(out, "{\nnumber_%: s64;\nif !toml_read_signed(r, *number_%, %, %, %) return false;\n% = cast,no_check(type_of(%)) number_%;\n}\n", id, id, number_text(low), number_text(high), message, target, target, id);
            } else {
                low := 0;
                high := unsigned_limit(size);

                if leaf.has_range {
                    low = max(low, leaf.low);
                    high = min(high, leaf.high);
                }

                print(out, "{\nnumber_%: u64;\nif !toml_read_unsigned(r, *number_%, %, %, %) return false;\n% = cast,no_check(type_of(%)) number_%;\n}\n", id, id, low, high, message, target, target, id);
            }
        case .FLOAT;
            if info.runtime_size != 4 && info.runtime_size != 8 {
                fail(g, "only float32 and float64 are supported");

                return;
            }

            id := next_id(g);

            print(out, "{\nnumber_%: float64;\nif !toml_read_float(r, *number_%, %) return false;\n% = cast(type_of(%)) number_%;\n}\n", id, id, message, target, target, id);
        case .BOOL;
            print(out, "if !toml_read_bool(r, %, %) return false;\n", address_of(target), message);
        case .STRING;
            print(out, "if !toml_read_string(r, %, %) return false;\n", address_of(target), message);
        case .ENUM;
            emit_read_enum(g, target, cast(*Type_Info_Enum) info, leaf);
        case .POINTER;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;

            if pointee.type == .VOID {
                fail(g, "a `*void` cannot be decoded");

                return;
            }

            print(out, "% = New(type_of(%.*));\n", target, target);
            emit_read(g, tprint("%.*", target), pointee, leaf);
        case .ARRAY;
            emit_read_array(g, target, cast(*Type_Info_Array) info, leaf);
        case .STRUCT;
            if is_datetime(info) {
                print(out, "if !toml_read_datetime(r, %, %) return false;\n", address_of(target), message);
            } else if is_value(info) {
                print(out, "if !toml_read_value(r, %) return false;\n", address_of(target));
            } else if is_map(info) && !has_string_keys(info) {
                fail(g, "a Hash_Table must have string keys to be decoded from a table");
            } else {
                print(out, "if !toml_inline(r, %, %) return false;\n", address_of(target), message);
            }
        case;
            fail(g, tprint("cannot decode %", type_label(info)));
    }
}

emit_read_enum :: (g: *Gen, target: string, info: *Type_Info_Enum, leaf: *Leaf) {
    out := *g.out;
    message := jai_quote(leaf.message);
    signed := info.internal_type.signed;
    flags := (info.enum_type_flags & .FLAGS) != 0;
    size := info.internal_type.runtime_size;
    id := next_id(g);

    if leaf.as_number || flags {
        if signed {
            low, high := signed_limits(size);

            print(out, "{\nnumber_%: s64;\nif !toml_read_signed(r, *number_%, %, %, %) return false;\n", id, id, number_text(low), number_text(high), message);
        } else {
            print(out, "{\nnumber_%: u64;\nif !toml_read_unsigned(r, *number_%, 0, %, %) return false;\n", id, id, unsigned_limit(size), message);
        }

        if !flags {
            print(out, "if ");

            for value: info.values {
                if it_index print(out, " && ");
                print(out, "number_% != %", id, value);
            }

            if !info.values.count print(out, "true");

            print(out, " return toml_fail(r, .TYPE, %, r.statement_at);\n", ifx leaf.message.count then message else jai_quote("not a value of the enum"));
        }

        print(out, "% = cast,no_check(type_of(%)) number_%;\n}\n", target, target, id);

        return;
    }

    print(out, "{\nstart_% := r.pos;\nname_%: string;\nif !toml_read_string(r, *name_%, %) return false;\ndefer if name_%.count { free(name_%.data); r.doc.retained_bytes -= name_%.count; }\n", id, id, id, message, id, id, id);

    for name: info.names {
        print(out, "%if name_% == % { % = cast,no_check(type_of(%)) %; }\n", ifx it_index then "else " else "", id, jai_quote(apply_rule(name, leaf.enum_rule)), target, target, info.values[it_index]);
    }

    print(out, "%return toml_fail(r, .TYPE, %, start_%);\n}\n", ifx info.names.count then "else " else "", ifx leaf.message.count then message else jai_quote("unknown enum name"), id);
}

emit_read_array :: (g: *Gen, target: string, array: *Type_Info_Array, leaf: *Leaf) {
    out := *g.out;
    message := jai_quote(leaf.message);

    id := next_id(g);

    if array.array_type == .FIXED {
        // Exactly as many elements as the array holds; the extra one is the error.
        print(out, "{\nif !toml_array_begin(r, %) return false;\nfirst_% := true;\nindex_% := 0;\n", message, id, id);
        print(out, "while true {\nstatus_% := toml_array_next(r, *first_%);\nif status_% < 0 return false;\nif status_% == 0 break;\n", id, id, id, id);
        print(out, "if index_% >= % return toml_fail(r, .TYPE, %, r.pos);\n", id, array.array_count, jai_quote(tprint("array has more than % elements", array.array_count)));
        print(out, "mark_% := toml_path_index(r, index_%);\n", id, id);
        emit_read(g, tprint("%[index_%]", target, id), array.element_type, leaf);
        print(out, "r.path.count = mark_%;\nindex_% += 1;\n}\n", id, id);
        print(out, "if index_% != % return toml_fail(r, .TYPE, %, r.pos);\n}\n", id, array.array_count, jai_quote(tprint("array needs % elements", array.array_count)));

        return;
    }

    view := array.array_type == .VIEW;
    list := ifx view then tprint("list_%", id) else target;

    print(out, "{\n");

    // A view is built in a growing array and republished after every element, so the final
    // freer finds the storage wherever a failure stops.
    if view print(out, "list_%: [..] type_of(%[0]);\n% = .{};\n", id, target, target);
    else print(out, "%.count = 0;\n", target);

    print(out, "if !toml_array_begin(r, %) return false;\nfirst_% := true;\nindex_% := 0;\n", message, id, id);
    print(out, "while true {\nstatus_% := toml_array_next(r, *first_%);\nif status_% < 0 return false;\nif status_% == 0 break;\n", id, id, id, id);
    print(out, "item_% := array_add(*%);\n", id, list);

    if view print(out, "% = .{ list_%.count, list_%.data };\n", target, id, id);

    print(out, "mark_% := toml_path_index(r, index_%);\n", id, id);
    emit_read(g, tprint("item_%.*", id), array.element_type, leaf);
    print(out, "r.path.count = mark_%;\nindex_% += 1;\n}\n}\n", id, id);
}

// ---- the descent ----

// The condition that `key` is the field's name or one of its aliases.
key_test :: (leaf: *Leaf) -> string {
    builder: String_Builder;

    print(*builder, "key == %", jai_quote(leaf.key));
    for leaf.aliases print(*builder, " || key == %", jai_quote(it));

    return builder_to_string(*builder,, temp);
}

// What a key does once it reached `place` (of type `leaf.info`) and was declared as `decl`: store
// an assigned value, or take part in the table headers and dotted keys below it. Every path ends
// in a `return`.
emit_step :: (g: *Gen, place: string, leaf: *Leaf) {
    out := *g.out;
    shape := table_shape(leaf.info);

    if shape == .NONE {
        print(out, "if step != .FINAL_ASSIGN return toml_fail(r, .TYPE, \"key is not a table\", r.statement_at);\n");
        emit_read(g, place, leaf.info, leaf);
        print(out, "return true;\n");

        return;
    }

    print(out, "if step == .FINAL_ASSIGN {\n");
    emit_read(g, place, leaf.info, leaf);
    print(out, "return true;\n}\n");

    if shape == .STRUCT || shape == .VALUE || shape == .POINTER {
        table := place;

        if shape == .POINTER {
            print(out, "if !% % = New(type_of(%.*));\n", place, place, place);
            table = tprint("%.*", place);
        }

        print(out, "if step == .FINAL_ARRAY return toml_fail(r, .TYPE, \"key is not an array of tables\", r.statement_at);\nif !decl.child decl.child = toml_new_frame();\nif step == .FINAL_TABLE return true;\nreturn toml_descend(r, %, decl.child, keys, index + 1, op, nav);\n", address_of(table));

        return;
    }

    if shape == .FIXED {
        // The elements are counted in `decl.capacity`; `[[x]]` headers fill the array in order.
        count := (cast(*Type_Info_Array) leaf.info).array_count;

        print(out, "if step == .FINAL_ARRAY {\nif decl.capacity >= % return toml_fail(r, .TYPE, %, r.statement_at);\n", count, jai_quote(tprint("array has more than % elements", count)));
        print(out, "if decl.child {\nif decl.capacity {\nelement_mark := toml_path_index(r, decl.capacity - 1);\nif !toml_finish(r, *%[decl.capacity - 1], decl.child, decl.span.start) return false;\nr.path.count = element_mark;\n}\ntoml_free_frame(decl.child);\n}\n", place);
        print(out, "decl.child = toml_new_frame();\ndecl.capacity += 1;\nreturn true;\n}\n");
        print(out, "if step == .FINAL_TABLE || decl.kind != .ARRAY_TABLE || !decl.capacity return toml_fail(r, .TYPE, \"key is not a table\", r.statement_at);\n");
        print(out, "array_add(*r.path, Toml_Path_Part.{ index = decl.capacity - 1 });\nreturn toml_descend(r, *%[decl.capacity - 1], decl.child, keys, index + 1, op, nav);\n", place);

        return;
    }

    print(out, "if step == .FINAL_ARRAY {\nif decl.child {\nif %.count {\nelement_mark := toml_path_index(r, %.count - 1);\nif !toml_finish(r, *%[%.count - 1], decl.child, decl.span.start) return false;\nr.path.count = element_mark;\n}\ntoml_free_frame(decl.child);\n}\n", place, place, place, place);
    print(out, "decl.child = toml_new_frame();\n");

    if shape == .VIEW {
        // The view keeps no capacity of its own; the declaration does. The new view is
        // published at once, so a failure frees the storage wherever it stands.
        print(out, "{\ngrown: [..] type_of(%[0]);\ngrown.data = %.data;\ngrown.count = %.count;\ngrown.allocated = decl.capacity;\narray_add(*grown);\n% = .{ grown.count, grown.data };\ndecl.capacity = grown.allocated;\n}\n", place, place, place, place);
    } else {
        print(out, "array_add(*%);\n", place);
    }

    print(out, "return true;\n}\n");
    print(out, "if step == .FINAL_TABLE || decl.kind != .ARRAY_TABLE || !%.count return toml_fail(r, .TYPE, \"key is not a table\", r.statement_at);\n", place);
    print(out, "array_add(*r.path, Toml_Path_Part.{ index = %.count - 1 });\nreturn toml_descend(r, *%[%.count - 1], decl.child, keys, index + 1, op, nav);\n", place, place, place);
}

// The descent into a `Table(string, V)`: each key is a record in the frame, and the value is
// added to the table the first time the key is declared.
generate_map_descend :: (T: Type) -> string {
    g: Gen;
    info := type_info(T);
    key, element := map_types(info);

    g.type_name = tprint("%", T);

    if key.type != .STRING {
        fail(*g, "a Hash_Table must have string keys to be decoded from a table");

        return "return false;";
    }

    leaf: Leaf;

    leaf.info = element;

    print(*g.out, "key := keys[index];\nstep := toml_step(op, index, keys.count, nav);\npath_mark := r.path.count;\narray_add(*r.path, Toml_Path_Part.{ name = key });\ndefer r.path.count = path_mark;\nentry := toml_frame_entry(r, frame, key);\nfresh := !entry.decl.declared;\ndecl := *entry.decl;\nif !toml_enter(r, decl, step, r.statement_at) return false;\n");
    print(*g.out, "if fresh {\nitem: type_of(out.entries[0].value);\ntable_add(out, copy_string(key), item);\n}\nslot := table_find_pointer(out, key);\n");
    emit_step(*g, "slot.*", *leaf);

    if !g.ok return "return false;";

    return builder_to_string(*g.out,, temp);
}

generate_descend :: (T: Type) -> string {
    info := type_info(T);

    if is_value(info) return "return toml_descend_value(r, out, frame, keys, index, op, nav);";
    if is_map(info) return generate_map_descend(T);

    g: Gen;
    plan := analyze(T);

    g.type_name = plan.type_name;

    if !plan.ok return "return false;";

    out := *g.out;
    collector := "";

    for *leaf: plan.leaves if leaf.unknown collector = leaf.path;

    print(out, "toml_frame_fields(frame, %);\nkey := keys[index];\nstep := toml_step(op, index, keys.count, nav);\npath_mark := r.path.count;\narray_add(*r.path, Toml_Path_Part.{ name = key });\ndefer r.path.count = path_mark;\n", plan.leaves.count);

    chain := false;

    for *leaf: plan.leaves {
        if leaf.skip_deserializing continue;

        g.field = leaf.member;

        print(out, "%if % {\ndecl := *frame.fields[%];\nif !toml_enter(r, decl, step, r.statement_at) return false;\n", ifx chain then "else " else "", key_test(leaf), it_index);
        chain = true;
        emit_step(*g, tprint("out.%", leaf.path), leaf);
        print(out, "}\n");
    }

    g.field = "";

    print(out, "%{\n", ifx chain then "else " else "");

    if plan.deny_unknown print(out, "return toml_fail(r, .TYPE, \"unknown key\", r.statement_at);\n");
    else {
        print(out, "if r.options.deny_unknown_fields return toml_fail(r, .TYPE, \"unknown key\", r.statement_at);\nif !toml_unknown_step(r, frame, keys, index, op, nav) return false;\n");

        if collector.count print(out, "if op != .ASSIGN || index >= nav toml_record_unknown(r, *out.%, keys, index, op != .ASSIGN);\n", collector);

        print(out, "return true;\n");
    }

    print(out, "}\n");

    if !g.ok return "return false;";

    return builder_to_string(*g.out,, temp);
}

// Checks what a declared table or array of tables at `place` still needs once it is closed,
// with `decl` its declaration and `name` the key for error paths.
emit_finish_child :: (g: *Gen, place: string, info: *Type_Info, name: string) {
    shape := table_shape(info);
    out := *g.out;

    if shape == .NONE || shape == .VALUE return;

    print(out, "mark := r.path.count;\narray_add(*r.path, Toml_Path_Part.{ name = % });\n", name);

    if shape == .STRUCT print(out, "if !toml_finish(r, *%, decl.child, decl.span.start) return false;\n", place);
    else if shape == .POINTER print(out, "if % && !toml_finish(r, %, decl.child, decl.span.start) return false;\n", place, place);
    else if shape == .FIXED {
        count := (cast(*Type_Info_Array) info).array_count;

        print(out, "if decl.capacity != % return toml_fail(r, .TYPE, %, decl.span.start);\n", count, jai_quote(tprint("array needs % elements", count)));
        print(out, "{\nelement_mark := toml_path_index(r, decl.capacity - 1);\nif !toml_finish(r, *%[decl.capacity - 1], decl.child, decl.span.start) return false;\nr.path.count = element_mark;\n}\n", place);
    } else {
        print(out, "if %.count {\nelement_mark := toml_path_index(r, %.count - 1);\nif !toml_finish(r, *%[%.count - 1], decl.child, decl.span.start) return false;\nr.path.count = element_mark;\n}\n", place, place, place, place);
    }

    print(out, "r.path.count = mark;\n");
}

// The finish of a `Table(string, V)`: its values that are tables, found by the keys the frame
// recorded.
generate_map_finish :: (T: Type) -> string {
    g: Gen;
    info := type_info(T);
    _, element := map_types(info);

    g.type_name = tprint("%", T);

    shape := table_shape(element);

    if shape == .NONE || shape == .VALUE return "return true;";

    out := *g.out;

    print(out, "for *entry: frame.unknown {\nif !entry.decl.child || entry.decl.kind == .CLOSED continue;\ndecl := *entry.decl;\nslot := table_find_pointer(out, entry.name);\n");
    emit_finish_child(*g, "slot.*", element, "entry.name");
    print(out, "}\nreturn true;\n");

    return builder_to_string(*g.out,, temp);
}

// The required-key check, run once a table can no longer change: after the document for the
// root and its tables, after each inline table, and when a `[[x]]` element is replaced.
generate_finish :: (T: Type) -> string {
    info := type_info(T);

    if is_value(info) return "return true;";
    if is_map(info) return generate_map_finish(T);

    g: Gen;
    plan := analyze(T);

    g.type_name = plan.type_name;

    if !plan.ok return "return false;";

    out := *g.out;

    print(out, "toml_frame_fields(frame, %);\n", plan.leaves.count);

    for *leaf: plan.leaves {
        if leaf.skip_deserializing continue;

        place := tprint("out.%", leaf.path);

        print(out, "{\ndecl := *frame.fields[%];\n", it_index);

        if leaf.optional print(out, "if decl.declared && decl.kind != .CLOSED && decl.child {\n");
        else print(out, "if !decl.declared return toml_missing_key(r, %, at);\nif decl.kind != .CLOSED && decl.child {\n", jai_quote(leaf.key));

        emit_finish_child(*g, place, leaf.info, jai_quote(leaf.key));

        print(out, "}\n}\n");
    }

    print(out, "return true;\n");

    return builder_to_string(*g.out,, temp);
}

// ---- freeing ----
//
// What a failed decode releases, and what free_toml_value releases: strings, arrays, pointers
// and nested structs, each compared with the type's default first so a field that kept its
// declared default (`name := "guest"`) is left alone.

owns_memory :: (info: *Type_Info) -> bool {
    if info.type == {
        case .STRING; return true;
        case .POINTER; return true;
        case .STRUCT; return true;
        case .ARRAY;
            array := cast(*Type_Info_Array) info;

            if array.array_type == .FIXED return owns_memory(array.element_type);

            return true;
        case;
    }

    return false;
}

emit_free_elements :: (g: *Gen, target: string, element: *Type_Info) {
    if !owns_memory(element) return;

    id := next_id(g);

    print(*g.out, "for *item_%: % {\n", id, target);
    emit_free(g, tprint("item_%.*", id), element, "");
    print(*g.out, "}\n");
}

emit_free :: (g: *Gen, target: string, info: *Type_Info, default: string) {
    out := *g.out;

    if info.type == {
        case .STRING;
            if default.count print(out, "if %.count && %.data != (%).data { free(%.data); }\n% = %;\n", target, target, default, target, target, default);
            else print(out, "if %.count { free(%.data); }\n% = \"\";\n", target, target, target);
        case .POINTER;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;

            if pointee.type == .VOID return;

            if default.count print(out, "if % != (%) {\n", target, default);
            else print(out, "if % != null {\n", target);

            emit_free(g, tprint("%.*", target), pointee, "");
            print(out, "free(%);\n}\n% = %;\n", target, target, ifx default.count then default else "null");
        case .ARRAY;
            array := cast(*Type_Info_Array) info;

            if array.array_type == .FIXED {
                emit_free_elements(g, target, array.element_type);

                return;
            }

            if default.count print(out, "if %.data != (%).data {\n", target, default);
            else print(out, "if %.data {\n", target);

            emit_free_elements(g, target, array.element_type);

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
            if is_value(info) {
                print(out, "toml_free_tree(%);\n", address_of(target));
            } else if is_map(info) {
                key, element := map_types(info);
                id := next_id(g);

                print(out, "for *entry_%, key_%: % {\n", id, id, target);
                if key.type == .STRING print(out, "if key_%.count { free(key_%.data); }\n", id, id);

                emit_free(g, tprint("entry_%.*", id), element, "");
                print(out, "}\ndeinit(%);\n", address_of(target));
            } else if default.count {
                print(out, "toml_free_fields(%, %);\n", address_of(target), address_of(default));
            } else {
                id := next_id(g);

                print(out, "{\ndefault_%: type_of(%);\ntoml_free_fields(%, *default_%);\n}\n", id, target, address_of(target), id);
            }
        case;
    }
}

generate_freer :: (T: Type) -> string {
    g: Gen;
    info := type_info(T);

    if is_value(info) || is_map(info) {
        emit_free(*g, "value.*", info, "");

        return builder_to_string(*g.out,, temp);
    }

    plan := analyze(T, true);

    g.type_name = plan.type_name;

    if !plan.ok return "";

    for *leaf: plan.leaves {
        if !owns_memory(leaf.info) continue;

        emit_free(*g, tprint("value.%", leaf.path), leaf.info, tprint("defaults.%", leaf.path));
    }

    return builder_to_string(*g.out,, temp);
}

// ---- encoding ----

emit_write :: (g: *Gen, value: string, info: *Type_Info, leaf: *Leaf) {
    out := *g.out;

    if info.type == {
        case .INTEGER;
            integer := cast(*Type_Info_Integer) info;

            if info.runtime_size > 8 {
                fail(g, "128-bit integers are not supported");

                return;
            }

            print(out, "if !toml_count_leaf(w) return;\n");

            if integer.signed print(out, "toml_write_integer(w, cast(s64) %);\n", value);
            else print(out, "toml_write_unsigned(w, cast(u64) %);\n", value);
        case .FLOAT;
            if info.runtime_size != 4 && info.runtime_size != 8 {
                fail(g, "only float32 and float64 are supported");

                return;
            }

            print(out, "if !toml_count_leaf(w) return;\ntoml_write_float(w, %);\n", value);
        case .BOOL;
            print(out, "if !toml_count_leaf(w) return;\ntoml_emit(w, ifx % then \"true\" else \"false\");\n", value);
        case .STRING;
            print(out, "if !toml_count_leaf(w) return;\ntoml_write_string(w, %);\n", value);
        case .ENUM;
            print(out, "if !toml_count_leaf(w) return;\n");
            emit_write_enum(g, value, cast(*Type_Info_Enum) info, leaf);
        case .POINTER;
            pointee := (cast(*Type_Info_Pointer) info).pointer_to;

            if pointee.type == .VOID {
                fail(g, "a `*void` cannot be encoded");

                return;
            }

            print(out, "if !% toml_write_fail(w, .OUTPUT, \"null is not a TOML value\");\nelse {\n", value);
            emit_write(g, tprint("%.*", value), pointee, leaf);
            print(out, "}\n");
        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            id := next_id(g);

            print(out, "{\nif !toml_begin_value(w) return;\ntoml_emit(w, \"[\");\nfor index_%: 0..%.count - 1 {\nif index_% toml_emit(w, \", \");\nmark_% := w.path.count;\narray_add(*w.path, Toml_Path_Part.{ index = index_% });\n", id, value, id, id, id);
            emit_write(g, tprint("%[index_%]", value, id), array.element_type, leaf);
            print(out, "w.path.count = mark_%;\nif w.error.kind != .NONE break;\n}\ntoml_emit(w, \"]\");\ntoml_end_value(w);\n}\n", id);
        case .STRUCT;
            if is_datetime(info) print(out, "toml_write_datetime(w, %);\n", address_of(value));
            else if is_value(info) print(out, "toml_write_value(w, %, w.depth);\n", address_of(value));
            else print(out, "toml_encode_inline(w, %);\n", address_of(value));
        case;
            fail(g, tprint("cannot encode %", type_label(info)));
    }
}

emit_write_enum :: (g: *Gen, value: string, info: *Type_Info_Enum, leaf: *Leaf) {
    out := *g.out;
    signed := info.internal_type.signed;
    flags := (info.enum_type_flags & .FLAGS) != 0;

    if leaf.as_number || flags {
        if signed print(out, "toml_write_integer(w, cast(s64) %);\n", value);
        else print(out, "toml_write_unsigned(w, cast(u64) %);\n", value);

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
        print(out, "case %;\ntoml_write_string(w, %);\n", number, jai_quote(apply_rule(name, leaf.enum_rule)));
    }

    print(out, "case;\ntoml_write_integer(w, number_%);\n}\n}\n", id);
}

skip_condition :: (g: *Gen, leaf: *Leaf, value: string) -> string {
    info := leaf.info;

    if leaf.skip_if == "empty" {
        if info.type == .STRING || info.type == .ARRAY return tprint("%.count == 0", value);

        fail(g, "`skip_serializing_if=empty` needs a string or an array");

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

    fail(g, "`skip_serializing_if=null` needs a pointer");

    return "false";
}

// The condition under which a field is written at all: not skipped and, for a pointer, set.
write_condition :: (g: *Gen, leaf: *Leaf, value: string) -> string {
    parts: [..] string;

    if leaf.skip_if.count array_add(*parts, tprint("!(%)", skip_condition(g, leaf, value)));
    if leaf.info.type == .POINTER array_add(*parts, value);
    if is_datetime(leaf.info) array_add(*parts, tprint("%.kind != .TABLE && %.kind != .ARRAY && %.kind != .STRING && %.kind != .INTEGER && %.kind != .FLOAT && %.kind != .BOOL", value, value, value, value, value, value));

    if !parts.count return "true";

    builder: String_Builder;

    for parts {
        if it_index append(*builder, " && ");
        append(*builder, it);
    }

    return builder_to_string(*builder,, temp);
}

// The `key = value` line of one member (a field or a map entry; `key` is Jai source for its
// name). Members that are tables are left for `emit_member_table`.
emit_member_line :: (g: *Gen, leaf: *Leaf, key: string, value: string) {
    out := *g.out;
    shape := table_shape(leaf.info);
    condition := write_condition(g, leaf, value);

    // Arrays of tables are written as headers unless empty: an empty one would vanish.
    if shape == .ARRAY || shape == .VIEW condition = tprint("(%) && %.count == 0", condition, value);
    else if shape == .VALUE condition = tprint("(%) && !toml_is_header_table(%) && !toml_is_table_array(%)", condition, address_of(value), address_of(value));
    else if shape != .NONE return;

    print(out, "if % {\nmark := w.path.count;\narray_add(*w.path, Toml_Path_Part.{ name = % });\ntoml_write_key(w, %, \" = \");\n", condition, key, key);
    emit_write(g, value, leaf.info, leaf);
    print(out, "w.path.count = mark;\ntoml_emit(w, \"\\n\");\n}\n");
}

// The `[header]` or `[[header]]` tables of one member, after all the lines of its table.
emit_member_table :: (g: *Gen, leaf: *Leaf, key: string, value: string) {
    out := *g.out;
    shape := table_shape(leaf.info);

    if shape == .NONE return;

    condition := write_condition(g, leaf, value);

    print(out, "if % {\nmark := w.path.count;\narray_add(*w.path, Toml_Path_Part.{ name = % });\n", condition, key);

    if shape == .STRUCT {
        print(out, "toml_write_header(w, false);\ntoml_encode_body(w, %);\n", address_of(value));
    } else if shape == .POINTER {
        print(out, "toml_write_header(w, false);\ntoml_encode_body(w, %);\n", value);
    } else if shape == .VALUE {
        print(out, "if toml_is_header_table(%) {\ntoml_write_header(w, false);\ntoml_encode_body(w, %);\n} else if toml_is_table_array(%) {\n", address_of(value), address_of(value), address_of(value));
        print(out, "if !toml_begin_value(w) return;\nfor item: %.items {\nitem_mark := toml_path_index_for_write(w, it_index);\ntoml_write_header(w, true);\ntoml_encode_body(w, item);\nw.path.count = item_mark;\nif w.error.kind != .NONE break;\n}\ntoml_end_value(w);\n}\n", value);
    } else {
        print(out, "if !toml_begin_value(w) return;\nfor *item: % {\nitem_mark := toml_path_index_for_write(w, it_index);\ntoml_write_header(w, true);\ntoml_encode_body(w, item);\nw.path.count = item_mark;\nif w.error.kind != .NONE break;\n}\ntoml_end_value(w);\n", value);
    }

    print(out, "w.path.count = mark;\n}\n");
}

// The entries of a `Table(string, V)` as the members of a table, or null if it cannot be written.
map_leaf :: (g: *Gen, info: *Type_Info) -> *Leaf {
    key, element := map_types(info);

    if key.type != .STRING {
        fail(g, "a Hash_Table must have string keys to be encoded as a table");

        return null;
    }

    leaf := New(Leaf,, temp);

    leaf.info = element;

    return leaf;
}

// Writes `key = value` lines then the sub-tables, for the body of a table or array element.
generate_body_encoder :: (T: Type) -> string {
    g: Gen;
    info := type_info(T);

    if is_value(info) return "if !toml_begin_value(w) return;\ntoml_write_table(w, value, w.depth - 1, false);\ntoml_end_value(w);\n";

    out := *g.out;

    if is_map(info) {
        g.type_name = tprint("%", T);
        leaf := map_leaf(*g, info);

        if !leaf return "";

        print(out, "if !toml_begin_value(w) return;\n");
        print(out, "for *entry, key: value.* {\n");
        emit_member_line(*g, leaf, "key", "entry.*");
        print(out, "}\nfor *entry, key: value.* {\n");
        emit_member_table(*g, leaf, "key", "entry.*");
        print(out, "}\ntoml_end_value(w);\n");

        if !g.ok return "";

        return builder_to_string(*g.out,, temp);
    }

    plan := analyze(T);

    g.type_name = plan.type_name;

    if !plan.ok return "";

    print(out, "if !toml_begin_value(w) return;\n");

    for *leaf: plan.leaves {
        if leaf.skip_serializing continue;

        g.field = leaf.member;
        emit_member_line(*g, leaf, jai_quote(leaf.key), tprint("value.%", leaf.path));
    }

    for *leaf: plan.leaves {
        if leaf.skip_serializing continue;

        g.field = leaf.member;
        emit_member_table(*g, leaf, jai_quote(leaf.key), tprint("value.%", leaf.path));
    }

    print(out, "toml_end_value(w);\n");

    if !g.ok return "";

    return builder_to_string(*g.out,, temp);
}

// `{ key = value, ... }`, for a struct inside an array or another inline table.
generate_inline_encoder :: (T: Type) -> string {
    g: Gen;
    info := type_info(T);
    out := *g.out;

    if is_map(info) {
        g.type_name = tprint("%", T);
        leaf := map_leaf(*g, info);

        if !leaf return "";

        print(out, "if !toml_begin_value(w) return;\ntoml_emit(w, \"{\");\nfirst := true;\nfor *entry, key: value.* {\nif !first toml_emit(w, \", \");\nfirst = false;\nmark := w.path.count;\narray_add(*w.path, Toml_Path_Part.{ name = key });\ntoml_write_key(w, key, \" = \");\n");
        emit_write(*g, "entry.*", leaf.info, leaf);
        print(out, "w.path.count = mark;\n}\ntoml_emit(w, \"}\");\ntoml_end_value(w);\n");

        if !g.ok return "";

        return builder_to_string(*g.out,, temp);
    }

    plan := analyze(T);

    g.type_name = plan.type_name;

    if !plan.ok return "";

    print(out, "if !toml_begin_value(w) return;\ntoml_emit(w, \"{\");\nfirst := true;\n");

    for *leaf: plan.leaves {
        if leaf.skip_serializing continue;

        g.field = leaf.member;
        value := tprint("value.%", leaf.path);

        print(out, "if % {\nif !first toml_emit(w, \", \");\nfirst = false;\nmark := w.path.count;\narray_add(*w.path, Toml_Path_Part.{ name = % });\ntoml_write_key(w, %, \" = \");\n", write_condition(*g, leaf, value), jai_quote(leaf.key), jai_quote(leaf.key));
        emit_write(*g, value, leaf.info, leaf);
        print(out, "w.path.count = mark;\n}\n");
    }

    print(out, "toml_emit(w, \"}\");\ntoml_end_value(w);\n");

    if !g.ok return "";

    return builder_to_string(*g.out,, temp);
}
