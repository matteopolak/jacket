#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

#load "../../target/generator-model-template-layout-oracle.jai";

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
    filename := to_c_string(tprint("%generator-model-templates.hpp", #filepath));

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
    assert(capture.children.count == 12);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);

    push_context {
        S.activate(*generator, null, *session);
        record := B.create_struct(capture.children[0]);

        assert(record != null && record.template_type_params.count == 2 && record.declarations.count == 3);
        parameter := record.template_type_params[0];

        assert(parameter.name == "T" && parameter.type == generator.type_def_type);
        assert(parameter.parent == cast(*B.Declaration) record);
        assert(record.declarations[0].type.type_of_declaration == parameter);
        assert(record.declarations[1].type.pointer_to.type_of_declaration == parameter);
        second_parameter := record.template_type_params[1];

        assert(record.declarations[2].type.pointer_to.type_of_declaration == second_parameter);
        assert((record.declarations[2].type.pointer_to.flags & .CONST) != 0);
        fields: Capture;
        fields.saved = context;
        B.clang_visitChildren(capture.children[0], visit, *fields);
        defer array_free(fields.children);
        assert(fields.children.count == 6);
        // Dependent methods without a mangling retain a usable standalone signature.
        method_cur := fields.children[5];
        ft := B.create_function_type(method_cur, B.clang_getCursorType(method_cur));

        assert(ft.return_type.type_of_declaration == second_parameter && ft.arguments.count == 2);
        assert(ft.arguments[0].type.pointer_to.type_of_struct == record);
        assert(ft.arguments[1].type.type_of_declaration == parameter);
        assert(B.create_function(method_cur, capture.children[0]) == null);
        field := fields.children[2];
        refs: Capture;
        refs.saved = context;
        B.clang_visitChildren(field, visit, *refs);
        defer array_free(refs.children);
        assert(refs.children.count == 1);
        type, consumed := B.create_elaborated_type(B.clang_getCursorType(field), refs.children, field);

        assert(type.type_of_declaration == parameter && consumed == 1);
        substituted := B.create_type_inner(B.clang_getCursorType(field), field, false, substitute_children = refs.children);

        assert(substituted.type_of_declaration == parameter);
        info, generated := B.find_or_create_extra_info(record.type, cast(*B.Declaration) record);

        assert(info == *B.struct_extra_info_unknown && generated == null);
        native_instance := B.clang_getPointeeType(B.clang_getCursorType(capture.children[3]));
        native_record := B.clang_getTypeDeclaration(native_instance);
        native_template := B.clang_getSpecializedCursorTemplate(native_record);

        assert(B.clang_equalLocations(B.clang_getCursorLocation(native_record), B.clang_getCursorLocation(native_template)) != 0);
        instance := B.create_declaration(capture.children[3], root);

        assert(instance != null);
        instantiated := instance.type.pointer_to;

        assert(instantiated.type_of_struct == record && instantiated.template_instantiation_params.count == 2);
        first_type := cast(*B.Type_Reference) instantiated.template_instantiation_params[0];
        second_type := cast(*B.Type_Reference) instantiated.template_instantiation_params[1];

        assert(first_type.kind == .TYPE_REFERENCE && first_type.type == generator.type_def_s32);
        assert(second_type.kind == .TYPE_REFERENCE && second_type.type == generator.type_def_float64);
        assert(instantiated.size == Native_Template_Size && instantiated.alignment == Native_Template_Alignment);
        constant := B.create_declaration(capture.children[4], root).type.pointer_to;

        assert((constant.flags & .CONST) != 0 && constant.template_instantiation_params.count == 2);
        assert(constant.template_instantiation_params.data != instantiated.template_instantiation_params.data);
        assert(constant.template_instantiation_params[0] != instantiated.template_instantiation_params[0]);
        nested := B.create_declaration(capture.children[5], root).type.pointer_to;
        nested_argument := (cast(*B.Type_Reference) nested.template_instantiation_params[0]).type;

        assert(nested_argument.pointer_to.type_of_struct == record);
        assert(nested_argument.pointer_to.template_instantiation_params.count == 2);
        const_argument := (cast(*B.Type_Reference) nested.template_instantiation_params[1]).type;

        assert((const_argument.flags & .CONST) != 0);
        bases := B.create_struct(capture.children[7]);

        assert(bases.parents.count == 2);
        assert(bases.parents[0] != bases.parents[1]);
        assert(bases.parents[0].type.type_of_struct == bases.parents[1].type.type_of_struct);
        assert((cast(*B.Type_Reference) bases.parents[0].type.template_instantiation_params[0]).type == generator.type_def_s32);
        assert((cast(*B.Type_Reference) bases.parents[1].type.template_instantiation_params[0]).type == generator.type_def_float64);
        pointer_refs: Capture;
        pointer_refs.saved = context;
        B.clang_visitChildren(fields.children[3], visit, *pointer_refs);
        defer array_free(pointer_refs.children);
        substituted_pointer := B.create_type_inner(B.clang_getCursorType(fields.children[3]), root, false, substitute_children = pointer_refs.children);

        assert(substituted_pointer.pointer_to.type_of_declaration == parameter);
        overloads := B.create_namespace(capture.children[9]);

        assert(overloads.declarations.count == 4);
        equivalent_a := cast(*B.Function) overloads.declarations[0];
        equivalent_b := cast(*B.Function) overloads.declarations[1];
        distinct_a := cast(*B.Function) overloads.declarations[2];
        distinct_b := cast(*B.Function) overloads.declarations[3];

        assert(B.functions_are_equivalent(equivalent_a, equivalent_b));
        assert((equivalent_b.decl_flags & .OMIT_FROM_OUTPUT) != 0);
        assert(!B.functions_are_equivalent(distinct_a, distinct_b));
        collision, _other := B.find_name_clash(overloads.declarations, cast(*B.Declaration) distinct_b, true);

        assert(!collision);
        outer := B.create_struct(capture.children[10]);

        assert(outer != null && outer.template_type_params.count == 2 && outer.declarations.count == 1);
        inner := outer.declarations[0].type.pointer_to;

        assert(inner.type_of_struct == record && inner.template_instantiation_params.count == 2);
        assert((cast(*B.Type_Reference) inner.template_instantiation_params[0]).type.type_of_declaration == outer.template_type_params[0]);
        assert((cast(*B.Type_Reference) inner.template_instantiation_params[1]).type.type_of_declaration == outer.template_type_params[1]);
        recursive := B.create_struct(capture.children[11]);

        assert(recursive != null && recursive.declarations.count == 1);
        next := recursive.declarations[0].type.pointer_to;

        assert(next.type_of_struct == recursive && next.template_instantiation_params.count == 1);
        assert((cast(*B.Type_Reference) next.template_instantiation_params[0]).type.type_of_declaration == recursive.template_type_params[0]);

        push_context {
            context.logger = log;

            for i: 1..2 {
                assert(B.create_struct(capture.children[i]) == null);
                found, _declaration := H.table_find(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(capture.children[i]));
                assert(!found);
            }
        }

        assert(errors == 2 && !session.failed);

        push_context {
            context.logger = log;
            assert(B.create_type_inner(B.clang_getCursorType(fields.children[3]), root, false) == null);
            assert(errors == 3 && session.failed);
            malformed: [] B.CXCursor;
            malformed.count = 1;
            assert(B.create_type_inner(B.clang_getCursorType(fields.children[3]), root, false, substitute_children = malformed) == null);
            assert(errors == 4);
        }
    }

    aggregate_children: Capture;
    aggregate_children.saved = context;
    B.clang_visitChildren(capture.children[8], visit, *aggregate_children);
    defer array_free(aggregate_children.children);
    initializer: B.CXCursor;
    for child: aggregate_children.children if B.clang_isExpression(child.kind) != 0 initializer = child;

    assert(initializer.kind == .InitListExpr);
    rejection := S.open(*generator, B.TARGET_OS);

    push_context {
        S.activate(*generator, null, *rejection);
        context.logger = log;
        expression, skip := B.create_expression_from_cursor(initializer, capture.children[8]);
        assert(expression == null && !skip && errors == 5 && rejection.failed);
    }

    S.close(*rejection);
    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
