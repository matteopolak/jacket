#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    cursor: B.CXCursor;
    saved: #Context;
    children: [..] B.CXCursor;
}

visit_child :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        array_add(*capture.children, cur);
    }

    return .Continue;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;
    if cur.kind == .ClassTemplate capture.cursor = cur;
    return .Continue;
}

bound :: (type: *B.CType, declaration: *B.Declaration) {
    assert(type && type.array_element_count == -1 && type.array_element_count_expression);
    literal := cast(*B.Literal) type.array_element_count_expression;
    assert(literal.kind == .LITERAL && literal.literal_kind == .IDENTIFIER && literal.identifier == declaration);
}

main :: () {
    generator: B.Generator;

    push_context {
        B.init_generator(*generator, .{});
        defer B.deinit_generator(*generator);
        generator.clang_index = B.clang_createIndex(0, 0);
        filename := to_c_string(tprint("%generator-model-arrays.hpp", #filepath));

        defer free(filename);
        argument := to_c_string("-std=c++17");

        defer free(argument);
        assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, *argument, 1, null, 0, 0, *generator.translation_unit) == .Success);
        assert(B.clang_getNumDiagnostics(generator.translation_unit) == 0);
        capture: Capture;

        B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), visit, *capture);
        record := B.create_struct(capture.cursor);

        assert(record && record.template_type_params.count == 3 && record.declarations.count == 8);
        t := record.template_type_params[0];
        n := record.template_type_params[1];
        m := record.template_type_params[2];
        single := record.declarations[0];

        bound(single.type, n);
        assert(single.type.array_element_type.type_of_declaration == t && !single.expression);
        matrix := record.declarations[1];

        bound(matrix.type, n);
        bound(matrix.type.array_element_type, m);
        assert(matrix.type.array_element_type.array_element_type.type_of_declaration == t && !matrix.expression);
        mixed := record.declarations[2].type;

        assert(mixed.array_element_count == 2 && !mixed.array_element_count_expression);
        sum := cast(*B.Binary_Operator) mixed.array_element_type.array_element_count_expression;

        assert(sum && sum.kind == .BINARY_OPERATOR && sum.op == "+");
        assert((cast(*B.Literal) sum.lhs).identifier == n && (cast(*B.Literal) sum.rhs).int_value == 1);
        bound(mixed.array_element_type.array_element_type, m);
        bound(record.declarations[3].type.pointer_to, n);
        initializer := cast(*B.Literal) record.declarations[3].expression;

        assert(initializer && initializer.kind == .LITERAL && initializer.literal_kind == .NULL);
        callbacks := record.declarations[4].type;

        bound(callbacks, n);
        bound(callbacks.array_element_type.pointer_to.type_of_function.return_type.pointer_to, m);
        callback_arguments := record.declarations[5].type;

        bound(callback_arguments, n);
        bound(callback_arguments.array_element_type.pointer_to.type_of_function.arguments[0].type.pointer_to, m);
        both := record.declarations[6].type.array_element_type.pointer_to.type_of_function;

        bound(both.return_type.pointer_to, m);
        argument_sum := cast(*B.Binary_Operator) both.arguments[0].type.pointer_to.array_element_count_expression;

        assert(argument_sum && argument_sum.kind == .BINARY_OPERATOR && argument_sum.op == "+");
        assert((cast(*B.Literal) argument_sum.lhs).identifier == n);
        assert(B.create_struct(capture.cursor) == record);
        fields: Capture;
        fields.saved = context;
        B.clang_visitChildren(capture.cursor, visit_child, *fields);
        defer array_free(fields.children);

        for field: fields.children {
            if field.kind != .FieldDecl continue;

            refs: Capture;
            refs.saved = context;
            B.clang_visitChildren(field, visit_child, *refs);
            defer array_free(refs.children);
            native := B.clang_getCursorType(field);

            assert(B.create_type(native, field) != null);
            rebuilt := B.create_type_inner(native, field, false, substitute_children = refs.children);

            assert(rebuilt != null);
            elaborated, consumed := B.create_elaborated_type(native, refs.children, field);

            assert(elaborated && consumed > 0 && consumed <= refs.children.count);
            if native.kind == .DependentSizedArray && refs.children.count == 2 assert(consumed == 2);

            assert(B.create_elaborated_type(native, field) != null);

            if native.kind == .Pointer {
                pointee := B.clang_getPointeeType(native);
                if pointee.kind == .FunctionProto {
                    function: B.Function;
                    signature := B.create_function_type(field, pointee, *function, false);
                    assert(signature && (function.decl_flags & .NEEDS_ARGUMENT_WRAPPER) != 0);
                }
            }
        }
    }

    print("ok\n");
}
