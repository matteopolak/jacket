#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    saved: #Context;
    source_file: B.CXFile;
    children: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;

        if cur.kind == .MacroDefinition {
            location: B.Location;
            B.fill_location(*location, cur);
            if location.file == capture.source_file array_add(*capture.children, cur);
        }
    }

    return .Continue;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-macros.h", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 1, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    capture: Capture;
    capture.saved = context;
    capture.source_file = B.clang_getFile(unit, filename);
    root := B.clang_getTranslationUnitCursor(unit);

    B.clang_visitChildren(root, visit, *capture);
    defer array_free(capture.children);
    assert(capture.children.count == 4);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);

    push_context {
        context.generator = *generator;
        context._bindings_generator_session = *session;
        assert(!B.create_macro_definition(capture.children[0]));
        value := B.create_macro_definition(capture.children[1]);

        assert(value != null && value.name == "ORIGINAL_MODEL_VALUE" && value.kind == .MACRO_DEFINITION);
        assert(value.expression.kind == .LITERAL && (cast(*B.Literal) value.expression).literal_kind == .MACRO);
        literal := cast(*B.Literal) value.expression;

        assert(literal.macro_value.tokens.count == 5);
        assert(B.is_token(literal.macro_value.tokens[0], "("));
        assert(B.find_closing_paren_token_index(literal.macro_value.tokens, 1) == 4);
        assert(B.create_macro_definition(capture.children[1]) == value);
        assert(!B.create_macro_definition(capture.children[2]));
        assert(!B.create_macro_definition(capture.children[3]));
        assert(generator.tokens_to_free.count == 1 && !session.failed);
    }

    for info: generator.tokens_to_free B.clang_disposeTokens(unit, info.token_pointer, info.num_tokens);

    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
