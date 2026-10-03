parse_codeview_types :: (data: [] u8, expect_codeview_version := true, verbose := false) -> bool, [] *Codeview_Type {
    records: [..] *Codeview_Type;
    keep := false;

    defer if !keep {
        cv_release_types(records);
        log_error("Debug_Info: invalid or unsupported CodeView type stream\n");
    }

    input := Line_Wire_Input.{ data = data, limit = data.count };

    if data.count < 0 || (data.count > 0 && !data.data) return false, .{};
    if expect_codeview_version && line_wire_uint(*input, 4) != 4 return false, .{};

    while input.cursor < input.limit && !input.failed {
        length := cv_u16(*input);
        if input.failed || length < 2 || length > input.limit - input.cursor return false, .{};

        after := input.cursor + length;

        input.limit = after;
        kind := cast(Codeview_Leaf_Type) cv_u16(*input);
        record_input := Line_Wire_Input.{ data = .{ after - input.cursor, data.data + input.cursor }, limit = after - input.cursor };

        if !cv_read_type(*record_input, kind, *records) || record_input.failed || !cv_padding(*record_input) {
            log_error("Debug_Info: invalid or unsupported CodeView type % at %\n", kind, records.count + 0xfff);
            return false, .{};
        }

        input.cursor = after;

        input.limit = data.count;
    }

    if input.failed return false, .{};
    if verbose log("Debug_Info: decoded % CodeView types\n", records.count);

    keep = true;

    return true, .{ records.count, records.data };
}

get_type :: (types: [] *Codeview_Type, type: Type_Id) -> *Codeview_Type, success: bool {
    id := cast(u32) type;
    if id < 0x1000 {
        name := describe_builtin(type);
        if !name.count return null, false;

        free(name);
        builtin := New(Codeview_Builtin);
        builtin.id = type;

        return cast(*Codeview_Type) builtin, true;
    }

    if types.count < 0 || !types.data || id - 0x1000 >= types.count return null, false;

    result := types[id - 0x1000];

    return result, result != null;
}

get_id_type :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, type: Type_Id) -> *Codeview_Type, success: bool {
    id := cast(u32) type;
    if id & 0x80000000 {
        if (id & 0x7fffffff) < 0x1000 return null, false;
        return get_type(ids, cast(Type_Id) (id & 0x7fffffff));
    }

    return get_type(types, type);
}

describe_builtin :: (type_id: Type_Id) -> string {
    id := cast(u32) type_id;
    if id >= 0x1000 return "";

    name: string;

    if id & 0xff == {
        case 0x00; name = "no type";
        case 0x02; name = "absolute";
        case 0x03; name = "void";
        case 0x08; name = "HRESULT";
        case 0x10; name = "char";
        case 0x11; name = "short";
        case 0x12; name = "long";
        case 0x13; name = "long long";
        case 0x20; name = "unsigned char";
        case 0x21; name = "unsigned short";
        case 0x22; name = "unsigned long";
        case 0x23; name = "unsigned long long";
        case 0x30; name = "bool8";
        case 0x31; name = "bool16";
        case 0x32; name = "bool32";
        case 0x33; name = "bool64";
        case 0x40; name = "float";
        case 0x41; name = "double";
        case 0x46; name = "float16";
        case 0x68; name = "s8";
        case 0x69; name = "u8";
        case 0x70; name = "char";
        case 0x71; name = "wchar_t";
        case 0x72; name = "s16";
        case 0x73; name = "u16";
        case 0x74; name = "int";
        case 0x75; name = "unsigned int";
        case 0x76; name = "s64";
        case 0x77; name = "u64";
        case 0x7a; name = "char16_t";
        case 0x7b; name = "char32_t";
        case; return "";
    }

    mode := (id >> 8) & 15;
    if mode == 0 return sprint("%", name);
    if mode > 7 return "";

    return sprint("% *[mode=%]", name, mode);
}

describe_id_type :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, type_id: Type_Id) -> string, success: bool {
    id := cast(u32) type_id;
    return cv_describe_index(types, ids, cast(Type_Id) (id | 0x80000000), 0);
}

describe_type :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, type_id: Type_Id) -> string, success: bool {
    return cv_describe_index(types, ids, type_id, 0);
}

describe_type :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, type: *Codeview_Type) -> string, success: bool {
    return cv_describe_record(types, ids, type, 0);
}

dump_codeview_types :: (types: [] *Codeview_Type, ids: [] *Codeview_Type) -> bool {
    // Prepare all descriptions before writing, so a failed graph produces no
    // misleading partial dump.
    descriptions: [..] string;

    defer {
        for text: descriptions free(text);
        array_free(descriptions);
    }

    if types.count < 0 || ids.count < 0 || (types.count && !types.data) || (ids.count && !ids.data) return false;

    for type: types {
        text, ok := cv_describe_record(types, ids, type, 0);
        if !ok return false;
        array_add(*descriptions, text);
    }

    for type: ids {
        text, ok := cv_describe_record(types, ids, type, 0);
        if !ok return false;
        array_add(*descriptions, text);
    }

    for text, index: descriptions print("% %: %\n", ifx index < types.count then "type" else "id",
        0x1000 + (ifx index < types.count then index else index - types.count), text);

    return true;
}

is_procedure :: (symbol: *Codeview_Symbol) -> bool {
    if !symbol return false;

    kind := symbol.symbol_type;

    return kind == .LPROC32 || kind == .GPROC32 || kind == .LPROC32_ID || kind == .GPROC32_ID;
}

#scope_module
#import "Basic";

cv_u8 :: (input: *Line_Wire_Input) -> u8 {
    return cast(u8) line_wire_uint(input, 1);
}

cv_u16 :: (input: *Line_Wire_Input) -> u16 {
    return cast(u16) line_wire_uint(input, 2);
}

cv_u32 :: (input: *Line_Wire_Input) -> u32 {
    return cast(u32) line_wire_uint(input, 4);
}

cv_id :: (input: *Line_Wire_Input) -> Type_Id {
    return cast(Type_Id) cv_u32(input);
}

cv_new :: (input: *Line_Wire_Input, records: *[..] *Codeview_Type, $T: Type, kind: Codeview_Leaf_Type) -> *T {
    // Each root allocation owns an immutable copy of its payload. String views
    // point into this tail, so standalone/TPI parsing never retains hidden
    // input buffers or requires a second allocation for each name.
    result := cast(*T) alloc(size_of(T) + input.data.count);

    if !result return null;

    Initialize(result);
    tail := cast(*u8) result + size_of(T);

    memcpy(tail, input.data.data, input.data.count);
    input.data.data = tail;
    result.base.kind = kind;
    array_add(records, cast(*Codeview_Type) result);

    return result;
}

cv_numeric :: (input: *Line_Wire_Input, allow_negative := false) -> u64 {
    leaf := cv_u16(input);
    if leaf < 0x8000 return leaf;

    size: s64;
    signed := false;

    if leaf == {
        case 0x8000;
            size = 1;
            signed = true;
        case 0x8001;
            size = 2;
            signed = true;
        case 0x8002; size = 2;
        case 0x8003;
            size = 4;
            signed = true;
        case 0x8004; size = 4;
        case 0x8009;
            size = 8;
            signed = true;
        case 0x800a; size = 8;
        case;
            input.failed = true;
            return 0;
    }

    value := line_wire_uint(input, size);
    if signed && (value & (cast(u64) 1 << (size * 8 - 1))) {
        if !allow_negative input.failed = true;
        if size < 8 value |= 0xffffffffffffffff << (size * 8);
    }

    return value;
}

cv_padding :: (input: *Line_Wire_Input) -> bool {
    if input.failed return false;
    if input.cursor == input.limit return true;

    first := cv_u8(input);
    if first < 0xf1 || first > 0xff return false;

    count := first - 0xf0;
    if count - 1 > input.limit - input.cursor return false;
    for index: 1..count - 1 if cv_u8(input) != first - index return false;

    return !input.failed && input.cursor == input.limit;
}

cv_member_padding :: (input: *Line_Wire_Input) -> bool {
    if input.cursor >= input.limit || input.data[input.cursor] < 0xf1 return !input.failed;

    first := cv_u8(input);
    count := first - 0xf0;

    if count - 1 > input.limit - input.cursor return false;
    for index: 1..count - 1 if cv_u8(input) != first - index return false;

    return !input.failed;
}

cv_read_type :: (input: *Line_Wire_Input, kind: Codeview_Leaf_Type, records: *[..] *Codeview_Type) -> bool {
    if kind == {
        case .STRUCTURE, .CLASS, .UNION;
            result := cv_new(input, records, Codeview_Struct, kind);
            if !result return false;

            result.num_fields = cv_u16(input);
            result.properties = cast(Struct_Properties) cv_u16(input);
            result.field_list = cv_id(input);

            if kind != .UNION {
                result.derived = cv_id(input);
                result.vshape = cv_u32(input);
            }

            result.size = cv_numeric(input);
            result.name = line_wire_string(input);
            if result.properties & .UNIQUE_NAME result.unique_name = line_wire_string(input);
        case .ENUM;
            result := cv_new(input, records, Codeview_Enum, kind);
            if !result return false;

            result.num_fields = cv_u16(input);
            result.properties = cast(Struct_Properties) cv_u16(input);
            result.type = cv_id(input);
            result.field_list = cv_id(input);
            result.name = line_wire_string(input);
            if result.properties & .UNIQUE_NAME result.unique_name = line_wire_string(input);
        case .ARRAY;
            result := cv_new(input, records, Codeview_Array, kind);
            if !result return false;

            result.element_type = cv_id(input);
            result.index_type = cv_id(input);
            result.size = cv_numeric(input);
            result.name = line_wire_string(input);
        case .POINTER;
            result := cv_new(input, records, Codeview_Pointer, kind);
            if !result return false;

            result.type = cv_id(input);
            result.flags = cv_u32(input);
            mode := (result.flags >> 5) & 7;
            if mode == 2 || mode == 3 return false; // No member-pointer tail in the public model.
        case .MODIFIER;
            result := cv_new(input, records, Codeview_Modifier, kind);
            if !result return false;

            result.type = cv_id(input);
            result.modifier = cast(Codeview_Modifier.Modifier_Type) cv_u16(input);
        case .PROCEDURE;
            result := cv_new(input, records, Codeview_Procedure, kind);
            if !result return false;

            result.return_type = cv_id(input);
            result.calling_convention = cv_u8(input);
            result.attributes = cv_u8(input);
            result.num_arguments = cv_u16(input);
            result.arguments = cv_id(input);
        case .MFUNCTION;
            result := cv_new(input, records, Codeview_Method_Function, kind);
            if !result return false;

            result.return_type = cv_id(input);
            result.class_type = cv_id(input);
            result.this_type = cv_id(input);
            result.calling_convention = cv_u8(input);
            result.attributes = cv_u8(input);
            result.num_arguments = cv_u16(input);
            result.arguments = cv_id(input);
            result.this_adjuster = cast(s32) cv_u32(input);
        case .ARGLIST, .BUILDINFO, .SUBSTR_LIST;
            result := cv_new(input, records, Codeview_Arguments, kind);
            if !result return false;

            count := cast(s64) (ifx kind == .BUILDINFO then cv_u16(input) else cv_u32(input));

            if input.failed || count > (input.limit - input.cursor) / 4 return false;

            if count {
                result.arguments = .{ count, cast(*Type_Id) alloc(count * size_of(Type_Id)) };
                if !result.arguments.data return false;
                for *argument: result.arguments argument.* = cv_id(input);
            }
        case .STRING_ID;
            result := cv_new(input, records, Codeview_String_Id, kind);
            if !result return false;

            result.id = cv_u32(input);
            result.name = line_wire_string(input);
        case .FUNC_ID, .MFUNC_ID;
            result := cv_new(input, records, Codeview_Function_Id, kind);
            if !result return false;

            result.scope = cv_id(input);
            result.type = cv_id(input);
            result.name = line_wire_string(input);
        case .UDT_SRC_LINE;
            result := cv_new(input, records, Codeview_Udt_Source_Line, kind);
            if !result return false;

            result.type = cv_id(input);
            result.src = cv_id(input);
            result.line = cv_u32(input);
        case .ALIAS;
            result := cv_new(input, records, Codeview_Alias, kind);
            if !result return false;

            result.type = cv_id(input);
            result.name = line_wire_string(input);
        case .TYPESERVER2;
            result := cv_new(input, records, Codeview_Typeserver2, kind);
            if !result return false;

            result.sig70.data1 = cv_u32(input);
            result.sig70.data2 = cv_u16(input);
            result.sig70.data3 = cv_u16(input);
            for *byte: result.sig70.data4 byte.* = cv_u8(input);

            result.age = cv_u32(input);
            result.name = line_wire_string(input);
        case .METHODLIST;
            result := cv_new(input, records, Codeview_Method_List, kind);
            if !result return false;

            while input.cursor < input.limit && !input.failed {
                if input.data[input.cursor] >= 0xf1 break;

                method: Codeview_Method;
                method.attributes = cv_u16(input);
                cv_u16(input); // Format-required reserved word.
                method.procedure_type = cv_id(input);
                mode := (method.attributes >> 2) & 7;
                if mode == 4 || mode == 6 method.base_offset = cv_u32(input);

                array_add(*result.elements, method);
            }
        case .FIELDLIST;
            result := cv_new(input, records, Codeview_List, kind);
            if !result return false;

            while input.cursor < input.limit && !input.failed {
                if !cv_member_padding(input) return false;
                if input.cursor == input.limit break;

                member_kind := cast(Codeview_Leaf_Type) cv_u16(input);
                if member_kind == {
                    case .MEMBER;
                        member: Codeview_Member;
                        member.access = cv_u16(input);
                        member.type = cv_id(input);
                        member.offset = cv_numeric(input);
                        member.name = line_wire_string(input);
                        array_add(*result.members, member);
                    case .ENUMERATE;
                        member: Codeview_Enumerate;
                        member.access = cv_u16(input);
                        member.value = cv_numeric(input, true);
                        member.name = line_wire_string(input);
                        array_add(*result.enumerates, member);
                    case .NESTTYPE;
                        cv_u16(input);
                        member: Codeview_Nested_Type;
                        member.type = cv_id(input);
                        member.name = line_wire_string(input);
                        array_add(*result.nested, member);
                    case .METHOD;
                        member: Codeview_Overloaded_Method;
                        member.count = cv_u16(input);
                        member.list_type = cv_id(input);
                        member.name = line_wire_string(input);
                        array_add(*result.overloaded_methods, member);
                    case .ONEMETHOD;
                        member: Codeview_Method;
                        member.attributes = cv_u16(input);
                        member.procedure_type = cv_id(input);
                        mode := (member.attributes >> 2) & 7;
                        if mode == 4 || mode == 6 member.base_offset = cv_u32(input);

                        member.name = line_wire_string(input);
                        array_add(*result.methods, member);
                    case .INDEX;
                        cv_u16(input);
                        if cast(u32) result.previous return false;

                        result.previous = cv_id(input);
                    case; return false;
                }
            }
        case; return false;
    }

    return !input.failed;
}

cv_release_types :: (types: [] *Codeview_Type) {
    for type: types {
        if !type continue;
        if type.kind == .ARGLIST || type.kind == .BUILDINFO || type.kind == .SUBSTR_LIST free((cast(*Codeview_Arguments) type).arguments.data);
        if type.kind == .METHODLIST array_free((cast(*Codeview_Method_List) type).elements);

        if type.kind == .FIELDLIST {
            list := cast(*Codeview_List) type;

            array_free(list.members);
            array_free(list.overloaded_methods);
            array_free(list.methods);
            array_free(list.enumerates);
            array_free(list.nested);
        }

        free(type);
    }

    free(types.data);
}

cv_describe_index :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, id: Type_Id, depth: s64) -> string, bool {
    numeric := cast(u32) id;
    if numeric < 0x1000 {
        text := describe_builtin(id);
        return text, text.count > 0;
    }

    table := types;

    if numeric & 0x80000000 {
        table = ids;
        numeric &= 0x7fffffff;
    }

    if numeric < 0x1000 || table.count < 0 || !table.data || numeric - 0x1000 >= table.count return "", false;

    return cv_describe_record(types, ids, table[numeric - 0x1000], depth + 1);
}

cv_describe_record :: (types: [] *Codeview_Type, ids: [] *Codeview_Type, type: *Codeview_Type, depth: s64) -> string, bool {
    if !type || depth > 64 return "", false;

    if type.kind == {
        case .BUILTIN;
            text := describe_builtin((cast(*Codeview_Builtin) type).id);

            return text, text.count > 0;
        case .STRUCTURE, .CLASS, .UNION; return sprint("%", (cast(*Codeview_Struct) type).name), true;
        case .ENUM; return sprint("%", (cast(*Codeview_Enum) type).name), true;
        case .STRING_ID; return sprint("string %", (cast(*Codeview_String_Id) type).name), true;
        case .FUNC_ID, .MFUNC_ID; return sprint("%", (cast(*Codeview_Function_Id) type).name), true;
        case .ALIAS; return sprint("%", (cast(*Codeview_Alias) type).name), true;
        case .TYPESERVER2; return sprint("type-server %", (cast(*Codeview_Typeserver2) type).name), true;
        case .FIELDLIST;
            list := cast(*Codeview_List) type;

            return sprint("field-list(% members, % enumerators, % methods)", list.members.count, list.enumerates.count, list.methods.count + list.overloaded_methods.count), true;
        case .METHODLIST; return sprint("method-list(%)", (cast(*Codeview_Method_List) type).elements.count), true;
        case .UDT_SRC_LINE; return sprint("source-line(%)", (cast(*Codeview_Udt_Source_Line) type).line), true;
        case .ARGLIST, .BUILDINFO, .SUBSTR_LIST;
            args := (cast(*Codeview_Arguments) type).arguments;
            if args.count < 0 || (args.count && !args.data) return "", false;

            builder: String_Builder;

            defer free_buffers(*builder);
            append(*builder, "(");

            for arg, index: args {
                reference := arg;
                if type.kind == .BUILDINFO || type.kind == .SUBSTR_LIST reference = cast(Type_Id) (cast(u32) arg | 0x80000000);

                text, ok := cv_describe_index(types, ids, reference, depth + 1);
                if !ok return "", false;
                if index append(*builder, ", ");

                append(*builder, text);
                free(text);
            }

            append(*builder, ")");

            return builder_to_string(*builder), true;
        case .POINTER;
            pointer := cast(*Codeview_Pointer) type;
            text, ok := cv_describe_index(types, ids, pointer.type, depth + 1);

            if !ok return "", false;

            defer free(text);
            mode := (pointer.flags >> 5) & 7;

            return sprint("% %", text, ifx mode == 1 then "&" else ifx mode == 4 then "&&" else "*"), true;
        case .MODIFIER;
            modifier := cast(*Codeview_Modifier) type;
            text, ok := cv_describe_index(types, ids, modifier.type, depth + 1);

            if !ok return "", false;

            defer free(text);

            return sprint("%%%%", ifx modifier.modifier & .CONST then "const " else "",
                ifx modifier.modifier & .VOLATILE then "volatile " else "", ifx modifier.modifier & .UNALINGED then "unaligned " else "", text), true;
        case .ARRAY;
            array := cast(*Codeview_Array) type;
            text, ok := cv_describe_index(types, ids, array.element_type, depth + 1);

            if !ok return "", false;

            defer free(text);

            return sprint("% [bytes=%]", text, array.size), true;
        case .PROCEDURE, .MFUNCTION;
            return_id: Type_Id;
            arguments_id: Type_Id;

            if type.kind == .PROCEDURE {
                procedure := cast(*Codeview_Procedure) type;
                return_id = procedure.return_type;
                arguments_id = procedure.arguments;
            } else {
                procedure := cast(*Codeview_Method_Function) type;
                return_id = procedure.return_type;
                arguments_id = procedure.arguments;
            }

            result, result_ok := cv_describe_index(types, ids, return_id, depth + 1);
            if !result_ok return "", false;

            defer free(result);
            arguments, arguments_ok := cv_describe_index(types, ids, arguments_id, depth + 1);
            if !arguments_ok return "", false;

            defer free(arguments);

            return sprint("% %", result, arguments), true;
    }

    return "", false;
}
