#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    saved: #Context;
    children: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        array_add(*capture.children, cur);
    }

    return .Continue;
}

errors: int;

log :: (_text: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-record-edges.hpp", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    capture: Capture;
    capture.saved = context;
    root := B.clang_getTranslationUnitCursor(unit);

    B.clang_visitChildren(root, visit, *capture);
    defer array_free(capture.children);
    assert(capture.children.count == 9);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);

    push_context {
        S.activate(*generator, null, *session);
        left := B.create_namespace(capture.children[0]);
        shared := cast(*B.Struct) left.declarations[0];
        shared.output_name = "RenamedShared";
        right := B.create_namespace(capture.children[1]);

        assert(right.declarations.count == 2);
        alias := cast(*B.Typedef) right.declarations[0];

        assert(alias.name == "Shared" && (alias.decl_flags & .OMIT_FROM_OUTPUT) == 0);
        assert(alias.type.type_of_struct == shared && shared.output_name == "RenamedShared");
        assert(right.declarations[1].type.pointer_to.type_of_typedef == alias);

        push_context {
            context.logger = log;

            for index: 3..4 {
                cur := capture.children[index];

                assert(B.create_struct(cur, resolving_type_only = true) == null);
                assert(B.create_struct(cur) == null);
                assert(B.create_type(B.clang_getCursorType(cur), cur) == null);
                registered, _entry := H.table_find(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(cur));

                assert(!registered);
            }
        }

        assert(errors == 6 && !session.failed);
        after := B.create_struct(capture.children[5]);

        assert(after != null && after.declarations.count == 1 && !session.failed);
        derived_shell := B.create_struct(capture.children[7], resolving_type_only = true);

        assert(derived_shell.parents.count == 0 && derived_shell.declarations.count == 0);
        retained_type := B.create_declaration(capture.children[7], root, true);

        assert(retained_type == cast(*B.Declaration) derived_shell);
        assert(derived_shell.parents.count == 1 && derived_shell.declarations.count == 1);
        derived := B.create_struct(capture.children[7]);
        base := B.create_struct(capture.children[6]);

        assert(derived == derived_shell && derived.parents.count == 1 && derived.declarations.count == 1);
        assert(derived.parents[0].type.type_of_struct == base);
        assert(derived.parents[0].name == "OriginalBase" && derived.parents[0].parent == cast(*B.Declaration) derived);
        assert(base.declarations.count == 1 && !session.failed);

        push_context {
            context.logger = log;
            assert(B.create_enum(capture.children[8]) == null);
            assert(errors == 7 && !session.failed && session.model_errors > 0);
        }
    }

    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
