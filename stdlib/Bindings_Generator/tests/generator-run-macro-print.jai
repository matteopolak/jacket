// Exact-input public reducer for checked Macro_Value printing. Private access
// configures only the canonical receipt; it supplies no alternate evaluator.
#import "Basic";
B :: #import "Bindings_Generator";
S :: #import "internal/developer-tools/generator-session";

Capture :: struct {
    cursor: B.CXCursor;
    saved: #Context;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context, defer_pop;
    context = capture.saved;
    native := B.clang_getCursorSpelling(cur);

    defer B.clang_disposeString(native);
    if to_string(B.clang_getCString(native)) == "RUN_PRINT" capture.cursor = cur;

    return .Continue;
}

main :: () {
    push_context, defer_pop;
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    S.activate(*generator, *options, context._bindings_generator_session);
    filename := "owned-generator-run-print.c";
    source := "#define RUN_PRINT 17\n";
    input: S.Parse_Input;
    input.filename = filename;
    input.arguments = string.["-x", "c"];
    input.overlays = S.Source_Overlay.[.{ filename = filename, contents = source }];
    context._bindings_generator_session.parse_input = *input;
    native_name := to_c_string(filename);
    language_flag := to_c_string("-x");
    language := to_c_string("c");

    defer free(native_name);
    defer free(language_flag);
    defer free(language);
    arguments := (*u8).[language_flag, language];
    overlay: B.CXUnsavedFile;
    overlay.Filename = native_name;
    overlay.Contents = source.data;
    overlay.Length = cast(type_of(overlay.Length)) source.count;
    generator.clang_index = B.clang_createIndex(0, 0);
    error := B.clang_parseTranslationUnit2(generator.clang_index, native_name, arguments.data, 2, *overlay, 1, 1, *generator.translation_unit);

    assert(error == .Success && generator.translation_unit != null);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), visit, *capture);
    declaration := B.create_macro_definition(capture.cursor);

    assert(declaration != null);
    B.check_macro_value(declaration);
    assert(declaration.macro_value_result == .KEEP);
    builder: String_Builder;

    defer free_buffers(*builder);
    B.print_expression_to_builder(*builder, declaration.expression);
    assert(!builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(text == "17");
    print("macro-print ok\n");
}
