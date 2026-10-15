#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    saved: #Context;
    children: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    if cur.kind == .ObjCInterfaceDecl || cur.kind == .ObjCProtocolDecl || cur.kind == .ObjCCategoryDecl || cur.kind == .TemplateTypeParameter {
        push_context {
            context = capture.saved;
            array_add(*capture.children, cur);
        }
    }

    return .Continue;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-objc.h", #filepath));

    defer free(filename);
    args: [2] *u8;
    args[0] = to_c_string("-x");
    args[1] = to_c_string("objective-c");
    defer free(args[0]);
    defer free(args[1]);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, args.data, 2, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *capture);
    defer array_free(capture.children);
    assert(capture.children.count == 4);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);

    push_context {
        context.generator = *generator;
        context._bindings_generator_session = *session;
        protocol := B.create_objc_interface(capture.children[0]);

        assert(protocol != null && (protocol.flags & .OBJC_PROTOCOL) != 0);
        assert(protocol.parents.count == 1 && protocol.parents[0].name == "ID");
        shell := B.create_objc_interface(capture.children[1], true);

        assert(shell.declarations.count == 0);
        root := B.create_objc_interface(capture.children[1]);

        assert(root == shell && root.declarations.count == 2);
        assert(root.declarations[0].foreign_name == "add:" && root.declarations[0].output_name == "add");
        assert((root.declarations[0].type.type_of_function.flags & .OBJC_METHOD) != 0);
        assert((root.declarations[1].type.type_of_function.flags & .OBJC_CLASS_METHOD) != 0);
        assert(root.declarations[0].type.type_of_function.arguments.count == 1);
        category := B.create_objc_interface(capture.children[2]);

        assert(category != null && category.parent == cast(*B.Declaration) root && category.declarations.count == 1);
        box := B.create_objc_interface(capture.children[3]);

        assert(box != null && box.template_type_params.count == 1 && box.declarations.count == 1);
        assert(box.template_type_params[0].type == generator.type_def_type);
        assert(box.declarations[0].type.type_of_function.return_type.type_of_declaration == box.template_type_params[0]);
        parameters: Capture;
        parameters.saved = context;
        B.clang_visitChildren(capture.children[3], visit, *parameters);
        defer array_free(parameters.children);
        assert(parameters.children.count == 1);
        assert(B.create_objc_type_param(parameters.children[0]) == box.template_type_params[0]);
        assert(!session.failed);
    }

    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
