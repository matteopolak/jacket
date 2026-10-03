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
        else print_to_builder(builder, " return cast,truncate(%) bits;\n}\n", value_type);

        print_to_builder(builder, "set_%% :: (fields: *%, value: %) {\n bits := cast(u64) value;\n", name_prefix, member.output_name, scope_name, value_type);

        for bit: 0..member.bitfield_width - 1 {
            position := start + bit;
            if array print_to_builder(builder, " fields.%%[%] = cast,truncate(u8) ((cast(u64) fields.%%[%] & ~(cast(u64) 1 << %)) | (((bits >> %) & 1) << %));\n", access_prefix, group.output_name, position / 8, access_prefix, group.output_name, position / 8, position % 8, bit, position % 8);
            else {
                storage_type := type_to_string(group.type, cast(*Declaration) scope);
                defer free(storage_type);
                if !storage_type return false;
                print_to_builder(builder, " fields.%% = cast,truncate(%) ((cast(u64) fields.%% & ~(cast(u64) 1 << %)) | (((bits >> %) & 1) << %));\n", access_prefix, group.output_name, storage_type, access_prefix, group.output_name, position, bit, position);
            }
        }

        append(builder, "}\n");
    }

    return !builder.failed;
}
