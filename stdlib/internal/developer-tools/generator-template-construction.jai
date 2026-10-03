#scope_module
#import "Basic";

gm_template_parameter_has_token :: (cur: CXCursor, text: string) -> bool {
    unit := gm_token_unit();
    if !unit return false;

    tokens: *CXToken;
    count: u32;

    clang_tokenize(unit, clang_getCursorExtent(cur), *tokens, *count);
    defer if tokens clang_disposeTokens(unit, tokens, count);
    for i: 0..cast(s64) count - 1 if gm_is_token(tokens[i], text) return true;

    return false;
}

gm_template_default_location :: (cur: CXCursor) -> bool, CXFile, u32 {
    unit := gm_token_unit();
    if !unit return false, null, 0;

    name_file: CXFile;
    name_offset: u32;

    clang_getExpansionLocation(clang_getCursorLocation(cur), *name_file, null, null, *name_offset);
    tokens: *CXToken;
    count: u32;

    clang_tokenize(unit, clang_getCursorExtent(cur), *tokens, *count);
    defer if tokens clang_disposeTokens(unit, tokens, count);

    for i: 0..cast(s64) count - 1 {
        if !gm_is_token(tokens[i], "=") continue;

        file: CXFile;
        offset: u32;

        clang_getExpansionLocation(clang_getTokenLocation(unit, tokens[i]), *file, null, null, *offset);
        if file == name_file && offset >= name_offset return true, file, offset;
    }

    return false, null, 0;
}

gm_template_shape_supported :: (cur: CXCursor) -> bool {
    children := native_children(cur);

    defer array_free(children);

    for child: children {
        if child.kind != .TemplateTypeParameter && child.kind != .NonTypeTemplateParameter && child.kind != .TemplateTemplateParameter continue;

        if gm_template_parameter_has_token(child, "...") {
            log_error("Bindings_Generator: template parameter packs are unsupported\n");
            return false;
        }
    }

    return true;
}

gm_dependent_type :: (native: CXType, children: [] CXCursor, depth := 0) -> *CType, int {
    if depth > 128 {
        gm_construction_error("dependent type-reference nesting exceeds supported depth");
        return null, 0;
    }

    for child, index: children {
        if child.kind == .ParmDecl {
            nested := native_children(child);
            defer array_free(nested);
            type := gm_dependent_type(native, nested, depth + 1);
            if type return type, index + 1;
        }

        if child.kind != .TypeRef continue;

        referred := clang_getCursorReferenced(child);
        if referred.kind != .TemplateTypeParameter continue;
        if clang_equalTypes(clang_getUnqualifiedType(native), clang_getUnqualifiedType(clang_getCursorType(referred))) == 0 continue;

        declaration := gm_objc_type_param(referred);
        if !declaration return null, 0;

        result := GM_Sessions.new_owned(gm_current_session(), CType);
        result.type_of_declaration = declaration;
        result.size = clang_Type_getSizeOf(native);
        result.alignment = clang_Type_getAlignOf(native);

        return gm_apply_qualifiers(result, native), index + 1;
    }

    return null, 0;
}

gm_template_instance :: (native: CXType, cur: CXCursor, template_cur: CXCursor, location_cur: CXCursor, depth: int, supplied: [] CXCursor, use_supplied: bool) -> *CType {
    definition := clang_getCursorDefinition(template_cur);
    if clang_Cursor_isNull(definition) == 0 template_cur = definition;

    // Supported instances must use the primary definition's source body.
    // Explicit/partial specializations have independent bodies and remain unsupported.
    if template_cur.kind != .ClassTemplate || clang_equalLocations(clang_getCursorLocation(cur), clang_getCursorLocation(template_cur)) == 0 {
        log_error("Bindings_Generator: unsupported native record specialization\n");
        return null;
    }

    return gm_template_application(native, template_cur, location_cur, depth, supplied, use_supplied);
}

gm_recover_template_type :: (native: CXType, children: [] CXCursor, location_cur: CXCursor, depth: int) -> *CType, int {
    if clang_Type_getNumTemplateArguments(native) < 0 return null, 0;

    for child: children {
        if child.kind == .NamespaceRef continue;
        if child.kind != .TemplateRef return null, 0;

        template_cur := clang_getCursorReferenced(child);
        if template_cur.kind != .ClassTemplate return null, 0;

        result := gm_template_application(native, template_cur, location_cur, depth, children, true);
        if !result return null, 0;

        consumed := 0;

        for entry: children {
            if entry.kind != .NamespaceRef && entry.kind != .TemplateRef && entry.kind != .TypeRef break;
            consumed += 1;
        }

        return result, consumed;
    }

    return null, 0;
}

gm_template_application :: (native: CXType, template_cur: CXCursor, location_cur: CXCursor, depth: int, supplied: [] CXCursor, use_supplied: bool) -> *CType {
    canonical_declaration := clang_getTypeDeclaration(clang_getCanonicalType(native));
    if clang_Cursor_isNull(canonical_declaration) == 0 && clang_Cursor_isNull(clang_getSpecializedCursorTemplate(canonical_declaration)) == 0 &&
       clang_equalLocations(clang_getCursorLocation(canonical_declaration), clang_getCursorLocation(template_cur)) == 0 {
        gm_construction_error("unsupported native record specialization body");
        return null;
    }

    declaration := gm_build_declaration(template_cur, clang_getCursorSemanticParent(template_cur), false);
    if !declaration || declaration.kind != .STRUCT return null;

    record := cast(*Struct) declaration;
    count := clang_Type_getNumTemplateArguments(native);
    specialization := clang_getTypeDeclaration(native);
    resolved_count := clang_Cursor_getNumTemplateArguments(specialization);

    if resolved_count >= 0 count = resolved_count;

    if count < 0 || count != record.template_type_params.count {
        gm_construction_error("native template argument count does not match the primary type parameters");
        return null;
    }

    session := gm_current_session();
    if !session return null;

    result := GM_Sessions.new_owned(session, CType);
    result.type_of_struct = record;
    result.size = clang_Type_getSizeOf(native);
    result.alignment = clang_Type_getAlignOf(native);

    for i: 0..cast(s64) count - 1 {
        argument := clang_Type_getTemplateArgumentAsType(native, cast(u32) i);
        if resolved_count >= 0 && clang_Cursor_getTemplateArgumentKind(specialization, cast(u32) i) == .Type argument = clang_Cursor_getTemplateArgumentType(specialization, cast(u32) i);

        if record.template_type_params[i].type != gm_active_generator().type_def_type {
            if clang_Cursor_getNumTemplateArguments(specialization) != count {
                gm_construction_error("installed SDK lacks exact non-type specialization argument metadata");
                return null;
            }

            kind := clang_Cursor_getTemplateArgumentKind(specialization, cast(u32) i);
            literal := GM_Sessions.new_owned(session, Literal);

            if kind == .NullPtr literal.literal_kind = .NULL;
            else if kind == .Integral {
                value_type := gm_underlying(record.template_type_params[i].type);
                if value_type && value_type.type_of_declaration {
                    for parameter, index: record.template_type_params if parameter == value_type.type_of_declaration {
                        actual := clang_Type_getTemplateArgumentAsType(native, cast(u32) index);
                        if resolved_count >= 0 && clang_Cursor_getTemplateArgumentKind(specialization, cast(u32) index) == .Type actual = clang_Cursor_getTemplateArgumentType(specialization, cast(u32) index);
                        value_type = gm_build_type(actual, location_cur, false, depth + 1, supplied, use_supplied);
                        break;
                    }
                }

                integer_type := ifx value_type && (value_type.flags & .BOOL) != 0 then value_type else gm_integer(value_type);
                if !integer_type || integer_type.size <= 0 || integer_type.size > 8 {
                    gm_construction_error("non-type template integral payload exceeds the exact public64-bit representation");
                    return null;
                }

                if (integer_type.number_flags & .SIGNED) != 0 {
                    literal.literal_kind = .INTEGER;
                    literal.int_value = clang_Cursor_getTemplateArgumentValue(specialization, cast(u32) i);
                } else {
                    literal.literal_kind = .UNSIGNED_INTEGER;
                    literal.uint_value = clang_Cursor_getTemplateArgumentUnsignedValue(specialization, cast(u32) i);
                }
            } else {
                gm_construction_error("unsupported native non-type template argument category");
                return null;
            }

            GM_Sessions.append_owned(session, *result.template_instantiation_params, cast(*Expression) literal);
            continue;
        }

        if argument.kind == .Invalid {
            gm_construction_error("native template argument is not a representable type argument");
            return null;
        }

        type := gm_build_type(argument, location_cur, false, depth + 1, supplied, use_supplied);
        if !type return null;

        reference := GM_Sessions.new_owned(session, Type_Reference);
        reference.kind = .TYPE_REFERENCE;
        reference.type = type;
        GM_Sessions.append_owned(session, *result.template_instantiation_params, cast(*Expression) reference);
    }

    return gm_apply_qualifiers(result, native);
}

gm_clone_template_expression :: (expression: *Expression, depth := 0) -> *Expression {
    if !expression || depth >= 128 {
        gm_construction_error("malformed or excessively deep template argument expression");
        return null;
    }

    session := gm_current_session();

    if expression.kind == .TYPE_REFERENCE {
        result := GM_Sessions.new_owned(session, Type_Reference);
        result.kind = .TYPE_REFERENCE;
        result.type = (cast(*Type_Reference) expression).type;
        return cast(*Expression) result;
    }

    if expression.kind == .LITERAL {
        source := cast(*Literal) expression;
        if source.template_instantiation_params.count < 0 || (source.template_instantiation_params.count > 0 && !source.template_instantiation_params.data) {
            gm_construction_error("malformed literal template argument storage");
            return null;
        }

        result := GM_Sessions.new_owned(session, Literal);
        result.literal_kind = source.literal_kind;
        result.location = source.location;
        result.raw_value = GM_Sessions.copy_text(session, source.raw_value);

        for argument: source.template_instantiation_params {
            copy := gm_clone_template_expression(argument, depth + 1);
            if !copy return null;
            GM_Sessions.append_owned(session, *result.template_instantiation_params, copy);
        }

        if source.literal_kind == .STRING || source.literal_kind == .RAW_CPP_TOKENS result.string_value = GM_Sessions.copy_text(session, source.string_value);
        else if source.literal_kind == .INTEGER result.int_value = source.int_value;
        else if source.literal_kind == .UNSIGNED_INTEGER result.uint_value = source.uint_value;
        else if source.literal_kind == .FLOAT result.float_value = source.float_value;
        else if source.literal_kind == .IDENTIFIER result.identifier = source.identifier;
        else if source.literal_kind == .ENUM {
            result.enum_type = source.enum_type;
            result.enum_value = source.enum_value;
        } else if source.literal_kind != .NULL {
            gm_construction_error("unsupported template literal copy category");
            return null;
        }

        return cast(*Expression) result;
    }

    if expression.kind == .PAREN_EXPRESSION {
        result := GM_Sessions.new_owned(session, Paren_Expression);
        result.kind = .PAREN_EXPRESSION;
        result.child = gm_clone_template_expression((cast(*Paren_Expression) expression).child, depth + 1);
        if !result.child return null;

        return cast(*Expression) result;
    }

    if expression.kind == .UNARY_OPERATOR {
        source := cast(*Unary_Operator) expression;
        result := GM_Sessions.new_owned(session, Unary_Operator);
        result.kind = .UNARY_OPERATOR;
        result.op = GM_Sessions.copy_text(session, source.op);
        result.child = gm_clone_template_expression(source.child, depth + 1);
        if !result.child return null;

        return cast(*Expression) result;
    }

    if expression.kind == .BINARY_OPERATOR {
        source := cast(*Binary_Operator) expression;
        result := GM_Sessions.new_owned(session, Binary_Operator);
        result.kind = .BINARY_OPERATOR;
        result.op = GM_Sessions.copy_text(session, source.op);
        result.lhs = gm_clone_template_expression(source.lhs, depth + 1);
        result.rhs = gm_clone_template_expression(source.rhs, depth + 1);
        if !result.lhs || !result.rhs return null;

        return cast(*Expression) result;
    }

    gm_construction_error("unsupported template argument expression copy");

    return null;
}
