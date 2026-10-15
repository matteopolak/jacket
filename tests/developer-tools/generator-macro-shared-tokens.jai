// Original SDK input. Public API checks preserve borrowed-token and copied-text
// lifetimes without changing native token storage or production configuration.
#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    macro: B.CXCursor;
    utf8: B.CXCursor;
    selected: B.CXCursor;
    empty: B.CXCursor;
    saved: #Context;
}

visit :: (cursor: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;

        if cursor.kind == .MacroDefinition || cursor.kind == .VarDecl {
            native := B.clang_getCursorSpelling(cursor);
            name := to_string(B.clang_getCString(native));

            if name == "OWN_TOKENS" capture.macro = cursor;
            if name == "OWN_UTF8" capture.utf8 = cursor;
            if name == "selected" capture.selected = cursor;
            if name == "empty" capture.empty = cursor;

            B.clang_disposeString(native);
        }
    }

    return .Recurse;
}

main :: () {
    copied: string;
    copied_utf8: string;

    defer free(copied);
    defer free(copied_utf8);

    {
        index := B.clang_createIndex(0, 0);

        assert(index != null);
        source := "#define OWN_TOKENS (1+2)\n#define OWN_UTF8 \"héllo\"\nint selected = 9; int empty;\n";
        filename := to_c_string("owned-generator-macro-tokens.c");

        defer free(filename);
        unsaved: B.CXUnsavedFile;
        unsaved.Filename = filename;
        unsaved.Contents = source.data;
        unsaved.Length = cast(type_of(unsaved.Length)) source.count;
        unit: B.CXTranslationUnit;
        error := B.clang_parseTranslationUnit2(index, filename, null, 0, *unsaved, 1, 1, *unit);

        assert(error == .Success && unit != null);
        generator: B.Generator;
        generator.clang_index = index;
        generator.translation_unit = unit;
        push_context, defer_pop;
        options: B.Generate_Bindings_Options;

        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        capture: Capture;
        capture.saved = context;
        B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *capture);
        declaration := B.create_macro_definition(capture.macro);
        utf8 := B.create_macro_definition(capture.utf8);

        assert(declaration != null && utf8 != null);
        value := (cast(*B.Literal) declaration.expression).macro_value;
        utf8_value := (cast(*B.Literal) utf8.expression).macro_value;

        assert(value.tokens.count == 5 && utf8_value.tokens.count == 1);
        copied = B.raw_string_from_macro_tokens(value);
        copied_utf8 = B.raw_string_from_macro_tokens(utf8_value);
        assert(copied == "( 1 + 2 )" && copied_utf8 == "\"héllo\"");
        spelling := B.get_scoped_token_string(utf8_value.tokens[0]);
        other := B.get_scoped_token_string(value.tokens[2]);

        assert(spelling == "\"héllo\"" && other == "+");
        match := B.expect_tokens(value.tokens, 0, "(", "1", "+", "2", ")");
        mismatch := B.expect_tokens(value.tokens, 1, "1", "-");
        short := B.expect_tokens(value.tokens, 4, ")", "extra");
        empty_match := B.expect_tokens(value.tokens, value.tokens.count);

        assert(match && !mismatch && !short && empty_match);
        expression := B.convert_children_to_expression(capture.selected);

        assert(expression != null && expression.kind == .LITERAL);
        literal := cast(*B.Literal) expression;

        assert(literal.literal_kind == .INTEGER && literal.int_value == 9);
        no_child := B.convert_children_to_expression(capture.empty);

        assert(no_child == null);
        // Its final native child is a declaration, not an expression. Earlier
        // usable expressions must not be selected instead of the final child.
        final_declaration := B.convert_children_to_expression(B.clang_getTranslationUnitCursor(unit));

        assert(final_declaration == null);
    }

    assert(copied == "( 1 + 2 )" && copied_utf8 == "\"héllo\"");
    copied.data[0] = #char "[";
    assert(copied_utf8 == "\"héllo\"");
    print("tokens ok\n");
}
