#scope_module
#import "Basic";

// Eligibility screen only: the SDK still parses the type-id. Without a parse
// receipt, identifiers/macros and configuration-sensitive keywords cannot be
// replayed truthfully from target information alone.
gm_default_context_independent :: (cur: CXCursor, equal_offset: u32) -> bool {
    tokens: *CXToken;
    count: u32;
    unit := gm_token_unit();

    clang_tokenize(unit, clang_getCursorExtent(cur), *tokens, *count);
    defer if tokens clang_disposeTokens(unit, tokens, count);

    for i: 0..cast(s64) count - 1 {
        offset: u32;

        clang_getExpansionLocation(clang_getTokenLocation(unit, tokens[i]), null, null, null, *offset);
        if offset <= equal_offset continue;

        kind := clang_getTokenKind(tokens[i]);
        if kind == .Identifier return false;
        if kind != .Keyword continue;

        supported := false;

        for word: string.["void", "bool", "signed", "unsigned", "short", "int", "float", "double", "const", "volatile", "__int128"] {
            if gm_is_token(tokens[i], word) supported = true;
        }

        if !supported return false;
    }

    return true;
}

gm_default_bindings_supported :: (cur: CXCursor, equal_offset: u32) -> bool {
    unit := gm_token_unit();
    tokens: *CXToken;
    count: u32;

    clang_tokenize(unit, clang_getCursorExtent(cur), *tokens, *count);
    defer if tokens clang_disposeTokens(unit, tokens, count);

    for i: 0..cast(s64) count - 1 {
        location := clang_getTokenLocation(unit, tokens[i]);
        offset: u32;

        clang_getExpansionLocation(location, null, null, null, *offset);
        if offset <= equal_offset || clang_getTokenKind(tokens[i]) != .Identifier continue;

        binding := clang_getCursor(unit, location);
        referred := clang_getCursorReferenced(binding);

        if binding.kind == .MacroExpansion {
            // Flat object macros are replayable at the declaration point.
            // Indirect/location/counter-dependent expansion is not proven by
            // the public C API and must not silently acquire a different type.
            if referred.kind != .MacroDefinition || clang_Cursor_isMacroBuiltin(referred) != 0 || clang_Cursor_isMacroFunctionLike(referred) != 0 return false;

            replacement: *CXToken;
            replacement_count: u32;

            clang_tokenize(unit, clang_getCursorExtent(referred), *replacement, *replacement_count);
            defer if replacement clang_disposeTokens(unit, replacement, replacement_count);
            if replacement_count <= 1 return false;
            for j: 1..cast(s64) replacement_count - 1 if clang_getTokenKind(replacement[j]) == .Identifier return false;
        } else {
            if clang_Cursor_isNull(referred) != 0 || referred.kind == .TemplateTypeParameter || referred.kind == .NonTypeTemplateParameter || referred.kind == .TemplateTemplateParameter return false;
        }
    }

    return true;
}

// Official SDK type query, not a second declaration model. Preflight excludes
// every shape that would register a nominal cursor from the temporary TU.
gm_default_query_type_supported :: (native: CXType) -> bool {
    pending, visited: [..] CXType;

    defer array_free(pending);
    defer array_free(visited);
    array_add(*pending, native);
    index := 0;
    while index < pending.count {
        type := clang_getCanonicalType(pending[index]);

        index += 1;
        seen := false;
        for previous: visited if previous.kind == type.kind && clang_equalTypes(previous, type) != 0 seen = true;
        if seen continue;

        array_add(*visited, type);
        if gm_scalar_slot(gm_active_generator(), type) continue;

        if type.kind == .Pointer || type.kind == .LValueReference || type.kind == .RValueReference {
            array_add(*pending, clang_getPointeeType(type));
        } else if type.kind == .ConstantArray || type.kind == .IncompleteArray {
            array_add(*pending, clang_getArrayElementType(type));
        } else if type.kind == .FunctionProto || type.kind == .FunctionNoProto {
            array_add(*pending, clang_getResultType(type));
            count := clang_getNumArgTypes(type);
            for i: 0..cast(s64) count - 1 array_add(*pending, clang_getArgType(type, cast(u32) i));
        } else return false;
    }

    return true;
}

gm_template_default_type :: (cur: CXCursor, file: CXFile, equal_offset: u32) -> *CType {
    generator := gm_active_generator();
    session := gm_current_session();

    if !generator || !session || session.failed || !generator.translation_unit return null;

    end_file: CXFile;
    end_offset: u32;

    clang_getExpansionLocation(clang_getRangeEnd(clang_getCursorExtent(cur)), *end_file, null, null, *end_offset);
    bytes: u64;
    data := clang_getFileContents(generator.translation_unit, file, *bytes);

    if file != end_file || !data || end_offset > bytes || equal_offset >= end_offset return null;

    default_text: string = .{ cast(s64) end_offset - equal_offset - 1, data + equal_offset + 1 };
    if !default_text.count return null;

    input := session.parse_input;
    if !input && !gm_default_context_independent(cur, equal_offset) return null;
    if input && !gm_default_bindings_supported(cur, equal_offset) return null;

    declaration := clang_getCursorSemanticParent(cur);
    insertion_file: CXFile;
    insertion_offset: u32;

    clang_getExpansionLocation(clang_getRangeStart(clang_getCursorExtent(declaration)), *insertion_file, null, null, *insertion_offset);

    if input {
        if insertion_file != file || insertion_offset > equal_offset return null;

        // A directive inside the template prefix changes preprocessing before
        // the formal. Such a query needs more than insertion before its parent.
        tokens: *CXToken;
        count: u32;

        clang_tokenize(generator.translation_unit, clang_getCursorExtent(declaration), *tokens, *count);
        defer if tokens clang_disposeTokens(generator.translation_unit, tokens, count);

        for i: 0..cast(s64) count - 1 {
            offset: u32;
            clang_getExpansionLocation(clang_getTokenLocation(generator.translation_unit, tokens[i]), null, null, null, *offset);
            if offset >= equal_offset break;
            if gm_is_token(tokens[i], "#") return null;
        }
    }

    native_filename := clang_getTranslationUnitSpelling(generator.translation_unit);

    defer clang_disposeString(native_filename);
    filename_text := to_string(clang_getCString(native_filename));
    if !filename_text return null;

    filename := to_c_string(filename_text);

    defer free(filename);
    source_name_native := clang_getFileName(file);

    defer clang_disposeString(source_name_native);
    source_name := ifx input then to_string(clang_getCString(source_name_native)) else filename_text;
    if !source_name return null;
    if bytes > 0x7fffffffffffffff return null;

    source_count := ifx input then cast(s64) bytes else 0;

    if !input insertion_offset = 0;

    children := native_children(clang_getCursorSemanticParent(declaration));

    defer array_free(children);
    suffix := 0;
    query_name, result_name: string;

    while true {
        query_name = tprint("__jai_model_default_query_%", suffix);
        result_name = tprint("__jai_model_default_result_%", suffix);
        collision := false;

        for child: children {
            spelling := clang_getCursorSpelling(child);
            defer clang_disposeString(spelling);
            name := to_string(clang_getCString(spelling));
            if name == query_name || name == result_name collision = true;
        }

        if !collision break;

        suffix += 1;
    }

    scaffold := sprint("\ntemplate<typename T> struct % { typedef T type; };\ntypedef %<% >::type %;\n", query_name, query_name, default_text, result_name);

    defer free(scaffold);
    source: String_Builder;

    defer free_buffers(*source);
    if input append(*source, string.{ cast(s64) insertion_offset, data });

    append(*source, scaffold);
    if input append(*source, string.{ source_count - insertion_offset, data + insertion_offset });
    if source.failed return null;

    contents := builder_to_string(*source);

    defer free(contents);
    arguments: [..] *u8;
    overlays: [..] CXUnsavedFile;

    defer {
        for argument: arguments free(argument);
        for overlay: overlays free(overlay.Filename);
        array_free(arguments);
        array_free(overlays);
    }

    if input {
        if !gm_dependency_view_valid(input.overlays) || input.overlays.count > 0xfffffffe ||
           !gm_lifecycle_arguments_valid(input.arguments) || input.arguments.count > 0x7fffffff {
            gm_input_error("template default query has malformed native input receipt");
            return null;
        }

        for argument: input.arguments array_add(*arguments, to_c_string(argument));

        for overlay: input.overlays {
            if overlay.filename == source_name continue;

            if !overlay.filename.count || !gm_lifecycle_arguments_valid(string.[overlay.filename]) || overlay.contents.count < 0 || (overlay.contents.count > 0 && !overlay.contents.data) {
                gm_input_error("template default query has malformed native overlay");
                return null;
            }

            native: CXUnsavedFile;
            native.Filename = to_c_string(overlay.filename);
            if clang_getFile(generator.translation_unit, native.Filename) == file {
                free(native.Filename);
                continue;
            }

            native.Contents = overlay.contents.data;
            native.Length = cast(type_of(native.Length)) overlay.contents.count;
            array_add(*overlays, native);
        }
    } else {
        // Only context-independent builtin syntax reaches this branch. Parse
        // an isolated SDK type query for the caller TU's actual native target.
        target := clang_getTranslationUnitTargetInfo(generator.translation_unit);
        if !target return null;

        defer clang_TargetInfo_dispose(target);
        triple := clang_TargetInfo_getTriple(target);

        defer clang_disposeString(triple);
        array_add(*arguments, to_c_string("-x"), to_c_string("c++"), to_c_string("-target"), to_c_string(to_string(clang_getCString(triple))));
    }

    if arguments.count > 0x7fffffff return null;

    primary: CXUnsavedFile;
    primary.Filename = to_c_string(source_name);
    primary.Contents = contents.data;
    primary.Length = cast(type_of(primary.Length)) contents.count;
    array_add(*overlays, primary);
    index := generator.clang_index;
    owns_index := !index;

    if owns_index index = clang_createIndex(0, 0);
    if !index return null;

    defer if owns_index clang_disposeIndex(index);
    unit: CXTranslationUnit;
    status := clang_parseTranslationUnit2(index, filename, arguments.data, cast(s32) arguments.count, overlays.data, cast(u32) overlays.count, 0, *unit);

    defer if unit clang_disposeTranslationUnit(unit);
    if status != .Success || !unit return null;

    for i: 0..cast(s64) clang_getNumDiagnostics(unit) - 1 {
        diagnostic := clang_getDiagnostic(unit, cast(u32) i);
        defer clang_disposeDiagnostic(diagnostic);
        severity := clang_getDiagnosticSeverity(diagnostic);
        if severity == .Error || severity == .Fatal return null;
    }

    results := native_children(clang_getTranslationUnitCursor(unit));

    defer array_free(results);
    query_file := clang_getFile(unit, primary.Filename);
    original_scope := clang_getCursorSemanticParent(declaration);
    original_scope_usr := clang_getCursorUSR(original_scope);

    defer clang_disposeString(original_scope_usr);
    found := false;
    found_type: CXType;
    result_index := 0;

    while result_index < results.count {
        result := results[result_index];

        result_index += 1;

        if result.kind == .Namespace || result.kind == .StructDecl || result.kind == .ClassDecl || result.kind == .UnionDecl || result.kind == .ClassTemplate {
            nested := native_children(result);
            defer array_free(nested);
            for child: nested array_add(*results, child);
        }

        if result.kind != .TypedefDecl continue;

        result_file: CXFile;
        offset: u32;

        clang_getExpansionLocation(clang_getCursorLocation(result), *result_file, null, null, *offset);
        if result_file != query_file || offset < insertion_offset || offset >= insertion_offset + scaffold.count continue;

        spelling := clang_getCursorSpelling(result);

        defer clang_disposeString(spelling);
        if to_string(clang_getCString(spelling)) != result_name continue;

        scope := clang_getCursorSemanticParent(result);

        if input {
            scope_usr := clang_getCursorUSR(scope);
            defer clang_disposeString(scope_usr);
            if scope.kind != original_scope.kind || to_string(clang_getCString(scope_usr)) != to_string(clang_getCString(original_scope_usr)) continue;
        }

        // Guardless includes can generate multiple aliases at the same source
        // offsets. Never select an arbitrary first matching native result.
        if found return null;

        found = true;
        found_type = clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(result));
    }

    if !found || !gm_default_query_type_supported(found_type) return null;

    // No temporary cursor/parameter/source location enters canonical state.
    return gm_build_type(found_type, clang_getNullCursor(), false, 0);
}
