fill_template_arguments :: (ty: CXType, arguments: *[..] *Expression, children: [] CXCursor, location_cur: CXCursor) -> bool, int {
    if !arguments return false, 0;

    native := ty;
    if native.kind == .Elaborated native = clang_Type_getNamedType(native);
    if clang_Type_getNumTemplateArguments(native) < 0 return false, 0;

    application, consumed := create_elaborated_type(ty, children, location_cur);
    if !application return false, 0;
    for parameter: application.template_instantiation_params array_add(arguments, parameter);

    return true, consumed;
}

fill_template_arguments_for_expression :: (expr: *Literal, arguments: *[..] *Expression, children: [] CXCursor, parent: CXCursor) -> bool {
    if !expr || expr.literal_kind != .IDENTIFIER || !expr.identifier || !arguments || children.count < 0 || (children.count && !children.data) return false;

    session := gm_current_session();
    if !session return false;

    initial_reference := false;

    for child: children {
        if child.kind == .NamespaceRef continue;

        if child.kind == .TemplateRef {
            if initial_reference || find_referenced_declaration(child) != expr.identifier {
                log_error("Bindings_Generator: ambiguous nested template-reference arguments\n");
                return false;
            }

            initial_reference = true;
            continue;
        }

        converted: *Expression;

        if child.kind == .TypeRef {
            type := create_type(clang_getCursorType(child), child);
            if !type return false;

            reference := GM_Sessions.new_owned(session, Type_Reference);
            reference.kind = .TYPE_REFERENCE;
            reference.type = type;
            converted = cast(*Expression) reference;
        } else {
            expression, skip := create_expression_from_cursor(child, parent);
            if !skip converted = expression;
        }

        if !converted {
            type := GM_Sessions.new_owned(session, CType);
            type.hardcoded_jai_string = "__Unsupported_Template_Argument";
            reference := GM_Sessions.new_owned(session, Type_Reference);
            reference.kind = .TYPE_REFERENCE;
            reference.type = type;
            array_add(arguments, cast(*Expression) reference);
            log_error("Bindings_Generator: unsupported native template argument child\n");

            return false;
        }

        array_add(arguments, converted);
    }

    return true;
}

as_str :: (cur: CXCursor, all_info := false) -> string {
    spelling := clang_getCursorSpelling(cur);

    defer clang_disposeString(spelling);
    text := to_string(clang_getCString(spelling));
    if !all_info return copy_string(text);

    kind := clang_getCursorKindSpelling(cur.kind);

    defer clang_disposeString(kind);
    type := as_str(clang_getCursorType(cur));

    defer free(type);

    return sprint("% (%, %)", text, to_string(clang_getCString(kind)), type);
}

as_str :: (ty: CXType) -> string {
    spelling := clang_getTypeSpelling(ty);
    defer clang_disposeString(spelling);
    return copy_string(to_string(clang_getCString(spelling)));
}

to_string :: (location: Location) -> string {
    generator := gp_current_generator();

    assert(generator != null && location.file != null, "Bindings_Generator: location requires live native file and generator");
    found, filename := GM_Table.table_find(*generator.filenames, location.file);
    if !found {
        native := clang_getFileName(location.file);

        defer clang_disposeString(native);
        session := gm_current_session();

        assert(session != null, "Bindings_Generator: filename cache needs active ownership");
        filename = GM_Sessions.copy_text(session, to_string(clang_getCString(native)));
        to_lower_in_place(filename);
        GM_Table.table_add(*generator.filenames, location.file, filename);
    }

    return sprint("%:%:%", filename, location.line, location.column);
}

log :: (cur: CXCursor, format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0) {
    location: Location;
    fill_location(*location, cur);
    log(location, format_string, ..args, loc = loc, flags = flags, user_flags = user_flags);
}

log_error :: (cur: CXCursor, format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0) {
    location: Location;
    fill_location(*location, cur);
    log(location, format_string, ..args, loc = loc, flags = flags | .ERROR, user_flags = user_flags);
}

log :: (location: Location, format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0) {
    prefix := to_string(location);
    defer free(prefix);
    body := tprint(format_string, ..args);
    Binding_Diagnostic_Basic.log("%: %", prefix, body, loc = loc, flags = flags, user_flags = user_flags);
}

log_error :: (location: Location, format_string: string, args: ..Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0) {
    log(location, format_string, ..args, loc = loc, flags = flags | .ERROR, user_flags = user_flags);
}

raw_tokens_string_from_cursor :: (cur: CXCursor) -> string {
    if clang_Cursor_isNull(cur) != 0 {
        log_error("Bindings_Generator: cannot reconstruct tokens from a null cursor\n");
        return "";
    }

    unit := clang_Cursor_getTranslationUnit(cur);
    if !unit {
        log_error("Bindings_Generator: cursor token reconstruction requires a live translation unit\n");
        return "";
    }

    tokens: *CXToken;
    count: u32;

    clang_tokenize(unit, clang_getCursorExtent(cur), *tokens, *count);
    defer if tokens clang_disposeTokens(unit, tokens, count);

    if count && !tokens {
        log_error("Bindings_Generator: libclang did not provide cursor token storage\n");
        return "";
    }

    builder: String_Builder;

    defer free_buffers(*builder);

    if !count {
        token := clang_getToken(unit, clang_getCursorLocation(cur));
        if !token return "";

        defer clang_disposeTokens(unit, token, 1);
        spelling := clang_getTokenSpelling(unit, token.*);

        defer clang_disposeString(spelling);

        return copy_string(to_string(clang_getCString(spelling)));
    }

    for index: 0..cast(int) count - 1 {
        token := tokens[index];
        // This API returns concatenated spellings, not reconstructed source.
        if clang_getTokenKind(token) == .Comment continue;

        spelling := clang_getTokenSpelling(unit, token);
        text := to_string(clang_getCString(spelling));

        append(*builder, text);
        clang_disposeString(spelling);
    }

    return builder_to_string(*builder);
}

#scope_module

Binding_Evaluation_Capture :: struct {
    saved_context: #Context;
    name: string;
    cursor: CXCursor;
}

binding_evaluation_cursor :: (cur: CXCursor, _parent: CXCursor, data: CXClientData) -> CXChildVisitResult #c_call {
    capture := cast(*Binding_Evaluation_Capture) data;

    push_context {
        context = capture.saved_context;
        if cur.kind != .VarDecl return .Continue;

        spelling := clang_getCursorSpelling(cur);

        defer clang_disposeString(spelling);
        if to_string(clang_getCString(spelling)) == capture.name capture.cursor = cur;
    }

    return .Continue;
}

binding_open_evaluation_probe :: (input: *GM_Sessions.Parse_Input, expression: string) -> CXTranslationUnit, CXCursor, available: bool, supported: bool {
    empty := clang_getNullCursor();
    generator := gp_current_generator();

    if !input || !generator || !generator.clang_index || !generator.translation_unit || !input.filename || !expression {
        log_error("Bindings_Generator: macro evaluation requires exact live canonical parse inputs\n");
        return null, empty, false, false;
    }

    if input.arguments.count < 0 || input.arguments.count > 0x7fffffff || (input.arguments.count && !input.arguments.data) || input.overlays.count < 0 || input.overlays.count > 0xfffffffe || (input.overlays.count && !input.overlays.data) {
        log_error("Bindings_Generator: malformed macro evaluation parse-input receipt\n");
        return null, empty, false, false;
    }

    if !binding_parse_text_valid(input.filename, false) {
        log_error("Bindings_Generator: malformed actual native source filename\n");
        return null, empty, false, false;
    }

    for overlay: input.overlays {
        if !binding_parse_text_valid(overlay.filename, false) || overlay.contents.count < 0 || (overlay.contents.count && !overlay.contents.data) {
            log_error("Bindings_Generator: malformed actual native unsaved overlay\n");
            return null, empty, false, false;
        }
    }

    for index: 0..cast(s64) clang_getNumDiagnostics(generator.translation_unit) - 1 {
        diagnostic := clang_getDiagnostic(generator.translation_unit, cast(u32) index);

        defer clang_disposeDiagnostic(diagnostic);
        severity := clang_getDiagnosticSeverity(diagnostic);
        if severity == .Error || severity == .Fatal {
            log_error("Bindings_Generator: original parse has native errors; evaluation configuration unavailable\n");
            return null, empty, false, false;
        }
    }

    primary: string;
    found_primary := false;

    for overlay: input.overlays if overlay.filename == input.filename {
        primary = overlay.contents;
        found_primary = true;
    }

    strings: [..] *u8;
    arguments: [..] *u8;
    overlays: [..] CXUnsavedFile;

    defer {
        for text: strings free(text);
        array_free(strings);
        array_free(arguments);
        array_free(overlays);
    }

    filename := to_c_string(input.filename);

    array_add(*strings, filename);

    if !found_primary {
        file := clang_getFile(generator.translation_unit, filename);
        count: u64;
        data := clang_getFileContents(generator.translation_unit, file, *count);

        if !file || (!data && count > 0) || count > 0x7fffffffffffffff {
            log_error("Bindings_Generator: actual primary parse bytes unavailable for macro evaluation\n");
            return null, empty, false, false;
        }

        primary = .{ cast(s64) count, data };
    }

    if primary.count < 0 || (primary.count && !primary.data) {
        log_error("Bindings_Generator: malformed primary parse bytes\n");
        return null, empty, false, false;
    }

    children := native_children(clang_getTranslationUnitCursor(generator.translation_unit));

    defer array_free(children);
    name: string;

    defer free(name);

    for suffix: 0..children.count + 1 {
        candidate := sprint("__jai_bindings_evaluation_%", suffix);
        collision := candidate == expression;

        for child: children {
            spelling := clang_getCursorSpelling(child);
            defer clang_disposeString(spelling);
            if to_string(clang_getCString(spelling)) == candidate collision = true;
        }

        if !collision {
            name = candidate;
            break;
        }

        free(candidate);
    }

    if !name return null, empty, false, false;

    source: String_Builder;

    defer free_buffers(*source);
    append(*source, primary);
    print_to_builder(*source, "\n#undef %\n__typeof__((%)) % = %;\n", name, expression, name, expression);
    if source.failed return null, empty, false, false;

    contents := builder_to_string(*source);

    defer free(contents);

    for argument: input.arguments {
        if !binding_parse_text_valid(argument, true) {
            log_error("Bindings_Generator: malformed actual native parse argument\n");
            return null, empty, false, false;
        }

        native := to_c_string(argument);

        array_add(*strings, native);
        array_add(*arguments, native);
    }

    for overlay: input.overlays {
        if overlay.filename == input.filename continue;

        native: CXUnsavedFile;
        native.Filename = to_c_string(overlay.filename);
        array_add(*strings, native.Filename);
        native.Contents = overlay.contents.data;
        native.Length = cast(type_of(native.Length)) overlay.contents.count;
        array_add(*overlays, native);
    }

    primary_overlay: CXUnsavedFile;
    primary_overlay.Filename = filename;
    primary_overlay.Contents = contents.data;
    primary_overlay.Length = cast(type_of(primary_overlay.Length)) contents.count;
    array_add(*overlays, primary_overlay);
    unit: CXTranslationUnit;
    status := clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, cast(s32) arguments.count, overlays.data, cast(u32) overlays.count, 0, *unit);

    if status != .Success || !unit {
        if unit clang_disposeTranslationUnit(unit);
        log_error("Bindings_Generator: evaluation parse failed with native status %\n", status);
        return null, empty, false, false;
    }

    for index: 0..cast(s64) clang_getNumDiagnostics(unit) - 1 {
        diagnostic := clang_getDiagnostic(unit, cast(u32) index);

        defer clang_disposeDiagnostic(diagnostic);
        severity := clang_getDiagnosticSeverity(diagnostic);
        if severity == .Error || severity == .Fatal {
            options := gp_current_options();
            if options && options.log_unsupported {
                text := clang_formatDiagnostic(diagnostic, clang_defaultDiagnosticDisplayOptions());
                defer clang_disposeString(text);
                log_error("Bindings_Generator: unsupported macro evaluation: %\n", to_string(clang_getCString(text)));
            }

            return unit, empty, true, false;
        }
    }

    capture: Binding_Evaluation_Capture;
    capture.saved_context = context;
    capture.name = name;
    capture.cursor = empty;
    clang_visitChildren(clang_getTranslationUnitCursor(unit), binding_evaluation_cursor, *capture);

    return unit, capture.cursor, true, clang_Cursor_isNull(capture.cursor) == 0;
}

binding_parse_text_valid :: (text: string, allow_empty: bool) -> bool {
    if text.count < 0 || (text.count && !text.data) || (!allow_empty && !text.count) return false;
    for byte: text if byte == 0 return false;
    return true;
}

#scope_file
#import "Basic";
#import "String";
Binding_Diagnostic_Basic :: #import "Basic";
