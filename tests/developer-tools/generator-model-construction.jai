#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

errors: int;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

#import "generator-model-layout-oracle";

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

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-construction.h", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    root := B.clang_getTranslationUnitCursor(unit);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(root, visit, *capture);
    defer array_free(capture.children);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);
    defer H.table_reset(*generator.extra_infos);

    push_context {
        context.generator = *generator;
        context._bindings_generator_session = *session;
        shell := B.create_struct(capture.children[0], resolving_type_only = true);

        assert(shell.declarations.count == 0 && shell.type.type_of_struct == shell);
        shell_info, shell_record := B.find_or_create_extra_info(shell.type, cast(*B.Declaration) shell);

        assert(shell_info == *B.struct_extra_info_unknown && shell_record == null);
        assert(generator.extra_infos.count == 0);

        for cur: capture.children {
            declaration := B.create_declaration(cur, root);

            assert(declaration != null);
            repeated := B.create_declaration_inner(cur, root);
            if declaration.kind == .FUNCTION assert(!repeated);
            else assert(repeated == declaration);

            S.append_owned(*session, *generator.global_scope.members, declaration);
        }

        assert(generator.global_scope.members.count == 11);
        node := cast(*B.Struct) generator.global_scope.members[0];

        assert(node.kind == .STRUCT && node.declarations.count == 2);
        assert(node == shell);
        assert(node.type.size == Native_Node_Size && node.type.alignment == Native_Node_Alignment);
        assert(node.declarations[1].bit_offset_in_parent == Native_Next_Bit_Offset);
        assert(node.declarations[1].type.pointer_to.type_of_struct == node);
        assert(node.declarations[0].parent == cast(*B.Declaration) node);
        assert(B.create_struct(capture.children[0]) == node);
        assert(B.create_type(B.clang_getCursorType(capture.children[0]), capture.children[0]).type_of_struct == node);
        color := cast(*B.Enum) generator.global_scope.members[1];

        assert(color.enumerates.count == 2 && color.type.number_flags & .SIGNED);
        assert((cast(*B.Literal) color.enumerates[0].expression).int_value == -1);
        assert((cast(*B.Literal) color.enumerates[1].expression).int_value == 7);
        assert(B.create_enum(capture.children[1]) == color);
        alias := cast(*B.Typedef) generator.global_scope.members[2];

        assert(alias.type.type_of_struct == node && B.create_typedef(capture.children[2]) == alias);
        variable := generator.global_scope.members[3];

        assert(variable.name == "original_counter" && variable.type == generator.type_def_s32);
        variable.output_name.data[0] = #char "X";
        assert(variable.name == "original_counter");
        variable.output_name.data[0] = #char "o";
        assert(B.is_prefix_comment(variable.comment));
        assert((cast(*B.Literal) variable.expression).int_value == 9);
        native_type := B.clang_getCursorType(capture.children[3]);

        assert(B.create_type_inner(native_type, capture.children[3]) == variable.type);
        variable_again, flags := B.create_variable_declaration(capture.children[3], root);

        assert(variable_again == variable && flags == 0);
        function := cast(*B.Function) generator.global_scope.members[4];

        assert(function.kind == .FUNCTION && function.type.type_of_function.arguments.count == 3);
        assert(B.create_function(capture.children[4], root) == null);
        ft := function.type.type_of_function;

        assert(ft.return_type == generator.type_def_s32);
        assert(ft.arguments[0].parent == cast(*B.Declaration) function);
        assert(ft.arguments[0].type.pointer_to.type_of_typedef == alias);
        type_children: Capture;
        type_children.saved = context;
        B.clang_visitChildren(capture.children[4], visit, *type_children);
        defer array_free(type_children.children);
        argument_children: Capture;
        argument_children.saved = context;
        B.clang_visitChildren(type_children.children[0], visit, *argument_children);
        defer array_free(argument_children.children);
        native_alias := B.clang_getPointeeType(B.clang_getCursorType(type_children.children[0]));
        elaborated, consumed := B.create_elaborated_type(native_alias, argument_children.children, type_children.children[0]);

        assert(elaborated.type_of_typedef == alias && consumed == argument_children.children.count);
        assert(B.create_elaborated_type(native_alias, type_children.children[0]).type_of_typedef == alias);
        assert(B.is_char_pointer_type(ft.arguments[1].type.*));
        assert((ft.arguments[1].type.pointer_to.flags & .CONST) != 0);
        assert((generator.type_def_char.flags & .CONST) == 0);
        assert(ft.arguments[2].type.pointer_to.type_of_function.arguments.count == 1);
        expr := generator.global_scope.members[5].expression;

        assert(generator.global_scope.members[5].comment.text == "///< trailing model comment.");
        assert(!B.is_prefix_comment(generator.global_scope.members[5].comment));

        assert(expr.kind == .BINARY_OPERATOR);
        binary := cast(*B.Binary_Operator) expr;

        assert(binary.op == "*" && binary.lhs.kind == .PAREN_EXPRESSION && binary.rhs.kind == .UNARY_OPERATOR);
        paren := cast(*B.Paren_Expression) binary.lhs;

        assert(paren.child.kind == .BINARY_OPERATOR && (cast(*B.Binary_Operator) paren.child).op == "+");
        assert((cast(*B.Unary_Operator) binary.rhs).op == "-");
        expression_children: Capture;
        expression_children.saved = context;
        B.clang_visitChildren(capture.children[5], visit, *expression_children);
        defer array_free(expression_children.children);
        scalar := B.create_literal(expression_children.children[0], capture.children[5]);

        assert(scalar.literal_kind == .INTEGER && scalar.int_value == -20);
        rebuilt, skip := B.create_expression_from_cursor(expression_children.children[0], capture.children[5]);

        assert(rebuilt.kind == .BINARY_OPERATOR && !skip);
        assert(generator.global_scope.members[6].type.array_element_count == 3);
        assert(generator.global_scope.members[6].type.array_element_type == generator.type_def_s32);
        assert(generator.global_scope.members[7].type.pointer_to.type_of_function.arguments.count == 1);
        assert(B.find_first_member_offset(B.clang_getCursorType(capture.children[0]), node, node) == 0);
        saved_first := node.declarations[0];

        node.declarations[0] = node.declarations[1];
        node.declarations[1] = saved_first;
        assert(B.find_first_member_offset(B.clang_getCursorType(capture.children[0]), node, node) == Native_Next_Bit_Offset);
        node.declarations[1] = node.declarations[0];
        node.declarations[0] = saved_first;
        grouped, is_new := B.find_or_create_enum_for_macro("ORIGINAL_VALUES", variable);

        assert(is_new && grouped.type == null && grouped.location.line == variable.location.line);
        again, is_new_again := B.find_or_create_enum_for_macro("ORIGINAL_VALUES", variable);

        assert(again == grouped && !is_new_again);
        existing_enum, existing_is_new := B.find_or_create_enum_for_macro("OriginalColor", variable);

        assert(existing_enum == color && !existing_is_new);
        found_name, output_name, identifier_type, parent_enum := B.find_and_check_macro_identifier("original_counter");

        assert(found_name && output_name == "original_counter" && identifier_type == variable.type && !parent_enum);
        found_name, output_name, identifier_type, parent_enum = B.find_and_check_macro_identifier("ORIGINAL_SEVEN", color);
        assert(found_name && output_name == "ORIGINAL_SEVEN" && parent_enum == color && identifier_type.type_of_enum == color);
        info, generated := B.find_or_create_extra_info(node.type, cast(*B.Declaration) node);

        assert(info != null && generated == node && info.has_field && info.has_nonzero_sized_field);
        assert(info.struct_alignment == Native_Node_Alignment && info.largest_natural_alignment == Native_Node_Alignment);
        cached_info, cached_generated := B.find_or_create_extra_info(node.type, cast(*B.Declaration) node);

        assert(cached_info == info && cached_generated == node);

        push_context {
            context._bindings_generator_session = null;
            context.logger = capture_error;
            borrowed_info, borrowed_record := B.find_or_create_extra_info(node.type, cast(*B.Declaration) node);
            assert(borrowed_info == info && borrowed_record == node && errors == 0);
        }

        template_parameter: B.Declaration;
        template_parameter.decl_flags = .TEMPLATE;
        dependent: B.CType;
        dependent.type_of_declaration = *template_parameter;
        unknown_info, unknown_record := B.find_or_create_extra_info(*dependent, null);

        assert(unknown_info == *B.struct_extra_info_unknown && unknown_record == null);
        assert(unknown_info.struct_alignment == -1 && unknown_info.largest_natural_alignment == -1);
        boolean: B.CType;
        boolean.flags = .BOOL;
        assert(B.can_omit_type(variable, 0, *boolean));
        assert(!B.can_omit_type(variable, .NO_ARGUMENT_DEFAULTS, *boolean));
        assert(!B.can_omit_type(variable, 0, variable.type));
        assert(!B.can_omit_type(null, 0, *boolean));
        raw := B.create_literal("(source + text)", .{ line = 31 });

        assert(raw.literal_kind == .RAW_CPP_TOKENS && raw.raw_value == "(source + text)" && raw.location.line == 31);
        raw.raw_value.data[0] = #char "X";
        assert(raw.string_value == "(source + text)");
        signed_wide := generator.global_scope.members[8].type;
        unsigned_wide := generator.global_scope.members[9].type;

        assert(signed_wide == generator.type_def_s128 && unsigned_wide == generator.type_def_u128);
        assert(signed_wide.size == Native_Wide_Size && unsigned_wide.size == Native_Wide_Size);
        assert(signed_wide.alignment == Native_Wide_Alignment && unsigned_wide.alignment == Native_Wide_Alignment);
        assert(B.is_16byte_integer_type(signed_wide) && B.is_16byte_integer_type(unsigned_wide));
        assert(B.is_target_type_signed(signed_wide) && B.is_target_type_unsigned(unsigned_wide));
        assert(B.get_primitive_or_array_primitive_type_size(unsigned_wide, null) == 16);
        assert(generator.int128_was_referenced);
        assert(generator.global_scope.members[10].expression.kind == .BINARY_OPERATOR);
        assert(!session.failed);
    }

    assert(context.generator == null && context._bindings_generator_session == null);
    rejected_session := S.open(*generator, B.TARGET_OS);

    push_context {
        S.activate(*generator, null, *rejected_session);
        context.logger = capture_error;
        assert(B.create_literal(capture.children[10], root) == null);
        assert(!rejected_session.failed && rejected_session.model_errors > 0 && errors == 1);
    }

    S.close(*rejected_session);
    assert(!session.failed);
    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
