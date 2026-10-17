#import "Basic";
#import "String";
B :: #import "Bindings_Generator";
H :: #import "Hash_Table";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    saved: #Context;
    rejected, retained, dependent, objc, macro, formal: B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        spelling := B.clang_getCursorSpelling(cur);

        defer B.clang_disposeString(spelling);
        name := to_string(B.clang_getCString(spelling));
        if name == "InvalidCycle" capture.rejected = cur;
        if name == "AcceptedAfter" capture.retained = cur;
        if name == "LoggerAccepted" capture.dependent = cur;
        if name == "BoundaryObjRoot" capture.objc = cur;
        if name == "MODEL_BOUNDARY_VALUE" capture.macro = cur;
        if cur.kind == .TemplateTypeParameter capture.formal = cur;
    }

    return ifx cur.kind == .ClassTemplate then .Recurse else .Continue;
}

errors: int;
fatal_seen: bool;
fatal_reentry: *B.Declaration;
fatal_cursor, fatal_parent: B.CXCursor;
fatal_objc_cursor, fatal_macro_cursor, fatal_formal_cursor: B.CXCursor;
fatal_objc: *B.Struct;
fatal_macro, fatal_formal: *B.Declaration;

log :: (message: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;

    if contains(message, "malformed") {
        fatal_seen = context._bindings_generator_session.failed;
        fatal_reentry = B.create_declaration(fatal_cursor, fatal_parent);
        fatal_objc = B.create_objc_interface(fatal_objc_cursor, true);
        fatal_macro = B.create_macro_definition(fatal_macro_cursor);
        fatal_formal = B.create_objc_type_param(fatal_formal_cursor);
    }
}

exercise :: (mode: int) {
    push_context, defer_pop;
    context.logger = log;
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    B.init_generator(*generator, options);
    defer B.deinit_generator(*generator);
    session := context._bindings_generator_session;

    S.activate(*generator, *options, session);
    generator.clang_index = B.clang_createIndex(0, 0);
    filename := to_c_string(tprint("%generator-model-boundaries.hpp", #filepath));

    defer free(filename);
    arguments := (*u8).[to_c_string("-std=c++17"), to_c_string("-x"), to_c_string("objective-c++")];

    defer for argument: arguments free(argument);
    assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 3, null, 0, cast(u32) B.CXTranslationUnit_Flags.DetailedPreprocessingRecord, *generator.translation_unit) == .Success);
    assert(B.clang_getNumDiagnostics(generator.translation_unit) == 0);
    root := B.clang_getTranslationUnitCursor(generator.translation_unit);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(root, visit, *capture);
    target := B.create_struct(capture.rejected, resolving_type_only = true);
    retained := B.create_declaration(capture.retained, root);
    dependent := B.create_declaration(capture.dependent, root);

    assert(target && retained && dependent);
    fatal_cursor = capture.retained;
    fatal_parent = root;
    fatal_objc_cursor = capture.objc;
    fatal_macro_cursor = capture.macro;
    fatal_formal_cursor = capture.formal;
    assert(capture.objc.kind == .ObjCInterfaceDecl && capture.macro.kind == .MacroDefinition && capture.formal.kind == .TemplateTypeParameter);
    fatal_seen = false;
    fatal_reentry = null;

    if mode == 6 {
        malformed: [] B.CXCursor;
        malformed.count = 1;
        assert(B.create_type_inner(B.clang_getCursorType(capture.retained), capture.retained, false, substitute_children = malformed) == null);
        assert(session.failed && fatal_seen && fatal_reentry == null && !fatal_objc && !fatal_macro && !fatal_formal);

        return;
    }

    // Public model edits form finite deep graphs and cycles. Cleanup must
    // follow actual identity edges, independent of nesting or allocation order.
    if mode == 0 {
        retained_leaf := generator.type_def_s32;
        dependent_leaf := target.type;

        for 0..319 {
            left := S.new_owned(session, B.CType);
            right := S.new_owned(session, B.CType);

            left.pointer_to = retained_leaf;
            right.pointer_to = dependent_leaf;
            retained_leaf = left;
            dependent_leaf = right;
        }

        retained.type = retained_leaf;
        dependent.type = dependent_leaf;
    } else if mode == 1 {
        retained_literal := S.new_owned(session, B.Literal);
        retained_literal.kind = .LITERAL;
        retained_leaf := cast(*B.Expression) retained_literal;
        dependent_literal := S.new_owned(session, B.Literal);
        dependent_literal.kind = .LITERAL;
        dependent_literal.literal_kind = .IDENTIFIER;
        dependent_literal.identifier = cast(*B.Declaration) target;
        dependent_leaf := cast(*B.Expression) dependent_literal;

        for 0..319 {
            left := S.new_owned(session, B.Paren_Expression);
            right := S.new_owned(session, B.Paren_Expression);

            left.kind = .PAREN_EXPRESSION;
            right.kind = .PAREN_EXPRESSION;
            left.child = retained_leaf;
            right.child = dependent_leaf;
            retained_leaf = cast(*B.Expression) left;
            dependent_leaf = cast(*B.Expression) right;
        }

        retained.expression = retained_leaf;
        dependent.expression = dependent_leaf;
    } else if mode == 2 {
        independent := S.new_owned(session, B.Declaration);
        independent.parent = independent;
        retained_leaf := independent;
        dependent_leaf := cast(*B.Declaration) target;

        for 0..319 {
            left := S.new_owned(session, B.Declaration);
            right := S.new_owned(session, B.Declaration);

            left.parent = retained_leaf;
            right.parent = dependent_leaf;
            retained_leaf = left;
            dependent_leaf = right;
        }

        retained.parent = retained_leaf;
        dependent.parent = dependent_leaf;
    } else if mode == 3 {
        // A cycle crossing type and expression nodes must terminate, without
        // concealing a dependency on a second branch of the same expression.
        left := S.new_owned(session, B.CType);
        right := S.new_owned(session, B.CType);
        left_reference := S.new_owned(session, B.Type_Reference);
        right_reference := S.new_owned(session, B.Type_Reference);

        left_reference.kind = .TYPE_REFERENCE;
        right_reference.kind = .TYPE_REFERENCE;
        left_reference.type = left;
        right_reference.type = right;
        left.pointer_to = left;
        right.pointer_to = right;
        left.array_element_count_expression = cast(*B.Expression) left_reference;
        binary := S.new_owned(session, B.Binary_Operator);
        binary.kind = .BINARY_OPERATOR;
        binary.lhs = cast(*B.Expression) right_reference;
        literal := S.new_owned(session, B.Literal);
        literal.kind = .LITERAL;
        literal.literal_kind = .IDENTIFIER;
        literal.identifier = cast(*B.Declaration) target;
        binary.rhs = cast(*B.Expression) literal;
        right.array_element_count_expression = cast(*B.Expression) binary;
        retained.type = left;
        dependent.type = right;
    } else {
        // Literal template arguments remain dependency edges when the
        // literal itself names an independent canonical declaration.
        left := S.new_owned(session, B.Literal);
        right := S.new_owned(session, B.Literal);

        left.kind = .LITERAL;
        right.kind = .LITERAL;
        left.literal_kind = .IDENTIFIER;
        right.literal_kind = .IDENTIFIER;
        left.identifier = retained;
        right.identifier = retained;
        reference := S.new_owned(session, B.Type_Reference);
        reference.kind = .TYPE_REFERENCE;
        reference.type = target.type;
        S.append_owned(session, *left.template_instantiation_params, cast(*B.Expression) left);
        S.append_owned(session, *right.template_instantiation_params, cast(*B.Expression) reference);
        if mode == 5 right.template_instantiation_params.count = 0x7fffffffffffffff;

        retained.expression = cast(*B.Expression) left;
        dependent.expression = cast(*B.Expression) right;
    }

    S.append_owned(session, *generator.global_scope.members, retained);
    S.append_owned(session, *generator.global_scope.members, dependent);
    generator.flattened_global_scope_members = S.copy_array(session, generator.global_scope.members);
    before := errors;

    assert(B.create_struct(capture.rejected) == null && errors > before);
    if mode == 5 assert(session.failed && fatal_seen && fatal_reentry == null);
    else assert(!session.failed);

    assert(H.table_contains(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(capture.retained)));
    assert(!H.table_contains(*generator.declarations_by_cursor, B.clang_getCanonicalCursor(capture.dependent)));
    assert(generator.global_scope.members.count == 1 && generator.global_scope.members[0] == retained);
    assert(generator.flattened_global_scope_members.count == 1 && generator.flattened_global_scope_members[0] == retained);
    if mode != 5 assert(B.create_declaration(capture.retained, root) == retained);
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);

    if args.count > 1 {
        mode, valid, rest := string_to_int(args[1]);
        assert(valid && !rest);
        exercise(mode);
    } else for mode: 0..6 exercise(mode);

    print("ok\n");
}
