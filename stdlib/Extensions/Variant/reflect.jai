// Run-time half of Variant that works from type information: equality and release of a payload
// whose type is only known as a `*Type_Info`.

#scope_module

// The fields of a resizable array `[..] T`, whatever T is.
Raw_Array :: struct {
    count: s64;
    data: *u8;
    allocated: s64;
    allocator: Allocator;
}

// The fields of a view `[] T`.
Raw_View :: struct {
    count: s64;
    data: *u8;
}

offset_of_member :: (info: *Type_Info_Struct, name: string) -> s64 {
    for info.members if it.name == name return it.offset_in_bytes;

    return 0;
}

bytes_equal :: (a: *void, b: *void, size: s64) -> bool {
    if size <= 0 return true;

    return memcmp(a, b, size) == 0;
}

values_equal :: (info: *Type_Info, a: *void, b: *void) -> bool {
    if info.type == {
        case .STRING;
            return (cast(*string) a).* == (cast(*string) b).*;
        case .FLOAT;
            // Equal as numbers are: 0.0 equals -0.0, and NaN equals nothing.
            if info.runtime_size == 4 return (cast(*float32) a).* == (cast(*float32) b).*; // jailint: allow(float_equality)
            if info.runtime_size == 8 return (cast(*float64) a).* == (cast(*float64) b).*; // jailint: allow(float_equality)

            return bytes_equal(a, b, info.runtime_size);
        case .ARRAY;
            array := cast(*Type_Info_Array) info;

            if array.array_type == .FIXED return elements_equal(array.element_type, a, b, array.array_count);

            first := cast(*Raw_View) a;
            second := cast(*Raw_View) b;

            if first.count != second.count return false;

            return elements_equal(array.element_type, first.data, second.data, first.count);
        case .STRUCT;
            return structs_equal(cast(*Type_Info_Struct) info, a, b);
        case .VARIANT;
            return values_equal((cast(*Type_Info_Variant) info).variant_of, a, b);
        case;
            return bytes_equal(a, b, info.runtime_size);
    }
}

elements_equal :: (element: *Type_Info, a: *void, b: *void, count: s64) -> bool {
    stride := element.runtime_size;

    for 0..count - 1 {
        if !values_equal(element, cast(*u8) a + it * stride, cast(*u8) b + it * stride) return false;
    }

    return true;
}

structs_equal :: (info: *Type_Info_Struct, a: *void, b: *void) -> bool {
    if is_variant_type(info) {
        first := cast(*s32) a;
        second := cast(*s32) b;

        if first.* != second.* return false;

        cases := variant_case_infos(info);
        payload := offset_of_member(info, "payload");

        if first.* < 0 || first.* >= cases.count return false;

        return values_equal(cases[first.*], cast(*u8) a + payload, cast(*u8) b + payload);
    }

    if (info.textual_flags & .UNION) != 0 return bytes_equal(a, b, info.runtime_size);

    for member: info.members {
        if (member.flags & .CONSTANT) != 0 continue;

        if !values_equal(member.type, cast(*u8) a + member.offset_in_bytes, cast(*u8) b + member.offset_in_bytes) return false;
    }

    return true;
}

// Releases what the value at `p` owns and leaves it empty.
free_value :: (info: *Type_Info, p: *void) {
    if info.type == {
        case .STRING;
            text := cast(*string) p;

            if text.count free(text.data);

            text.* = "";
        case .ARRAY;
            array := cast(*Type_Info_Array) info;

            if array.array_type == .FIXED {
                free_elements(array.element_type, p, array.array_count);
            } else if array.array_type == .RESIZABLE {
                list := cast(*Raw_Array) p;

                if list.data {
                    free_elements(array.element_type, list.data, list.count);

                    if list.allocator.proc free(list.data,, list.allocator);
                    else free(list.data);
                }

                list.* = .{};
            }
        case .STRUCT;
            free_struct(cast(*Type_Info_Struct) info, p);
        case .VARIANT;
            free_value((cast(*Type_Info_Variant) info).variant_of, p);
    }
}

free_elements :: (element: *Type_Info, p: *void, count: s64) {
    for 0..count - 1 free_value(element, cast(*u8) p + it * element.runtime_size);
}

free_struct :: (info: *Type_Info_Struct, p: *void) {
    if is_variant_type(info) {
        free_variant_value(info, p);

        return;
    }

    if (info.textual_flags & .UNION) != 0 return;

    for member: info.members {
        if (member.flags & .CONSTANT) != 0 continue;

        free_value(member.type, cast(*u8) p + member.offset_in_bytes);
    }
}

// Frees the live case of the Variant at `p`, then zeroes it.
free_variant_value :: (info: *Type_Info, p: *void) {
    struct_info := cast(*Type_Info_Struct) info;
    cases := variant_case_infos(info);
    payload := cast(*u8) p + offset_of_member(struct_info, "payload");
    index := cast(*s32) p;

    if index.* >= 0 && index.* < cases.count free_value(cases[index.*], payload);

    memset(payload, 0, cases_size(struct_info));
    index.* = 0;
}

// The size of the payload bytes of a Variant.
cases_size :: (info: *Type_Info_Struct) -> s64 {
    for info.members {
        if it.name == "payload" return (cast(*Type_Info_Array) it.type).array_count;
    }

    return 0;
}
