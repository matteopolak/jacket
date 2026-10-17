#scope_module
#import "Basic";
#import "String";

Native_Model :: struct {
    allocations: [..] *void;
    root: Namespace;
    cursors: [..] CXCursor;
    declaration_cursors: [..] CXCursor;
    declarations: [..] *Declaration;
    failed: bool;
    emission_diagnosed: bool;
    unit: CXTranslationUnit;
    index: CXIndex;
    args: [] *u8;
    filename: *u8;
    cpp_functions: [..] *Declaration;
    cpp_cursors: [..] CXCursor;
    string_type: *CType;
    apple_target: bool;
}

Cursor_Capture :: struct {
    saved_context: #Context;
    cursors: [..] CXCursor;
}

capture_native_cursor :: (cursor: CXCursor, _parent: CXCursor, data: CXClientData) -> CXChildVisitResult #c_call {
    capture := cast(*Cursor_Capture) data;

    push_context {
        context = capture.saved_context;
        array_add(*capture.cursors, cursor);
    }

    return .Continue;
}

native_children :: (cursor: CXCursor) -> [..] CXCursor {
    capture: Cursor_Capture;
    capture.saved_context = context;
    clang_visitChildren(cursor, capture_native_cursor, *capture);
    return capture.cursors;
}

model_new :: (model: *Native_Model, $T: Type) -> *T {
    result := New(T);
    array_add(*model.allocations, result);
    return result;
}

model_string :: (model: *Native_Model, native: CXString) -> string {
    defer clang_disposeString(native);
    text := copy_string(to_string(clang_getCString(native)));
    array_add(*model.allocations, text.data);
    return text;
}

model_named :: (model: *Native_Model, name: string, kind: C_Kind) -> *Declaration {
    for declaration: model.root.declarations if declaration.name == name && declaration.kind == kind return declaration;
    return null;
}

model_declaration :: (model: *Native_Model, cursor: CXCursor, kind: C_Kind, parent: *Declaration) -> *Declaration {
    result: *Declaration;
    if kind == .STRUCT result = cast(*Declaration) model_new(model, Struct);
    else if kind == .ENUM result = cast(*Declaration) model_new(model, Enum);
    else if kind == .TYPEDEF result = cast(*Declaration) model_new(model, Typedef);
    else if kind == .FUNCTION result = cast(*Declaration) model_new(model, Function);
    else if kind == .NAMESPACE result = cast(*Declaration) model_new(model, Namespace);
    else result = model_new(model, Declaration);

    result.kind = kind;
    result.name = model_string(model, clang_getCursorSpelling(cursor));
    result.output_name = result.name;
    result.parent = parent;
    fill_location(*result.location, cursor);
    result.comment.text = model_string(model, clang_Cursor_getRawCommentText(cursor));
    result.comment.location = result.location;

    return result;
}

model_type :: (model: *Native_Model, native: CXType, depth := 0) -> *CType {
    result := model_new(model, CType);

    if depth > 64 {
        model.failed = true;
        return result;
    }

    result.size = clang_Type_getSizeOf(native);
    result.alignment = get_clang_type_alignment(native);
    if clang_isConstQualifiedType(native) != 0 result.flags |= .CONST;
    if clang_isVolatileQualifiedType(native) != 0 result.flags |= .VOLATILE;

    if native.kind == .Void {
        result.flags |= .VOID;
        return result;
    }

    if native.kind == .Bool {
        result.flags |= .BOOL;
        return result;
    }

    if native.kind == .Elaborated return model_type(model, clang_getCanonicalType(native), depth + 1);

    if native.kind == .Record || native.kind == .Enum || native.kind == .Typedef {
        cursor := clang_getTypeDeclaration(native);
        name := model_string(model, clang_getCursorSpelling(cursor));
        kind := ifx native.kind == .Record then C_Kind.STRUCT else ifx native.kind == .Enum then C_Kind.ENUM else C_Kind.TYPEDEF;
        declaration := model_cursor_declaration(model, cursor);

        if !declaration {
            if native.kind == .Typedef return model_type(model, clang_getCanonicalType(native), depth + 1);
            log_error("Bindings_Generator: unresolved native type %\n", name);
            model.failed = true;
            return result;
        }

        // A typedef that supplied its anonymous tag's name is not emitted as
        // another declaration. References must use the shared nominal object,
        // so later visitor renames remain visible through every alias/argument.
        if kind == .TYPEDEF && declaration.decl_flags & .OMIT_FROM_OUTPUT
            return model_type(model, clang_getCanonicalType(native), depth + 1);

        if kind == .STRUCT result.type_of_struct = cast(*Struct) declaration;
        if kind == .ENUM result.type_of_enum = cast(*Enum) declaration;
        if kind == .TYPEDEF result.type_of_typedef = cast(*Typedef) declaration;

        return result;
    }

    if native.kind == .LValueReference {
        result.pointer_to = model_type(model, clang_getPointeeType(native), depth + 1);
        result.flags |= .REFERENCE;
        return result;
    }

    if native.kind == .Pointer {
        result.pointer_to = model_type(model, clang_getPointeeType(native), depth + 1);
        return result;
    }

    if native.kind == .ConstantArray {
        result.array_element_count = clang_getArraySize(native);
        result.array_element_type = model_type(model, clang_getArrayElementType(native), depth + 1);
        return result;
    }

    if native.kind == .FunctionProto {
        function := model_new(model, Function_Type);
        function.return_type = model_type(model, clang_getResultType(native), depth + 1);
        count := clang_getNumArgTypes(native);
        for i: 0..count - 1 {
            argument := model_new(model, Declaration);
            argument.kind = .DECLARATION;
            argument.name = copy_string(tprint("arg%", i));
            array_add(*model.allocations, argument.name.data);
            argument.output_name = argument.name;
            argument.type = model_type(model, clang_getArgType(native, cast(u32) i), depth + 1);
            array_add(*function.arguments, argument);
        }

        array_add(*model.allocations, function.arguments.data);
        if clang_isFunctionTypeVariadic(native) != 0 function.flags |= .VARIADIC;

        result.type_of_function = function;

        return result;
    }

    signed := native.kind == .SChar || native.kind == .Short || native.kind == .Int || native.kind == .Long || native.kind == .LongLong;
    unsigned := native.kind == .Char_S || native.kind == .Char_U || native.kind == .UChar || native.kind == .UShort || native.kind == .UInt || native.kind == .ULong || native.kind == .ULongLong;
    floating := native.kind == .Float || native.kind == .Double;

    if signed || unsigned || floating {
        if result.size == 1 result.number_flags |= ._8BIT;
        else if result.size == 2 result.number_flags |= ._16BIT;
        else if result.size == 4 result.number_flags |= ._32BIT;
        else if result.size == 8 result.number_flags |= ._64BIT;
        else model.failed = true;
        if signed result.number_flags |= .SIGNED;
        if floating result.number_flags |= .FLOAT;
        if native.kind == .Char_S || native.kind == .SChar || native.kind == .Char_U || native.kind == .UChar result.number_flags |= .CHAR;

        return result;
    }

    spelling := model_string(model, clang_getTypeSpelling(native));

    log_error("Bindings_Generator: unsupported callback model type %\n", spelling);
    model.failed = true;

    return result;
}

find_probe_expression :: (cursor: CXCursor, depth := 0) -> CXCursor {
    if depth > 64 return clang_getNullCursor();
    if cursor.kind == .VarDecl return cursor;
    if clang_isExpression(cursor.kind) != 0 && clang_getCursorType(cursor).kind != .Void return cursor;

    children := native_children(cursor);

    defer array_free(children);

    for child: children {
        found := find_probe_expression(child, depth + 1);
        if clang_Cursor_isNull(found) == 0 return found;
    }

    return clang_getNullCursor();
}

model_byte_string_extent :: (cursor: CXCursor, depth := 0) -> s64 {
    if depth > 64 return -1;

    if cursor.kind == .StringLiteral {
        type := clang_getCanonicalType(clang_getCursorType(cursor));
        if type.kind != .ConstantArray return -1;

        element := clang_getCanonicalType(clang_getArrayElementType(type));
        if element.kind != .Char_S && element.kind != .Char_U return -1;

        return clang_getArraySize(type);
    }

    if cursor.kind != .ParenExpr && cursor.kind != .UnexposedExpr && cursor.kind != .VarDecl return -1;

    children := native_children(cursor);

    defer array_free(children);
    if children.count == 0 || (cursor.kind != .VarDecl && children.count != 1) return -1;

    return model_byte_string_extent(children[children.count - 1], depth + 1);
}

model_numeric_macro :: (model: *Native_Model, declaration: *Declaration) {
    source: String_Builder;

    defer free_buffers(*source);
    append(*source, "#include ");
    emit_model_quoted(*source, to_string(model.filename));
    print_to_builder(*source, "\nvoid __developer_tools_macro_probe(void);\nvoid __developer_tools_macro_probe(void) { __typeof__((%)) __developer_tools_macro_value = (%); (void)__developer_tools_macro_value; }\n", declaration.name, declaration.name);
    text := builder_to_string(*source);

    defer free(text);
    source_name := to_c_string("developer-tools-macro-probe.c");

    defer free(source_name);
    header_size: u64;
    header := clang_getFileContents(model.unit, clang_getFile(model.unit, model.filename), *header_size);

    if source.failed || !header {
        model.failed = true;
        return;
    }

    unsaved: [2] CXUnsavedFile;
    unsaved[0].Filename = source_name;
    unsaved[0].Contents = text.data;
    unsaved[0].Length = cast(type_of(unsaved[0].Length)) text.count;
    unsaved[1].Filename = model.filename;
    unsaved[1].Contents = header;
    unsaved[1].Length = cast(type_of(unsaved[1].Length)) header_size;
    probe: CXTranslationUnit;
    status := clang_parseTranslationUnit2(model.index, source_name, model.args.data, cast(s32) model.args.count, unsaved.data, 2, 0, *probe);

    defer if probe clang_disposeTranslationUnit(probe);

    if status != .Success || !probe {
        model.failed = true;
        return;
    }

    count := cast(s64) clang_getNumDiagnostics(probe);
    for i: 0..count - 1 {
        diagnostic := clang_getDiagnostic(probe, cast(u32) i);

        defer clang_disposeDiagnostic(diagnostic);

        if clang_getDiagnosticSeverity(diagnostic) >= .Warning {
            message := clang_formatDiagnostic(diagnostic, clang_defaultDiagnosticDisplayOptions());
            defer clang_disposeString(message);
            log_error("Bindings_Generator: %\n", to_string(clang_getCString(message)));
            model.failed = true;
        }
    }

    if model.failed return;

    cursors := native_children(clang_getTranslationUnitCursor(probe));

    defer array_free(cursors);
    expression := clang_getNullCursor();

    for cursor: cursors {
        if cursor.kind != .FunctionDecl continue;

        spelling := clang_getCursorSpelling(cursor);
        is_probe := to_string(clang_getCString(spelling)) == "__developer_tools_macro_probe";

        clang_disposeString(spelling);
        if !is_probe continue;

        candidate := find_probe_expression(cursor);
        if clang_Cursor_isNull(candidate) == 0 expression = candidate;
    }

    if clang_Cursor_isNull(expression) != 0 {
        log_error("Bindings_Generator: macro % is not a supported numeric constant expression\n", declaration.name);
        model.failed = true;
        return;
    }

    evaluated := clang_Cursor_Evaluate(expression);

    defer if evaluated clang_EvalResult_dispose(evaluated);

    if !evaluated {
        log_error("Bindings_Generator: macro % is not a supported numeric constant expression\n", declaration.name);
        model.failed = true;
        return;
    }

    literal := model_new(model, Literal);
    literal.location = declaration.location;
    kind := clang_EvalResult_getKind(evaluated);
    if kind == .Int {
        if clang_EvalResult_isUnsignedInt(evaluated) != 0 {
            literal.literal_kind = .UNSIGNED_INTEGER;
            literal.uint_value = clang_EvalResult_getAsUnsigned(evaluated);
        } else {
            literal.literal_kind = .INTEGER;
            literal.int_value = clang_EvalResult_getAsLongLong(evaluated);
        }
    } else if kind == .Float {
        literal.literal_kind = .FLOAT;
        literal.float_value = clang_EvalResult_getAsDouble(evaluated);
    } else if kind == .StrLiteral {
        native := clang_EvalResult_getAsStr(evaluated);
        text := to_string(native);
        extent := model_byte_string_extent(expression);

        if !native || extent <= 0 || text.count != extent - 1 {
            log_error("Bindings_Generator: macro % is not a supported byte string constant (wide, embedded NUL or nonliteral)\n", declaration.name);
            model.failed = true;
            return;
        }

        literal.literal_kind = .STRING;
        literal.string_value = copy_string(text);
        array_add(*model.allocations, literal.string_value.data);

        if !model.string_type {
            model.string_type = model_new(model, CType);
            model.string_type.hardcoded_jai_string = "string";
            model.string_type.flags = .SHARED;
        }

        declaration.type = model.string_type;
        declaration.expression = cast(*Expression) literal;
        declaration.macro_value_result = .KEEP;

        return;
    } else {
        log_error("Bindings_Generator: macro % is not a supported numeric constant expression\n", declaration.name);
        model.failed = true;
        return;
    }

    declaration.type = model_type(model, clang_getCanonicalType(clang_getCursorType(expression)));
    declaration.expression = cast(*Expression) literal;
    declaration.macro_value_result = .KEEP;
}

model_cursor_declaration :: (model: *Native_Model, cursor: CXCursor) -> *Declaration {
    canonical := clang_getCanonicalCursor(cursor);
    if clang_Cursor_isNull(canonical) != 0 return null;
    for previous, index: model.declaration_cursors
        if clang_equalCursors(clang_getCanonicalCursor(previous), canonical) != 0 return model.declarations[index];

    return null;
}

collect_model_scope :: (model: *Native_Model, cursors: [] CXCursor, scope: *Namespace, depth := 0) {
    if depth > 64 {
        log_error("Bindings_Generator: native declaration scopes exceed supported depth\n");
        model.failed = true;
        return;
    }

    for cursor: cursors {
        if clang_Location_isInSystemHeader(clang_getCursorLocation(cursor)) != 0 continue;

        if cursor.kind == .LinkageSpec {
            children := native_children(cursor);
            defer array_free(children);
            collect_model_scope(model, children, scope, depth + 1);
            continue;
        }

        kind := C_Kind.NONE;
        if cursor.kind == .Namespace kind = .NAMESPACE;
        if cursor.kind == .StructDecl || cursor.kind == .UnionDecl kind = .STRUCT;
        if cursor.kind == .EnumDecl kind = .ENUM;
        if cursor.kind == .TypedefDecl kind = .TYPEDEF;
        if cursor.kind == .FunctionDecl kind = .FUNCTION;

        if cursor.kind == .MacroDefinition {
            file: CXFile;

            clang_getSpellingLocation(clang_getCursorLocation(cursor), *file, null, null, null);
            if !file continue;

            name := model_string(model, clang_getCursorSpelling(cursor));
            ignored := false;

            for candidate: MACROS_TO_IGNORE if candidate == name ignored = true;
            if ignored || model_named(model, name, .MACRO_DEFINITION) continue;

            tokens: *CXToken;
            count: u32;

            clang_tokenize(model.unit, clang_getCursorExtent(cursor), *tokens, *count);
            defer if tokens clang_disposeTokens(model.unit, tokens, count);
            if count <= 1 continue;

            kind = .MACRO_DEFINITION;
        }

        if kind == .NONE {
            if clang_isDeclaration(cursor.kind) != 0 {
                log_error("Bindings_Generator: unsupported native declaration category %\n", cursor.kind);
                model.failed = true;
            }

            continue;
        }

        if (kind == .STRUCT || kind == .ENUM) && clang_isCursorDefinition(cursor) == 0 && clang_Cursor_isNull(clang_getCursorDefinition(cursor)) == 0 continue;

        existing := model_cursor_declaration(model, cursor);
        if existing {
            if kind == .NAMESPACE {
                children := native_children(cursor);
                defer array_free(children);
                collect_model_scope(model, children, cast(*Namespace) existing, depth + 1);
            }

            continue;
        }

        declaration := model_declaration(model, cursor, kind, cast(*Declaration) scope);
        anonymous_tag := (kind == .STRUCT || kind == .ENUM) && clang_Cursor_isAnonymous(cursor) != 0;

        if !looks_like_valid_identifier(declaration.name) && !anonymous_tag {
            log_error("Bindings_Generator: unsupported native declaration name %\n", declaration.name);
            model.failed = true;
            continue;
        }

        array_add(*scope.declarations, declaration);
        array_add(*model.declarations, declaration);
        array_add(*model.declaration_cursors, cursor);

        if kind == .NAMESPACE {
            children := native_children(cursor);
            defer array_free(children);
            collect_model_scope(model, children, cast(*Namespace) declaration, depth + 1);
        }
    }
}

name_model_anonymous_types :: (model: *Native_Model) {
    // The canonical SDK tag can already carry an inferred typedef spelling.
    // Assign an absent spelling once; later aliases keep their own identity.
    for declaration, index: model.declarations {
        if declaration.kind != .TYPEDEF continue;

        underlying := clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(model.declaration_cursors[index]));
        if underlying.kind != .Record && underlying.kind != .Enum continue;

        cursor := clang_getTypeDeclaration(underlying);
        if !model_tag_is_unnamed(model, cursor) continue;

        nominal := model_cursor_declaration(model, cursor);
        if !nominal continue;

        if !looks_like_valid_identifier(nominal.name) {
            nominal.name = declaration.name;
            nominal.output_name = declaration.output_name;
        }

        if nominal.name == declaration.name declaration.decl_flags |= .OMIT_FROM_OUTPUT;
    }
}

model_tag_is_unnamed :: (model: *Native_Model, cursor: CXCursor) -> bool {
    if clang_Cursor_isAnonymous(cursor) != 0 return true;

    // Clang gives an unnamed typedef tag an inferred identifier and then its
    // anonymous query returns false. Its declaration location still identifies
    // the original struct/union/enum keyword rather than an explicit tag name.
    token := clang_getToken(model.unit, clang_getCursorLocation(cursor));
    if !token return false;

    defer clang_disposeTokens(model.unit, token, 1);
    spelling := clang_getTokenSpelling(model.unit, token.*);

    defer clang_disposeString(spelling);
    text := to_string(clang_getCString(spelling));

    return (cursor.kind == .StructDecl && text == "struct") ||
           (cursor.kind == .UnionDecl && text == "union") ||
           (cursor.kind == .EnumDecl && text == "enum");
}

populate_model :: (model: *Native_Model, unit: CXTranslationUnit) {
    model.unit = unit;
    target := clang_getTranslationUnitTargetInfo(unit);
    if !target {
        log_error("Bindings_Generator: native target information is unavailable\n");
        model.failed = true;
        return;
    }

    triple := clang_TargetInfo_getTriple(target);

    model.apple_target = contains(to_string(clang_getCString(triple)), "-apple-");
    clang_disposeString(triple);
    clang_TargetInfo_dispose(target);
    model.root.kind = .NAMESPACE;
    model.cursors = native_children(clang_getTranslationUnitCursor(unit));
    collect_model_scope(model, model.cursors, *model.root);
    name_model_anonymous_types(model);
    array_add(*model.allocations, model.root.declarations.data);
    for declaration: model.declarations if declaration.kind == .NAMESPACE
        array_add(*model.allocations, (cast(*Namespace) declaration).declarations.data);
    if model.failed return;

    for declaration, declaration_index: model.declarations {
        if declaration.kind == .NAMESPACE continue;

        cursor := model.declaration_cursors[declaration_index];

        if declaration.kind == .MACRO_DEFINITION {
            model_numeric_macro(model, declaration);
            continue;
        }

        if declaration.kind == .TYPEDEF declaration.type = model_type(model, clang_getTypedefDeclUnderlyingType(cursor));
        else if declaration.kind == .ENUM declaration.type = model_type(model, clang_getEnumDeclIntegerType(cursor));
        else if declaration.kind == .STRUCT {
            declaration.type = model_new(model, CType);
            declaration.type.type_of_struct = cast(*Struct) declaration;
            native := clang_getCursorType(cursor);

            declaration.type.size = clang_Type_getSizeOf(native);
            declaration.type.alignment = get_clang_type_alignment(native);
            if declaration.type.size >= 0 && clang_isPODType(native) == 0 {
                log_error("Bindings_Generator: non-POD record modeling requires canonical constructor support (%)\n", declaration.name);
                model.failed = true;
                continue;
            }
        } else declaration.type = model_type(model, clang_getCursorType(cursor));

        if declaration.kind == .FUNCTION {
            mangling := model_string(model, clang_Cursor_getMangling(cursor));

            if model.apple_target && begins_with(mangling, "_") mangling = slice(mangling, 1, mangling.count - 1);

            if !mangling {
                log_error("Bindings_Generator: native function % has no usable foreign symbol\n", declaration.name);
                model.failed = true;
                continue;
            }

            declaration.foreign_name = mangling;

            if begins_with(mangling, "_Z") || begins_with(mangling, "__Z") || begins_with(mangling, "?") {
                array_add(*model.cpp_functions, declaration);
                array_add(*model.cpp_cursors, cursor);
            }

            function := declaration.type.type_of_function;
            if !function {
                model.failed = true;
                continue;
            }

            for argument, i: function.arguments {
                native := clang_Cursor_getArgument(cursor, cast(u32) i);

                argument.name = model_string(model, clang_getCursorSpelling(native));
                if !argument.name {
                    argument.name = copy_string(tprint("arg%", i));
                    array_add(*model.allocations, argument.name.data);
                }

                argument.output_name = argument.name;
                argument.parent = declaration;
            }
        }

        if declaration.kind != .STRUCT && declaration.kind != .ENUM continue;

        children := native_children(cursor);

        defer array_free(children);

        if declaration.kind == .STRUCT {
            record := cast(*Struct) declaration;
            if cursor.kind == .UnionDecl record.flags |= .IS_UNION;

            for child: children {
                if child.kind != .FieldDecl {
                    model.failed = true;
                    continue;
                }

                field := model_declaration(model, child, .DECLARATION, declaration);
                field.type = model_type(model, clang_getCursorType(child));
                field.bit_offset_in_parent = clang_Cursor_getOffsetOfField(child);

                if clang_Cursor_isBitField(child) != 0 {
                    model.failed = true;
                    continue;
                }

                array_add(*record.declarations, field);
            }

            array_add(*model.allocations, record.declarations.data);
        } else {
            enumeration := cast(*Enum) declaration;

            for child: children {
                if child.kind != .EnumConstantDecl {
                    model.failed = true;
                    continue;
                }

                value := model_declaration(model, child, .DECLARATION, declaration);
                literal := model_new(model, Literal);

                if declaration.type.number_flags & .SIGNED {
                    literal.literal_kind = .INTEGER;
                    literal.int_value = clang_getEnumConstantDeclValue(child);
                } else {
                    literal.literal_kind = .UNSIGNED_INTEGER;
                    literal.uint_value = clang_getEnumConstantDeclUnsignedValue(child);
                }

                value.expression = cast(*Expression) literal;
                array_add(*enumeration.enumerates, value);
            }

            array_add(*model.allocations, enumeration.enumerates.data);
        }
    }
}

emit_model_type :: (builder: *String_Builder, type: *CType, depth := 0) -> bool {
    return jai_model_type(builder, type, null, 0, depth);
}

emit_model_function :: (builder: *String_Builder, declaration: *Function, function: *Function_Type, get_arguments: Get_Function_Arguments_For_Printing_Func, depth := 0) -> bool {
    if !function || !get_arguments || depth > 64 return false;

    append(builder, "(");
    arguments := get_arguments(declaration, function, .FOREIGN_FUNC_DECLARATION);
    for argument, i: arguments {
        if i > 0 append(builder, ", ");
        if !argument || !looks_like_valid_identifier(argument.output_name) return false;
        print_to_builder(builder, "%: ", argument.output_name);
        if !emit_model_type(builder, argument.type, depth + 1) return false;
    }

    if function.flags & .VARIADIC {
        if !append_jai_variadic_tail(builder, arguments) return false;
    }

    append(builder, ") -> ");

    return emit_model_type(builder, function.return_type, depth + 1);
}

emit_model_quoted :: (builder: *String_Builder, text: string) {
    append(builder, "\"");

    for c: text {
        if c == #char "\n" append(builder, "\\n");
        else if c == #char "\r" append(builder, "\\r");
        else if c < 32 || c > 126 {
            digits := "0123456789abcdef";
            append(builder, "\\x");
            append(builder, string.{ 1, digits.data + (c >> 4) });
            append(builder, string.{ 1, digits.data + (c & 15) });
        } else {
            if c == #char "\\" || c == #char "\"" append(builder, "\\");
            append(builder, string.{ 1, *c });
        }
    }

    append(builder, "\"");
}

Macro_Group :: struct {
    record: *Enum;
    values: [..] *Declaration;
}

group_numeric_macros :: (model: *Native_Model, options: Generate_Bindings_Options) -> bool {
    if !options.convert_macro_value_to_enum_callback return true;

    roots: [..] *Declaration;
    groups: [..] Macro_Group;
    adopted := false;

    defer {
        if !adopted {
            array_free(roots);
            for group: groups array_free(group.values);
        }

        array_free(groups);
    }

    for declaration: model.root.declarations {
        if declaration.kind != .MACRO_DEFINITION || declaration.macro_value_result == .DISCARD || declaration.decl_flags & .OMIT_FROM_OUTPUT {
            array_add(*roots, declaration);
            continue;
        }

        name := options.convert_macro_value_to_enum_callback(declaration);
        if !name {
            array_add(*roots, declaration);
            continue;
        }

        valid_name := looks_like_valid_identifier(name);
        for reserved: RESERVED_NAMES if reserved == name valid_name = false;
        for reserved: RESERVED_TYPE_NAMES if reserved == name valid_name = false;

        if !valid_name {
            log_error("Bindings_Generator: invalid macro enum name %\n", name);
            return false;
        }

        if !declaration.type || !declaration.expression || declaration.expression.kind != .LITERAL {
            log_error("Bindings_Generator: macro enum needs a typed numeric literal\n");
            return false;
        }

        literal := cast(*Literal) declaration.expression;
        if literal.literal_kind != .INTEGER && literal.literal_kind != .UNSIGNED_INTEGER {
            log_error("Bindings_Generator: macro enum values must have integer type (%)\n", declaration.name);
            return false;
        }

        macro_type := declaration.type;
        if macro_type.type_of_enum macro_type = macro_type.type_of_enum.type;

        if !macro_type || !macro_type.number_flags || macro_type.number_flags & .FLOAT || (macro_type.size != 1 && macro_type.size != 2 && macro_type.size != 4 && macro_type.size != 8) {
            log_error("Bindings_Generator: macro enum needs an ordinary integer underlying type (%)\n", declaration.name);
            return false;
        }

        group_index: s64 = -1;
        for group, i: groups if group.record.name == name group_index = i;

        if group_index < 0 {
            record := cast(*Enum) model_named(model, name, .ENUM);
            if !record {
                for other: model.root.declarations if other.kind != .MACRO_DEFINITION && other.name == name {
                    log_error("Bindings_Generator: macro enum name collides with %\n", name);
                    return false;
                }

                record = model_new(model, Enum);
                record.kind = .ENUM;
                record.name = copy_string(name);
                array_add(*model.allocations, record.name.data);
                record.output_name = record.name;
                record.location = declaration.location;
                record.parent = cast(*Declaration) *model.root;
                record.type = macro_type;
                array_add(*roots, cast(*Declaration) record);
            }

            group: Macro_Group;
            group.record = record;
            for value: record.enumerates array_add(*group.values, value);

            array_add(*groups, group);
            group_index = groups.count - 1;
        }

        group := *groups[group_index];
        if !group.record.type || group.record.type.size != macro_type.size || ((group.record.type.number_flags & .SIGNED) != (macro_type.number_flags & .SIGNED)) {
            log_error("Bindings_Generator: macro enum requires matching native integer widths/signedness (%)\n", name);
            return false;
        }

        array_add(*group.values, declaration);
        declaration.parent = cast(*Declaration) group.record;
    }

    for group: groups {
        group.record.enumerates = group.values;
        array_add(*model.allocations, group.values.data);
    }

    model.root.declarations = roots;
    array_add(*model.allocations, roots.data);
    adopted = true;

    return true;
}

emit_model :: (builder: *String_Builder, model: *Native_Model, options: Generate_Bindings_Options, library: Library) -> bool {
    if !custom_model_names_valid(model) || !custom_cpp_declarations_valid(model) return false;

    if options.generate_library_declarations {
        print_to_builder(builder, "% :: #foreign_library ", library.identifier);
        emit_model_quoted(builder, tprint("%%", options.generated_library_path_prefix, library.filename));
        append(builder, ";\n");
    }

    for declaration: model.root.declarations {
        if declaration.decl_flags & .OMIT_FROM_OUTPUT continue;
        if declaration.kind == .MACRO_DEFINITION && declaration.macro_value_result == .DISCARD continue;
        if !declaration.type || !looks_like_valid_identifier(declaration.output_name) return false;

        if options.try_to_preserve_comments && declaration.comment.text {
            lines := split(declaration.comment.text, "\n");
            defer free(lines.data);
            for line: lines print_to_builder(builder, "// %\n", line);
        }

        if declaration.kind == .TYPEDEF && declaration.type.type_of_struct && declaration.output_name == declaration.type.type_of_struct.output_name continue;

        print_to_builder(builder, "% :: ", declaration.output_name);

        if declaration.kind == .MACRO_DEFINITION {
            if !declaration.expression || declaration.expression.kind != .LITERAL return false;

            literal := cast(*Literal) declaration.expression;

            print_literal_to_builder(builder, literal, declaration.type);
            if builder.failed return false;

            append(builder, ";\n");
        } else if declaration.kind == .STRUCT {
            record := cast(*Struct) declaration;

            if declaration.type.size < 0 {
                append(builder, "struct {} // Native incomplete type; pointers only.\n");
                continue;
            }

            append(builder, ifx record.flags & .IS_UNION then "union {\n" else "struct {\n");

            for field: record.declarations {
                if field.decl_flags & .OMIT_FROM_OUTPUT continue;
                if !looks_like_valid_identifier(field.output_name) return false;

                print_to_builder(builder, "    %: ", field.output_name);
                if !emit_model_type(builder, field.type) return false;

                append(builder, ";\n");
            }

            append(builder, "}\n");

            if options.generate_compile_time_struct_checks && !(record.flags & .NO_STRUCT_CHECKS) {
                print_to_builder(builder, "#assert size_of(%) == %;\n#assert align_of(%) == %;\n", declaration.output_name, declaration.type.size, declaration.output_name, declaration.type.alignment);

                for field: record.declarations {
                    if field.decl_flags & .OMIT_FROM_OUTPUT || field.decl_flags & .OMIT_FROM_CHECKS continue;
                    if field.bit_offset_in_parent < 0 || field.bit_offset_in_parent % 8 != 0 return false;
                    print_to_builder(builder, "#assert offset_of(%, \"%\") == %;\n", declaration.output_name, field.output_name, field.bit_offset_in_parent / 8);
                }
            }
        } else if declaration.kind == .ENUM {
            enumeration := cast(*Enum) declaration;

            append(builder, ifx enumeration.flags & .IS_ENUM_FLAGS then "enum_flags " else "enum ");
            if !emit_model_type(builder, declaration.type) return false;

            append(builder, " {\n");

            for value: enumeration.enumerates {
                if value.decl_flags & .OMIT_FROM_OUTPUT continue;
                if value.kind == .MACRO_DEFINITION && value.macro_value_result == .DISCARD continue;
                if !looks_like_valid_identifier(value.output_name) || !value.expression || value.expression.kind != .LITERAL return false;

                literal := cast(*Literal) value.expression;
                if literal.literal_kind == .INTEGER print_to_builder(builder, "    % :: %;\n", value.output_name, literal.int_value);
                else if literal.literal_kind == .UNSIGNED_INTEGER print_to_builder(builder, "    % :: %;\n", value.output_name, literal.uint_value);
                else return false;
                if (options.alias_original_enum_names || enumeration.flags & .ALIAS_ORIGINAL_NAMES) && value.output_name != value.name print_to_builder(builder, "    % :: %;\n", value.name, value.output_name);
            }

            append(builder, "}\n");
        } else if declaration.kind == .FUNCTION {
            cpp := array_find(model.cpp_functions, declaration);

            if !declaration.type.type_of_function return false;

            for source_declaration, source_index: model.declarations {
                if source_declaration != declaration continue;

                native := clang_getCursorType(model.declaration_cursors[source_index]);
                if ((declaration.type.type_of_function.flags & .VARIADIC) != 0) != (clang_isFunctionTypeVariadic(native) != 0) {
                    log_error("Bindings_Generator: callback changed the native variadic ABI of %\n", declaration.name);
                    model.emission_diagnosed = true;
                    return false;
                }
            }

            if cpp {
                if !emit_custom_cpp_jai(builder, cast(*Function) declaration, options.get_func_args_for_printing) return false;
            } else if !emit_model_function(builder, cast(*Function) declaration, declaration.type.type_of_function, options.get_func_args_for_printing) return false;

            print_to_builder(builder, " #foreign % ", library.identifier);
            emit_model_quoted(builder, ifx cpp then declaration.output_name else ifx declaration.foreign_name then declaration.foreign_name else declaration.name);
            append(builder, ";\n");
        } else {
            if !emit_model_type(builder, declaration.type) return false;
            append(builder, ";\n");
        }
    }

    return !builder.failed;
}

flatten_model_scopes :: (declarations: [] *Declaration, result: *[..] *Declaration, path: *[..] *Declaration) -> bool {
    for declaration: declarations {
        if !declaration return false;
        if declaration.decl_flags & .OMIT_FROM_OUTPUT continue;

        if declaration.kind != .NAMESPACE {
            if !array_find(result.*, declaration) array_add(result, declaration);
            continue;
        }

        if path.count >= 64 || array_find(path.*, declaration) {
            log_error("Bindings_Generator: cyclic or excessive output namespace graph\n");
            return false;
        }

        namespace := cast(*Namespace) declaration;
        if !(namespace.flags & .FLATTEN) {
            log_error("Bindings_Generator: namespace % requires explicit flattened output in the current emitter\n", declaration.name);
            return false;
        }

        array_add(path, declaration);
        valid := flatten_model_scopes(namespace.declarations, result, path);

        path.count -= 1;
        if !valid return false;
    }

    return true;
}

generate_callback_model :: (command: [] string, options: Generate_Bindings_Options, library: Library, bridge := false) -> String_Builder, bool {
    result: String_Builder;
    index := clang_createIndex(0, 0);

    if !index return result, false;

    defer clang_disposeIndex(index);
    args: [..] *u8;

    array_add(*args, to_c_string("-x"));
    array_add(*args, to_c_string(ifx bridge then "c++" else "c"));
    array_add(*args, to_c_string(ifx bridge then "-std=c++17" else "-std=c11"));

    defer {
        for arg: args free(arg);
        array_free(args);
    }

    for argument, i: command {
        if i < 4 || begins_with(argument, "--jai-") continue;
        array_add(*args, to_c_string(argument));
    }

    combined_source: String_Builder;

    defer free_buffers(*combined_source);
    input_name := options.source_files[0];

    if options.source_files.count > 1 {
        // One real TU preserves cross-header types, macro state, redeclarations
        // and caller-supplied include order. No physical temporary file is used.
        for source: options.source_files {
            absolute, resolved := get_absolute_path(source);

            defer free(absolute);

            if !resolved {
                log_error("Bindings_Generator: could not resolve source header %\n", source);
                return result, false;
            }

            for byte: absolute if byte == 0 || byte == #char "\n" || byte == #char "\r" || byte == #char "\"" {
                log_error("Bindings_Generator: resolved source header path contains an unsupported byte\n");
                return result, false;
            }

            print_to_builder(*combined_source, "#include \"%\"\n", absolute);
        }

        input_name = tprint("%.__independent_aggregate__.h", options.source_files[0]);
    }

    if combined_source.failed {
        log_error("Bindings_Generator: could not construct aggregate header input\n");
        return result, false;
    }

    filename := to_c_string(input_name);

    defer free(filename);
    combined_text := builder_to_string(*combined_source);

    defer free(combined_text);
    unsaved: CXUnsavedFile;
    #if size_of(type_of(unsaved.Length)) == 4 {
        if combined_text.count > 0xffffffff {
            log_error("Bindings_Generator: aggregate source exceeds the target libclang input length\n");
            return result, false;
        }
    }

    unsaved.Filename = filename;
    unsaved.Contents = combined_text.data;
    unsaved.Length = cast(type_of(unsaved.Length)) combined_text.count;
    unit: CXTranslationUnit;
    status := clang_parseTranslationUnit2(index, filename, args.data, cast(s32) args.count,
                                        ifx options.source_files.count > 1 then *unsaved else null,
                                        ifx options.source_files.count > 1 then 1 else 0, 1, *unit);

    if !unit || status != .Success {
        if unit clang_disposeTranslationUnit(unit);
        return result, false;
    }

    defer clang_disposeTranslationUnit(unit);
    diagnostic_count := cast(s64) clang_getNumDiagnostics(unit);
    for i: 0..diagnostic_count - 1 {
        diagnostic := clang_getDiagnostic(unit, cast(u32) i);

        defer clang_disposeDiagnostic(diagnostic);

        if clang_getDiagnosticSeverity(diagnostic) >= .Warning {
            text := clang_formatDiagnostic(diagnostic, clang_defaultDiagnosticDisplayOptions());
            defer clang_disposeString(text);
            log_error("Bindings_Generator: %\n", to_string(clang_getCString(text)));
            return result, false;
        }
    }

    model: Native_Model;
    model.index = index;
    model.args = args;
    model.filename = filename;

    defer {
        for allocation: model.allocations free(allocation);

        array_free(model.allocations);
        array_free(model.cursors);
        array_free(model.declaration_cursors);
        array_free(model.declarations);
        array_free(model.cpp_functions);
        array_free(model.cpp_cursors);
    }

    populate_model(*model, unit);

    if model.failed {
        log_error("Bindings_Generator: callback model materialization failed\n");
        return result, false;
    }

    if !group_numeric_macros(*model, options) return result, false;

    if options.strip_enum_prefixes && options.auto_detect_enum_prefixes {
        for declaration: model.root.declarations {
            if declaration.kind != .ENUM continue;

            enumeration := cast(*Enum) declaration;
            if !enumeration.enumerates.count continue;

            prefix := enumeration.enumerates[0].name;

            for value: enumeration.enumerates {
                count: s64;
                while count < prefix.count && count < value.name.count && prefix[count] == value.name[count] count += 1;
                prefix.count = count;
            }

            boundary: s64;
            for c, i: prefix if c == #char "_" boundary = i + 1;
            if !boundary continue;

            for value: enumeration.enumerates {
                shortened := slice(value.name, boundary, value.name.count - boundary);
                if looks_like_valid_identifier(shortened) value.output_name = shortened;
            }
        }
    }

    for declaration: model.root.declarations {
        if options.visitor visit_declarations(options.visitor, declaration, cast(*Declaration) *model.root);
        if options.visitor_with_data visit_declarations(options.visitor_with_data, options.visitor_data, declaration, cast(*Declaration) *model.root);
    }

    flattened: [..] *Declaration;
    namespace_path: [..] *Declaration;

    defer array_free(namespace_path);

    if !flatten_model_scopes(model.root.declarations, *flattened, *namespace_path) {
        array_free(flattened);
        return result, false;
    }

    model.root.declarations = flattened;
    array_add(*model.allocations, flattened.data);

    if options.will_print_bindings options.will_print_bindings();

    emitted := ifx bridge then emit_custom_cpp_bridge(*result, *model, options) else emit_model(*result, *model, options, library);
    if !emitted {
        free_buffers(*result);
        result = .{};
        if !model.emission_diagnosed log_error("Bindings_Generator: unsupported callback output model\n");
        return result, false;
    }

    return result, true;
}
