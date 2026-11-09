// Equality and release of a payload. The code is written for each Variant type when the program
// compiles (`variant_equal_code`, `variant_free_code`, `value_equal_code`, `value_free_code`), so
// `==` and `free_variant` run straight-line code, not a walk over type information. Printing still
// works from type information: it takes an `Any`.

#scope_module

// The offset and type of the field `name`, looking inside unnamed unions and structs too.
find_member :: (info: *Type_Info_Struct, name: string) -> offset: s64, type: *Type_Info {
    for info.members {
        if it.flags & .CONSTANT continue;
        if it.name == name return it.offset_in_bytes, it.type;

        if it.name.count == 0 && it.type.type == .STRUCT {
            inner, type := find_member(cast(*Type_Info_Struct) it.type, name);

            if type return it.offset_in_bytes + inner, type;
        }
    }

    return 0, null;
}

offset_of_member :: (info: *Type_Info_Struct, name: string) -> s64 {
    offset := find_member(info, name);

    return offset;
}

// The live case of the Variant at `p`: its position among the cases.
live_case :: (info: *Type_Info_Struct, p: *void) -> s64 {
    for info.members {
        if it.name != "which" continue;

        tag: s64;

        if it.type.runtime_size == 1 tag = (cast(*u8) (cast(*u8) p + it.offset_in_bytes)).*;
        else tag = (cast(*u16) (cast(*u8) p + it.offset_in_bytes)).*;

        return tag;
    }

    return -1;
}

// The type a `.VARIANT` type info was made from.
underlying_type :: (T: Type) -> Type {
    return cast(Type) (cast(*Type_Info_Variant) type_info(T)).variant_of;
}

// Types whose values are equal when their bytes are.
compared_as_bytes :: (info: *Type_Info) -> bool {
    return info.type == .INTEGER || info.type == .BOOL || info.type == .ENUM || info.type == .POINTER;
}

bytes_equal :: (a: *void, b: *void, size: s64) -> bool {
    if size <= 0 return true;

    return memcmp(a, b, size) == 0;
}

// ---- equality ----

// Whether the Variants are the same case with equal values: one comparison per case.
variants_equal :: (a: *$V, b: *V) -> bool {
    #insert #run variant_equal_code(V);
}

// Whether the two values are equal: strings by content, floats as numbers (0.0 equals -0.0, and
// NaN equals nothing), arrays element by element, structs member by member, anything else by bytes.
values_equal :: (a: *$T, b: *T) -> bool {
    #insert #run value_equal_code(T);
}

variant_equal_code :: (V: Type) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    infos := variant_case_infos(type_info(V));
    names := variant_case_names(type_info(V));

    append(*builder, "if a.which != b.which return false;\n");
    append(*builder, "if cast(s64) a.which == {\n");

    for infos {
        if it.type == .VOID continue;

        print_to_builder(*builder, "    case %; return values_equal(*a.%, *b.%);\n", it_index, names[it_index], names[it_index]);
    }

    append(*builder, "}\n\nreturn true;");

    return builder_to_string(*builder);
}

value_equal_code :: (T: Type) -> string {
    info := type_info(T);

    if info.type == {
        case .STRING;
            return "return a.* == b.*;";

        case .FLOAT;
            // Equal as numbers are: 0.0 equals -0.0, and NaN equals nothing.
            if info.runtime_size == 4 || info.runtime_size == 8 return "return a.* == b.*; // jailint: allow(float_equality)";

        case .INTEGER; #through;
        case .BOOL; #through;
        case .ENUM; #through;
        case .POINTER;
            return "return a.* == b.*;";

        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            same_bytes := compared_as_bytes(array.element_type);

            if array.array_type == .FIXED {
                if same_bytes return "return bytes_equal(a, b, size_of(T));";

                return "for 0..a.count - 1 if !values_equal(*a.*[it], *b.*[it]) return false;\n\nreturn true;";
            }

            if same_bytes return "if a.count != b.count return false;\n\nreturn bytes_equal(a.data, b.data, a.count * size_of(type_of(a.data.*)));";

            return "if a.count != b.count return false;\n\nfor 0..a.count - 1 if !values_equal(*a.*[it], *b.*[it]) return false;\n\nreturn true;";

        case .STRUCT;
            struct_info := cast(*Type_Info_Struct) info;

            if is_variant_type(info) return "return variants_equal(a, b);";
            if (struct_info.textual_flags & .UNION) != 0 return "return bytes_equal(a, b, size_of(T));";

            builder: String_Builder;
            builder.allocator = temp;

            for struct_info.members {
                if (it.flags & .CONSTANT) != 0 continue;

                if !it.name.count {
                    compiler_report(tprint("Variant: % has an unnamed member, which == cannot compare", T));
                    continue;
                }

                print_to_builder(*builder, "if !values_equal(*a.%, *b.%) return false;\n", it.name, it.name);
            }

            append(*builder, "\nreturn true;");

            return builder_to_string(*builder);

        case .VARIANT;
            return "return values_equal(cast(*#run underlying_type(T)) a, cast(*#run underlying_type(T)) b);";

        case; // Compared as bytes.
    }

    return "return bytes_equal(a, b, size_of(T));";
}

// ---- release ----

// Releases what the live case of the Variant owns and leaves it at its zero value: case 0 with a
// zeroed payload.
variant_free :: (v: *$V) {
    #insert #run variant_free_code(V);
}

// Releases what the value at `p` owns and leaves it empty.
value_free :: (p: *$T) {
    #insert #run value_free_code(T);
}

// Whether a value of this type owns memory that `free_variant` releases.
owns_memory :: (info: *Type_Info) -> bool {
    if info.type == {
        case .STRING;
            return true;

        case .ARRAY;
            array := cast(*Type_Info_Array) info;

            if array.array_type == .RESIZABLE return true;

            return array.array_type == .FIXED && owns_memory(array.element_type);

        case .STRUCT;
            if is_variant_type(info) return true;
            if ((cast(*Type_Info_Struct) info).textual_flags & .UNION) != 0 return false;

            for (cast(*Type_Info_Struct) info).members {
                if (it.flags & .CONSTANT) == 0 && owns_memory(it.type) return true;
            }

        case .VARIANT;
            return owns_memory((cast(*Type_Info_Variant) info).variant_of);

        case; // Other types own nothing.
    }

    return false;
}

variant_free_code :: (V: Type) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    infos := variant_case_infos(type_info(V));
    names := variant_case_names(type_info(V));
    first := true;

    for infos {
        if it.type == .VOID || !owns_memory(it) continue;

        if first append(*builder, "if cast(s64) v.which == {\n");
        first = false;

        print_to_builder(*builder, "    case %; value_free(*v.%);\n", it_index, names[it_index]);
    }

    if !first append(*builder, "}\n\n");

    append(*builder, "memset(v.payload.data, 0, V.payload_size);\nv.which = cast(V.Which) 0;");

    return builder_to_string(*builder);
}

value_free_code :: (T: Type) -> string {
    info := type_info(T);

    if !owns_memory(info) return "";

    if info.type == {
        case .STRING;
            return "if p.count free(p.data);\n\np.* = \"\";";

        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            elements := ifx owns_memory(array.element_type) then "for *p.* value_free(it);\n" else "";

            if array.array_type == .FIXED return elements;

            return tprint("if p.data {\n%    if p.allocator.proc free(p.data,, p.allocator);\n    else free(p.data);\n}\n\np.* = .{};", elements);

        case .STRUCT;
            if is_variant_type(info) return "variant_free(p);";

            builder: String_Builder;
            builder.allocator = temp;

            for (cast(*Type_Info_Struct) info).members {
                if (it.flags & .CONSTANT) != 0 || !owns_memory(it.type) continue;

                if !it.name.count {
                    compiler_report(tprint("Variant: % has an unnamed member that owns memory, which free_variant cannot release", T));
                    continue;
                }

                print_to_builder(*builder, "value_free(*p.%);\n", it.name);
            }

            return builder_to_string(*builder);

        case .VARIANT;
            return "value_free(cast(*#run underlying_type(T)) p);";

        case; // Other types own nothing.
    }

    return "";
}
