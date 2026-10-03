#scope_module
render_enum :: (builder: *String_Builder, item: Any, info: *Type_Info_Enum, style: *Print_Style) {
    assert(info.internal_type != null && info.values.count == info.names.count);
    bits := get_u64_from_void_pointer(item.value_pointer, info.internal_type.runtime_size);
    mask: u64 = 0xffffffffffffffff;

    if info.internal_type.runtime_size < 8 mask = ((cast(u64) 1) << (info.internal_type.runtime_size * 8)) - 1;

    if (info.enum_type_flags & .FLAGS) == 0 {
        for value, index: info.values {
            if ((cast,no_check(u64) value) & mask) == bits {
                append(builder, info.names[index]);
                return;
            }
        }

        if bits == 0 {
            append(builder, "0");
            return;
        }

        append(builder, "(");
        append(builder, info.name);
        append(builder, " out of range: ");
        format := style.default_format_int;
        format.value = Any.{ info.internal_type, item.value_pointer };
        print_integer(builder, format);
        append(builder, ")");

        return;
    }

    if bits == 0 {
        for value, index: info.values if value == 0 {
            append(builder, info.names[index]);
            return;
        }

        append(builder, "0");

        return;
    }

    remaining := bits;
    printed := false;

    for value, index: info.values {
        flag := (cast,no_check(u64) value) & mask;
        if flag == 0 || (remaining & flag) != flag continue;
        if printed append(builder, " | ");

        append(builder, info.names[index]);
        remaining &= ~flag;
        printed = true;
    }

    if remaining != 0 {
        if printed append(builder, " | ");
        append(builder, "0x");
        print_u64_base_n(builder, remaining, FormatInt.{ base = 16 });
    }
}
