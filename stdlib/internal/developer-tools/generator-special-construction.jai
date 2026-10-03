#scope_module
#import "Basic";
#import "String";

gm_pointer_size :: () -> int {
    generator := gm_active_generator();
    if !generator return -1;
    if generator.pointer_size > 0 return generator.pointer_size;
    if !generator.translation_unit return -1;

    target := clang_getTranslationUnitTargetInfo(generator.translation_unit);
    if !target return -1;

    defer clang_TargetInfo_dispose(target);
    bits := clang_TargetInfo_getPointerWidth(target);
    if bits <= 0 || bits % 8 != 0 return -1;

    generator.pointer_size = bits / 8;

    return generator.pointer_size;
}

gm_named_canonical_type :: (slot: **CType, name: string) -> *CType {
    if slot.* return slot.*;

    session := gm_current_session();
    if !session return null;

    result := GM_Sessions.new_owned(session, CType);
    result.hardcoded_jai_string = GM_Sessions.copy_text(session, name);
    result.size = gm_pointer_size();
    result.alignment = result.size;
    result.flags = .SHARED;
    slot.* = result;

    return result;
}

gm_opaque_block_type :: () -> *CType {
    generator := gm_active_generator();
    session := gm_current_session();

    if !generator || !session return null;
    if generator.type_def_objc_block_pointer return generator.type_def_objc_block_pointer;

    record := GM_Sessions.new_owned(session, Struct);
    record.kind = .STRUCT;
    record.name = GM_Sessions.copy_text(session, "Block");
    record.output_name = GM_Sessions.copy_text(session, "Block");
    record.type = GM_Sessions.new_owned(session, CType);
    record.type.type_of_struct = record;
    record.type.flags = .SHARED;
    record.type.size = gm_pointer_size();
    record.type.alignment = record.type.size;
    generator.type_def_objc_block_pointer = record.type;

    return record.type;
}

gm_function_output_name :: (name: string) -> string {
    options := gm_active_options();
    if options for prefix: options.strip_prefixes {
        if prefix && begins_with(name, prefix) {
            name = slice(name, prefix.count, name.count - prefix.count);
            break;
        }
    }

    for reserved: RESERVED_NAMES if name == reserved return GM_Sessions.copy_text(gm_current_session(), tprint("_%", name));

    return GM_Sessions.copy_text(gm_current_session(), name);
}

gm_objc_type_param :: (cur: CXCursor) -> *Declaration {
    if cur.kind != .TemplateTypeParameter return null;
    return gm_template_formal(cur);
}

gm_template_formal :: (cur: CXCursor) -> *Declaration {
    if cur.kind != .TemplateTypeParameter && cur.kind != .NonTypeTemplateParameter && cur.kind != .TemplateTemplateParameter return null;

    generator := gm_active_generator();
    session := gm_current_session();

    if !generator || !session return null;
    if session.failed return null;

    canonical := clang_getCanonicalCursor(cur);
    found, existing := GM_Table.table_find(*generator.declarations_by_cursor, canonical);

    if found return existing;

    result := GM_Sessions.new_owned(session, Declaration);
    result.kind = .DECLARATION;
    result.name = gm_native_text(clang_getCursorSpelling(cur));
    result.output_name = GM_Sessions.copy_text(session, result.name);
    result.decl_flags |= .TEMPLATE | .IS_CONST;
    if cur.kind == .NonTypeTemplateParameter result.type = gm_build_type(clang_getCursorType(cur), cur, true, 0);
    else result.type = gm_named_canonical_type(*generator.type_def_type, "Type");
    if !result.type return null;

    fill_location(*result.location, cur);
    parent := clang_getCursorSemanticParent(cur);
    parent_found, parent_declaration := GM_Table.table_find(*generator.declarations_by_cursor, clang_getCanonicalCursor(parent));

    if parent_found result.parent = parent_declaration;

    children := native_children(cur);

    defer array_free(children);
    has_default, default_file, default_offset := gm_template_default_location(cur);

    for child: children {
        if has_default && cur.kind == .NonTypeTemplateParameter && clang_isExpression(child.kind) != 0 {
            file: CXFile;
            offset: u32;

            clang_getExpansionLocation(clang_getRangeStart(clang_getCursorExtent(child)), *file, null, null, *offset);
            if file != default_file || offset <= default_offset continue;

            errors_before := session.model_errors;

            result.expression = gm_build_expression(child, cur);
            if !result.expression {
                if !session.failed && session.model_errors == errors_before gm_construction_error("formal template default expression cannot be resolved");
                return null;
            }

            break;
        }
    }

    if !result.expression && has_default {
        if cur.kind == .TemplateTypeParameter {
            has_native_reference := false;

            for child: children {
                if child.kind != .TypeRef continue;

                file: CXFile;
                offset: u32;

                clang_getExpansionLocation(clang_getRangeEnd(clang_getCursorExtent(child)), *file, null, null, *offset);
                if file == default_file && offset > default_offset has_native_reference = true;
            }

            type: *CType;
            // Keep aliases and nominal defaults on their original canonical
            // TypeRef route; query only shapes the C API does not expose.
            if !has_native_reference type = gm_template_default_type(cur, default_file, default_offset);

            if type {
                reference := GM_Sessions.new_owned(session, Type_Reference);
                reference.kind = .TYPE_REFERENCE;
                reference.type = type;
                result.expression = cast(*Expression) reference;
            }
        }
    }

    if !result.expression && has_default {
        if cur.kind == .TemplateTypeParameter {
            for token: string.["const", "volatile", "*", "&", "&&", "[", "(", "<"] {
                if gm_template_parameter_has_token(cur, token) {
                    gm_construction_error("formal type default needs unavailable qualified/composite native type metadata");
                    return null;
                }
            }
        }

        best := clang_getNullCursor();
        best_end: u32;
        ambiguous := false;

        if cur.kind != .NonTypeTemplateParameter for child: children {
            if cur.kind == .TemplateTypeParameter && child.kind != .TypeRef continue;
            if cur.kind == .TemplateTemplateParameter && child.kind != .TemplateRef continue;

            file: CXFile;
            offset: u32;

            clang_getExpansionLocation(clang_getRangeEnd(clang_getCursorExtent(child)), *file, null, null, *offset);
            if file != default_file || offset <= default_offset continue;

            if clang_Cursor_isNull(best) != 0 || offset > best_end {
                best = child;
                best_end = offset;
                ambiguous = false;
            } else if offset == best_end && clang_equalCursors(child, best) == 0 ambiguous = true;
        }

        if ambiguous {
            gm_construction_error("formal type default has ambiguous native reference locations");
            return null;
        }

        if clang_Cursor_isNull(best) == 0 {
            type: *CType;
            if best.kind == .TypeRef type = gm_build_type(clang_getCursorType(best), best, true, 0);
            else {
                native := clang_getCursorReferenced(best);
                if native.kind == .ClassTemplate {
                    declaration := gm_build_declaration(native, clang_getCursorSemanticParent(native), false, true);
                    if declaration type = declaration.type;
                }
            }

            if type {
                reference := GM_Sessions.new_owned(session, Type_Reference);
                reference.kind = .TYPE_REFERENCE;
                reference.type = type;
                result.expression = cast(*Expression) reference;
            }
        }

        if !result.expression {
            gm_construction_error("formal template default lacks a convertible native expression");
            return null;
        }
    }

    if session.failed return null;

    GM_Table.table_add(*generator.declarations_by_cursor, canonical, result);

    return result;
}

gm_objc_parameter_type :: (native: CXType, cur: CXCursor, accurate: bool, index: int, supplied: [] CXCursor) -> *CType {
    owned: [..] CXCursor;

    defer array_free(owned);
    children := supplied;

    if accurate {
        owned = native_children(cur);
        children = owned;
    }

    position := 0;

    for child: children {
        if child.kind != .TypeRef continue;

        parameter_cur := clang_getCursorReferenced(child);
        if parameter_cur.kind != .TemplateTypeParameter continue;

        if position == index {
            parameter := gm_objc_type_param(parameter_cur);
            if !parameter return null;

            result := GM_Sessions.new_owned(gm_current_session(), CType);
            result.type_of_declaration = parameter;
            result.size = clang_Type_getSizeOf(native);
            result.alignment = clang_Type_getAlignOf(native);

            return result;
        }

        position += 1;
    }

    return null;
}

gm_elaborated_type :: (native: CXType, children: [] CXCursor, location: CXCursor) -> *CType, int {
    if children.count < 0 || (children.count > 0 && !children.data) {
        gm_input_error("malformed elaborated-type child view");
        return null, 0;
    }

    type := native;
    if type.kind == .Elaborated type = clang_Type_getNamedType(type);

    if type.kind == .Unexposed {
        dependent, consumed := gm_dependent_type(type, children);
        if dependent return dependent, consumed;

        application, template_consumed := gm_recover_template_type(type, children, location, 0);
        if application return application, template_consumed;

        canonical := clang_getCanonicalType(type);
        if canonical.kind == .Unexposed && clang_equalTypes(type, canonical) != 0 return null, 0;

        type = canonical;
    }

    result := gm_build_type(type, location, false, 0, children, true);
    if !result return null, 0;

    consumed := 0;

    for child: children {
        if child.kind == .NamespaceRef || child.kind == .TypeRef || child.kind == .TemplateRef || child.kind == .ParmDecl || gm_array_bound_position(location, child) >= 0 consumed += 1;
        else break;
    }

    return result, consumed;
}

gm_macro_definition :: (cur: CXCursor) -> *Declaration {
    if cur.kind != .MacroDefinition return null;

    generator := gm_active_generator();
    session := gm_current_session();

    if !generator || !session return null;
    if session.failed return null;
    if !gm_token_unit() return null;
    if clang_Cursor_isMacroBuiltin(cur) != 0 || clang_Cursor_isMacroFunctionLike(cur) != 0 return null;

    spelling := clang_getCursorSpelling(cur);

    defer clang_disposeString(spelling);
    name := to_string(clang_getCString(spelling));
    for ignored: MACROS_TO_IGNORE if name == ignored return null;
    if ends_with(name, "_H") || ends_with(name, "_H_") || ends_with(name, "_INCLUDED") return null;

    canonical := clang_getCanonicalCursor(cur);
    found, existing := GM_Table.table_find(*generator.declarations_by_cursor, canonical);

    if found return existing;

    tokens: *CXToken;
    count: u32;

    clang_tokenize(generator.translation_unit, clang_getCursorExtent(cur), *tokens, *count);

    if count <= 1 {
        if tokens clang_disposeTokens(generator.translation_unit, tokens, count);
        return null;
    }

    GM_Sessions.append_owned(session, *generator.tokens_to_free, Token_Info.{ tokens, count });
    result := GM_Sessions.new_owned(session, Declaration);
    result.kind = .MACRO_DEFINITION;
    result.name = GM_Sessions.copy_text(session, name);
    result.output_name = GM_Sessions.copy_text(session, result.name);
    result.decl_flags |= .IS_CONST;
    fill_location(*result.location, cur);
    gm_fill_native_comment(*result.comment, cur, result.location);
    value: Macro_Value;
    value.tokens = .{ cast(s64) count - 1, tokens + 1 };
    result.expression = cast(*Expression) gm_macro_literal(value, result.location);
    GM_Table.table_add(*generator.declarations_by_cursor, canonical, result);

    return result;
}

gm_objc_parent :: (record: *Struct, type: *CType, name: string) {
    session := gm_current_session();
    if !session || !type return;
    for parent: record.parents if parent && parent.type == type && parent.name == name return;

    entry := GM_Sessions.new_owned(session, Declaration);
    entry.kind = .DECLARATION;
    entry.name = GM_Sessions.copy_text(session, name);
    entry.output_name = GM_Sessions.copy_text(session, entry.name);
    entry.type = type;
    entry.parent = cast(*Declaration) record;
    GM_Sessions.append_owned(session, *record.parents, entry);
}

gm_objc_interface :: (cur: CXCursor, type_only: bool) -> *Struct {
    if cur.kind != .ObjCInterfaceDecl && cur.kind != .ObjCProtocolDecl && cur.kind != .ObjCCategoryDecl return null;

    generator := gm_active_generator();
    session := gm_current_session();

    if !generator || !session return null;
    if session.failed return null;

    canonical := clang_getCanonicalCursor(cur);
    found, existing := GM_Table.table_find(*generator.declarations_by_cursor, canonical);
    record: *Struct;

    if found {
        record = cast(*Struct) existing;
        state := gm_scope_index(session, existing);
        if type_only || state < 0 || session.scope_states[state].stage == .FULL || session.scope_states[state].stage == .BUILDING return record;
    } else {
        record = GM_Sessions.new_owned(session, Struct);
        record.kind = .STRUCT;
        record.name = gm_native_text(clang_getCursorSpelling(cur));
        record.output_name = GM_Sessions.copy_text(session, record.name);
        fill_location(*record.location, cur);
        gm_fill_native_comment(*record.comment, cur, record.location);
        record.type = GM_Sessions.new_owned(session, CType);
        record.type.type_of_struct = record;
        record.type.flags = .SHARED;
        if cur.kind == .ObjCProtocolDecl record.flags |= .OBJC_PROTOCOL;
        if cur.kind == .ObjCInterfaceDecl record.flags |= .OBJC_INTERFACE;
        if cur.kind == .ObjCCategoryDecl record.flags |= .OBJC_CATEGORY;

        GM_Table.table_add(*generator.declarations_by_cursor, canonical, cast(*Declaration) record);
        state: GM_Sessions.Scope_State;
        state.declaration = record;
        array_add(*session.scope_states, state);
    }

    if type_only return record;

    scope_index := gm_scope_index(session, cast(*Declaration) record);
    session.scope_states[scope_index].stage = .BUILDING;

    defer {
        if session.scope_states[scope_index].stage == .BUILDING session.scope_states[scope_index].stage = .SHELL;
    }

    definition := clang_getCursorDefinition(cur);
    if clang_Cursor_isNull(definition) == 0 cur = definition;

    children := native_children(cur);

    defer array_free(children);
    has_superclass := false;

    for child: children {
        if child.kind == .ObjCSuperClassRef || child.kind == .ObjCProtocolRef || child.kind == .ObjCClassRef {
            referenced := clang_getCursorReferenced(child);
            parent := gm_objc_interface(referenced, true);

            if !parent return null;
            if child.kind == .ObjCSuperClassRef has_superclass = true;
            if cur.kind == .ObjCCategoryDecl && child.kind == .ObjCClassRef record.parent = cast(*Declaration) parent;
            else gm_objc_parent(record, parent.type, parent.name);
        }
    }

    if cur.kind == .ObjCProtocolDecl || (cur.kind == .ObjCInterfaceDecl && !has_superclass) gm_objc_parent(record, gm_named_canonical_type(*generator.type_def_objc_id, "ID"), "ID");

    for child: children {
        if child.kind == .TemplateTypeParameter {
            parameter := gm_objc_type_param(child);
            if parameter GM_Sessions.append_owned(session, *record.template_type_params, parameter);
        }

        if child.kind == .ObjCInstanceMethodDecl || child.kind == .ObjCClassMethodDecl {
            method := create_function(child, cur);
            if method GM_Sessions.append_owned(session, *record.declarations, cast(*Declaration) method);
        }
        // Objective-C ivars deliberately do not become C struct fields.
    }

    if session.failed return null;

    session.scope_states[scope_index].stage = .FULL;

    return record;
}
