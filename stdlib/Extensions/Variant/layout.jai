// Compile-time half of Variant: the declarations inside the struct, the names of the cases and the
// checks that name a case that is not there. Nothing here is called at run time.

#scope_module

max_of :: (a: s64, b: s64) -> s64 {
    return ifx a > b then a else b;
}

// The alignment a value of this type needs, from its type information alone.
alignment_of :: (info: *Type_Info) -> s64 {
    if info.type == {
        case .STRUCT;
            return max_of(1, (cast(*Type_Info_Struct) info).alignment);
        case .ARRAY;
            array := cast(*Type_Info_Array) info;

            if array.array_type == .FIXED return alignment_of(array.element_type);

            return 8;
        case .VARIANT;
            return alignment_of((cast(*Type_Info_Variant) info).variant_of);
        case .INTEGER; #through;
        case .FLOAT; #through;
        case .ENUM; #through;
        case .BOOL;
            return max_of(1, info.runtime_size);
        case .VOID;
            return 1;
        case;
            return 8;
    }
}

type_list_text :: (types: [] Type) -> string {
    builder: String_Builder;

    if is_named_form(types) {
        fields := 0;

        append(*builder, "struct {");

        for (cast(*Type_Info_Struct) type_info(types[0])).members {
            if it.flags & .CONSTANT continue;
            print(*builder, " %: %;", it.name, cast(Type) it.type);
            fields += 1;
        }

        append(*builder, ifx fields then " }" else "}");

        return builder_to_string(*builder,, temp);
    }

    for types {
        if it_index append(*builder, ", ");
        print(*builder, "%", it);
    }

    return builder_to_string(*builder,, temp);
}

// Names a struct may not use for a case, because the Variant declares them itself.
RESERVED_NAMES :: string.["which", "payload", "types", "case_count", "case_names", "case_infos", "payload_size", "payload_align"];

is_upper :: (c: u8) -> bool {
    return c >= #char "A" && c <= #char "Z";
}

is_lower :: (c: u8) -> bool {
    return c >= #char "a" && c <= #char "z";
}

is_digit :: (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
}

// `HttpRequest`, `Http_Request` and `HTTPRequest` all become `http_request`.
snake_case :: (name: string) -> string {
    builder: String_Builder;
    previous: u8 = #char "_";

    for i: 0..name.count - 1 {
        c := name[i];

        if c == #char "_" {
            if previous != #char "_" append(*builder, c);
            previous = c;

            continue;
        }

        if is_upper(c) && previous != #char "_" {
            next_is_lower := i + 1 < name.count && is_lower(name[i + 1]);

            if is_lower(previous) || is_digit(previous) || (is_upper(previous) && next_is_lower) append(*builder, #char "_");
        }

        lowered := ifx is_upper(c) then c + (#char "a" - #char "A") else c;

        append(*builder, lowered);
        previous = c;
    }

    return builder_to_string(*builder,, temp);
}

// `http_request` becomes `HTTP_REQUEST`; a trailing `_` (on a built-in type's name) is dropped.
tag_of :: (name: string) -> string {
    builder: String_Builder;
    text := name;

    if text.count > 1 && text[text.count - 1] == #char "_" text.count -= 1;

    for text append(*builder, ifx is_lower(it) then it - (#char "a" - #char "A") else it);

    return builder_to_string(*builder,, temp);
}

// The field name a case of a type list gets, or "" when the type has no name to give. `void` is
// the case `none`.
derived_name :: (info: *Type_Info) -> string {
    if info.type == {
        case .VOID;
            return "none";
        case .STRUCT;
            name := (cast(*Type_Info_Struct) info).name;

            if name.count == 0 return "";

            return snake_case(name);
        case .ENUM;
            return snake_case((cast(*Type_Info_Enum) info).name);
        case .POINTER;
            return derived_name((cast(*Type_Info_Pointer) info).pointer_to);
        case .INTEGER;
            // A pointer whose target the compiler has not laid out yet can show up as a 0-byte
            // integer; it has no usable name.
            if info.runtime_size == 0 return "";

            return tprint("%_", cast(Type) info);
        case .FLOAT; #through;
        case .BOOL; #through;
        case .STRING;
            // `s64: s64;` is not allowed, so a built-in type's case gets a trailing `_`.
            return tprint("%_", cast(Type) info);
        case;
            return "";
    }
} @TempAllocates

// The cases written as `struct { a: A; b: B; }`: one argument, a struct without a name.
is_named_form :: (types: [] Type) -> bool {
    if types.count != 1 return false;

    info := type_info(types[0]);

    return info.type == .STRUCT && (cast(*Type_Info_Struct) info).name.count == 0;
}

Case :: struct {
    name: string;
    info: *Type_Info;
    spelled: string; // how the struct's own declarations spell the type
}

// The cases of `Variant(types)`, or false after reporting why there are none.
collect_cases :: (types: [] Type) -> [..] Case, bool {
    cases: [..] Case;
    title := tprint("Variant(%)", type_list_text(types));

    if types.count == 0 {
        compiler_report("Variant needs at least one case: `Variant(Leaf, *Branch)` or `Variant(struct { email: string; phone: string; })`");

        return cases, false;
    }

    if is_named_form(types) {
        for member: (cast(*Type_Info_Struct) type_info(types[0])).members {
            if member.flags & .CONSTANT continue;

            array_add(*cases, .{ name = member.name, info = member.type, spelled = tprint("type_of(types[0].%)", member.name) });
        }

        if cases.count == 0 {
            compiler_report(tprint("%: the struct has no fields, so the Variant has no cases", title));

            return cases, false;
        }
    } else {
        for types {
            name := derived_name(type_info(it));

            if name.count == 0 {
                compiler_report(tprint("%: `%` has no name to give its case. Name the cases: `Variant(struct { name: %; ... })`", title, it, it));

                return cases, false;
            }

            array_add(*cases, .{ name = name, info = type_info(it), spelled = tprint("types[%]", it_index) });
        }
    }

    for case, case_index: cases {
        for RESERVED_NAMES {
            if case .name != it continue;

            compiler_report(tprint("%: a case cannot be named `%`, the Variant uses that name itself", title, it));

            return cases, false;
        }

        for other, other_index: cases {
            if other_index <= case_index || tag_of(other.name) != tag_of(case .name) continue;

            compiler_report(tprint("%: two cases would both be tagged .%. Name the cases yourself: `Variant(struct { a: %; b: %; })`", title, tag_of(case .name), cast(Type) case .info, cast(Type) other.info));

            return cases, false;
        }
    }

    return cases, true;
}

// The Jai source for what lies inside `Variant(types)`.
declare_variant :: (types: ..Type) -> string {
    cases, ok := collect_cases(types);

    if !ok return "";

    size := 1;
    align := 1;

    for cases {
        size = max_of(size, it.info.runtime_size);
        align = max_of(align, alignment_of(it.info));
    }

    builder: String_Builder;

    print(*builder, "Which :: enum % {\n", ifx cases.count < 256 then "u8" else "u16");
    for cases print(*builder, "    % :: %;\n", tag_of(it.name), it_index);

    append(*builder, "}\n");

    print(*builder, "case_count :: %;\n", cases.count);
    print(*builder, "payload_size :: %;\npayload_align :: %;\n", size, align);

    for cases print(*builder, "case_type_% :: %;\n", it_index, it.spelled);

    append(*builder, "case_infos :: #run collect_infos(");

    for cases {
        if it_index append(*builder, ", ");
        print(*builder, "case_type_%", it_index);
    }

    append(*builder, ");\n");

    append(*builder, "case_names :: #run collect_names(");

    for cases {
        if it_index append(*builder, ", ");
        print(*builder, "\"%\"", it.name);
    }

    append(*builder, ");\n");

    // The payload bytes come first and the cases are left uninitialized, so a Variant that is
    // not set starts as case 0 with every payload byte zero, whatever defaults the case types have.
    append(*builder, "which: Which;\nunion {\n    payload: [payload_size] u8 #align payload_align;\n");
    for cases print(*builder, "    %: case_type_%%;\n", it.name, it_index, ifx it.info.type == .VOID then "" else " = ---");

    append(*builder, "}\n");

    return builder_to_string(*builder,, temp);
}

collect_infos :: (types: ..Type) -> [] *Type_Info {
    infos: [..] *Type_Info;

    for types array_add(*infos, type_info(it));

    return infos;
}

collect_names :: (names: ..string) -> [] string {
    result: [..] string;

    for names array_add(*result, it);

    return result;
}

// `Variant(s64, string)`, as a person would write it.
variant_title :: ($V: Type) -> string {
    return tprint("Variant(%)", type_list_text(V.types));
}

// The cases of `V` as `.LEAF (Leaf), .BRANCH (*Branch)`, for messages.
case_list_text :: ($V: Type) -> string {
    builder: String_Builder;

    for V.case_infos {
        if it_index append(*builder, ", ");
        print(*builder, ".% (%)", tag_of(V.case_names[it_index]), cast(Type) it);
    }

    return builder_to_string(*builder,, temp);
}

// The position of the case of type `T` in `V` (0 for the first case), or -1 after reporting why
// there is none. `user` names the procedure that asked.
case_of :: ($V: Type, $T: Type, user: string) -> s32 {
    #if !#run is_variant_type(type_info(V)) {
        compiler_report(tprint("%: `%` is not a Variant", user, V));

        return -1;
    } else {
        found: s32 = -1;

        for V.case_infos {
            if it != type_info(T) continue;

            if found >= 0 {
                compiler_report(tprint("%: `%` is the type of more than one case of `%` (%). Set the case by name instead: `v.which = .%; v.% = value;`", user, T, variant_title(V), case_list_text(V), tag_of(V.case_names[found]), V.case_names[found]));

                return -1;
            }

            found = cast(s32) it_index;
        }

        if found < 0 {
            compiler_report(tprint("%: `%` is not a case of `%`. The cases are %", user, T, variant_title(V), case_list_text(V)));
        }

        return found;
    }
}

// The position of the `void` case of `V`, or -1 after reporting why there is none (or why it is
// ambiguous). `user` names the procedure that asked.
void_case_of :: ($V: Type, user: string) -> s32 {
    #if !#run is_variant_type(type_info(V)) {
        compiler_report(tprint("%: `%` is not a Variant", user, V));

        return -1;
    } else {
        found: s32 = -1;

        for V.case_infos {
            if it.type != .VOID continue;

            if found >= 0 {
                compiler_report(tprint("%: `%` has more than one `void` case (%). Set the case by name instead: `v.which = .%;`", user, variant_title(V), case_list_text(V), tag_of(V.case_names[found])));

                return -1;
            }

            found = cast(s32) it_index;
        }

        if found < 0 {
            compiler_report(tprint("%: `%` has no `void` case. Add one: `Variant(void, ...)`, or a field `none: void;` in the named form. The cases are %", user, variant_title(V), case_list_text(V)));
        }

        return found;
    }
}
