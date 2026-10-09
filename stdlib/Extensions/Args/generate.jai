// Compile-time half of Args: reads a struct's fields and notes, rejects bad declarations with
// a message naming the field, and writes the Jai source of the parser (an `#insert` in
// parse_command), of the help description (describe) and of the Help and Is_Set structs.
//
// Everything here runs inside the compiler. Nothing here is called at run time.

#scope_module

Field_Kind :: enum u8 {
    FLAG;
    COUNT;
    INT;
    FLOAT;
    STRING;
    ENUM;
    LIST;
    SUBCOMMAND;
    SELECTOR;
    INVALID;
}

Field :: struct {
    name: string;
    info: *Type_Info;
    kind: Field_Kind;
    element: *Type_Info; // LIST
    element_kind: Field_Kind;
    long: string;
    short: u8;
    env: string;
    value_name: string;
    has_range: bool;
    range_lo: string;
    range_hi: string;
    positional: bool;
    required: bool;
    hidden: bool;
    global: bool;
    optional: bool;
    conflicts: [..] string;
    requires: [..] string;
}

Analysis :: struct {
    type_name: string;
    ok := true;
    quiet: bool;
    fields: [..] Field;
}

// Names the Help struct reserves for itself.
help_meta_field :: (name: string) -> bool {
    return name == "about" || name == "version";
}

complain :: (analysis: *Analysis, field: string, message: string) {
    analysis.ok = false;
    if analysis.quiet return;
    where := ifx field.count then tprint("%.%", analysis.type_name, field) else analysis.type_name;
    compiler_report(tprint("Args: %: %", where, message));
}

has_note :: (member: *Type_Info_Struct_Member, note: string) -> bool {
    for member.notes if it == note return true;
    return false;
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

is_number :: (text: string) -> bool {
    start := 0;
    if text.count && text[0] == #char "-" start = 1;
    digits := 0;
    dots := 0;
    for start..text.count - 1 {
        c := text[it];
        if c == #char "." dots += 1;
        else if c >= #char "0" && c <= #char "9" digits += 1;
        else return false;
    }
    return digits > 0 && dots <= 1;
}

classify_scalar :: (info: *Type_Info) -> Field_Kind {
    if info.type == {
        case .BOOL; return .FLAG;
        case .INTEGER; return .INT;
        case .FLOAT; return .FLOAT;
        case .STRING; return .STRING;
        case .ENUM;
            enum_info := cast(*Type_Info_Enum) info;
            if enum_info.enum_type_flags & .FLAGS return .INVALID;
            return .ENUM;
        case; return .INVALID;
    }
}

// Reads one field: its type, then its notes.
read_field :: (analysis: *Analysis, member: *Type_Info_Struct_Member) -> Field {
    field: Field;
    field.name = member.name;
    field.info = member.type;
    field.long = kebab(member.name);
    name := member.name;

    is_subcommand := has_note(member, "subcommand");
    info := member.type;
    if info.type == .ARRAY {
        array := cast(*Type_Info_Array) info;
        if array.array_type != .RESIZABLE {
            complain(analysis, name, "arrays other than `[..] T` are not supported; use a resizable array for a repeatable option");
            field.kind = .INVALID;
        } else {
            field.element = array.element_type;
            field.element_kind = classify_scalar(array.element_type);
            if field.element_kind == .INVALID || field.element_kind == .FLAG {
                complain(analysis, name, tprint("a list of % is not supported; the elements can be integers, floats, strings or enums", type_label(array.element_type)));
                field.kind = .INVALID;
            } else {
                field.kind = .LIST;
            }
        }
    } else if info.type == .STRUCT {
        if is_subcommand {
            field.kind = .SUBCOMMAND;
        } else {
            complain(analysis, name, tprint("%: a struct field is only supported as a subcommand; add `@subcommand` (and an enum field marked `@subcommand` to select it)", type_label(info)));
            field.kind = .INVALID;
        }
    } else if info.type == .ENUM && is_subcommand {
        field.kind = .SELECTOR;
        if (cast(*Type_Info_Enum) info).enum_type_flags & .FLAGS {
            complain(analysis, name, "an enum_flags cannot select a subcommand");
            field.kind = .INVALID;
        }
    } else {
        field.kind = classify_scalar(info);
        if field.kind == .INVALID {
            complain(analysis, name, tprint("unsupported field type: %. Use bool, an integer, a float, a string, an enum, a `[..] T` of those, or a struct marked `@subcommand`", type_label(info)));
        }
    }

    for member.notes {
        note := it;
        if note == "---" continue;
        if note == "positional" field.positional = true;
        else if note == "required" field.required = true;
        else if note == "count" {
            if field.kind != .INT && field.kind != .INVALID complain(analysis, name, "`@count` needs an integer field");
            else if field.kind == .INT field.kind = .COUNT;
        } else if note == "hidden" field.hidden = true;
        else if note == "global" field.global = true;
        else if note == "optional" field.optional = true;
        else if note == "subcommand" {
            if field.kind != .SUBCOMMAND && field.kind != .SELECTOR && field.kind != .INVALID {
                complain(analysis, name, "`@subcommand` goes on a struct field (one per subcommand) and on the enum field that selects it");
            }
        } else if begins_with(note, "short=") {
            value := slice(note, 6, note.count - 6);
            if value.count != 1 || !is_alnum_byte(value[0]) {
                complain(analysis, name, tprint("`@%` needs a single letter or digit, as in `@short=c`", note));
            } else {
                field.short = value[0];
            }
        } else if begins_with(note, "long=") {
            value := slice(note, 5, note.count - 5);
            if !value.count || !is_name_text(value) complain(analysis, name, tprint("`@%` needs a name of letters, digits, `-` and `_`, as in `@long=output-file`", note));
            else field.long = copy_string(value,, temp);
        } else if begins_with(note, "env=") {
            value := slice(note, 4, note.count - 4);
            if !value.count || !is_name_text(value) complain(analysis, name, tprint("`@%` needs an environment variable name, as in `@env=MY_TOOL_JOBS`", note));
            else field.env = copy_string(value,, temp);
        } else if begins_with(note, "value=") {
            value := slice(note, 6, note.count - 6);
            if !value.count complain(analysis, name, "`@value=` needs a name to show in help, as in `@value=FILE`");
            else field.value_name = copy_string(value,, temp);
        } else if begins_with(note, "conflicts=") {
            array_add(*field.conflicts, copy_string(slice(note, 10, note.count - 10),, temp));
        } else if begins_with(note, "requires=") {
            array_add(*field.requires, copy_string(slice(note, 9, note.count - 9),, temp));
        } else if begins_with(note, "range(") && ends_with(note, ")") {
            body := slice(note, 6, note.count - 7);
            found, low, high := split_from_left(body, #char ",");
            if !found || !is_number(low) || !is_number(high) {
                complain(analysis, name, tprint("`@%` needs two numbers, as in `@range(1,64)`", note));
            } else {
                field.has_range = true;
                field.range_lo = copy_string(low,, temp);
                field.range_hi = copy_string(high,, temp);
            }
        } else {
            complain(analysis, name, tprint("unknown note `@%`. The notes are positional, short=, long=, env=, range(a,b), value=, count, required, conflicts=, requires=, hidden, subcommand, global and optional", note));
        }
    }
    return field;
}

is_alnum_byte :: (c: u8) -> bool {
    return (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || (c >= #char "0" && c <= #char "9");
}

is_name_text :: (text: string) -> bool {
    for 0..text.count - 1 {
        c := text[it];
        if !is_alnum_byte(c) && c != #char "-" && c != #char "_" return false;
    }
    return true;
}

find_field :: (fields: [] Field, name: string) -> *Field {
    for *fields if it.name == name return it;
    return null;
}

numeric_kind :: (kind: Field_Kind) -> bool {
    return kind == .INT || kind == .FLOAT || kind == .COUNT;
}

// Reads and validates every field of `T`. With `quiet`, problems are not reported (the caller
// that does report has already seen them).
analyze :: (T: Type, has_version: bool, quiet: bool) -> Analysis {
    analysis: Analysis;
    analysis.quiet = quiet;
    struct_info := cast(*Type_Info_Struct) type_info(T);
    analysis.type_name = struct_info.name;

    for *member: struct_info.members {
        if member.flags & .CONSTANT continue;
        array_add(*analysis.fields, read_field(*analysis, member));
    }
    fields := analysis.fields;

    selectors := 0;
    subcommands := 0;
    for *field: fields {
        name := field.name;
        if help_meta_field(name) complain(*analysis, name, "this name is reserved by `Args.Help`; rename the field");
        if field.kind == .INVALID continue;

        if field.kind == .SELECTOR selectors += 1;
        if field.kind == .SUBCOMMAND subcommands += 1;

        if field.has_range && !numeric_kind(field.kind) && !(field.kind == .LIST && numeric_kind(field.element_kind)) {
            complain(*analysis, name, tprint("`@range` needs a numeric field (an integer or a float), not %", type_label(field.info)));
        }
        if field.has_range && field.kind == .COUNT complain(*analysis, name, "`@range` cannot be combined with `@count`");
        if field.has_range && (field.kind == .INT || (field.kind == .LIST && field.element_kind == .INT)) {
            if contains(field.range_lo, #char ".") || contains(field.range_hi, #char ".") {
                complain(*analysis, name, "an integer field needs an integer `@range`");
            }
        }
        if field.env.count {
            if field.kind == .LIST || field.kind == .SUBCOMMAND || field.kind == .SELECTOR complain(*analysis, name, "`@env` is for options with a single value");
            if field.positional complain(*analysis, name, "`@env` cannot be combined with `@positional`");
        }
        if field.kind == .FLAG || field.kind == .COUNT || field.kind == .SUBCOMMAND || field.kind == .SELECTOR {
            if field.value_name.count complain(*analysis, name, "`@value` names the value of an option; this field has none");
        }
        if field.required && (field.kind == .FLAG || field.kind == .COUNT || field.kind == .SUBCOMMAND || field.kind == .SELECTOR) {
            complain(*analysis, name, "`@required` does not apply to flags and counters (and subcommands are required unless the selector is `@optional`)");
        }
        if field.optional && field.kind != .SELECTOR complain(*analysis, name, "`@optional` goes on the enum field that selects the subcommand");
        if field.global && (field.positional || field.kind == .SUBCOMMAND || field.kind == .SELECTOR) complain(*analysis, name, "`@global` applies to options");

        if field.positional {
            if field.kind == .FLAG || field.kind == .COUNT || field.kind == .SUBCOMMAND || field.kind == .SELECTOR {
                complain(*analysis, name, "a positional must be a value, not a flag, counter or subcommand");
            }
            if field.short complain(*analysis, name, "`@short` cannot be combined with `@positional`");
            if field.long != kebab(name) complain(*analysis, name, "`@long` cannot be combined with `@positional`");
        }
        if field.kind == .SUBCOMMAND || field.kind == .SELECTOR {
            if field.short complain(*analysis, name, "`@short` does not apply to subcommands");
        }
    }

    // Positionals: in declaration order, only the last may be a list, a required one may not
    // follow an optional one.
    last_positional: *Field;
    count := 0;
    for *field: fields {
        if !field.positional || field.kind == .INVALID continue;
        if last_positional && last_positional.kind == .LIST {
            complain(*analysis, last_positional.name, "only the last positional may be an array; this one is followed by another positional");
        }
        if last_positional && !last_positional.required && field.required {
            complain(*analysis, field.name, tprint("a required positional cannot follow the optional `%`; mark that one `@required` too or reorder them", last_positional.name));
        }
        last_positional = field;
        count += 1;
    }

    if selectors > 1 complain(*analysis, "", "only one enum field can select the subcommand (`@subcommand`)");
    if subcommands && !selectors complain(*analysis, "", "subcommand structs need an enum field marked `@subcommand` that says which one was chosen");
    if selectors && !subcommands complain(*analysis, "", "the `@subcommand` enum field has no subcommand struct fields to select");
    if subcommands && count complain(*analysis, "", "a command with subcommands cannot also take positionals");

    // The selector's values and the subcommand fields name each other.
    for *selector: fields {
        if selector.kind != .SELECTOR continue;
        enum_info := cast(*Type_Info_Enum) selector.info;
        for value_name: enum_info.names {
            matched := false;
            for fields if it.kind == .SUBCOMMAND && kebab(it.name) == kebab(value_name) matched = true;
            if !matched complain(*analysis, selector.name, tprint("the value `%` has no subcommand field named `%`", value_name, to_lower_copy(value_name,, temp)));
        }
        for *sub: fields {
            if sub.kind != .SUBCOMMAND continue;
            matched := false;
            for enum_info.names if kebab(it) == kebab(sub.name) matched = true;
            if !matched complain(*analysis, sub.name, tprint("the enum `%` has no value for this subcommand", enum_info.name));
        }
    }

    // Names: each long and short spelling once, and not the built-in --help/-h (--version/-V).
    taken_long: [..] string;
    taken_short: [..] u8;
    array_add(*taken_long, "help");
    array_add(*taken_short, #char "h");
    if has_version {
        array_add(*taken_long, "version");
        array_add(*taken_short, #char "V");
    }
    for *field: fields {
        if field.kind == .INVALID || field.positional || field.kind == .SELECTOR continue;
        longs: [..] string;
        array_add(*longs, field.long);
        if field.kind == .FLAG array_add(*longs, tprint("no-%", field.long));
        for spelling: longs {
            if field.kind == .SUBCOMMAND break;
            for taken_long if it == spelling {
                complain(*analysis, field.name, tprint("the long name `--%` is already used (by another option or built in); pick another with `@long=`", spelling));
            }
            array_add(*taken_long, spelling);
        }
        if field.kind == .SUBCOMMAND {
            for taken_sub: fields {
                if taken_sub.kind == .SUBCOMMAND && taken_sub.name != field.name && taken_sub.long == field.long {
                    complain(*analysis, field.name, tprint("the subcommand name `%` is used twice", field.long));
                }
            }
            continue;
        }
        if field.short {
            for taken_short if it == field.short {
                complain(*analysis, field.name, tprint("the short name `-%` is already used (by another option or built in)", short_text(field.short)));
            }
            array_add(*taken_short, field.short);
        }
    }

    // @conflicts / @requires name real fields.
    for *field: fields {
        for other: field.conflicts {
            target := find_field(fields, other);
            if !target complain(*analysis, field.name, tprint("`@conflicts=%` names a field that does not exist", other));
            else if target == field complain(*analysis, field.name, "`@conflicts` names the field itself");
            else if target.kind == .SUBCOMMAND || target.kind == .SELECTOR complain(*analysis, field.name, tprint("`@conflicts=%` names a subcommand; conflicts are between options and positionals", other));
        }
        for other: field.requires {
            target := find_field(fields, other);
            if !target complain(*analysis, field.name, tprint("`@requires=%` names a field that does not exist", other));
            else if target == field complain(*analysis, field.name, "`@requires` names the field itself");
            else if target.kind == .SUBCOMMAND || target.kind == .SELECTOR complain(*analysis, field.name, tprint("`@requires=%` names a subcommand; requirements are between options and positionals", other));
        }
    }
    return analysis;
}

// ---- source writing ----

quote :: (text: string) -> string {
    builder: String_Builder;
    append(*builder, "\"");
    for 0..text.count - 1 {
        c := text[it];
        if c == #char "\"" append(*builder, "\\\"");
        else if c == #char "\\" append(*builder, "\\\\");
        else if c == #char "\n" append(*builder, "\\n");
        else append(*builder, string.{ 1, *c });
    }
    append(*builder, "\"");
    return builder_to_string(*builder,, temp);
}

// The display name of a field in messages: `--jobs`, or `<PATHS>` for a positional.
display_name :: (field: *Field) -> string {
    if field.positional {
        return tprint("<%>", ifx field.value_name.count then field.value_name else to_upper_copy(field.long,, temp));
    }
    return tprint("--%", field.long);
}

range_literal :: (field: *Field) -> string {
    if !field.has_range return "NO_RANGE";
    kind := ifx field.kind == .LIST then field.element_kind else field.kind;
    text := quote(tprint("%..%", field.range_lo, field.range_hi));
    if kind == .FLOAT return tprint("Range.{ present = true, flo = %, fhi = %, text = % }", field.range_lo, field.range_hi, text);
    return tprint("Range.{ present = true, lo = %, hi = %, text = % }", field.range_lo, field.range_hi, text);
}

// The statements that convert `value` (a string variable in scope) into `destination`, with
// `display` naming the option in errors; on failure they return `failure`.
store_statement :: (field: *Field, destination: string, display: string, failure: string) -> string {
    kind := ifx field.kind == .LIST then field.element_kind else field.kind;
    if kind == .ENUM {
        return tprint("if !store_enum_%(state, %, value, %) return %;\n", field.name, display, destination, failure);
    }
    if kind == .FLAG {
        return tprint("if !store_flag_text(state, %, value, %) return %;\n", display, destination, failure);
    }
    return tprint("if !store_value(state, %, value, %, %) return %;\n", display, destination, range_literal(field), failure);
}

generate_command :: (analysis: *Analysis, has_version: bool) -> string {
    out: String_Builder;
    fields := analysis.fields;

    // One conversion procedure per enum field.
    for *field: fields {
        enum_info: *Type_Info_Enum;
        element_text: string;
        if field.kind == .ENUM {
            enum_info = cast(*Type_Info_Enum) field.info;
            element_text = tprint("type_of(T.{}.%)", field.name);
        } else if field.kind == .LIST && field.element_kind == .ENUM {
            enum_info = cast(*Type_Info_Enum) field.element;
            element_text = tprint("type_of(T.{}.%[0])", field.name);
        } else continue;
        names: [..] string;
        for enum_info.names array_add(*names, kebab(it));
        print(*out, "store_enum_% :: (state: *Parse_State, display: string, text: string, dest: *%) -> bool {\n", field.name, element_text);
        append(*out, "    lowered := to_lower_copy(text,, temp);\n");
        for enum_info.names {
            print(*out, "    if lowered == % { dest.* = .%; return true; }\n", quote(names[it_index]), it);
        }
        append(*out, "    bad_choice(state, display, text, .[");
        for names {
            if it_index append(*out, ", ");
            append(*out, quote(it));
        }
        append(*out, "]);\n    return false;\n}\n");
    }

    // One procedure per option.
    for *field: fields {
        if field.positional || field.kind == .SUBCOMMAND || field.kind == .SELECTOR || field.kind == .INVALID continue;
        print(*out, "opt_% :: (state: *Parse_State, out: *T, set: *Is_Set(T), display: string, is_short: bool, rest: string, has_rest: bool, negated: bool) -> Outcome {\n", field.name);
        if field.kind == .FLAG {
            print(*out, "    if has_rest && !is_short return unexpected_value(state, display);\n    out.% = !negated;\n    set.% = true;\n    return .FLAG;\n", field.name, field.name);
        } else if field.kind == .COUNT {
            print(*out, "    if has_rest && !is_short return unexpected_value(state, display);\n    bump(*out.%);\n    set.% = true;\n    return .FLAG;\n", field.name, field.name);
        } else {
            append(*out, "    value: string;\n    if !take_value(state, display, rest, has_rest, is_short, *value) return .ERROR;\n");
            if field.kind == .LIST {
                print(*out, "    item: type_of(T.{}.%[0]);\n", field.name);
                print(*out, "    %", store_statement(field, "*item", "display", ".ERROR"));
                print(*out, "    array_add(*out.%, item);\n", field.name);
            } else {
                print(*out, "    %", store_statement(field, tprint("*out.%", field.name), "display", ".ERROR"));
                // A Color_Mode field takes effect at once, so errors and --help later on use it.
                if field.kind == .ENUM && field.info == type_info(Color_Mode) print(*out, "    set_color_mode(out.%);\n", field.name);
            }
            print(*out, "    set.% = true;\n    return .DONE;\n", field.name);
        }
        append(*out, "}\n");
    }

    // The option table.
    append(*out, "h_option :: (out_v: *void, set_v: *void, state: *Parse_State, name: string, is_short: bool, rest: string, has_rest: bool, only_globals: bool) -> Outcome {\n");
    append(*out, "    out := cast(*T) out_v;\n    set := cast(*Is_Set(T)) set_v;\n");
    for pass: 0..1 {
        has_any := false;
        for *field: fields {
            if field.positional || field.kind == .SUBCOMMAND || field.kind == .SELECTOR || field.kind == .INVALID continue;
            if pass == 0 && !field.short continue;
            has_any = true;
        }
        if !has_any continue;
        append(*out, ifx pass == 0 then "    if is_short {\n        if name == {\n" else "    if !is_short {\n        if name == {\n");
        for *field: fields {
            if field.positional || field.kind == .SUBCOMMAND || field.kind == .SELECTOR || field.kind == .INVALID continue;
            guard := ifx field.global then "" else "if only_globals return .NOT_FOUND; ";
            if pass == 0 {
                if !field.short continue;
                print(*out, "            case %; %return opt_%(state, out, set, %, true, rest, has_rest, false);\n", quote(short_text(field.short)), guard, field.name, quote(tprint("-%", short_text(field.short))));
            } else {
                print(*out, "            case %; %return opt_%(state, out, set, %, false, rest, has_rest, false);\n", quote(field.long), guard, field.name, quote(tprint("--%", field.long)));
                if field.kind == .FLAG {
                    print(*out, "            case %; %return opt_%(state, out, set, %, false, rest, has_rest, true);\n", quote(tprint("no-%", field.long)), guard, field.name, quote(tprint("--no-%", field.long)));
                }
            }
        }
        append(*out, "            case; return .NOT_FOUND;\n        }\n    }\n");
    }
    append(*out, "    return .NOT_FOUND;\n}\n");

    // Positionals.
    append(*out, "h_positional :: (out_v: *void, set_v: *void, state: *Parse_State, text: string) -> bool {\n");
    append(*out, "    out := cast(*T) out_v;\n    set := cast(*Is_Set(T)) set_v;\n    index := state.positional_index;\n    state.positional_index += 1;\n    value := text;\n");
    position := 0;
    for *field: fields {
        if !field.positional || field.kind == .INVALID continue;
        display := quote(display_name(field));
        if field.kind == .LIST {
            print(*out, "    if index >= % {\n        item: type_of(T.{}.%[0]);\n        %", position, field.name, store_statement(field, "*item", display, "false"));
            print(*out, "        array_add(*out.%, item);\n        set.% = true;\n        return true;\n    }\n", field.name, field.name);
        } else {
            print(*out, "    if index == % {\n        %", position, store_statement(field, tprint("*out.%", field.name), display, "false"));
            print(*out, "        set.% = true;\n        return true;\n    }\n", field.name);
        }
        position += 1;
    }
    append(*out, "    fail(state, .UNEXPECTED_ARGUMENT, text, text, \"unexpected argument `%`\", text);\n    return false;\n}\n");

    // Subcommands.
    append(*out, "h_subcommand :: (out_v: *void, set_v: *void, state: *Parse_State, name: string) -> Outcome {\n");
    selector: *Field;
    for *field: fields if field.kind == .SELECTOR selector = field;
    if selector {
        append(*out, "    out := cast(*T) out_v;\n    set := cast(*Is_Set(T)) set_v;\n    if name == {\n");
        enum_info := cast(*Type_Info_Enum) selector.info;
        for *field: fields {
            if field.kind != .SUBCOMMAND continue;
            value_name: string;
            for enum_info.names if kebab(it) == kebab(field.name) value_name = it;
            print(*out, "        case %;\n", quote(field.long));
            print(*out, "            out.% = .%;\n            set.% = true;\n", selector.name, value_name, selector.name);
            append(*out, "            link := Global_Link.{ out = out_v, set = set_v, handle = h_option, next = state.globals };\n            state.globals = *link;\n");
            print(*out, "            array_add(*state.path, %);\n", quote(field.long));
            print(*out, "            ok := parse_command(type_of(T.{}.%), help.%, state, *out.%, *set.%);\n", field.name, field.name, field.name, field.name);
            append(*out, "            state.globals = link.next;\n            return ifx ok then Outcome.DONE else Outcome.ERROR;\n");
        }
        append(*out, "    }\n");
    }
    append(*out, "    return .NOT_FOUND;\n}\n");

    // Defaults from the environment, required options, conflicts.
    append(*out, "h_finish :: (out_v: *void, set_v: *void, state: *Parse_State) -> bool {\n");
    append(*out, "    out := cast(*T) out_v;\n    set := cast(*Is_Set(T)) set_v;\n");
    for *field: fields {
        if !field.env.count || field.kind == .INVALID continue;
        print(*out, "    if !set.% {\n        env := environment_lookup(state, %);\n        if env.count {\n            value := env;\n", field.name, quote(field.env));
        display := quote(tprint("$%", field.env));
        print(*out, "            %", store_statement(field, tprint("*out.%", field.name), display, "false"));
        print(*out, "            set.% = true;\n        }\n    }\n", field.name);
    }
    if selector && !selector.optional {
        print(*out, "    if !set.% {\n        fail(state, .MISSING_COMMAND, \"\", \"\", \"a subcommand is required\");\n        return false;\n    }\n", selector.name);
    }
    for *field: fields {
        if !field.required || field.kind == .INVALID continue;
        print(*out, "    if !set.% {\n        fail(state, .MISSING_REQUIRED, %, \"\", \"the % `%` is required\");\n        return false;\n    }\n", field.name, quote(display_name(field)), ifx field.positional then "argument" else "option", display_name(field));
    }
    for *field: fields {
        if field.kind == .INVALID continue;
        for other: field.conflicts {
            target := find_field(fields, other);
            if !target continue;
            print(*out, "    if set.% && set.% {\n        fail(state, .CONFLICT, %, %, \"`%` cannot be used with `%`\");\n        return false;\n    }\n", field.name, other, quote(display_name(field)), quote(display_name(target)), display_name(field), display_name(target));
        }
        for other: field.requires {
            target := find_field(fields, other);
            if !target continue;
            print(*out, "    if set.% && !set.% {\n        fail(state, .REQUIRES, %, %, \"`%` requires `%`\");\n        return false;\n    }\n", field.name, other, quote(display_name(field)), quote(display_name(target)), display_name(field), display_name(target));
        }
    }
    append(*out, "    return true;\n}\n");

    append(*out, "handlers: Command_Handlers;\nhandlers.option = h_option;\nhandlers.positional = h_positional;\nhandlers.subcommand = h_subcommand;\nhandlers.finish = h_finish;\n");
    print(*out, "handlers.has_subcommands = %;\n", ifx selector then "true" else "false");
    print(*out, "handlers.has_version = %;\n", ifx has_version then "true" else "false");
    append(*out, "return run_command(*handlers, out, set, state);\n");
    return builder_to_string(*out);
}

// The statements of `describe` that fill a Command_Spec.
generate_description :: (analysis: *Analysis) -> string {
    out: String_Builder;
    for *field: analysis.fields {
        if field.kind == .INVALID || field.kind == .SELECTOR continue;
        if field.kind == .SUBCOMMAND {
            print(*out, "array_add(*spec.subcommands, describe(type_of(T.{}.%), help.%, %));\n", field.name, field.name, quote(field.long));
            continue;
        }
        append(*out, "{\n    option: Option_Spec;\n");
        print(*out, "    option.field = %;\n    option.long = %;\n", quote(field.name), quote(field.long));
        if field.short print(*out, "    option.short = %;\n", cast(s64) field.short);
        if field.env.count print(*out, "    option.env = %;\n", quote(field.env));
        if field.value_name.count print(*out, "    option.value_name = %;\n", quote(field.value_name));
        if !field.hidden print(*out, "    option.help = help.%;\n", field.name);
        kind_name := "VALUE";
        if field.kind == .FLAG kind_name = "FLAG";
        else if field.kind == .COUNT kind_name = "COUNT";
        else if field.kind == .LIST kind_name = "LIST";
        print(*out, "    option.kind = .%;\n", kind_name);
        print(*out, "    option.required = %;\n    option.hidden = %;\n    option.global = %;\n", field.required, field.hidden, field.global);
        if field.kind == .FLAG && !field.positional print(*out, "    option.negatable = true;\n");
        enum_info: *Type_Info_Enum;
        if field.kind == .ENUM enum_info = cast(*Type_Info_Enum) field.info;
        if field.kind == .LIST && field.element_kind == .ENUM enum_info = cast(*Type_Info_Enum) field.element;
        if enum_info {
            append(*out, "    option.choices = .[");
            for enum_info.names {
                if it_index append(*out, ", ");
                append(*out, quote(kebab(it)));
            }
            append(*out, "];\n");
        }
        if field.kind == .FLAG print(*out, "    if defaults.% option.default_text = \"true\";\n", field.name);
        if field.kind == .INT || field.kind == .FLOAT || field.kind == .STRING || field.kind == .ENUM {
            print(*out, "    option.default_text = default_text(defaults.%);\n", field.name);
        }
        if field.has_range print(*out, "    option.range_text = %;\n", quote(tprint("%..%", field.range_lo, field.range_hi)));
        print(*out, "    array_add(*spec.%, option);\n}\n", ifx field.positional then "positionals" else "options");
    }
    selector: *Field;
    for *field: analysis.fields if field.kind == .SELECTOR selector = field;
    if selector print(*out, "spec.subcommand_required = %;\n", !selector.optional);
    return builder_to_string(*out);
}

// Warns about options with no help text. Runs in the compiler.
warn_missing_help :: (type_name: string, field: string, text: string) {
    if text.count return;
    compiler_report(tprint("Args: %.%: no help text; add `%` to the Args.Help literal, or mark the field `@hidden`", type_name, field, field), mode = .WARNING);
}

generate_help_checks :: (analysis: *Analysis) -> string {
    out: String_Builder;
    for *field: analysis.fields {
        if field.kind == .INVALID || field.kind == .SELECTOR || field.kind == .SUBCOMMAND || field.hidden continue;
        print(*out, "#run warn_missing_help(%, %, help.%);\n", quote(analysis.type_name), quote(field.name), field.name);
    }
    return builder_to_string(*out);
}

// ---- the Help and Is_Set structs ----

is_subcommand_member :: (member: *Type_Info_Struct_Member) -> bool {
    return member.type.type == .STRUCT && has_note(member, "subcommand");
}

help_members :: (T: Type) -> string {
    out: String_Builder;
    struct_info := cast(*Type_Info_Struct) type_info(T);
    for *member: struct_info.members {
        if member.flags & .CONSTANT continue;
        if help_meta_field(member.name) continue;
        if member.type.type == .ENUM && has_note(member, "subcommand") continue;
        if has_note(member, "hidden") continue;
        if is_subcommand_member(member) print(*out, "%: Help(type_of(T.{}.%));\n", member.name, member.name);
        else print(*out, "%: string;\n", member.name);
    }
    return builder_to_string(*out);
}

set_members :: (T: Type) -> string {
    out: String_Builder;
    struct_info := cast(*Type_Info_Struct) type_info(T);
    for *member: struct_info.members {
        if member.flags & .CONSTANT continue;
        if is_subcommand_member(member) print(*out, "%: Is_Set(type_of(T.{}.%));\n", member.name, member.name);
        else print(*out, "%: bool;\n", member.name);
    }
    return builder_to_string(*out);
}
