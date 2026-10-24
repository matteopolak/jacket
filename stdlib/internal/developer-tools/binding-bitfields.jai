maybe_add_to_bitfield :: (using status: *Bitfield_Status, decl: *Declaration) -> *Declaration {
    assert(status != null && decl != null);
    storage := strip_type(decl.type, typedefs = true, enums = true);
    primitive := storage && (storage.number_flags != 0 || (storage.flags & .BOOL) != 0) && !(storage.number_flags & .FLOAT);
    eligible := primitive && (storage.size == 1 || storage.size == 2 || storage.size == 4 || storage.size == 8);

    if decl.bitfield_width < 0 || !eligible {
        if bitfield finish_bitfield(bitfield, 0);
        bitfield = null;
        first_decl = null;
        return decl;
    }

    if decl.bitfield_width == 0 return null;

    options := gp_current_options();

    assert(options != null && parent != null, "Bindings_Generator: bitfield grouping requires active options/parent");

    if bitfield && options.os == .WINDOWS && !is_same_type(first_decl.type, decl.type) {
        finish_bitfield(bitfield, 0);
        bitfield = null;
        first_decl = null;
    }

    first := bitfield == null;
    if first {
        session := context._bindings_generator_session;
        if session bitfield = GM_Sessions.new_owned(session, Bitfield);
        else bitfield = New(Bitfield);

        bitfield.kind = .BITFIELD;
        bitfield.name = decl.name;
        bitfield.output_name = "__bitfield";
        bitfield.location = decl.location;
        bitfield.parent = cast(*Declaration) parent;
        bitfield.bit_offset_in_parent = decl.bit_offset_in_parent;
        bitfield.type = decl.type;
        if options.generate_bitfield_helpers bitfield.flags |= .GENERATE_HELPERS;

        first_decl = decl;
    }

    session := context._bindings_generator_session;
    if session GM_Sessions.append_owned(session, *bitfield.members, decl);
    else array_add(*bitfield.members, decl);
    if first return cast(*Declaration) bitfield;

    return null;
}

finish_bitfield :: (using bitfield: *Bitfield, index: int) {
    assert(bitfield != null && members.count > 0 && bit_offset_in_parent >= 0);
    largest: *CType;
    last_end := bit_offset_in_parent;

    for member: members {
        assert(member != null && member.bitfield_width > 0 && member.bitfield_width <= 64);
        storage := strip_type(member.type, typedefs = true, enums = true);

        assert(storage != null && storage.size > 0 && storage.size <= 8);
        if !largest || storage.size > largest.size largest = storage;

        assert(member.bit_offset_in_parent >= bit_offset_in_parent && member.bit_offset_in_parent <= 0x7fffffffffffffff - member.bitfield_width);
        last_end = max(last_end, member.bit_offset_in_parent + member.bitfield_width);
    }

    span_bits := last_end - bit_offset_in_parent;

    assert(span_bits > 0 && span_bits <= 0x7fffffffffffffff - 7);
    bytes := (span_bits + 7) / 8;
    options := gp_current_options();
    generator := gp_current_generator();

    assert(options != null && generator != null);
    type = largest;

    if bytes <= largest.size {
        if options.os != .WINDOWS && bytes < largest.size {
            if bytes <= 1 type = generator.type_def_u8;
            else if bytes <= 2 type = generator.type_def_u16;
            else if bytes <= 4 type = generator.type_def_u32;
            else type = generator.type_def_u64;
        }
    } else {
        type = binding_new_type();
        type.array_element_type = generator.type_def_u8;
        type.array_element_count = bytes;
        type.size = bytes;
        type.alignment = 1;
    }

    assert(type != null);
    alignment_type = ifx type.alignment < largest.alignment then largest else null;
}

maybe_unwrap_bitfield_declaration :: (decl: *Declaration) -> *Declaration {
    if !decl || decl.kind != .BITFIELD return decl;

    group := cast(*Bitfield) decl;
    if group.members.count != 1 return decl;

    member := group.members[0];

    assert(member != null);
    member.type = group.type;

    return member;
}

append_parent_field_path :: (builder: *String_Builder, decl: *Declaration, current_scope: *Declaration, separator := ".") -> bool {
    names: [..] string;
    visited: [..] *Declaration;

    defer array_free(names);
    defer array_free(visited);
    node := decl;
    while node && node != current_scope {
        assert(node.kind == .STRUCT && (cast(*Struct) node).flags & .IS_ANON, "Bindings_Generator: field path requires anonymous record ancestors");

        if array_find(visited, node) {
            log_error("Bindings_Generator: cyclic anonymous record containment\n");
            builder.failed = true;
            return false;
        }

        array_add(*visited, node);
        assert(node.parent && node.parent.kind == .STRUCT, "Bindings_Generator: anonymous record requires containing record");
        parent := cast(*Struct) node.parent;
        parent_type: CType;
        parent_type.type_of_struct = parent;
        info, _generated := find_or_create_extra_info(*parent_type, node.parent);
        if !info || info == *struct_extra_info_unknown {
            builder.failed = true;
            return false;
        }

        containing: *Declaration;
        for field: info.modified_declarations if binding_field_contains(field, cast(*Struct) node) containing = field;

        for overlay: parent.overlays {
            if binding_field_contains(overlay.declaration, cast(*Struct) node) containing = overlay.declaration;
            else if binding_field_contains(overlay.place, cast(*Struct) node) containing = overlay.place;
        }

        assert(containing != null, "Bindings_Generator: missing anonymous containing declaration");
        array_add(*names, containing.output_name);
        node = node.parent;
    }

    for index: 0..names.count - 1 {
        name := names[names.count - 1 - index];
        if !name continue;
        append(builder, name);
        append(builder, separator);
    }

    return visited.count > 0;
}

append_bitfield_helpers :: (builder: *String_Builder, bitfield: *Bitfield, scope: *Struct) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := bitfield != null && scope != null;
    if valid valid = binding_bitfield_accessors(*fragment, bitfield, scope);

    commit_type_fragment(builder, *fragment, valid);
}

#scope_module
#import "Basic";
#import "String";

// Prints one bitfield storage unit: the integer (or byte array) field, then its members in a comment.
binding_bitfield_unit :: (builder: *String_Builder, group: *Bitfield, scope: *Declaration, flags: Print_Decl_Flags, alignment: int) -> bool {
    generator := gp_current_generator();
    if !generator || !group.type || !looks_like_valid_identifier(group.output_name) return false;

    append(builder, group.output_name);
    append(builder, ": ");
    print_type_to_builder(builder, group.type, scope);
    if group.explicit_field_alignment > 0 print_to_builder(builder, " #align %", group.explicit_field_alignment);

    append(builder, ";\n");
    append_indentation(builder);
    append(builder, "/*\n");

    generator.tab_depth += 1;

    for member: group.members {
        if !member || !member.type return false;

        prefix: String_Builder;

        defer free_buffers(*prefix);
        append_prefix_comment(*prefix, member.comment, false);
        lines := split(builder_to_string(*prefix), "\n");

        defer array_free(lines);

        for line: lines if line.count {
            safe := binding_nested_comment_text(line);

            defer free(safe);
            append_indentation(builder);
            append(builder, safe);
            append(builder, "\n");
        }

        append_indentation(builder);
        append(builder, ifx member.output_name.count then member.output_name else "_");
        append(builder, ": ");
        print_type_to_builder(builder, member.type, scope);
        print_to_builder(builder, "; /* % bits */", member.bitfield_width);

        suffix: String_Builder;

        defer free_buffers(*suffix);
        append_postfix_comment(*suffix, member.comment);
        text := builder_to_string(*suffix);

        defer free(text);
        safe := binding_nested_comment_text(text);

        defer free(safe);
        append(builder, safe);
        append(builder, "\n");
    }

    generator.tab_depth -= 1;
    append_indentation(builder);
    append(builder, "*/\n");

    return !builder.failed;
}

// Source comments are copied into a block comment, so their own comment markers must nest.
// Text whose markers do not pair up has them broken apart instead.
binding_nested_comment_text :: (text: string) -> string {
    depth := 0;
    balanced := true;

    for index: 0..text.count - 2 {
        if text[index] == #char "/" && text[index + 1] == #char "*" depth += 1;
        else if text[index] == #char "*" && text[index + 1] == #char "/" {
            depth -= 1;
            if depth < 0 balanced = false;
        }
    }

    if depth != 0 balanced = false;
    if balanced return copy_string(text);

    result: String_Builder;

    defer free_buffers(*result);
    index := 0;

    while index < text.count {
        if index + 1 < text.count && ((text[index] == #char "/" && text[index + 1] == #char "*") || (text[index] == #char "*" && text[index + 1] == #char "/")) {
            append(*result, text[index]);
            append(*result, " ");
            index += 1;
            continue;
        }

        append(*result, text[index]);
        index += 1;
    }

    return builder_to_string(*result);
}

// The accessors for every named member of a storage unit. A name another declaration or accessor
// already uses gets a numeric suffix, as other generated names do, on both procedures of the pair.
append_layout_unit_accessors :: (builder: *String_Builder, group: *Bitfield, scope: *Struct, existing: [] *Declaration, used: *[..] string) {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    valid := group != null && scope != null;
    if valid valid = binding_layout_unit_accessors(*fragment, group, scope, existing, used);

    commit_type_fragment(builder, *fragment, valid);
}

binding_field_contains :: (field: *Declaration, record: *Struct) -> bool {
    if !field return false;
    type := strip_type(field.type, typedefs = true);
    return type != null && type.type_of_struct == record;
}

binding_bitfield_accessors :: (builder: *String_Builder, group: *Bitfield, scope: *Struct) -> bool {
    if !group.type || !looks_like_valid_identifier(group.output_name) return false;

    path: String_Builder;

    defer free_buffers(*path);
    append_parent_field_path(*path, group.parent, cast(*Declaration) scope);
    if path.failed return false;

    access_prefix := builder_to_string(*path);

    defer free(access_prefix);
    append_parent_field_path(*path, group.parent, cast(*Declaration) scope, "_");
    if path.failed return false;

    name_prefix := builder_to_string(*path);

    defer free(name_prefix);
    scope_name := type_to_string(*(CType.{ type_of_struct = scope }), null);

    defer free(scope_name);
    if !scope_name return false;

    array := group.type.array_element_type != null;

    for member: group.members {
        if !member || !member.output_name continue;
        if !looks_like_valid_identifier(member.output_name) || member.bitfield_width <= 0 || member.bitfield_width > 64 return false;

        start := member.bit_offset_in_parent - group.bit_offset_in_parent;
        if start < 0 || start > group.type.size * 8 - member.bitfield_width return false;

        value_type := ifx member.bitfield_width == 1 then "bool" else type_to_string(member.type, cast(*Declaration) scope);
        if !value_type return false;

        defer if member.bitfield_width != 1 free(value_type);
        print_to_builder(builder, "get_%% :: (fields: %) -> % {\n bits: u64;\n", name_prefix, member.output_name, scope_name, value_type);

        for bit: 0..member.bitfield_width - 1 {
            position := start + bit;
            if array print_to_builder(builder, " bits |= ((cast(u64) fields.%%[%] >> %) & 1) << %;\n", access_prefix, group.output_name, position / 8, position % 8, bit);
            else print_to_builder(builder, " bits |= ((cast(u64) fields.%% >> %) & 1) << %;\n", access_prefix, group.output_name, position, bit);
        }

        if member.bitfield_width == 1 append(builder, " return bits != 0;\n}\n");
        else print_to_builder(builder, " return cast,trunc(%) bits;\n}\n", value_type);

        print_to_builder(builder, "set_%% :: (fields: *%, value: %) {\n bits := cast(u64) value;\n", name_prefix, member.output_name, scope_name, value_type);

        for bit: 0..member.bitfield_width - 1 {
            position := start + bit;
            if array print_to_builder(builder, " fields.%%[%] = cast,trunc(u8) ((cast(u64) fields.%%[%] & ~(cast(u64) 1 << %)) | (((bits >> %) & 1) << %));\n", access_prefix, group.output_name, position / 8, access_prefix, group.output_name, position / 8, position % 8, bit, position % 8);
            else {
                storage_type := type_to_string(group.type, cast(*Declaration) scope);
                defer free(storage_type);
                if !storage_type return false;
                print_to_builder(builder, " fields.%% = cast,trunc(%) ((cast(u64) fields.%% & ~(cast(u64) 1 << %)) | (((bits >> %) & 1) << %));\n", access_prefix, group.output_name, storage_type, access_prefix, group.output_name, position, bit, position);
            }
        }

        append(builder, "}\n");
    }

    return !builder.failed;
}

binding_accessor_name_taken :: (name: string, existing: [] *Declaration, used: [] string) -> bool {
    for declaration: existing if declaration && declaration.output_name == name return true;
    for taken: used if taken == name return true;

    return false;
}

binding_mask_text :: (width: int) -> string {
    if width >= 64 return "(~cast(u64) 0)";

    return tprint("(cast(u64) %)", (cast(u64) 1 << cast(u64) width) - 1);
}

binding_layout_unit_accessors :: (builder: *String_Builder, group: *Bitfield, scope: *Struct, existing: [] *Declaration, used: *[..] string) -> bool { // jailint: allow(cognitive_complexity)
    if !group.type || !looks_like_valid_identifier(group.output_name) return false;

    path: String_Builder;

    defer free_buffers(*path);
    append_parent_field_path(*path, group.parent, cast(*Declaration) scope);
    if path.failed return false;

    access_prefix := builder_to_string(*path);

    defer free(access_prefix);
    append_parent_field_path(*path, group.parent, cast(*Declaration) scope, "_");
    if path.failed return false;

    name_prefix := builder_to_string(*path);

    defer free(name_prefix);
    scope_name := type_to_string(*(CType.{ type_of_struct = scope }), null);

    defer free(scope_name);
    if !scope_name.count || !looks_like_valid_identifier(scope.output_name) return false;

    storage_name := type_to_string(group.type, cast(*Declaration) scope);

    defer free(storage_name);
    array := group.type.array_element_type != null;
    storage_bytes := group.type.size;
    field := tprint("s.%%", access_prefix, group.output_name);

    if storage_bytes < 1 || storage_bytes > 8 || (!array && !storage_name.count) return true;

    for member: group.members {
        if !member || !member.output_name.count || !looks_like_valid_identifier(member.output_name) continue;

        width := member.bitfield_width;
        shift := member.bit_offset_in_parent - group.bit_offset_in_parent;

        if width < 1 || width > 64 || shift < 0 || shift + width > storage_bytes * 8 continue;

        value_type := type_to_string(member.type, cast(*Declaration) scope);

        defer free(value_type);
        if !value_type.count continue;

        base := strip_type(member.type, typedefs = true, enums = true);
        named := strip_type(member.type, typedefs = true);

        if !base || !named continue;

        is_bool := (base.flags & .BOOL) != 0;
        is_enum := named.type_of_enum != null;
        is_signed := (base.number_flags & .SIGNED) != 0;
        integer_name := ifx is_enum then type_to_string(base, cast(*Declaration) scope) else copy_string(value_type);

        defer free(integer_name);
        if !integer_name.count continue;

        base_name := tprint("%_%%", scope.output_name, name_prefix, member.output_name);
        getter := base_name;
        setter := tprint("%_set_%%", scope.output_name, name_prefix, member.output_name);

        for suffix: 1..existing.count + used.count + 2 {
            if !binding_accessor_name_taken(getter, existing, used.*) && !binding_accessor_name_taken(setter, existing, used.*) break;

            getter = tprint("%_%", base_name, suffix);
            setter = tprint("%_set_%%_%", scope.output_name, name_prefix, member.output_name, suffix);
        }

        array_add(used, copy_string(getter));
        array_add(used, copy_string(setter));
        mask := binding_mask_text(width);
        load := ifx array then "bits" else tprint("(cast(u64) %)", field);

        print_to_builder(builder, "% :: inline (s: %) -> % {\n", getter, scope_name, value_type);
        if array print_to_builder(builder, " bits: u64;\n for i: 0..% bits |= (cast(u64) %[i]) << (cast(u64) (8 * i));\n", storage_bytes - 1, field);

        print_to_builder(builder, " value := (% >> %) & %;\n", load, shift, mask);
        if is_signed && width < 64 print_to_builder(builder, " if (value & ((cast(u64) 1) << %)) != 0 value |= ~%;\n", width - 1, mask);

        if is_bool append(builder, " return value != 0;\n}\n");
        else if is_enum print_to_builder(builder, " return cast(%) cast,trunc(%) value;\n}\n", value_type, integer_name);
        else print_to_builder(builder, " return cast,trunc(%) value;\n}\n", value_type);

        print_to_builder(builder, "% :: inline (s: *%, value: %) {\n", setter, scope_name, value_type);
        if array print_to_builder(builder, " bits: u64;\n for i: 0..% bits |= (cast(u64) %[i]) << (cast(u64) (8 * i));\n", storage_bytes - 1, field);

        replaced := tprint("((% & ~(% << %)) | (((cast,no_check(u64) value) & %) << %))", ifx array then "bits" else tprint("(cast(u64) %)", field), mask, shift, mask, shift);

        if array {
            print_to_builder(builder, " bits = %;\n for i: 0..% %[i] = cast,trunc(u8) (bits >> (cast(u64) (8 * i)));\n}\n", replaced, storage_bytes - 1, field);
        } else print_to_builder(builder, " % = cast,trunc(%) %;\n}\n", field, storage_name, replaced);
    }

    return !builder.failed;
}
