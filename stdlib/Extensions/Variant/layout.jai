// Compile-time half of Variant: the declarations inside the struct and the checks that name a
// case that is not there. Nothing here is called at run time.

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
        case;
            return 8;
    }
}

type_list_text :: (types: [] Type) -> string {
    builder: String_Builder;

    for types {
        if it_index append(*builder, ", ");
        print(*builder, "%", it);
    }

    return builder_to_string(*builder,, temp);
}

// The Jai source for what lies inside `Variant(types)`. The case types are named
// `case_type_0`... so that code generated for a variant can spell them.
declare_variant :: (types: ..Type) -> string {
    name := tprint("Variant(%)", type_list_text(types));

    if types.count == 0 {
        compiler_report("Variant needs at least one case type");

        return "";
    }

    size := 1;
    align := 1;

    for types {
        info := type_info(it);

        size = max_of(size, info.runtime_size);
        align = max_of(align, alignment_of(info));

        for other, other_index: types {
            if other_index <= it_index || other != it continue;

            compiler_report(tprint("%: the case `%` appears twice. A Variant picks its case by the value's type, so use a wrapper struct per case", name, it));

            return "";
        }
    }

    builder: String_Builder;

    print(*builder, "case_count :: %;\n", types.count);
    print(*builder, "payload_size :: %;\npayload_align :: %;\n", size, align);
    print(*builder, "case_infos :: #run collect_infos(..types);\n");

    for types print(*builder, "case_type_% :: types[%];\n", it_index, it_index);

    print(*builder, "payload: [payload_size] u8 #align payload_align;\n");

    return builder_to_string(*builder,, temp);
}

collect_infos :: (types: ..Type) -> [] *Type_Info {
    infos: [..] *Type_Info;

    for types array_add(*infos, type_info(it));

    return infos;
}

// `Variant(s64, string)`, as a person would write it.
variant_name :: ($V: Type) -> string {
    return tprint("Variant(%)", type_list_text(V.types));
}

index_of_case :: (types: [] Type, wanted: Type) -> s32 {
    for types if it == wanted return cast(s32) it_index;

    return -1;
}

// The position of `T` among the cases of `V`, or -1 after reporting why not. `user` names the
// procedure that asked.
case_of :: ($V: Type, $T: Type, user: string) -> s32 {
    #if !#run is_variant_type(type_info(V)) {
        compiler_report(tprint("%: `%` is not a Variant", user, V));

        return -1;
    } else {
        found := index_of_case(V.types, T);

        if found < 0 {
            compiler_report(tprint("%: `%` is not a case of `%`. The cases are %", user, T, variant_name(V), type_list_text(V.types)));
        }

        return found;
    }
}
