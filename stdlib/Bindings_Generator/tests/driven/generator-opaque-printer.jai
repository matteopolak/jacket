#import "Basic";
#import "String";
#import "File";
B :: #import "Bindings_Generator";
H :: #import "Hash_Table";

errors: s64;
diagnostics: String_Builder;
selected_function: string;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR {
        errors += 1;
        append(*diagnostics, _text);
    }
}

check_value_use :: (generator: *B.Generator, opaque: *B.Struct) {
    alias: B.Typedef;
    alias.kind = .TYPEDEF;
    alias.output_name = "OpaqueAliasControl";
    alias.type = opaque.type;
    alias_type, array_type, forged_view: B.CType;
    alias_type.type_of_typedef = *alias;
    array_type.array_element_type = opaque.type;
    array_type.array_element_count = 2;
    // A separate type view cannot override the canonical record's unknown layout.
    forged_view.type_of_struct = opaque;
    forged_view.size = 4;
    forged_view.alignment = 4;
    cases: [4] *B.CType;
    cases[0] = opaque.type;
    cases[1] = *alias_type;
    cases[2] = *array_type;
    cases[3] = *forged_view;
    for type: cases {
        for mode: 0..2 {
            builder: String_Builder;

            defer free_buffers(*builder);
            argument: B.Declaration;
            argument.kind = .DECLARATION;
            argument.output_name = "value";
            argument.type = type;
            signature: B.Function_Type;
            signature.return_type = generator.type_def_void;
            defer array_free(signature.arguments);
            before := errors;

            push_context {
                context.logger = capture_error;

                if mode == 0 {
                    signature.return_type = type;
                    B.print_function_type_to_builder(*builder, *signature, null);
                } else if mode == 1 {
                    array_add(*signature.arguments, *argument);
                    B.print_function_type_to_builder(*builder, *signature, null);
                } else B.print_declaration_to_builder(*builder, *argument, null);
            }

            assert(builder.failed && errors > before && builder_string_length(*builder) == 0);
        }
    }

    // An alias declaration identifies a type; it does not allocate or pass it.
    builder: String_Builder;

    defer free_buffers(*builder);
    B.print_typedef(*builder, *alias);
    assert(!builder.failed);

    // Actual canonical pointer signatures remain supported.
    for declaration: generator.global_scope.members {
        if declaration.kind == .FUNCTION && declaration.name == "opaque_identity" {
            B.print_function_type_to_builder(*builder, declaration.type.type_of_function, null);
            assert(!builder.failed);
        }
    }
}

check_overrides :: (generator: *B.Generator, opaque: *B.Struct) {
    for mode: 0..6 {
        builder: String_Builder;
        existing: [..] *B.Declaration;

        defer free_buffers(*builder);
        defer array_free(existing);
        fields := opaque.declarations.count;
        parents := opaque.parents.count;
        virtuals := opaque.virtual_functions.count;
        overlays := opaque.overlays.count;
        entries := generator.declarations_by_cursor.entries.count;

        defer {
            opaque.declarations.count = fields;
            opaque.parents.count = parents;
            opaque.virtual_functions.count = virtuals;
            opaque.overlays.count = overlays;
            generator.declarations_by_cursor.entries.count = entries;
        }

        alignment: int;
        if mode == 0 alignment = 16;
        if mode == 1 opaque.declarations.count = 1;
        if mode == 2 opaque.parents.count = 1;
        if mode == 3 opaque.virtual_functions.count = 1;
        if mode == 4 opaque.overlays.count = 1;
        if mode == 5 generator.declarations_by_cursor.entries.count = 0;
        if mode == 6 generator.declarations_by_cursor.entries.count = 0x7fffffffffffffff;

        before := errors;

        push_context {
            context.logger = capture_error;
            B.print_struct(*builder, opaque, *existing, explicit_alignment = alignment);
        }

        assert(builder.failed && errors > before && builder_string_length(*builder) == 0);
    }
}

check_type_info :: (opaque: *B.Struct) {
    original := opaque.decl_flags;

    defer opaque.decl_flags = original;

    for mode: 0..2 {
        opaque.decl_flags = original;
        if mode != 1 opaque.decl_flags |= .TYPE_INFO_NONE;
        if mode != 0 opaque.decl_flags |= .TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS;

        builder: String_Builder;
        existing: [..] *B.Declaration;

        defer free_buffers(*builder);
        defer array_free(existing);
        B.print_struct(*builder, opaque, *existing);
        assert(!builder.failed);
        text := builder_to_string(*builder);

        defer free(text);
        if mode == 1 assert(contains(text, "#type_info_procedures_are_void_pointers") && !contains(text, "#type_info_none"));
        else assert(contains(text, "#type_info_none") && !contains(text, "#type_info_procedures_are_void_pointers"));
    }
}

native_cursor :: (generator: *B.Generator, record: *B.Struct) -> B.CXCursor {
    for entry: generator.declarations_by_cursor.entries {
        if entry.hash >= H.FIRST_VALID_HASH && entry.value == cast(*B.Declaration) record return entry.key;
    }

    return B.clang_getNullCursor();
}

check_canonical_print :: (options: B.Generate_Bindings_Options) {
    push_context, defer_pop;
    generator: B.Generator;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    assert(B.run_generator(*generator, options));
    opaque, forward, opaque_union: *B.Struct;

    for declaration: generator.global_scope.members {
        if declaration.kind != .STRUCT continue;
        if declaration.name == "Opaque" opaque = cast(*B.Struct) declaration;
        if declaration.name == "Forward" forward = cast(*B.Struct) declaration;
        if declaration.name == "OpaqueUnion" opaque_union = cast(*B.Struct) declaration;
    }

    assert(opaque && forward && opaque_union && opaque != forward);
    assert(opaque.type.type_of_struct == opaque && opaque.type.size == -2 && opaque.type.alignment == -2);
    assert(opaque.declarations.count == 0);
    assert(forward.type.type_of_struct == forward && forward.type.size == 4 && forward.type.alignment == 4);
    assert(forward.declarations.count == 1 && forward.declarations[0].name == "value");
    // The canonical public cursor table provides independent definition evidence;
    // native error codes and an empty member list alone do not provide that proof.
    opaque_cursor := native_cursor(*generator, opaque);
    forward_cursor := native_cursor(*generator, forward);

    assert(opaque_cursor.kind == .StructDecl && forward_cursor.kind == .StructDecl);
    assert(B.clang_Cursor_getTranslationUnit(opaque_cursor) == generator.translation_unit);
    assert(B.clang_isCursorDefinition(opaque_cursor) == 0);
    assert(B.clang_Cursor_isNull(B.clang_getCursorDefinition(opaque_cursor)) != 0);
    native_type := B.clang_getCursorType(opaque_cursor);

    assert(B.clang_Type_getSizeOf(native_type) == -2 && B.clang_Type_getAlignOf(native_type) == -2);
    definition := B.clang_getCursorDefinition(forward_cursor);

    assert(B.clang_Cursor_isNull(definition) == 0 && B.clang_isCursorDefinition(definition) != 0);
    count := generator.extra_infos.count;
    info, created := B.find_or_create_extra_info(opaque.type, cast(*B.Declaration) opaque);

    assert(info == *B.struct_extra_info_unknown && created == null && generator.extra_infos.count == count);
    builder: String_Builder;
    existing: [..] *B.Declaration;

    defer free_buffers(*builder);
    defer array_free(existing);
    B.print_struct(*builder, opaque, *existing);
    assert(!builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(contains(text, "Opaque") && !contains(text, "#align") && !contains(text, "#assert"));
    assert(opaque.type.size == -2 && opaque.type.alignment == -2 && opaque.declarations.count == 0);
    assert(generator.extra_infos.count == count && !context._bindings_generator_session.failed);
    info, created = B.find_or_create_extra_info(opaque.type, cast(*B.Declaration) opaque);
    assert(info == *B.struct_extra_info_unknown && created == null && generator.extra_infos.count == count);
    assert((opaque_union.flags & .IS_UNION) != 0);
    assert(opaque_union.type.size == -2 && opaque_union.type.alignment == -2 && opaque_union.declarations.count == 0);
    union_cursor := native_cursor(*generator, opaque_union);

    assert(union_cursor.kind == .UnionDecl && B.clang_Cursor_isNull(B.clang_getCursorDefinition(union_cursor)) != 0);
    B.print_struct(*builder, opaque_union, *existing);
    assert(!builder.failed && generator.extra_infos.count == count);
    check_value_use(*generator, opaque);
    check_overrides(*generator, opaque);
    check_type_info(opaque);
}

select_by_value :: (declaration: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if declaration.kind == .FUNCTION && declaration.name != selected_function declaration.decl_flags |= .OMIT_FROM_OUTPUT;
    return .RECURSE;
}

check_by_value_bindings :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%generator-opaque-by-value.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_opaque", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    options.add_generator_command = false;
    options.visitor = select_by_value;
    path := tprint("%../../../../target/generator-opaque-by-value-rejected.jai", #filepath);

    for name: string.["opaque_by_value_return", "opaque_by_value_argument"] {
        selected_function = name;
        reset(*diagnostics);
        assert(write_entire_file(path, "preserved destination\n"));

        push_context {
            context.logger = capture_error;
            assert(!B.generate_bindings(options, path, log_results = false));
        }

        text := builder_to_string(*diagnostics);

        defer free(text);
        expected := ifx name == "opaque_by_value_return" then "cannot be returned by value" else "native argument cannot be emitted by value";

        assert(contains(text, expected));
        preserved, ok := read_entire_file(path);

        defer free(preserved);
        assert(ok && preserved == "preserved destination\n");
    }
}

check_invalid_metadata :: (options: B.Generate_Bindings_Options) {
    push_context, defer_pop;
    generator: B.Generator;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    assert(B.run_generator(*generator, options));

    // Neither invalid metadata nor a forged Incomplete shell has native proof.
    for candidate: 0..1 {
        type: B.CType;
        record: B.Struct;

        if candidate == 1 {
            type.size = -2;
            type.alignment = -2;
        }

        type.type_of_struct = *record;
        record.kind = .STRUCT;
        record.name = "Malformed";
        record.output_name = "Malformed";
        record.type = *type;
        builder: String_Builder;
        existing: [..] *B.Declaration;

        defer free_buffers(*builder);
        defer array_free(existing);
        append(*builder, "preserved prefix");
        before := errors;

        push_context {
            context.logger = capture_error;
            B.print_struct(*builder, *record, *existing);
        }

        text := builder_to_string(*builder);

        defer free(text);
        assert(builder.failed && errors > before && text == "");
        assert(builder_string_length(*builder) == "preserved prefix".count);
    }
}

main :: () {
    defer free_buffers(*diagnostics);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%generator-opaque-printer.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_opaque", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    options.add_generator_command = false;
    check_invalid_metadata(options);
    check_canonical_print(options);
    assert(B.generate_bindings(options, tprint("%../../../../target/generator-opaque-generated.jai", #filepath), log_results = false));
    check_by_value_bindings();
    print("ok\n");
}
