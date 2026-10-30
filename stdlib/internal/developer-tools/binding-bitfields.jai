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

// The accessors of one bitfield group, for programs that print records themselves. The names are checked against
// the global declarations of the active generator.
append_bitfield_helpers :: (builder: *String_Builder, bitfield: *Bitfield, scope: *Struct) {
    fragment: String_Builder;
    used: [..] Helper_Name;

    defer free_buffers(*fragment);

    defer {
        for entry: used free(entry.name);
        array_free(used);
    }

    valid := bitfield != null && scope != null;

    if valid {
        existing := binding_global_declarations(gp_current_generator());

        defer array_free(existing);
        valid = binding_bitfield_accessors(*fragment, bitfield, scope, existing, *used);
    }

    commit_type_fragment(builder, *fragment, valid);
}

#scope_module
#import "Basic";
#import "String";

// Prints one bitfield storage unit: the integer (or byte array) field, then its members in a comment.
binding_bitfield_unit :: (builder: *String_Builder, group: *Bitfield, scope: *Declaration, flags: Print_Decl_Flags, alignment: int, explicit_alignment := 0) -> bool {
    generator := gp_current_generator();
    if !generator || !group.type || !looks_like_valid_identifier(group.output_name) return false;

    append(builder, group.output_name);
    append(builder, ": ");
    print_type_to_builder(builder, group.type, scope);
    field_alignment := ifx explicit_alignment > 0 then explicit_alignment else group.explicit_field_alignment;
    if field_alignment > 0 print_to_builder(builder, " #align %", field_alignment);

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

binding_field_contains :: (field: *Declaration, record: *Struct) -> bool {
    if !field return false;
    type := strip_type(field.type, typedefs = true);
    return type != null && type.type_of_struct == record;
}

// A helper name already given out for a record, so that a second helper for the same record is told apart.
Helper_Name :: struct {
    name: string;
    scope: *Struct;
}

// The names of everything printed at global scope so far; an accessor never reuses one that is not a procedure.
binding_global_declarations :: (generator: *Generator) -> [..] *Declaration {
    result: [..] *Declaration;

    if !generator return result;

    for declaration: generator.flattened_global_scope_members array_add(*result, declaration);
    for declaration: generator.declarations_for_global_scope array_add(*result, declaration);

    return result;
}

binding_accessor_name_taken :: (name: string, scope: *Struct, existing: [] *Declaration, used: [] Helper_Name) -> bool {
    for declaration: existing if declaration && declaration.kind != .FUNCTION && declaration.output_name == name return true;
    for entry: used if entry.name == name && entry.scope == scope return true;

    return false;
}

// `0b1111_1111`: `width` set bits in groups of four.
binding_ones_literal :: (width: int) -> string {
    text: String_Builder;

    defer free_buffers(*text);
    append(*text, "0b");
    remaining := width;
    group := width % 4;

    if group == 0 group = 4;

    while remaining > 0 {
        for 1..min(group, remaining) append(*text, "1");

        remaining -= group;
        group = 4;
        if remaining > 0 append(*text, "_");
    }

    return builder_to_string(*text);
}

binding_unsigned_name :: (bytes: s64) -> string {
    if bytes == 1 return "u8";
    if bytes == 2 return "u16";
    if bytes == 4 return "u32";

    return "u64";
}

// What an accessor needs to know about the unit it reads and writes.
Bitfield_Accessor_Unit :: struct {
    record: string;
    field: string;
    storage: string;
    bytes: s64;
    bytes_array: bool;
    scope: *Struct;
}

// The accessors for every named member of a storage unit, as `set_<name>` and `get_<name>` after the record.
// A name another declaration or accessor already uses gets a numeric suffix, as other generated names do, on both
// procedures of the pair.
binding_bitfield_accessors :: (builder: *String_Builder, group: *Bitfield, scope: *Struct, existing: [] *Declaration, used: *[..] Helper_Name) -> bool {
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
    record := type_to_string(*(CType.{ type_of_struct = scope }), null);

    defer free(record);
    if !record.count return false;

    unit: Bitfield_Accessor_Unit;

    unit.record = record;
    unit.field = tprint("%%", access_prefix, group.output_name);
    unit.bytes = group.type.size;
    unit.bytes_array = group.type.array_element_type != null;
    unit.scope = scope;

    // A unit wider than a machine word has no integer to compute with.
    if unit.bytes < 1 || unit.bytes > 8 return true;

    storage := ifx unit.bytes_array then copy_string("u8") else type_to_string(group.type, cast(*Declaration) scope);

    defer free(storage);
    if !storage.count return true;

    unit.storage = storage;

    for member: group.members {
        if !member || !member.output_name.count || !looks_like_valid_identifier(member.output_name) continue;

        width := member.bitfield_width;
        shift := member.bit_offset_in_parent - group.bit_offset_in_parent;

        if width < 1 || width > 64 || shift < 0 || shift + width > unit.bytes * 8 continue;

        base_name := tprint("%%", name_prefix, member.output_name);
        getter := tprint("get_%", base_name);
        setter := tprint("set_%", base_name);

        for suffix: 1..existing.count + used.count + 2 {
            if !binding_accessor_name_taken(getter, scope, existing, used.*) && !binding_accessor_name_taken(setter, scope, existing, used.*) break;

            getter = tprint("get_%_%", base_name, suffix);
            setter = tprint("set_%_%", base_name, suffix);
        }

        if !binding_member_accessors(builder, unit, member, shift, getter, setter) continue;

        array_add(used, .{ copy_string(getter), scope });
        array_add(used, .{ copy_string(setter), scope });
    }

    return !builder.failed;
}

// One setter and getter. Returns false, printing nothing, for a member whose type has no name to compute with.
binding_member_accessors :: (builder: *String_Builder, unit: Bitfield_Accessor_Unit, member: *Declaration, shift: s64, getter: string, setter: string) -> bool {
    width := member.bitfield_width;
    base := strip_type(member.type, typedefs = true, enums = true);
    named := strip_type(member.type, typedefs = true);

    if !base || !named return false;

    is_bool := (base.flags & .BOOL) != 0;
    is_enum := named.type_of_enum != null;
    is_signed := (base.number_flags & .SIGNED) != 0 && !is_bool;
    flag := width == 1 && (is_bool || (!is_signed && !is_enum));
    value_type := ifx flag then copy_string("bool") else type_to_string(member.type, cast(*Declaration) unit.scope);

    defer free(value_type);
    if !value_type.count return false;

    integer := ifx is_enum then type_to_string(base, cast(*Declaration) unit.scope) else copy_string(value_type);

    defer free(integer);
    if !integer.count return false;

    mask := binding_ones_literal(width);

    defer free(mask);
    field := unit.field;
    storage := unit.storage;

    if unit.bytes_array {
        binding_array_accessors(builder, unit, member, shift, getter, setter, value_type, integer, value_type == "bool", is_enum, is_signed);
        return !builder.failed;
    }

    shifted_mask := ifx shift == 0 then mask else tprint("(% << %)", mask, shift);

    // The setter.
    print_to_builder(builder, "% :: (fields: *%, value: %) {\n", setter, unit.record, value_type);

    if flag {
        print_to_builder(builder, "    if value fields.% |= 1 << %;\n    else     fields.% &= ~cast(%)(1 << %);\n}\n", field, shift, field, storage, shift);
    } else {
        needs_cast := value_type != storage;
        value := ifx needs_cast then tprint("(cast,no_check(%) value)", storage) else "value";

        if shift != 0 value = tprint("(% << %)", value, shift);

        print_to_builder(builder, "    MASK0: % : %;\n    fields.% = (fields.% & ~MASK0) | (% & MASK0);\n}\n", storage, shifted_mask, field, field, value);
    }

    // The getter.
    print_to_builder(builder, "% :: (fields: %) -> % {\n", getter, unit.record, value_type);

    raw := ifx shift == 0 then tprint("(fields.% & %)", field, mask) else tprint("((fields.% & %) >> %)", field, shifted_mask, shift);
    integer_bits := base.size * 8;

    if flag {
        print_to_builder(builder, "    return (fields.% & (1 << %)) != 0;\n}\n", field, shift);
    } else if is_bool {
        print_to_builder(builder, "    return % != 0;\n}\n", raw);
    } else if is_signed && width < integer_bits {
        // Move the member's top bit to the integer's, then shift back with sign extension.
        lifted := binding_unsigned_name(base.size);
        moved := tprint("(cast,no_check(%) (fields.%%)) << %", lifted, field, ifx shift == 0 then "" else tprint(" >> %", shift), integer_bits - width);
        extended := tprint("((cast,no_check(%) (%)) >> %)", integer, moved, integer_bits - width);

        print_to_builder(builder, "    return %;\n}\n", ifx value_type != integer then tprint("(cast,no_check(%) %)", value_type, extended) else extended);
    } else {
        print_to_builder(builder, "    return %;\n}\n", ifx value_type != storage then tprint("(cast,no_check(%) %)", value_type, raw) else raw);
    }

    append(builder, "\n");

    return !builder.failed;
}

// Units that are byte arrays (packed records) are read and written through a 64-bit copy of their bytes.
binding_array_accessors :: (builder: *String_Builder, unit: Bitfield_Accessor_Unit, member: *Declaration, shift: s64, getter: string, setter: string, value_type: string, integer: string, is_bool: bool, is_enum: bool, is_signed: bool) {
    width := member.bitfield_width;
    mask := ifx width >= 64 then "(~cast(u64) 0)" else tprint("(cast(u64) %)", (cast(u64) 1 << cast(u64) width) - 1);
    last := unit.bytes - 1;

    print_to_builder(builder, "% :: (fields: *%, value: %) {\n    bits: u64;\n", setter, unit.record, value_type);
    print_to_builder(builder, "    for i: 0..% bits |= (cast(u64) fields.%[i]) << (cast(u64) (8 * i));\n", last, unit.field);
    print_to_builder(builder, "    bits = (bits & ~(% << %)) | (((cast,no_check(u64) value) & %) << %);\n", mask, shift, mask, shift);
    print_to_builder(builder, "    for i: 0..% fields.%[i] = cast,trunc(u8) (bits >> (cast(u64) (8 * i)));\n}\n", last, unit.field);

    print_to_builder(builder, "% :: (fields: %) -> % {\n    bits: u64;\n", getter, unit.record, value_type);
    print_to_builder(builder, "    for i: 0..% bits |= (cast(u64) fields.%[i]) << (cast(u64) (8 * i));\n", last, unit.field);
    print_to_builder(builder, "    value := (bits >> %) & %;\n", shift, mask);

    if is_signed && width < 64 print_to_builder(builder, "    if (value & ((cast(u64) 1) << %)) != 0 value |= ~%;\n", width - 1, mask);

    if is_bool append(builder, "    return value != 0;\n}\n");
    else if is_enum print_to_builder(builder, "    return cast(%) cast,trunc(%) value;\n}\n", value_type, integer);
    else print_to_builder(builder, "    return cast,trunc(%) value;\n}\n", value_type);

    append(builder, "\n");
}
