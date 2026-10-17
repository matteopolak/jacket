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

main :: () {
    index := B.clang_createIndex(0, 0);

    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-aggregates.hpp", #filepath));

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
    assert(capture.children.count == 6);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);

    push_context {
        S.activate(*generator, null, *session);

        for cur: capture.children {
            declaration := B.create_declaration(cur, root);
            assert(declaration != null);
            S.append_owned(*session, *generator.global_scope.members, declaration);
        }

        value := cast(*B.Struct_Literal) generator.global_scope.members[2].expression;

        assert(value.kind == .STRUCT_LITERAL && value.type_name == "OriginalAggregate" && value.values.count == 2);
        assert((cast(*B.Literal) value.values[0]).int_value == 4);
        assert((cast(*B.Literal) value.values[1]).float_value == 2.5);
        nested := cast(*B.Struct_Literal) generator.global_scope.members[3].expression;

        assert(nested.kind == .STRUCT_LITERAL && nested.type_name == "OriginalNestedAggregate" && nested.values.count == 1);
        child := cast(*B.Struct_Literal) nested.values[0];

        assert(child.kind == .STRUCT_LITERAL && child.values.count == 2);
        assert((cast(*B.Literal) child.values[0]).int_value == 7);
        empty := cast(*B.Struct_Literal) generator.global_scope.members[4].expression;

        assert(empty.kind == .STRUCT_LITERAL && empty.values.count == 0);
        function := cast(*B.Function) generator.global_scope.members[5];

        assert((function.decl_flags & .NEEDS_ARGUMENT_WRAPPER) != 0);
        argument := function.type.type_of_function.arguments[0];

        assert((argument.decl_flags & .POINTER_TO_VALUE) != 0);
        default_value := cast(*B.Struct_Literal) argument.expression;

        assert(default_value.kind == .STRUCT_LITERAL && default_value.values.count == 2);
        assert((cast(*B.Literal) default_value.values[0]).int_value == 9);
        assert(!session.failed);
    }

    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
