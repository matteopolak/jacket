#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    saved: #Context;
    alias: B.CXCursor;
    children: [..] B.CXCursor;
}

visit_alias :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;
    if cur.kind == .TypedefDecl capture.alias = cur;
    return .Continue;
}

visit_child :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        array_add(*capture.children, cur);
    }

    return .Continue;
}

main :: () {
    generator: B.Generator;

    push_context {
        B.init_generator(*generator, .{});
        defer B.deinit_generator(*generator);
        generator.clang_index = B.clang_createIndex(0, 0);
        filename := to_c_string(tprint("%bindings-template-arguments.hpp", #filepath));

        defer free(filename);
        arguments := (*u8).[to_c_string("-std=c++17")];

        defer free(arguments[0]);
        parsed := B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 1, null, 0, 0, *generator.translation_unit);

        assert(parsed == .Success && B.clang_getNumDiagnostics(generator.translation_unit) == 0);
        capture: Capture;
        capture.saved = context;
        B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), visit_alias, *capture);
        B.clang_visitChildren(capture.alias, visit_child, *capture);
        defer array_free(capture.children);
        native := B.clang_getTypedefDeclUnderlyingType(capture.alias);
        params: [..] *B.Expression;

        defer array_free(params);
        success, consumed := B.fill_template_arguments(native, *params, capture.children, capture.alias);

        assert(success && consumed > 0 && consumed <= capture.children.count && params.count == 2);
        type_ref := cast(*B.Type_Reference) params[0];

        assert(type_ref.kind == .TYPE_REFERENCE && type_ref.type.type_of_struct.name == "OriginalTemplateType");
        value := cast(*B.Literal) params[1];

        assert(value.kind == .LITERAL && value.int_value == 3);
        application, _used := B.create_elaborated_type(native, capture.children, capture.alias);
        literal: B.Literal;
        literal.literal_kind = .IDENTIFIER;
        literal.identifier = cast(*B.Declaration) application.type_of_struct;
        expression_params: [..] *B.Expression;

        defer array_free(expression_params);
        assert(B.fill_template_arguments_for_expression(*literal, *expression_params, capture.children, capture.alias));
        assert(expression_params.count == 2 && literal.identifier == cast(*B.Declaration) application.type_of_struct);
        assert((cast(*B.Type_Reference) expression_params[0]).type.type_of_struct == type_ref.type.type_of_struct);
        assert((cast(*B.Literal) expression_params[1]).int_value == 3);
    }

    print("ok\n");
}
