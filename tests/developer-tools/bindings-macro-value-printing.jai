#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

Parse_Input :: type_of(context._bindings_generator_session.parse_input.*);
errors: int;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

Capture :: struct {
    saved: #Context;
    cursors: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    if cur.kind != .MacroDefinition return .Continue;

    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        spelling := B.clang_getCursorSpelling(cur);
        defer B.clang_disposeString(spelling);
        if begins_with(to_string(B.clang_getCString(spelling)), "OWN_PRINT_") array_add(*capture.cursors, cur);
    }

    return .Continue;
}

main :: () {
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        generator.clang_index = B.clang_createIndex(0, 0);
        path := tprint("%bindings-macro-value-printing.h", #filepath);
        input: Parse_Input;
        input.filename = path;
        input.arguments = string.["-x", "c"];
        context._bindings_generator_session.parse_input = *input;
        filename := to_c_string(path);

        defer free(filename);
        arguments := (*u8).[to_c_string("-x"), to_c_string("c")];

        defer for argument: arguments free(argument);
        assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 2, null, 0, 1, *generator.translation_unit) == .Success);
        capture: Capture;
        capture.saved = context;
        B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), visit, *capture);
        defer array_free(capture.cursors);
        assert(capture.cursors.count == 3);
        builder: String_Builder;

        defer free_buffers(*builder);

        for cursor, index: capture.cursors {
            declaration := B.create_macro_definition(cursor);

            assert(declaration != null);
            B.check_macro_value(declaration);
            assert(declaration.macro_value_result == .KEEP);
            B.print_expression_to_builder(*builder, declaration.expression);
            assert(!builder.failed);
            text := builder_to_string(*builder);

            defer free(text);
            expected := string.["17", "2 + 3 * 4", "-9"];

            assert(text == expected[index]);
            B.print_literal_to_builder(*builder, cast(*B.Literal) declaration.expression, generator.type_def_s32);
            assert(!builder.failed);
            typed := builder_to_string(*builder);

            defer free(typed);
            assert(typed == tprint("cast(s32) (%)", expected[index]));
        }

        // No native token view is available: retain caller bytes and report failure.
        append(*builder, "preserved");
        invalid: B.Literal;
        invalid.literal_kind = .MACRO;

        push_context {
            context.logger = capture_error;
            B.print_literal_to_builder(*builder, *invalid);
        }

        assert(builder.failed && errors == 1);
        assert(builder_string_length(*builder) == "preserved".count);
    }

    print("ok\n");
}
