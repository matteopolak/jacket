#scope_module
#import "Basic";
#import "String";
#load "generator-vtable-abi.jai";
#load "generator-bitfield-layout.jai";

gm_generated_member :: (session: *GM_Sessions.Session, record: *Struct, name: string, type: *CType, offset: int) -> *Declaration {
    member := GM_Sessions.new_owned(session, Declaration);
    member.kind = .DECLARATION;
    member.parent = cast(*Declaration) record;
    member.name = GM_Sessions.copy_text(session, name);
    member.output_name = GM_Sessions.copy_text(session, name);
    member.type = type;
    member.bit_offset_in_parent = offset;
    member.field_alignment = type.alignment;

    return member;
}

// A slot is named for its method, with a numeric suffix when an earlier slot has the name.
// Destructor slots and names that are not identifiers keep a positional name.
gm_vtable_slot_name :: (function: *Function, index: int, table: *Struct) -> string {
    positional := tprint("slot_%", index);
    if function.type && function.type.type_of_function && function.type.type_of_function.flags & .DESTRUCTOR return positional;

    base := ifx function.output_name.count then function.output_name else function.name;
    if begins_with(base, "operator") base = replace_operator_name(base);

    base = avoid_reserved_names(base);
    if !looks_like_valid_identifier(base) return positional;

    name := base;

    for suffix: 1..table.declarations.count + 1 {
        taken := false;
        for member: table.declarations if member && member.output_name == name taken = true;
        if !taken break;

        name = tprint("%_%", base, suffix);
    }

    return name;
}

gm_metadata_byte_storage :: (bytes: s64) -> *CType {
    session := gm_current_session();
    generator := gm_active_generator();

    if !generator.type_def_u8 {
        generator.type_def_u8 = GM_Sessions.new_owned(session, CType);
        generator.type_def_u8.size = 1;
        generator.type_def_u8.alignment = 1;
        generator.type_def_u8.number_flags = ._8BIT;
        generator.type_def_u8.flags = .SHARED;
    }

    type := GM_Sessions.new_owned(session, CType);
    type.array_element_type = generator.type_def_u8;
    type.array_element_count = bytes;
    type.size = bytes;
    type.alignment = 1;

    return type;
}

gm_empty_record_storage :: (record: *Struct, info: *Struct_Extra_Info) {
    if record.type.size <= 0 || info.vtable_struct return;
    for field: info.modified_declarations if field && (field.kind == .DECLARATION || field.kind == .BITFIELD) && field.storage_class != .Static && field.type && field.type.size > 0 return;

    session := gm_current_session();
    name: string;

    for suffix: 0..record.declarations.count * 3 {
        candidate := tprint("__storage_%", suffix);
        taken := false;

        for member: record.declarations if member && (member.name == candidate || member.output_name == candidate || member.outputted_jai_name == candidate) taken = true;

        if !taken {
            name = GM_Sessions.copy_text(session, candidate);
            break;
        }
    }

    info.padding_declaration = gm_generated_member(session, record, name, gm_metadata_byte_storage(record.type.size), 0);
    GM_Sessions.append_movable_owned(session, *info.modified_declarations, info.padding_declaration);
}

gm_metadata_cache :: (record: *Struct, info: *Struct_Extra_Info) -> *Struct_Extra_Info {
    generator := gm_active_generator();
    result := GM_Table.table_add(*generator.extra_infos, record, info.*);

    info.* = .{};

    // Hash-table growth relocates values. Repair every retained parent-info link.
    for *entry: generator.extra_infos.entries {
        if entry.hash < GM_Table.FIRST_VALID_HASH continue;
        if entry.value.vtable_parent entry.value.vtable_parent_info = GM_Table.table_find_pointer(*generator.extra_infos, entry.value.vtable_parent);
    }

    return result;
}

gm_declaration_cursor :: (declaration: *Declaration) -> CXCursor {
    generator := gm_active_generator();
    for entry: generator.declarations_by_cursor.entries {
        if entry.hash >= GM_Table.FIRST_VALID_HASH && entry.value == declaration return entry.key;
    }

    return clang_getNullCursor();
}

gm_native_override :: (function: *Function, inherited: *Function) -> bool {
    cursor := gm_declaration_cursor(cast(*Declaration) function);
    base := gm_declaration_cursor(cast(*Declaration) inherited);

    if clang_Cursor_isNull(cursor) != 0 || clang_Cursor_isNull(base) != 0 {
        gm_construction_error("virtual override metadata requires canonical native function cursors");
        return false;
    }

    overridden: *CXCursor;
    count: u32;

    clang_getOverriddenCursors(cursor, *overridden, *count);
    defer if overridden clang_disposeOverriddenCursors(overridden);
    for i: 0..cast(s64) count - 1 if clang_equalCursors(clang_getCanonicalCursor(overridden[i]), clang_getCanonicalCursor(base)) != 0 return true;

    return false;
}

gm_polymorphic_field_layout :: (record: *Struct, info: *Struct_Extra_Info) -> bool {
    session := gm_current_session();
    output: [..] *Declaration;
    position: s64;

    for field: info.modified_declarations {
        if !field || field.kind != .DECLARATION || field.storage_class == .Static || (field.decl_flags & .IS_METADATA) != 0 {
            GM_Sessions.append_movable_owned(session, *output, field);
            continue;
        }

        alignment, _ := gm_alignment(field.type, field);
        if !field.type || field.type.size < 0 || field.bitfield_width >= 0 || field.bit_offset_in_parent < 0 || field.bit_offset_in_parent % 8 != 0 ||
           alignment <= 0 || (alignment & (alignment - 1)) != 0 || position > 0x7fffffffffffffff - (alignment - 1) {
            gm_construction_error("polymorphic field layout needs supported byte-aligned complete fields");
            return false;
        }

        offset := field.bit_offset_in_parent / 8;
        natural_offset := (position + alignment - 1) & ~(alignment - 1);

        if offset < natural_offset || offset > record.type.size || field.type.size > record.type.size - offset {
            gm_construction_error("polymorphic field layout overlaps or exceeds native record bounds");
            return false;
        }

        if offset > natural_offset {
            type := gm_metadata_byte_storage(offset - position);
            name: string;

            for suffix: 0..info.modified_declarations.count * 3 {
                candidate := tprint("__padding_%_%", position, suffix);
                taken := false;

                for member: info.modified_declarations if member && (member.name == candidate || member.output_name == candidate || member.outputted_jai_name == candidate) taken = true;

                if !taken {
                    name = GM_Sessions.copy_text(session, candidate);
                    break;
                }
            }

            padding := gm_generated_member(session, record, name, type, position * 8);
            if !info.padding_declaration info.padding_declaration = padding;

            GM_Sessions.append_movable_owned(session, *output, padding);
        }

        GM_Sessions.append_movable_owned(session, *output, field);
        position = offset + field.type.size;
    }

    array_free(info.modified_declarations);
    info.modified_declarations = output;
    output = .{};

    return true;
}

gm_metadata_virtual_members :: (record: *Struct) -> bool {
    cursor := gm_declaration_cursor(cast(*Declaration) record);
    if clang_Cursor_isNull(cursor) != 0 {
        if record.virtual_functions.count == 0 return true;
        gm_construction_error("virtual metadata requires the canonical native record cursor");
        return false;
    }

    generator := gm_active_generator();
    definition := clang_getCursorDefinition(cursor);

    if clang_Cursor_isNull(definition) == 0 cursor = definition;

    children := native_children(cursor);

    defer array_free(children);

    for child: children {
        if child.kind != .CXXMethod && child.kind != .ConversionFunction && child.kind != .Destructor continue;
        if clang_CXXMethod_isVirtual(child) == 0 continue;

        found, declaration := GM_Table.table_find(*generator.declarations_by_cursor, clang_getCanonicalCursor(child));
        if found && declaration && declaration.kind == .FUNCTION {
            function := cast(*Function) declaration;
            if (function.flags & .VIRTUAL) != 0 && array_find(record.virtual_functions, function) continue;
        }

        gm_construction_error("record metadata contains an unrepresented native virtual member");

        return false;
    }

    return true;
}

gm_record_metadata :: (record: *Struct, depth := 0) -> *Struct_Extra_Info { // jailint: allow(cognitive_complexity)
    generator := gm_active_generator();
    session := gm_current_session();

    if !generator || !session return null;

    if !record || !record.type || record.type.type_of_struct != record {
        gm_construction_error("record metadata lacks its canonical type backlink");
        return null;
    }

    cached := GM_Table.table_find_pointer(*generator.extra_infos, record);
    if cached return cached;

    if depth >= 64 {
        gm_construction_error("cyclic or excessively deep inheritance metadata");
        return null;
    }

    if record.parents.count < 0 || record.virtual_functions.count < 0 || record.declarations.count < 0 ||
       (record.parents.count > 0 && !record.parents.data) || (record.virtual_functions.count > 0 && !record.virtual_functions.data) || (record.declarations.count > 0 && !record.declarations.data) {
        gm_construction_error("malformed record metadata member storage");
        return null;
    }

    if record.declarations.count > 0x7fffffffffffffff / size_of(*Declaration) || record.virtual_functions.count > 0x7fffffffffffffff / size_of(*Function) || record.parents.count > 0x7fffffffffffffff / size_of(*Declaration) {
        gm_construction_error("record metadata member storage size overflows");
        return null;
    }

    if record.virtual_functions.count > 0 || record.parents.count > 0 {
        for member: record.declarations if member && member.kind == .BITFIELD {
            gm_construction_error("grouped bitfield inheritance/polymorphic layout requires unsupported ABI metadata");
            return null;
        }
    }

    if record.parents.count > 1 {
        gm_construction_error("multiple-inheritance adjustment metadata is unsupported");
        return null;
    }

    if !gm_metadata_virtual_members(record) return null;

    info: Struct_Extra_Info;
    info.struct_alignment = record.type.alignment;
    parent_info: *Struct_Extra_Info;

    if record.parents.count {
        parent_declaration := record.parents[0];
        parent_type := ifx parent_declaration then gm_underlying(parent_declaration.type) else null;

        if !parent_type || !parent_type.type_of_struct || parent_type.template_instantiation_params.count {
            gm_construction_error("inheritance metadata requires an ordinary represented parent");
            return null;
        }

        parent := parent_type.type_of_struct;
        phase := gm_scope_index(session, cast(*Declaration) parent);

        if phase >= 0 && session.scope_states[phase].stage != .FULL {
            native := gm_declaration_cursor(cast(*Declaration) parent);
            if clang_Cursor_isNull(native) != 0 || !gm_build_declaration(native, clang_getCursorSemanticParent(native), false) {
                if !session.failed gm_construction_error("inheritance metadata cannot populate the canonical parent");
                return null;
            }
        }

        parent_info = gm_record_metadata(parent, depth + 1);
        if !parent_info return null;

        if !parent_info.vtable_struct {
            gm_construction_error("nonpolymorphic base placement and padding metadata is unsupported");
            return null;
        }

        info.vtable_parent = parent;
        info.vtable_parent_info = parent_info;
        info.vtable_parent_declaration = parent_declaration;
        for function: parent_info.vtable GM_Sessions.append_movable_owned(session, *info.vtable, function);
    }

    inherited_slots := info.vtable.count;

    // MSVC groups newly declared same-name methods in reverse declaration order.
    // Overrides below still replace their inherited slots by native identity.
    ordered: [..] *Function;

    defer array_free(ordered);

    for function: record.virtual_functions {
        if TARGET_OS != .WINDOWS {
            array_add(*ordered, function);
            continue;
        }

        if array_find(ordered, function) continue;

        for reverse_index: 0..record.virtual_functions.count - 1 {
            candidate := record.virtual_functions[record.virtual_functions.count - 1 - reverse_index];
            if candidate && function && candidate.name == function.name array_add(*ordered, candidate);
        }
    }

    for function: ordered {
        if !function || !function.type || !function.type.type_of_function {
            gm_construction_error("virtual metadata lacks a function signature");
            return null;
        }

        signature := function.type.type_of_function;
        destructor := (signature.flags & .DESTRUCTOR) != 0;

        replaced := false;

        for index: 0..inherited_slots - 1 {
            inherited := info.vtable[index];
            if inherited.name != function.name continue;

            errors_before := session.model_errors;

            if !gm_native_override(function, inherited) {
                if session.failed || session.model_errors != errors_before return null;
                continue;
            }

            if !destructor && !functions_are_equivalent(inherited, function, true) {
                gm_construction_error("native override signature exceeds public type-comparison support");
                return null;
            }

            if !destructor && !is_same_type(inherited.type.type_of_function.return_type, signature.return_type) {
                gm_construction_error("covariant virtual return adjustment is unsupported");
                return null;
            }

            info.vtable[index] = function;
            replaced = true;
            if !destructor break;
        }

        if !replaced {
            GM_Sessions.append_movable_owned(session, *info.vtable, function);
            if destructor && TARGET_OS != .WINDOWS GM_Sessions.append_movable_owned(session, *info.vtable, function);
        }
    }

    if info.vtable.count {
        if !gm_metadata_native_abi() {
            gm_construction_error("vtable metadata requires a supported 64-bit macOS, Linux or MSVC Windows native ABI");
            return null;
        }

        pointer_size := gm_pointer_size();
        table := GM_Sessions.new_owned(session, Struct);
        table.kind = .STRUCT;
        table.parent = cast(*Declaration) record;
        table.name = GM_Sessions.copy_text(session, tprint("%_VTable", record.name));
        table.output_name = GM_Sessions.copy_text(session, tprint("%_VTable", gm_name(cast(*Declaration) record)));
        table.flags = .IS_VTABLE;
        table.decl_flags = .IS_METADATA;
        table.type = GM_Sessions.new_owned(session, CType);
        table.type.type_of_struct = table;
        table.type.size = info.vtable.count * pointer_size;
        table.type.alignment = pointer_size;
        info.vtable_struct = table;
        for function, index: info.vtable {
            type := gm_vtable_slot_type(function);
            if !type return null;

            field := gm_generated_member(session, table, gm_vtable_slot_name(function, index, table), type, index * pointer_size * 8);
            field.comment.text = GM_Sessions.copy_text(session, tprint("// C++ virtual: %", function.name));

            GM_Sessions.append_owned(session, *table.declarations, field);
        }

        pointer := GM_Sessions.new_owned(session, CType);
        pointer.pointer_to = table.type;
        pointer.size = pointer_size;
        pointer.alignment = pointer_size;
        info.vtable_pointer_declaration = gm_generated_member(session, record, "__vtable", pointer, 0);
        GM_Sessions.append_movable_owned(session, *info.modified_declarations, info.vtable_pointer_declaration);

        if parent_info for field: parent_info.modified_declarations {
            if !field || field.kind != .DECLARATION || field == parent_info.vtable_pointer_declaration continue;
            GM_Sessions.append_movable_owned(session, *info.modified_declarations, field);
        }
    }

    grouped := false;

    if !info.vtable_struct && !record.parents.count && !record.template_type_params.count grouped = gm_group_bitfields(session, record, *info);

    for declaration: record.declarations {
        if !grouped GM_Sessions.append_movable_owned(session, *info.modified_declarations, declaration);
        GM_Sessions.append_movable_owned(session, *info.all_scope_declarations, declaration);
    }

    if info.vtable_struct && !gm_polymorphic_field_layout(record, *info) return null;

    gm_empty_record_storage(record, *info);

    for field: info.modified_declarations {
        if !field || (field.kind != .DECLARATION && field.kind != .BITFIELD) || field.storage_class == .Static || (field.decl_flags & .IS_METADATA) != 0 continue;

        info.has_field = true;

        if field.type {
            if field.type.size > 0 info.has_nonzero_sized_field = true;
            info.largest_natural_alignment = max(info.largest_natural_alignment, field.type.alignment);
        }

        if field.kind == .BITFIELD {
            alignment_type := (cast(*Bitfield) field).alignment_type;
            if alignment_type info.largest_natural_alignment = max(info.largest_natural_alignment, alignment_type.alignment);
        }
    }

    if (record.flags & .IS_RAW) != 0 info.raw_struct = record;
    if info.vtable_struct GM_Sessions.append_owned(session, *record.metadata, cast(*Declaration) info.vtable_struct);

    return gm_metadata_cache(record, *info);
}
