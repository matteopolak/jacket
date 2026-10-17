#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

errors: int;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    empty: [] B.CXToken;

    assert(B.count_asterisks(empty) == 0);
    assert(B.find_closing_paren_token_index(empty, 0) == -1);

    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string("original-model-token-input.h");

    defer free(filename);
    source :: "int **original(int (*callback)(int));\nint **original(int (*callback)(int));\n";
    contents := to_c_string(source);

    defer free(contents);
    unsaved: B.CXUnsavedFile;
    unsaved.Filename = filename;
    unsaved.Contents = contents;
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, *unsaved, 1, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    tokens: *B.CXToken;
    count: u32;

    B.clang_tokenize(unit, B.clang_getCursorExtent(B.clang_getTranslationUnitCursor(unit)), *tokens, *count);
    assert(count == 30);
    defer B.clang_disposeTokens(unit, tokens, count);
    first: [] B.CXToken = .{ 15, tokens };
    second: [] B.CXToken = .{ 15, tokens + 15 };
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);

    push_context {
        context.generator = session.generator;
        context._bindings_generator_session = *session;
        assert(B.is_token(tokens[0], "int") && !B.is_token(tokens[0], "long"));
        assert(B.count_asterisks(first) == 0);
        leading: [] B.CXToken = .{ 14, tokens + 1 };

        assert(B.count_asterisks(leading) == 2);
        assert(B.find_closing_paren_token_index(first, 5) == 13);
        assert(B.find_closing_paren_token_index(first, 7) == 9);
        assert(B.find_closing_paren_token_index(first, 11) == 12);
        assert(B.find_closing_paren_token_index(first, 4) == -1);
        a: B.Macro_Value = .{ first };
        b: B.Macro_Value = .{ second };

        assert(B.macro_values_are_equal(a, b));
        short: B.Macro_Value = .{ .{ 14, second.data } };

        assert(!B.macro_values_are_equal(a, short));
        literal := B.create_literal(a, .{ line = 7 });

        assert(literal.literal_kind == .MACRO && literal.location.line == 7);
        assert(literal.macro_value.tokens.data != first.data);
        other := B.create_literal(b, .{});

        assert(B.macro_literals_are_equal(literal, other));
        scalar, other_scalar: B.Literal;
        scalar.literal_kind = .INTEGER;
        scalar.int_value = 42;
        other_scalar = scalar;
        assert(!B.macro_literals_are_equal(*scalar, *other_scalar));
        other_scalar.int_value = 41;
        assert(!B.macro_literals_are_equal(*scalar, *other_scalar));

        push_context {
            context.logger = capture;
            assert(B.find_closing_paren_token_index(first, -1) == -1);
            assert(B.find_closing_paren_token_index(first, 0) == -1);
            incomplete: [] B.CXToken = .{ 13, first.data };

            assert(B.find_closing_paren_token_index(incomplete, 4) == -1);
        }

        assert(errors == 1);
    }

    assert(context._bindings_generator_session == null);

    push_context {
        context.logger = capture;
        assert(!B.is_token(tokens[0], "int"));
    }

    assert(errors == 2);
    print("ok\n");
}
