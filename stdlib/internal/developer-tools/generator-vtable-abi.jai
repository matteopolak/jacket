#scope_module
#import "Basic";
#import "String";

gm_metadata_native_abi :: () -> bool {
    generator := gm_active_generator();
    if !generator.translation_unit || gm_pointer_size() != 8 return false;

    target := clang_getTranslationUnitTargetInfo(generator.translation_unit);
    if !target return false;

    defer clang_TargetInfo_dispose(target);
    triple := gm_native_text(clang_TargetInfo_getTriple(target));

    supported_cpu := begins_with(triple, "arm64-") || begins_with(triple, "aarch64-") || begins_with(triple, "x86_64-");

    if !supported_cpu return false;
    if TARGET_OS == .MACOS return contains(triple, "-apple-");
    if TARGET_OS == .LINUX return contains(triple, "-linux-");
    if TARGET_OS == .WINDOWS return contains(triple, "-windows-msvc");

    return false;
}

gm_vtable_value_type :: (type: *CType, depth := 0) -> bool {
    if depth >= 64 return false;

    type = gm_underlying(type);
    if !type return false;
    if type.pointer_to return true;
    if type.type_of_enum return gm_vtable_value_type(type.type_of_enum.type, depth + 1);
    if type.type_of_function || type.type_of_declaration || type.hardcoded_jai_string return false;
    if type.array_element_type return type.array_element_count > 0 && gm_vtable_value_type(type.array_element_type, depth + 1);

    if type.type_of_struct {
        record := type.type_of_struct;

        if type.size <= 0 || type.size > 32 || type.alignment <= 0 || type.alignment > 8 return false;

        cursor := gm_declaration_cursor(cast(*Declaration) record);
        if clang_Cursor_isNull(cursor) != 0 || !gm_build_declaration(cursor, clang_getCursorSemanticParent(cursor), false) return false;
        if record.flags & (.HAS_COPY_CONSTRUCTOR | .HAS_MOVE_CONSTRUCTOR | .HAS_DESTRUCTOR | .IS_UNION) return false;
        if record.decl_flags & .TEMPLATE return false;
        if record.parents.count || record.virtual_functions.count || record.template_type_params.count || record.type.template_instantiation_params.count return false;

        fields, position: int;

        for field: record.declarations {
            if !field || field.storage_class == .Static continue;
            if field.kind == .BITFIELD return false;
            if field.kind != .DECLARATION continue;
            if !gm_vtable_value_type(field.type, depth + 1) return false;

            if field.decl_flags & .OMIT_FROM_OUTPUT || field.type.size <= 0 return false;

            alignment, _ := gm_alignment(field.type, field);
            if alignment <= 0 || alignment > type.alignment || (alignment & (alignment - 1)) != 0 return false;

            position = (position + alignment - 1) & ~(alignment - 1);
            if field.bit_offset_in_parent != position * 8 return false;

            position += field.type.size;
            fields += 1;
        }

        return fields > 0 && ((position + type.alignment - 1) & ~(type.alignment - 1)) == type.size;
    }

    if (type.flags & .VOID) != 0 return true;

    return type.size > 0 && type.size <= 8 && (type.number_flags != 0 || (type.flags & .BOOL) != 0);
}

gm_vtable_signature :: (signature: *Function_Type) -> bool {
    if !signature || (signature.flags & .VARIADIC) != 0 || !gm_vtable_value_type(signature.return_type) return false;
    if signature.arguments.count < 0 || (signature.arguments.count > 0 && !signature.arguments.data) return false;
    for argument: signature.arguments if !argument || !gm_vtable_value_type(argument.type) return false;
    return true;
}

gm_vtable_slot_type :: (function: *Function) -> *CType {
    session := gm_current_session();
    generator := gm_active_generator();
    native := function.type.type_of_function;
    type := GM_Sessions.new_owned(session, CType);
    type.size = gm_pointer_size();
    type.alignment = type.size;

    if !generator.type_def_void {
        generator.type_def_void = GM_Sessions.new_owned(session, CType);
        generator.type_def_void.flags = .VOID | .SHARED;
        generator.type_def_void.size = 0;
        generator.type_def_void.alignment = 1;
    }

    if function.decl_flags & .OMIT_FROM_OUTPUT || native.flags & .DESTRUCTOR {
        type.pointer_to = generator.type_def_void;
        return type;
    }

    if !gm_vtable_signature(native) {
        gm_construction_error("virtual slot signature requires unverified aggregate/extended/variadic calling ABI");
        return null;
    }

    signature := GM_Sessions.new_owned(session, Function_Type);

    type.type_of_function = signature;
    signature.return_type = native.return_type;
    signature.flags = native.flags & ~(Function_Type.Flags.CPP_METHOD | .INLINE);
    result := gm_underlying(native.return_type);
    indirect := TARGET_OS == .WINDOWS && result && result.type_of_struct != null;

    for argument, index: native.arguments {
        GM_Sessions.append_owned(session, *signature.arguments, argument);
        if index != 0 || !indirect continue;

        // MSVC member returns put the result pointer after this, including small PODs.
        pointer := GM_Sessions.new_owned(session, CType);
        pointer.size = gm_pointer_size();
        pointer.alignment = pointer.size;
        pointer.pointer_to = native.return_type;
        output := GM_Sessions.new_owned(session, Declaration);
        output.kind = .DECLARATION;
        output.type = pointer;

        for suffix: 0..native.arguments.count {
            name := tprint("__result_%", suffix);
            taken := false;

            for parameter: native.arguments if parameter.output_name == name taken = true;
            if taken continue;

            output.name = GM_Sessions.copy_text(session, name);
            output.output_name = output.name;
            break;
        }

        GM_Sessions.append_owned(session, *signature.arguments, output);
        signature.return_type = generator.type_def_void;
    }

    return type;
}
