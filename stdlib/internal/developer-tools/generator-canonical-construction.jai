#scope_module
#import "Basic";

gm_construction_error :: (message: string) {
    log_error("Bindings_Generator: %\n", message);
    session := context._bindings_generator_session;
    if !session return;

    if session.model_errors == 0xffffffffffffffff {
        session.failed = true;
        return;
    }

    session.model_errors += 1;
}

gm_input_error :: (message: string) {
    if context._bindings_generator_session context._bindings_generator_session.failed = true;
    // A logger can reenter public construction; fatal state must already be
    // visible before it runs, including while canonical cleanup is iterating.
    log_error("Bindings_Generator: %\n", message);
}

gm_native_text :: (native: CXString) -> string {
    defer clang_disposeString(native);
    session := gm_current_session();
    if !session return "";
    return GM_Sessions.copy_text(session, to_string(clang_getCString(native)));
}

gm_cursor_name :: (cur: CXCursor) -> string {
    text := get_name_for_cursor(cur);
    defer free(text);
    return GM_Sessions.copy_text(gm_current_session(), text);
}

gm_comment_source_location :: (cur: CXCursor, declaration: Location) -> Location {
    result: Location;
    range := clang_Cursor_getCommentRange(cur);

    if clang_Range_isNull(range) != 0 return result;

    clang_getExpansionLocation(clang_getRangeStart(range), *result.file, *result.line, *result.column, null);
    if result.line >= declaration.line return .{};

    return result;
}

gm_fill_native_comment :: (comment: *Comment, cur: CXCursor, location: Location) {
    native := clang_Cursor_getRawCommentText(cur);

    defer clang_disposeString(native);
    text := trim(to_string(clang_getCString(native)));
    if !text return;

    session := gm_current_session();
    if !session return;

    comment.text = GM_Sessions.copy_text(session, text);
    before := gm_comment_source_location(cur, location);
    if before.line != 0 comment.location = before;
}

gm_scope_index :: (session: *GM_Sessions.Session, declaration: *Declaration) -> int {
    for state, index: session.scope_states if state.declaration == declaration return index;
    return -1;
}

gm_namespace_children :: (cur: CXCursor, depth := 0) -> [..] CXCursor {
    result: [..] CXCursor;

    if depth > 128 {
        gm_construction_error("namespace nesting exceeds supported construction depth");
        return result;
    }

    parent := clang_getCursorSemanticParent(cur);
    siblings: [..] CXCursor;

    if parent.kind == .Namespace siblings = gm_namespace_children(parent, depth + 1);
    else siblings = native_children(parent);

    defer array_free(siblings);
    canonical := clang_getCanonicalCursor(cur);

    for sibling: siblings {
        if sibling.kind != .Namespace || clang_equalCursors(clang_getCanonicalCursor(sibling), canonical) == 0 continue;
        children := native_children(sibling);
        defer array_free(children);
        for child: children array_add(*result, child);
    }

    return result;
}

gm_append_scope_member :: (session: *GM_Sessions.Session, members: *[..] *Declaration, declaration: *Declaration) {
    if array_find(members.*, declaration) return;

    if declaration.kind == .FUNCTION {
        for other: members.* {
            if !other || other.kind != .FUNCTION || gm_name(other) != gm_name(declaration) continue;
            if functions_are_equivalent(cast(*Function) other, cast(*Function) declaration) declaration.decl_flags |= .OMIT_FROM_OUTPUT;
        }
    }

    GM_Sessions.append_owned(session, members, declaration);
}

gm_prepare_typedef :: (alias: *Typedef, cur: CXCursor) {
    underlying := gm_underlying(alias.type);
    if !underlying return;

    nominal: *Declaration;
    if underlying.type_of_struct nominal = cast(*Declaration) underlying.type_of_struct;
    if underlying.type_of_enum nominal = cast(*Declaration) underlying.type_of_enum;

    if nominal {
        native := clang_getTypeDeclaration(clang_getTypedefDeclUnderlyingType(cur));
        if !nominal.name || clang_Cursor_isAnonymous(native) != 0 {
            nominal.name = GM_Sessions.copy_text(gm_current_session(), alias.name);
            nominal.output_name = GM_Sessions.copy_text(gm_current_session(), alias.output_name);
            alias.decl_flags |= .OMIT_FROM_OUTPUT;
            return;
        }

        // Libclang also synthesizes the alias's spelling for anonymous typedef tags.
        // Equal unqualified names in distinct scopes still introduce distinct public names.
        nominal_parent := clang_getCanonicalCursor(clang_getCursorSemanticParent(native));
        alias_parent := clang_getCanonicalCursor(clang_getCursorSemanticParent(cur));

        if nominal.name == alias.name && clang_equalCursors(nominal_parent, alias_parent) != 0 {
            alias.decl_flags |= .OMIT_FROM_OUTPUT;
            return;
        }
    }

    matches := false;
    for name: TYPEDEFS_TO_UNWRAP if alias.name == name matches = true;
    for prefix: TYPEDEF_PREFIXES_TO_UNWRAP if prefix && begins_with(alias.name, prefix) matches = true;

    options := gm_active_options();
    if options for prefix: options.typedef_prefixes_to_unwrap if prefix && begins_with(alias.name, prefix) matches = true;
    if !matches return;

    // Pointer/array/function descriptors have no independent represented name.
    named := nominal != null || underlying.hardcoded_jai_string.count > 0;

    if !underlying.pointer_to && !underlying.array_element_type && !underlying.type_of_function {
        if underlying.number_flags != 0 || underlying.size == 16 || (underlying.flags & (.VOID | .BOOL)) != 0 named = true;
    }

    if !named return;

    generator := gm_active_generator();
    if !generator return;

    GM_Table.table_add(*generator.unwrapped_typedefs, alias.name, alias);
}

gm_cpp_base :: (record: *Struct, cur: CXCursor) -> bool {
    generator := gm_active_generator();
    if !generator return false;

    canonical := clang_getCanonicalCursor(cur);
    found, existing := GM_Table.table_find(*generator.declarations_by_cursor, canonical);

    if found {
        if existing && existing.parent == cast(*Declaration) record return true;
        gm_construction_error("base cursor belongs to another canonical declaration");
        return false;
    }

    if clang_isVirtualBase(cur) != 0 {
        gm_construction_error("virtual base layout needs shared inheritance metadata integration");
        return false;
    }

    type := gm_build_type(clang_getCursorType(cur), cur, true, 0);
    underlying := gm_underlying(type);

    if !underlying || !underlying.type_of_struct {
        gm_construction_error("base specifier requires a represented record type");
        return false;
    }

    session := gm_current_session();
    if !session return false;

    base := GM_Sessions.new_owned(session, Declaration);
    base.kind = .DECLARATION;
    base.name = GM_Sessions.copy_text(session, underlying.type_of_struct.name);
    base.output_name = GM_Sessions.copy_text(session, gm_name(cast(*Declaration) underlying.type_of_struct));
    base.type = type;
    base.parent = cast(*Declaration) record;
    base.decl_flags = .QUALIFIER_USING;
    fill_location(*base.location, cur);
    access := clang_getCXXAccessSpecifier(cur);
    if access == .CXXPrivate base.decl_flags |= .PRIVATE;
    if access == .CXXProtected base.decl_flags |= .PROTECTED;

    GM_Sessions.append_owned(session, *record.parents, base);
    GM_Table.table_add(*generator.declarations_by_cursor, canonical, base);

    return true;
}

gm_public_type :: (native: CXType, cur: CXCursor, accurate: bool, argument_index: int, substitute_children: [] CXCursor) -> *CType {
    if substitute_children.count < 0 || (substitute_children.count > 0 && !substitute_children.data) {
        gm_input_error("malformed type-construction child view");
        return null;
    }

    if native.kind == .ObjCTypeParam return gm_objc_parameter_type(native, cur, accurate, argument_index, substitute_children);

    if (native.kind == .Elaborated || native.kind == .Unexposed) && substitute_children.count > 0 && !accurate {
        result := gm_elaborated_type(native, substitute_children, cur);
        return result;
    }

    return gm_build_type(native, cur, accurate, 0, substitute_children, !accurate);
}

gm_scalar_slot :: (generator: *Generator, native: CXType) -> **CType {
    kind := native.kind;
    if kind == .Void return *generator.type_def_void;
    if kind == .Bool return *generator.type_def_bool;
    if kind == .Char_S || kind == .Char_U return *generator.type_def_char;

    size := clang_Type_getSizeOf(native);
    signed := kind == .SChar || kind == .Short || kind == .Int || kind == .Long || kind == .LongLong || kind == .Int128;
    unsigned := kind == .UChar || kind == .UShort || kind == .UInt || kind == .ULong || kind == .ULongLong || kind == .UInt128 || kind == .Char16 || kind == .Char32;

    if kind == .WChar signed = true;

    if kind == .Float || kind == .Double || kind == .LongDouble {
        if size == 4 return *generator.type_def_float32;
        if size == 8 return *generator.type_def_float64;
        return null;
    }

    if !signed && !unsigned return null;
    if size == 1 return ifx signed then *generator.type_def_s8 else *generator.type_def_u8;
    if size == 2 return ifx signed then *generator.type_def_s16 else *generator.type_def_u16;
    if size == 4 return ifx signed then *generator.type_def_s32 else *generator.type_def_u32;
    if size == 8 return ifx signed then *generator.type_def_s64 else *generator.type_def_u64;
    if size == 16 return ifx signed then *generator.type_def_s128 else *generator.type_def_u128;

    return null;
}

gm_apply_qualifiers :: (base: *CType, native: CXType) -> *CType {
    if !base return null;

    qualifiers: CType.Flags;
    if clang_isConstQualifiedType(native) != 0 qualifiers |= .CONST;
    if clang_isVolatileQualifiedType(native) != 0 qualifiers |= .VOLATILE;
    if !qualifiers return base;

    session := gm_current_session();
    if !session return null;

    result := GM_Sessions.new_owned(session, CType);
    // Copy descriptor scalars/nominal references only, never a mutable child array.
    result.pointer_to = base.pointer_to;
    result.type_of_enum = base.type_of_enum;
    result.type_of_typedef = base.type_of_typedef;
    result.type_of_struct = base.type_of_struct;
    result.type_of_declaration = base.type_of_declaration;
    result.type_of_function = base.type_of_function;
    result.array_element_type = base.array_element_type;
    result.array_element_count = base.array_element_count;
    result.array_element_count_expression = base.array_element_count_expression;
    result.hardcoded_jai_string = GM_Sessions.copy_text(session, base.hardcoded_jai_string);
    result.size = base.size;
    result.alignment = base.alignment;
    result.number_flags = base.number_flags;
    result.flags = (base.flags & ~CType.Flags.SHARED) | qualifiers;
    result.comment = GM_Sessions.copy_text(session, base.comment);

    for argument: base.template_instantiation_params {
        copy := gm_clone_template_expression(argument);
        if !copy return null;
        GM_Sessions.append_owned(session, *result.template_instantiation_params, copy);
    }

    return result;
}

gm_build_type :: (native: CXType, cur: CXCursor, accurate: bool, depth: int, supplied: [] CXCursor = .[], use_supplied := false, array_dimension := 0) -> *CType {
    session := gm_current_session();
    if !session return null;

    generator := gm_active_generator();
    if !generator return null;
    if session.failed return null;

    if depth > 128 {
        gm_construction_error("native type graph exceeds supported construction depth");
        return null;
    }

    slot := gm_scalar_slot(generator, native);
    if slot {
        if !slot.* {
            result := GM_Sessions.new_owned(session, CType);
            result.size = clang_Type_getSizeOf(native);
            result.alignment = clang_Type_getAlignOf(native);
            result.flags = .SHARED;
            if native.kind == .Int128 result.hardcoded_jai_string = GM_Sessions.copy_text(session, "S128");
            if native.kind == .UInt128 result.hardcoded_jai_string = GM_Sessions.copy_text(session, "U128");

            if native.kind == .Void result.flags |= .VOID;
            else if native.kind == .Bool result.flags |= .BOOL;
            else {
                if result.size == 1 result.number_flags |= ._8BIT;
                if result.size == 2 result.number_flags |= ._16BIT;
                if result.size == 4 result.number_flags |= ._32BIT;
                if result.size == 8 result.number_flags |= ._64BIT;
                if native.kind == .WChar || native.kind == .SChar || native.kind == .Short || native.kind == .Int || native.kind == .Long || native.kind == .LongLong || native.kind == .Int128 result.number_flags |= .SIGNED;
                if native.kind == .Float || native.kind == .Double || native.kind == .LongDouble result.number_flags |= .FLOAT;
                if native.kind == .Char_S || native.kind == .Char_U result.number_flags |= .CHAR;
            }

            slot.* = result;
        }

        if native.kind == .Int128 || native.kind == .UInt128 generator.int128_was_referenced = true;

        return gm_apply_qualifiers(slot.*, native);
    }

    if native.kind == .ObjCId return gm_named_canonical_type(*generator.type_def_objc_id, "ID");
    if native.kind == .ObjCClass return gm_named_canonical_type(*generator.type_def_objc_class, "Class");
    if native.kind == .ObjCSel return gm_named_canonical_type(*generator.type_def_objc_selector, "SEL");
    if native.kind == .BlockPointer return gm_opaque_block_type();

    if native.kind == .ObjCInterface {
        record := gm_objc_interface(clang_getTypeDeclaration(native), true);
        if !record return null;
        return gm_apply_qualifiers(record.type, native);
    }

    if native.kind == .ObjCTypeParam return gm_objc_parameter_type(native, cur, !use_supplied, 0, supplied);

    if native.kind == .ObjCObjectPointer {
        spelling := clang_getTypeSpelling(native);
        is_id := to_string(clang_getCString(spelling)) == "id";

        clang_disposeString(spelling);
        if is_id return gm_named_canonical_type(*generator.type_def_objc_id, "ID");

        pointee := gm_build_type(clang_getPointeeType(native), cur, accurate, depth + 1, supplied, use_supplied, array_dimension);
        if !pointee return null;

        result := GM_Sessions.new_owned(session, CType);
        result.pointer_to = pointee;
        result.size = clang_Type_getSizeOf(native);
        result.alignment = clang_Type_getAlignOf(native);

        return gm_apply_qualifiers(result, native);
    }

    if native.kind == .Elaborated {
        named := clang_Type_getNamedType(native);
        return gm_apply_qualifiers(gm_build_type(named, cur, accurate, depth + 1, supplied, use_supplied, array_dimension), native);
    }

    if native.kind == .Unexposed {
        owned: [..] CXCursor;

        defer array_free(owned);
        children := supplied;

        if !use_supplied {
            owned = native_children(cur);
            children = owned;
        }

        dependent := gm_dependent_type(native, children);
        if dependent return dependent;

        errors_before := session.model_errors;
        application := gm_recover_template_type(native, children, cur, depth);

        if application return application;
        if session.model_errors != errors_before return null;

        canonical := clang_getCanonicalType(native);
        if canonical.kind != native.kind || clang_equalTypes(canonical, native) == 0 return gm_build_type(canonical, cur, accurate, depth + 1, supplied, use_supplied, array_dimension);
    }

    if native.kind == .Record || native.kind == .Enum || native.kind == .Typedef {
        declaration_cur := clang_getTypeDeclaration(native);

        if native.kind == .Record {
            template_cur := clang_getSpecializedCursorTemplate(declaration_cur);
            if clang_Cursor_isNull(template_cur) == 0 return gm_template_instance(native, declaration_cur, template_cur, cur, depth, supplied, use_supplied);
        }

        declaration := gm_build_declaration(declaration_cur, clang_getCursorSemanticParent(declaration_cur), false, true);

        if !declaration return null;
        if native.kind == .Record return gm_apply_qualifiers(declaration.type, native);

        if native.kind == .Typedef {
            unwrapped, alias := GM_Table.table_find(*generator.unwrapped_typedefs, declaration.name);
            if unwrapped && cast(*Declaration) alias == declaration return gm_apply_qualifiers(declaration.type, native);
        }

        result := GM_Sessions.new_owned(session, CType);
        result.size = clang_Type_getSizeOf(native);
        result.alignment = clang_Type_getAlignOf(native);

        if native.kind == .Enum {
            result.type_of_enum = cast(*Enum) declaration;
            if declaration.type result.number_flags = declaration.type.number_flags;
        } else result.type_of_typedef = cast(*Typedef) declaration;

        return gm_apply_qualifiers(result, native);
    }

    result := GM_Sessions.new_owned(session, CType);
    result.size = clang_Type_getSizeOf(native);
    result.alignment = clang_Type_getAlignOf(native);

    if native.kind == .Pointer || native.kind == .LValueReference || native.kind == .RValueReference {
        pointee := gm_build_type(clang_getPointeeType(native), cur, false, depth + 1, supplied, use_supplied, array_dimension);
        if !pointee return null;

        result.pointer_to = pointee;
        if native.kind != .Pointer result.flags |= .REFERENCE;

        return gm_apply_qualifiers(result, native);
    }

    if native.kind == .ConstantArray || native.kind == .IncompleteArray || native.kind == .DependentSizedArray {
        result.array_element_type = gm_build_type(clang_getArrayElementType(native), cur, false, depth + 1, supplied, use_supplied, array_dimension + 1);
        result.array_element_count = clang_getArraySize(native);

        if native.kind == .DependentSizedArray {
            result.array_element_count_expression = gm_array_bound_expression(cur, array_dimension, supplied, use_supplied);
            if !result.array_element_count_expression return null;
        }

        if !result.array_element_type return null;

        return gm_apply_qualifiers(result, native);
    }

    if native.kind == .FunctionProto || native.kind == .FunctionNoProto {
        result.type_of_function = gm_build_function_type(cur, native, null, accurate, depth + 1, supplied, use_supplied, array_dimension);
        if !result.type_of_function return null;
        return result;
    }

    spelling := clang_getTypeSpelling(native);

    defer clang_disposeString(spelling);
    gm_construction_error(tprint("unsupported native model type: %", to_string(clang_getCString(spelling))));

    return null;
}

gm_build_function_type :: (cur: CXCursor, native: CXType, function: *Function, accurate: bool, depth: int, supplied: [] CXCursor = .[], use_supplied := false, return_array_dimension := 0) -> *Function_Type {
    session := gm_current_session();
    if !session return null;

    objc := accurate && (cur.kind == .ObjCInstanceMethodDecl || cur.kind == .ObjCClassMethodDecl);
    if native.kind != .FunctionProto && native.kind != .FunctionNoProto && !objc {
        gm_construction_error("function type construction needs a native function type");
        return null;
    }

    result := GM_Sessions.new_owned(session, Function_Type);
    return_native := ifx objc then clang_getCursorResultType(cur) else clang_getResultType(native);

    result.return_type = gm_build_type(return_native, cur, true, depth + 1, supplied, use_supplied, return_array_dimension);
    if !result.return_type return null;

    native_count := ifx accurate then clang_Cursor_getNumArguments(cur) else -1;
    count := ifx objc then native_count else clang_getNumArgTypes(native);

    if native.kind == .FunctionNoProto && native_count >= 0 count = native_count;
    if count < 0 count = 0;

    cpp := accurate && (cur.kind == .CXXMethod || cur.kind == .Constructor || cur.kind == .Destructor);
    if cpp && clang_CXXMethod_isStatic(cur) == 0 {
        parent := clang_getCursorSemanticParent(cur);
        parent_type: *CType;
        generator := gm_active_generator();

        if !generator return null;

        found, declaration := GM_Table.table_find(*generator.declarations_by_cursor, clang_getCanonicalCursor(parent));
        if found && declaration.kind == .STRUCT parent_type = declaration.type;
        else parent_type = gm_build_type(clang_getCursorType(parent), parent, false, depth + 1);

        if !parent_type return null;

        pointer := GM_Sessions.new_owned(session, CType);
        pointer.pointer_to = parent_type;
        pointer.size = gm_pointer_size();
        pointer.alignment = pointer.size;
        if cur.kind == .CXXMethod && clang_CXXMethod_isConst(cur) != 0 pointer.flags |= .CONST;

        receiver := GM_Sessions.new_owned(session, Declaration);
        receiver.kind = .DECLARATION;
        receiver.name = GM_Sessions.copy_text(session, "this");
        receiver.output_name = GM_Sessions.copy_text(session, receiver.name);
        receiver.type = pointer;
        receiver.parent = cast(*Declaration) function;
        GM_Sessions.append_owned(session, *result.arguments, receiver);
    }

    fallback_children: [..] CXCursor;
    parameter_views: [..] CXCursor;

    defer array_free(fallback_children);
    defer array_free(parameter_views);
    parameter_views_match := false;

    if native_count != count && count > 0 {
        if use_supplied {
            for child: supplied array_add(*fallback_children, child);
        } else fallback_children = native_children(cur);

        for child: fallback_children if child.kind == .ParmDecl array_add(*parameter_views, child);

        parameter_views_match = parameter_views.count == count;
        if parameter_views_match {
            for parameter, i: parameter_views {
                if clang_equalTypes(clang_getCursorType(parameter), clang_getArgType(native, cast(u32) i)) == 0 parameter_views_match = false;
            }
        }
    }

    for i: 0..count - 1 {
        argument: *Declaration;

        if native_count == count {
            argument_cur := clang_Cursor_getArgument(cur, cast(u32) i);
            parent_flags: Declaration_Flags;
            // A standalone signature does not construct its absent owning function.
            argument_parent := ifx function then cur else clang_getNullCursor();

            argument, parent_flags = gm_build_variable(argument_cur, argument_parent, true, function);
            if function function.decl_flags |= parent_flags;
        } else if parameter_views_match {
            // Callback fields expose native parameter children even though the
            // enclosing field is not an accurate function-declaration cursor.
            // Keep each parameter's declarator/bound domain separate.
            parent_flags: Declaration_Flags;
            argument, parent_flags = gm_build_variable(parameter_views[i], clang_getNullCursor(), true, function);
            if function function.decl_flags |= parent_flags;
        } else {
            argument = GM_Sessions.new_owned(session, Declaration);
            argument.kind = .DECLARATION;
            argument.name = GM_Sessions.copy_text(session, tprint("unknown%", i));
            argument.output_name = GM_Sessions.copy_text(session, argument.name);
            // No proven parameter view: source-dependent bounds must diagnose,
            // never accidentally borrow an enclosing field's dimension zero.
            argument.type = gm_build_type(clang_getArgType(native, cast(u32) i), clang_getNullCursor(), false, depth + 1, fallback_children, true);
        }

        if !argument || !argument.type return null;

        if !argument.name {
            argument.name = GM_Sessions.copy_text(session, tprint("unknown%", i));
            argument.output_name = GM_Sessions.copy_text(session, argument.name);
        }

        if function argument.parent = cast(*Declaration) function;

        GM_Sessions.append_owned(session, *result.arguments, argument);
    }

    if clang_isFunctionTypeVariadic(native) != 0 result.flags |= .VARIADIC;
    if accurate && clang_Cursor_isFunctionInlined(cur) != 0 result.flags |= .INLINE;

    if accurate {
        if cur.kind == .CXXMethod result.flags |= .CPP_METHOD;
        if cur.kind == .Constructor result.flags |= .CPP_METHOD | .CONSTRUCTOR;
        if cur.kind == .Destructor result.flags |= .CPP_METHOD | .DESTRUCTOR;
        if cur.kind == .Constructor && clang_CXXConstructor_isCopyConstructor(cur) != 0 result.flags |= .COPY_CONSTRUCTOR;
        if cur.kind == .Constructor && clang_CXXConstructor_isMoveConstructor(cur) != 0 result.flags |= .MOVE_CONSTRUCTOR;
        if cur.kind == .ObjCInstanceMethodDecl result.flags |= .OBJC_METHOD;
        if cur.kind == .ObjCClassMethodDecl result.flags |= .OBJC_CLASS_METHOD;
    }

    return result;
}

gm_cursor_kind :: (cur: CXCursor) -> C_Kind {
    if cur.kind == .StructDecl || cur.kind == .UnionDecl || cur.kind == .ClassDecl || cur.kind == .ClassTemplate return .STRUCT;
    if cur.kind == .EnumDecl return .ENUM;
    if cur.kind == .TypedefDecl || cur.kind == .TypeAliasDecl return .TYPEDEF;
    if cur.kind == .Namespace return .NAMESPACE;
    if cur.kind == .FunctionDecl || cur.kind == .CXXMethod || cur.kind == .Constructor || cur.kind == .Destructor || cur.kind == .ObjCInstanceMethodDecl || cur.kind == .ObjCClassMethodDecl return .FUNCTION;
    if cur.kind == .VarDecl || cur.kind == .FieldDecl || cur.kind == .ParmDecl || cur.kind == .EnumConstantDecl return .DECLARATION;

    return .NONE;
}

gm_build_expected :: (cur: CXCursor, kind: C_Kind, type_only := false) -> *Declaration {
    if gm_cursor_kind(cur) != kind {
        gm_construction_error("cursor kind does not match requested model declaration");
        return null;
    }

    return gm_build_declaration(cur, clang_getCursorSemanticParent(cur), false, type_only);
}

gm_build_variable :: (cur: CXCursor, parent: CXCursor, is_argument: bool, function: *Function) -> *Declaration, Declaration_Flags {
    if gm_cursor_kind(cur) != .DECLARATION {
        gm_construction_error("variable construction needs a field, parameter or variable cursor");
        return null, 0;
    }

    declaration := gm_build_declaration(cur, parent, false);
    flags: Declaration_Flags;

    if declaration && is_argument && function declaration.parent = cast(*Declaration) function;

    if declaration && is_argument && declaration.type {
        if declaration.type.array_element_type {
            pointer := GM_Sessions.new_owned(gm_current_session(), CType);
            pointer.pointer_to = declaration.type.array_element_type;
            pointer.size = gm_pointer_size();
            pointer.alignment = pointer.size;
            declaration.type = pointer;
        }

        type := declaration.type;
        if (type.flags & .REFERENCE) != 0 && type.pointer_to && (type.pointer_to.flags & .CONST) != 0 && (!function || (function.flags & .VIRTUAL) == 0) flags |= .NEEDS_ARGUMENT_WRAPPER;

        if type.pointer_to && declaration.expression && declaration.expression.kind == .STRUCT_LITERAL {
            flags |= .NEEDS_ARGUMENT_WRAPPER;
            declaration.decl_flags |= .POINTER_TO_VALUE;
        }
    }

    return declaration, flags;
}

gm_record_categories :: (record: *Struct, cur: CXCursor) {
    if cur.kind == .UnionDecl record.flags |= .IS_UNION;
    if clang_Cursor_isAnonymous(cur) != 0 record.flags |= .IS_ANON;
    if clang_CXXRecord_isAbstract(cur) != 0 record.flags |= .IS_ABSTRACT;

    children := native_children(cur);

    defer array_free(children);

    for child: children {
        if child.kind == .Constructor {
            record.flags |= .HAS_CONSTRUCTOR;
            if clang_CXXConstructor_isCopyConstructor(child) != 0 record.flags |= .HAS_COPY_CONSTRUCTOR;
            if clang_CXXConstructor_isMoveConstructor(child) != 0 record.flags |= .HAS_MOVE_CONSTRUCTOR;
        }

        if child.kind == .Destructor record.flags |= .HAS_DESTRUCTOR;
    }
}

gm_build_declaration :: (cur: CXCursor, parent: CXCursor, only_structs: bool, type_only := false) -> *Declaration {
    session := gm_current_session();
    if !session return null;

    generator := gm_active_generator();
    if !generator return null;
    if session.failed return null;

    kind := gm_cursor_kind(cur);
    if only_structs && (kind == .FUNCTION || kind == .DECLARATION || kind == .MACRO_DEFINITION) return null;

    if kind == .STRUCT && clang_Cursor_isNull(clang_getSpecializedCursorTemplate(cur)) == 0 {
        log_error("Bindings_Generator: unsupported native record specialization\n");
        return null;
    }

    if cur.kind == .ClassTemplate && !gm_template_shape_supported(cur) return null;

    if kind == .NONE {
        if clang_isDeclaration(cur.kind) != 0 return null;
        gm_construction_error("unsupported native declaration kind in canonical model construction");
        return null;
    }

    canonical := clang_getCanonicalCursor(cur);
    found, existing := GM_Table.table_find(*generator.declarations_by_cursor, canonical);

    if found {
        if type_only return existing;
        if kind != .STRUCT && kind != .ENUM && kind != .NAMESPACE return existing;
        state := gm_scope_index(session, existing);
        if state < 0 || session.scope_states[state].stage == .FULL || session.scope_states[state].stage == .BUILDING return existing;
    }

    completed := false;

    definition := clang_getCursorDefinition(cur);

    if (kind == .STRUCT || kind == .ENUM) && clang_Cursor_isNull(definition) == 0 cur = definition;

    declaration: *Declaration;
    if found declaration = existing;
    else if kind == .STRUCT declaration = cast(*Declaration) GM_Sessions.new_owned(session, Struct);
    else if kind == .ENUM declaration = cast(*Declaration) GM_Sessions.new_owned(session, Enum);
    else if kind == .TYPEDEF declaration = cast(*Declaration) GM_Sessions.new_owned(session, Typedef);
    else if kind == .FUNCTION declaration = cast(*Declaration) GM_Sessions.new_owned(session, Function);
    else if kind == .NAMESPACE declaration = cast(*Declaration) GM_Sessions.new_owned(session, Namespace);
    else declaration = GM_Sessions.new_owned(session, Declaration);

    defer if !completed gm_reject_construction(generator, declaration);

    if !found {
        declaration.kind = kind;
        declaration.name = gm_cursor_name(cur);
        declaration.output_name = GM_Sessions.copy_text(session, declaration.name);
        fill_location(*declaration.location, cur);
        gm_fill_native_comment(*declaration.comment, cur, declaration.location);
        declaration.storage_class = clang_Cursor_getStorageClass(cur);
        if cur.kind == .ClassTemplate declaration.decl_flags |= .TEMPLATE;
        if clang_isConstQualifiedType(clang_getCursorType(cur)) != 0 || cur.kind == .EnumConstantDecl declaration.decl_flags |= .IS_CONST;
    }

    if !found GM_Table.table_add(*generator.declarations_by_cursor, canonical, declaration);

    if !found && (kind == .STRUCT || kind == .ENUM || kind == .NAMESPACE) {
        state: GM_Sessions.Scope_State;
        state.declaration = declaration;
        array_add(*session.scope_states, state);
    }

    if kind == .STRUCT && !declaration.type {
        declaration.type = GM_Sessions.new_owned(session, CType);
        declaration.type.type_of_struct = cast(*Struct) declaration;
        declaration.type.flags |= .SHARED;
        declaration.type.size = clang_Type_getSizeOf(clang_getCursorType(cur));
        declaration.type.alignment = clang_Type_getAlignOf(clang_getCursorType(cur));
        gm_record_categories(cast(*Struct) declaration, cur);
    }

    if clang_Cursor_isNull(parent) == 0 && parent.kind != .TranslationUnit {
        parent_canonical := clang_getCanonicalCursor(parent);
        parent_found, parent_declaration := GM_Table.table_find(*generator.declarations_by_cursor, parent_canonical);
        if parent_found declaration.parent = parent_declaration;
        else declaration.parent = gm_build_declaration(parent, clang_getCursorSemanticParent(parent), false, true);
    }

    if kind == .ENUM && !declaration.type {
        declaration.type = gm_build_type(clang_getEnumDeclIntegerType(cur), cur, false, 0);
        if !declaration.type return null;
    }

    if type_only && (kind == .STRUCT || kind == .ENUM || kind == .NAMESPACE) {
        completed = true;
        return declaration;
    }

    scope_index := gm_scope_index(session, declaration);
    if scope_index >= 0 {
        if session.scope_states[scope_index].stage == .TYPES_ONLY && !only_structs {
            if kind == .STRUCT {
                st := cast(*Struct) declaration;
                st.declarations.count = 0;
            }

            if kind == .ENUM {
                enumeration := cast(*Enum) declaration;
                enumeration.enumerates.count = 0;
            }

            if kind == .NAMESPACE {
                namespace := cast(*Namespace) declaration;
                namespace.declarations.count = 0;
            }
        }

        session.scope_states[scope_index].stage = .BUILDING;
    }

    defer {
        if scope_index >= 0 && session.scope_states[scope_index].stage == .BUILDING session.scope_states[scope_index].stage = .SHELL;
    }

    if kind == .TYPEDEF {
        declaration.type = gm_build_type(clang_getTypedefDeclUnderlyingType(cur), cur, true, 0);
        if !declaration.type return null;

        gm_prepare_typedef(cast(*Typedef) declaration, cur);
        completed = true;

        return declaration;
    }

    if kind == .FUNCTION {
        function := cast(*Function) declaration;

        if cur.kind == .CXXMethod || cur.kind == .Destructor {
            if clang_CXXMethod_isVirtual(cur) != 0 function.flags |= .VIRTUAL;
            if clang_CXXMethod_isPureVirtual(cur) != 0 function.flags |= .PURE_VIRTUAL;
        }

        if clang_getCursorAvailability(cur) == .Deprecated declaration.decl_flags |= .DEPRECATED;

        declaration.type = GM_Sessions.new_owned(session, CType);
        declaration.type.size = clang_Type_getSizeOf(clang_getCursorType(cur));
        declaration.type.alignment = clang_Type_getAlignOf(clang_getCursorType(cur));
        declaration.type.type_of_function = gm_build_function_type(cur, clang_getCursorType(cur), function, true, 0);
        if !declaration.type.type_of_function return null;

        if cur.kind == .ObjCInstanceMethodDecl || cur.kind == .ObjCClassMethodDecl {
            declaration.foreign_name = GM_Sessions.copy_text(session, declaration.name);
            output := declaration.name;
            for character, index: output if character == #char ":" {
                output.count = index;
                break;
            }

            declaration.output_name = GM_Sessions.copy_text(session, output);
        } else {
            declaration.foreign_name = gm_native_text(clang_Cursor_getMangling(cur));
            if !declaration.foreign_name return null;
        }

        if cur.kind == .Constructor || cur.kind == .Destructor {
            name := "Destructor";

            if cur.kind == .Constructor {
                name = "Constructor";
                if clang_CXXConstructor_isCopyConstructor(cur) != 0 name = "CopyConstructor";
                if clang_CXXConstructor_isMoveConstructor(cur) != 0 name = "MoveConstructor";
            }

            declaration.name = GM_Sessions.copy_text(session, name);
            declaration.output_name = gm_function_output_name(name);
            options := gm_active_options();
            if options && ((cur.kind == .Constructor && (options.strip_flags & .CONSTRUCTORS) != 0) || (cur.kind == .Destructor && (options.strip_flags & .DESTRUCTORS) != 0)) declaration.decl_flags |= .OMIT_FROM_OUTPUT;
        }

        completed = true;

        return declaration;
    }

    if kind == .DECLARATION {
        declaration.type = gm_build_type(clang_getCursorType(cur), cur, true, 0);
        if !declaration.type return null;

        if cur.kind == .FieldDecl {
            declaration.bit_offset_in_parent = clang_Cursor_getOffsetOfField(cur);
            declaration.field_alignment = declaration.type.alignment;
            if clang_Cursor_isBitField(cur) != 0 declaration.bitfield_width = clang_getFieldDeclBitWidth(cur);
        }

        if cur.kind == .EnumConstantDecl {
            literal := GM_Sessions.new_owned(session, Literal);

            if declaration.parent && declaration.parent.type && (declaration.parent.type.number_flags & .SIGNED) != 0 {
                literal.literal_kind = .INTEGER;
                literal.int_value = clang_getEnumConstantDeclValue(cur);
            } else {
                literal.literal_kind = .UNSIGNED_INTEGER;
                literal.uint_value = clang_getEnumConstantDeclUnsignedValue(cur);
            }

            literal.location = declaration.location;
            declaration.expression = cast(*Expression) literal;
        } else if declaration.bitfield_width < 0 && !(declaration.type.array_element_type && declaration.type.array_element_count >= 0) {
            children := native_children(cur);

            defer array_free(children);

            for child: children if clang_isExpression(child.kind) != 0 && gm_array_bound_position(cur, child) < 0 {
                errors_before := session.model_errors;

                declaration.expression = gm_build_expression(child, cur);
                if !declaration.expression {
                    if !session.failed && session.model_errors == errors_before gm_construction_error("unrepresentable native initializer expression");
                    return null;
                }

                break;
            }
        }

        completed = true;

        return declaration;
    }

    if kind == .ENUM {
        declaration.type = gm_build_type(clang_getEnumDeclIntegerType(cur), cur, false, 0);
        if !declaration.type return null;

        if declaration.type.size > 8 {
            gm_construction_error("enum values wider than 64 bits need an exact literal representation");
            return null;
        }
    }

    children: [..] CXCursor;
    if kind == .NAMESPACE children = gm_namespace_children(cur);
    else children = native_children(cur);

    defer array_free(children);

    if kind == .STRUCT {
        st := cast(*Struct) declaration;
        if cur.kind == .UnionDecl st.flags |= .IS_UNION;
        if clang_Cursor_isAnonymous(cur) != 0 st.flags |= .IS_ANON;
        if clang_CXXRecord_isAbstract(cur) != 0 st.flags |= .IS_ABSTRACT;
    }

    for child: children {
        if kind == .STRUCT && (child.kind == .TemplateTypeParameter || child.kind == .NonTypeTemplateParameter || child.kind == .TemplateTemplateParameter) {
            parameter := gm_template_formal(child);
            if !parameter return null;

            record := cast(*Struct) declaration;
            if !array_find(record.template_type_params, parameter) GM_Sessions.append_owned(session, *record.template_type_params, parameter);

            continue;
        }

        if kind == .STRUCT && child.kind == .CXXBaseSpecifier {
            if !gm_cpp_base(cast(*Struct) declaration, child) return null;
            continue;
        }

        if clang_isDeclaration(child.kind) == 0 continue;
        if child.kind == .CXXAccessSpecifier continue;

        child_types_only := only_structs && kind == .NAMESPACE;
        errors_before := session.model_errors;
        child_declaration := create_declaration(child, cur, child_types_only);

        if !child_declaration {
            if !session.failed && gm_cursor_kind(child) == .FUNCTION && !child_types_only {
                cached, pointer := GM_Table.table_find(*generator.declarations_by_cursor, clang_getCanonicalCursor(child));
                if cached && pointer && pointer.foreign_name child_declaration = pointer;
            }
        }

        if !child_declaration {
            if !session.failed {
                // Unsupported scope declarations can be omitted, but a failed
                // represented instance field cannot silently disappear.
                if kind == .STRUCT && child.kind == .FieldDecl {
                    if session.model_errors == errors_before gm_construction_error("record contains an unrepresentable native field");
                    return null;
                }

                continue;
            }

            return null;
        }

        if kind == .STRUCT {
            st := cast(*Struct) declaration;

            gm_append_scope_member(session, *st.declarations, child_declaration);

            if child_declaration.kind == .FUNCTION {
                function := cast(*Function) child_declaration;
                if (function.flags & .VIRTUAL) != 0 && !array_find(st.virtual_functions, function) GM_Sessions.append_owned(session, *st.virtual_functions, function);

                ft := function.type.type_of_function;
                if ft && (ft.flags & .CONSTRUCTOR) != 0 st.flags |= .HAS_CONSTRUCTOR;
                if ft && (ft.flags & .COPY_CONSTRUCTOR) != 0 st.flags |= .HAS_COPY_CONSTRUCTOR;
                if ft && (ft.flags & .MOVE_CONSTRUCTOR) != 0 st.flags |= .HAS_MOVE_CONSTRUCTOR;
                if ft && (ft.flags & .DESTRUCTOR) != 0 st.flags |= .HAS_DESTRUCTOR;
            }
        }

        if kind == .ENUM gm_append_scope_member(session, *(cast(*Enum) declaration).enumerates, child_declaration);
        if kind == .NAMESPACE gm_append_scope_member(session, *(cast(*Namespace) declaration).declarations, child_declaration);
    }

    if scope_index >= 0 session.scope_states[scope_index].stage = ifx only_structs && kind == .NAMESPACE then .TYPES_ONLY else .FULL;

    completed = true;

    return declaration;
}

gm_literal_error :: (message: string, report_errors: bool) {
    if report_errors gm_construction_error(message);
}

gm_native_literal :: (cur: CXCursor, _parent: CXCursor, report_errors := true) -> *Literal {
    session := gm_current_session();
    if !session return null;
    if cur.kind == .DeclRefExpr return gm_reference_expression(cur, .{});

    native_type := clang_getCanonicalType(clang_getCursorType(cur));
    payload_type := native_type;

    if payload_type.kind == .Enum payload_type = clang_getEnumDeclIntegerType(clang_getTypeDeclaration(payload_type));

    if payload_type.kind == .Int128 || payload_type.kind == .UInt128 {
        gm_literal_error("128-bit literal evaluation cannot be represented by libclang's 64-bit value accessors", report_errors);
        return null;
    }

    if cur.kind == .CXXNullPtrLiteralExpr || cur.kind == .GNUNullExpr {
        result := GM_Sessions.new_owned(session, Literal);
        result.literal_kind = .NULL;
        fill_location(*result.location, cur);
        return result;
    }

    extent := gm_native_string_extent(cur);
    string_shaped := gm_string_source_type(native_type);

    if string_shaped && extent.found && !extent.ambiguous && extent.width != 1 {
        gm_literal_error("wide native string literal requires an explicit encoding conversion", report_errors);
        return null;
    }

    evaluated := clang_Cursor_Evaluate(cur);

    defer if evaluated clang_EvalResult_dispose(evaluated);

    if !evaluated {
        if cur.kind == .StringLiteral && clang_Cursor_isNull(_parent) == 0 && clang_equalCursors(cur, _parent) == 0 && gm_string_source_type(clang_getCanonicalType(clang_getCursorType(_parent))) return gm_native_literal(_parent, clang_getNullCursor(), report_errors);
        return null;
    }

    kind := clang_EvalResult_getKind(evaluated);
    if kind == .UnExposed && extent.found && clang_Cursor_isNull(_parent) == 0 && clang_equalCursors(cur, _parent) == 0 && gm_string_source_type(clang_getCanonicalType(clang_getCursorType(_parent))) {
        return gm_native_literal(_parent, clang_getNullCursor(), report_errors);
    }

    string_kind := kind == .StrLiteral || kind == .ObjCStrLiteral || kind == .CFStr;
    if kind != .Int && kind != .Float && !string_kind return null;

    result := GM_Sessions.new_owned(session, Literal);

    fill_location(*result.location, cur);

    if string_kind {
        if !extent.found || extent.ambiguous || extent.width != 1 {
            gm_literal_error("native string evaluation lacks a verified byte extent", report_errors);
            return null;
        }

        pointer := clang_EvalResult_getAsStr(evaluated);
        if !pointer {
            gm_literal_error("native string evaluation returned a null value", report_errors);
            return null;
        }

        text := to_string(pointer);
        if text.count != extent.units {
            gm_literal_error("native string literal contains an embedded NUL or an unsupported evaluation extent", report_errors);
            return null;
        }

        result.literal_kind = .STRING;
        result.string_value = GM_Sessions.copy_text(session, text);
    } else if kind == .Float {
        result.literal_kind = .FLOAT;
        result.float_value = clang_EvalResult_getAsDouble(evaluated);
    } else if clang_EvalResult_isUnsignedInt(evaluated) != 0 {
        result.literal_kind = .UNSIGNED_INTEGER;
        result.uint_value = clang_EvalResult_getAsUnsigned(evaluated);
    } else {
        result.literal_kind = .INTEGER;
        result.int_value = clang_EvalResult_getAsLongLong(evaluated);
    }

    return result;
}

gm_build_expression :: (cur: CXCursor, parent: CXCursor) -> *Expression {
    session := gm_current_session();
    if !session return null;
    if cur.kind == .DeclRefExpr return cast(*Expression) gm_reference_expression(cur, .{});
    if cur.kind == .IntegerLiteral || cur.kind == .FloatingLiteral || cur.kind == .CharacterLiteral || cur.kind == .StringLiteral || cur.kind == .CXXBoolLiteralExpr || cur.kind == .CXXNullPtrLiteralExpr || cur.kind == .GNUNullExpr return cast(*Expression) gm_native_literal(cur, parent);

    children := native_children(cur);

    defer array_free(children);
    expressions: [..] CXCursor;

    defer array_free(expressions);
    for child: children if clang_isExpression(child.kind) != 0 array_add(*expressions, child);
    if (cur.kind == .UnexposedExpr || cur.kind == .CStyleCastExpr) && expressions.count > 0 return gm_build_expression(expressions[expressions.count - 1], cur);

    if cur.kind == .InitListExpr return gm_aggregate_expression(cur, expressions);
    if cur.kind == .CXXFunctionalCastExpr && expressions.count == 1 && expressions[0].kind == .InitListExpr return gm_build_expression(expressions[0], cur);

    if cur.kind == .ParenExpr && expressions.count == 1 {
        result := GM_Sessions.new_owned(session, Paren_Expression);
        result.kind = .PAREN_EXPRESSION;
        result.child = gm_build_expression(expressions[0], cur);
        if !result.child return null;

        return cast(*Expression) result;
    }

    if cur.kind == .UnaryOperator && expressions.count == 1 {
        result := GM_Sessions.new_owned(session, Unary_Operator);
        result.kind = .UNARY_OPERATOR;
        result.op = gm_native_text(clang_getUnaryOperatorKindSpelling(clang_getCursorUnaryOperatorKind(cur)));
        result.child = gm_build_expression(expressions[0], cur);
        if !result.child return null;

        return cast(*Expression) result;
    }

    if cur.kind == .BinaryOperator && expressions.count == 2 {
        result := GM_Sessions.new_owned(session, Binary_Operator);
        result.kind = .BINARY_OPERATOR;
        result.op = gm_native_text(clang_getBinaryOperatorKindSpelling(clang_getCursorBinaryOperatorKind(cur)));
        result.lhs = gm_build_expression(expressions[0], cur);
        result.rhs = gm_build_expression(expressions[1], cur);
        if !result.lhs || !result.rhs return null;

        return cast(*Expression) result;
    }

    gm_construction_error("unsupported or malformed native expression shape");

    return null;
}
