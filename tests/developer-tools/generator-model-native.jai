#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    cursor: B.CXCursor;
    saved: #Context;
    expected: *B.Declaration;
    reference: *B.Literal;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    state := cast(*Capture) data;

    if cur.kind == .VarDecl {
        state.cursor = cur;
        if state.expected {
            push_context {
                context = state.saved;
                assert(B.find_referenced_declaration(cur) == state.expected);
                state.reference = B.create_reference_expression(cur, .{ line = 19 });
                assert(state.reference.identifier == state.expected);
            }
        }
    }

    return .Continue;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string("generator-model-original.h");

    defer free(filename);
    source := "/** retained after native disposal */\nextern int original_value;\n";
    unsaved: B.CXUnsavedFile;
    unsaved.Filename = filename;
    unsaved.Contents = source.data;
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;
    args: [2] *u8;
    args[0] = to_c_string("-x");
    args[1] = to_c_string("c");
    defer free(args[0]);
    defer free(args[1]);
    unit: B.CXTranslationUnit;
    status := B.clang_parseTranslationUnit2(index, filename, args.data, 2, *unsaved, 1, 0, *unit);

    assert(status == .Success && unit != null);
    state: Capture;

    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *state);
    type := B.get_int_type(4, true);

    defer free(type);
    decl := B._create_declaration("original_value", type, state.cursor);
    generator: B.Generator;

    H.table_add(*generator.declarations_by_cursor, state.cursor, decl);
    found, same := H.table_find(*generator.declarations_by_cursor, state.cursor);

    assert(found && same == decl);
    assert(generator.declarations_by_cursor.hash_function(state.cursor) == B.clang_hashCursor(state.cursor));
    assert(generator.declarations_by_cursor.compare_function(state.cursor, state.cursor));
    session := Session.open(*generator, B.TARGET_OS);

    decl.decl_flags |= .IS_CONST;

    state.expected = decl;

    push_context {
        context.generator = session.generator;
        context._bindings_generator_session = *session;
        state.saved = context;
        B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *state);
        assert(state.reference.literal_kind == .IDENTIFIER && state.reference.location.line == 19);
        assert(B.find_referenced_declaration(B.clang_getNullCursor()) == null);
    }

    Session.close(*session);
    H.table_reset(*generator.declarations_by_cursor);

    // Public strings survive disposal of every libclang source object.
    B.clang_disposeTranslationUnit(unit);
    defer free(decl);
    defer free(decl.name);
    defer free(decl.output_name);
    defer free(decl.comment.text);
    assert(decl.kind == .DECLARATION && decl.name == "original_value" && decl.type == type);
    assert(decl.location.file != null && decl.location.line == 2 && decl.location.column == 12);
    assert(decl.comment.text == "/** retained after native disposal */");
    assert(decl.comment.location.line == 1 && B.is_prefix_comment(decl.comment));
    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";

Session :: type_of(context._bindings_generator_session.*);
