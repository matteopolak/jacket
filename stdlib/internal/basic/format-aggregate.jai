isFormatter :: (info: *Type_Info_Struct) -> *Type_Info_Struct_Member {
    for *member: info.members {
        if member.type == type_info(Formatter) && (member.flags & .AS) != 0 return member;
    }

    return null;
}

append_indentation :: (builder: *String_Builder, format: *FormatStruct) {
    style := formatter_style();
    formatter_repeat(builder, 32, cast(s64) style.indentation_depth * cast(s64) format.indentation_width);
}

append_struct_info :: (builder: *String_Builder, info: *Type_Info_Struct, base_pointer: *void, format: *FormatStruct = null) {
    style := formatter_style();
    selected := style.default_format_struct;

    if format selected = format.*;

    render_struct(builder, info, base_pointer, selected, style);
}

print_array :: (builder: *String_Builder, any: Any, info_array: *Type_Info_Array, format: FormatArray, may_be_uninitialized := false) {
    render_array(builder, any, info_array, format, may_be_uninitialized, formatter_style());
}

#scope_module
render_struct_format :: (builder: *String_Builder, format: FormatStruct, style: *Print_Style) {
    value := format.value;
    if !value.type || !value.value_pointer {
        append(builder, "(null Any)");
        return;
    }

    if value.type.type == .POINTER {
        value.type = (cast(*Type_Info_Pointer) value.type).pointer_to;
        value.value_pointer = (cast(**void) value.value_pointer).*;
        if !value.value_pointer {
            append(builder, "null");
            return;
        }
    }

    if !value.type || value.type.type != .STRUCT {
        formatter_error(style, "FormatStruct requires a struct or pointer to struct");
        append(builder, "error");
        return;
    }

    render_struct(builder, cast(*Type_Info_Struct) value.type, value.value_pointer, format, style);
}

render_struct :: (builder: *String_Builder, info: *Type_Info_Struct, base: *void, format: FormatStruct, style: *Print_Style) {
    assert(base != null);

    if (info.textual_flags & .UNION_IS_TAGGED) != 0 {
        render_tagged_union(builder, info, base, style);
        return;
    }

    // A union's members share storage, so none of them can be read without knowing which is live.
    if (info.textual_flags & .UNION) != 0 {
        append(builder, "(untagged union)");
        return;
    }

    count: s64;
    for *member: info.members if printable_member(member) count += 1;

    long_form := format.use_long_form_if_more_than_this_many_members < 0 || count > format.use_long_form_if_more_than_this_many_members;
    lines := long_form && format.use_newlines_if_long_form && count > 0;

    if format.draw_type_name render_type(builder, info, false, style);

    append(builder, format.begin_string);
    old_depth := style.indentation_depth;
    old_flags := style.state_flags;

    style.state_flags = .INSIDE_DATA_STRUCTURE_PRINTING;

    defer {
        style.indentation_depth = old_depth;
        style.state_flags = old_flags;
    }

    if lines {
        append(builder, cast(u8) 10);
        style.indentation_depth += 1;
    }

    printed: s64;

    for *member: info.members {
        if !printable_member(member) continue;

        if long_form {
            if lines formatter_repeat(builder, 32, cast(s64) style.indentation_depth * cast(s64) format.indentation_width);
            append(builder, member.name);
            append(builder, format.separator_between_name_and_value);
        } else if printed > 0 append(builder, format.short_form_separator_between_fields);

        value := Any.{ member.type, cast(*u8) base + member.offset_in_bytes };

        render_item(builder, value, true, false, style);
        if long_form append(builder, format.long_form_separator_between_fields);
        if lines append(builder, cast(u8) 10);

        printed += 1;
        if builder.failed break;
    }

    style.indentation_depth = old_depth;
    if lines formatter_repeat(builder, 32, cast(s64) old_depth * cast(s64) format.indentation_width);

    append(builder, format.end_string);
}

render_array :: (builder: *String_Builder, any: Any, info: *Type_Info_Array, format: FormatArray, may_be_uninitialized: bool, style: *Print_Style) {
    assert(any.value_pointer != null && info.element_type != null);
    count := info.array_count;
    data := cast(*u8) any.value_pointer;

    if info.array_type != .FIXED {
        view := cast(*Array_View_64) any.value_pointer;
        count = view.count;
        data = view.data;
    }

    assert(count >= 0 && (data != null || count == 0), "invalid array representation");
    emitted := count;
    if format.stop_printing_after_this_many_elements >= 0 emitted = min(emitted, format.stop_printing_after_this_many_elements);

    old_flags := style.state_flags;

    style.state_flags = .INSIDE_DATA_STRUCTURE_PRINTING;
    defer style.state_flags = old_flags;
    append(builder, format.begin_string);

    for index: 0..emitted - 1 {
        item := Any.{ info.element_type, data + index * info.element_type.runtime_size };
        render_item(builder, item, true, may_be_uninitialized, style);
        if index + 1 < emitted || format.draw_separator_after_last_element append(builder, format.separator);
        if builder.failed break;
    }

    if count > emitted {
        append(builder, format.printing_stopped_early_string);
    }

    append(builder, format.end_string);
}

#scope_file
render_tagged_union :: (builder: *String_Builder, info: *Type_Info_Struct, base: *void, style: *Print_Style) {
    assert(info.members.count > 0, "tagged union requires a discriminator");
    tag := info.members[0];
    tag_type := formatter_base_type(tag.type);

    assert(tag_type != null);
    if tag_type.type == .ENUM tag_type = (cast(*Type_Info_Enum) tag_type).internal_type;

    assert(tag_type != null && (tag_type.type == .INTEGER || tag_type.type == .TYPE), "unsupported tagged union discriminator");
    bits := get_u64_from_void_pointer(cast(*u8) base + tag.offset_in_bytes, tag_type.runtime_size);
    mask: u64 = 0xffffffffffffffff;

    if tag_type.runtime_size < 8 mask = ((cast(u64) 1) << (tag_type.runtime_size * 8)) - 1;

    selected: s64 = -1;

    for binding: info.tagged_union_bindings {
        if (binding.constant_value & mask) == bits {
            selected = binding.member_index;
            break;
        }
    }

    assert(selected > 0 && selected < info.members.count, "tagged union discriminator has no member binding");
    member := info.members[selected];

    append(builder, "{");
    render_item(builder, Any.{ tag.type, cast(*u8) base + tag.offset_in_bytes }, false, false, style);
    append(builder, ", ");
    render_item(builder, Any.{ member.type, cast(*u8) base + member.offset_in_bytes }, false, false, style);
    append(builder, "}");
}

printable_member :: (member: *Type_Info_Struct_Member) -> bool {
    assert(member.type != null);
    return member.type.runtime_size > 0 && (member.flags & (.CONSTANT | .IMPORTED | .OVERLAY)) == 0;
}
