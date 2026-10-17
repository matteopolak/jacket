get_u64_from_void_pointer :: (pointer: *void, size_in_bytes: int) -> u64 {
    if size_in_bytes == 1 return (cast(*u8) pointer).*;
    if size_in_bytes == 2 return (cast(*u16) pointer).*;
    if size_in_bytes == 4 return (cast(*u32) pointer).*;

    assert(size_in_bytes == 8);

    return (cast(*u64) pointer).*;
}

get_s64_from_void_pointer :: (pointer: *void, size_in_bytes: int) -> s64 {
    if size_in_bytes == 1 return (cast(*s8) pointer).*;
    if size_in_bytes == 2 return (cast(*s16) pointer).*;
    if size_in_bytes == 4 return (cast(*s32) pointer).*;

    assert(size_in_bytes == 8);

    return (cast(*s64) pointer).*;
}

print_item_to_builder :: (builder: *String_Builder, item: Any, inside_struct_or_array := true, may_be_uninitialized := false) {
    render_item(builder, item, inside_struct_or_array, may_be_uninitialized, formatter_style());
}

#scope_module
formatter_base_type :: (info: *Type_Info) -> *Type_Info {
    current := info;
    slow := info;
    move_slow := false;

    while current && current.type == .VARIANT {
        current = (cast(*Type_Info_Variant) current).variant_of;
        assert(current != null && current != slow, "invalid variant descriptor");
        if move_slow slow = (cast(*Type_Info_Variant) slow).variant_of;
        move_slow = !move_slow;
    }

    return current;
}

render_item :: (builder: *String_Builder, item: Any, inside: bool, may_be_uninitialized: bool, style: *Print_Style) {
    info := item.type;
    if !info && !item.value_pointer {
        append(builder, "Any.{}");
        return;
    }

    if !info {
        context.assertion_failed(#caller_location, "cannot format an Any without type metadata");
        return;
    }

    if !item.value_pointer && info.runtime_size > 0 {
        context.assertion_failed(#caller_location, "cannot format an Any without payload storage");
        return;
    }

    if !inside && info == type_info(FormatInt) {
        print_integer(builder, (cast(*FormatInt) item.value_pointer).*);
        return;
    }

    if !inside && info == type_info(FormatFloat) {
        print_float(builder, (cast(*FormatFloat) item.value_pointer).*);
        return;
    }

    if !inside && info == type_info(FormatStruct) {
        render_struct_format(builder, (cast(*FormatStruct) item.value_pointer).*, style);
        return;
    }

    if !inside && info == type_info(FormatArray) {
        format := (cast(*FormatArray) item.value_pointer).*;
        value := format.value;

        if !value.type || !value.value_pointer {
            append(builder, "(null Any)");
            return;
        }

        assert(value.type.type == .ARRAY, "FormatArray requires an array value");
        render_array(builder, value, cast(*Type_Info_Array) value.type, format, may_be_uninitialized, style);

        return;
    }

    if info.type == .FLOAT {
        format := style.default_format_float;
        format.value = item;
        print_float(builder, format);
        return;
    }

    if info.type == .STRING {
        text := (cast(*string) item.value_pointer).*;

        if inside {
            append(builder, cast(u8) 34);

            for byte: text {
                if byte == 10 append(builder, "\\n");
                else append(builder, byte);
            }

            append(builder, cast(u8) 34);
        } else append(builder, text);

        return;
    }

    if info.type == .BOOL {
        if (cast(*bool) item.value_pointer).* append(builder, "true");
        else append(builder, "false");
        return;
    }

    if info.type == .INTEGER {
        format := style.default_format_int;
        format.value = item;
        print_integer(builder, format);

        return;
    }

    if info.type == .STRUCT {
        if formatter_struct_callback(builder, item, style) return;
        render_struct(builder, cast(*Type_Info_Struct) info, item.value_pointer, style.default_format_struct, style);
        return;
    }

    if info.type == .VARIANT {
        if formatter_struct_callback(builder, item, style) return;

        target := (cast(*Type_Info_Variant) info).variant_of;

        assert(target != null && target != info, "invalid variant descriptor");
        render_item(builder, Any.{ target, item.value_pointer }, inside, may_be_uninitialized, style);

        return;
    }

    if info.type == .POINTER || info.type == .PROCEDURE {
        pointer := (cast(**void) item.value_pointer).*;
        if !pointer {
            append(builder, "null");
            return;
        }

        if info.type == .POINTER {
            target := (cast(*Type_Info_Pointer) info).pointer_to;
            if target && target.type == .STRUCT && formatter_struct_callback(builder, item, style) return;
        }

        print_u64_base_n(builder, cast(u64) pointer, style.default_format_absolute_pointer);

        return;
    }

    if info.type == .ARRAY {
        render_array(builder, item, cast(*Type_Info_Array) info, style.default_format_array, may_be_uninitialized, style);
        return;
    }

    if info.type == .ENUM {
        render_enum(builder, item, cast(*Type_Info_Enum) info, style);
        return;
    }

    if info.type == .ANY {
        nested := (cast(*Any) item.value_pointer).*;
        assert(nested.type != info || nested.value_pointer != item.value_pointer, "an Any cannot contain itself");
        render_item(builder, nested, inside, may_be_uninitialized, style);
        return;
    }

    if info.type == .TYPE {
        render_type(builder, (cast(**Type_Info) item.value_pointer).*, false, style);
        return;
    }

    if info.type == .VOID {
        append(builder, "void");
        return;
    }

    if info.type == .CODE {
        append(builder, "Code");
        return;
    }

    if info.type == .OVERLOAD_SET {
        append(builder, "(procedure overloads)");
        return;
    }

    assert(false, "unsupported reflected value kind");
}

#scope_export
print_to_builder :: (builder: *String_Builder, format_string: string, args: ..Any) -> bool {
    arg: s64;
    cursor: s64;
    style := formatter_style();

    while cursor < format_string.count {
        byte := format_string[cursor];

        cursor += 1;

        if byte == 31 {
            append(builder, cast(u8) 37);
            continue;
        }

        if byte == 37 {
            if cursor < format_string.count && is_digit(format_string[cursor]) {
                end, index := parse_number(format_string, cursor);
                if index == 0 && end - cursor >= 2 {
                    cursor = end;
                    continue;
                }

                cursor = end;
                if index != 0 arg = index - 1;
            }

            if arg >= args.count {
                formatter_error(style, "format argument index is out of range");
                arg = 0;
                continue;
            }

            render_item(builder, args[arg], false, false, style);
            arg += 1;
        } else append(builder, byte);

        if builder.failed break;
    }

    return !builder.failed;
}

my_is_digit :: (c: s64) -> bool {
    return c >= 48 && c <= 57;
}

parse_number :: (s: string, cursor: s64) -> (end_cursor: s64, sum: s64) {
    assert(cursor >= 0 && cursor <= s.count);
    index := cursor;
    value: s64;

    while index < s.count && my_is_digit(s[index]) {
        digit := cast(s64) (s[index] - 48);
        if value > (0x7fffffffffffffff - digit) / 10 value = 0x7fffffffffffffff;
        else value = value * 10 + digit;
        index += 1;
    }

    return index, value;
}

sprint :: (format_string: string, args: ..Any) -> string {
    builder: String_Builder;
    defer free_buffers(*builder);
    if !print_to_builder(*builder, format_string, ..args) return "";
    return builder_to_string(*builder);
} @UseResult

print :: print_to_builder;

tprint :: (format_string: string, args: ..Any) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    if !print_to_builder(*builder, format_string, ..args) return "";

    push_allocator(temp);

    return builder_to_string(*builder);
} @UseResult

print_character :: (_c: u8) {
    write_string(string.{ 1, *_c });
}
