#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    saved: #Context;
    cursors: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        array_add(*capture.cursors, cur);
    }

    return .Continue;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        S.activate(*generator, *options, context._bindings_generator_session);
        generator.clang_index = B.clang_createIndex(0, 0);
        filename := to_c_string(tprint("%bindings-canonical-printing.h", #filepath));

        defer free(filename);
        arguments := (*u8).[to_c_string("-x"), to_c_string("c")];

        defer for argument: arguments free(argument);
        parsed := B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 2, null, 0, 0, *generator.translation_unit);

        assert(parsed == .Success && B.clang_getNumDiagnostics(generator.translation_unit) == 0);
        capture: Capture;
        capture.saved = context;
        root := B.clang_getTranslationUnitCursor(generator.translation_unit);

        B.clang_visitChildren(root, visit, *capture);
        defer array_free(capture.cursors);
        output: String_Builder;

        defer free_buffers(*output);
        append(*output, "native :: #foreign_library \"canonical_printing\";\n");
        library: B.Library_Info;
        library.identifier = "native";
        literal: B.Literal;
        literal.literal_kind = .INTEGER;
        literal.int_value = 5;

        for cur: capture.cursors {
            declaration := B.create_declaration(cur, root);

            assert(declaration != null);
            declaration.library = *library;
            if declaration.kind == .FUNCTION {
                B.fix_macos_foreign_name(*declaration.foreign_name);
                if declaration.name == "original_sum" declaration.type.type_of_function.arguments[1].expression = cast(*B.Expression) *literal;
            }

            if declaration.kind == .DECLARATION declaration.foreign_name = declaration.name;

            B.print_declaration_to_builder(*output, declaration, null, .IS_GLOBAL);
            assert(!output.failed);
        }

        namespace: B.Namespace;
        namespace.kind = .NAMESPACE;
        namespace.output_name = "Constants";
        constant: B.Declaration;
        constant.kind = .DECLARATION;
        constant.output_name = "Answer";
        constant.type = generator.type_def_s32;
        constant.expression = cast(*B.Expression) *literal;
        constant.decl_flags = .IS_CONST;
        constant.parent = cast(*B.Declaration) *namespace;
        array_add(*namespace.declarations, *constant);
        defer array_free(namespace.declarations);
        B.print_namespace(*output, *namespace);
        assert(!output.failed && generator.counts.structs == 2 && generator.counts.functions == 4 && generator.counts.enums == 1);
        text := builder_to_string(*output);

        defer free(text);
        assert(write_entire_file(tprint("%/developer-tools-canonical-generated.jai", args[1]), text));
    }

    print("ok\n");
}
