#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

Capture :: struct {
    saved: #Context;
    macros: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    if cur.kind != .MacroDefinition return .Continue;

    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        name := B.clang_getCursorSpelling(cur);
        defer B.clang_disposeString(name);
        if begins_with(to_string(B.clang_getCString(name)), "OWN_CAST_") array_add(*capture.macros, cur);
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
        assert(generator.clang_index != null);
        filename := to_c_string(tprint("%bindings-cast-tokens.h", #filepath));

        defer free(filename);
        arguments := (*u8).[to_c_string("-x"), to_c_string("c")];

        defer for argument: arguments free(argument);
        assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 2, null, 0, cast(u32) B.CXTranslationUnit_Flags.DetailedPreprocessingRecord, *generator.translation_unit) == .Success);
        capture: Capture;
        capture.saved = context;
        B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), visit, *capture);
        defer array_free(capture.macros);
        assert(capture.macros.count == 4);
        builder: String_Builder;

        defer free_buffers(*builder);

        for cur, index: capture.macros {
            pointer: *B.CXToken;
            count: u32;

            B.clang_tokenize(generator.translation_unit, B.clang_getCursorExtent(cur), *pointer, *count);
            defer B.clang_disposeTokens(generator.translation_unit, pointer, count);
            tokens := ([] B.CXToken).{ count, pointer };
            success, last := B.maybe_append_cast(*builder, tokens, 1, null);

            if index == 3 {
                assert(!success && last == -1 && builder_string_length(*builder) == 0);
                continue;
            }

            assert(success && last > 1 && B.is_token(tokens[last], ")"));
            text := builder_to_string(*builder);

            defer free(text);

            if index == 0 {
                assert(contains(text, "cast,truncate(s32)"));
                enumeration: B.Enum;
                enumeration.type = generator.type_def_s32;
                stripped, stripped_last := B.maybe_append_cast(*builder, tokens, 1, *enumeration);

                assert(stripped && stripped_last == last && builder_string_length(*builder) == 0);
            } else assert(contains(text, "cast(*void)"));
        }
    }

    print("ok\n");
}
