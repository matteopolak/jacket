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
    filename := to_c_string(tprint("%generator-model-cpp.hpp", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
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
        S.activate(*generator, null, *session);
        standalone_type := B.create_function_type(capture.children[3], B.clang_getCursorType(capture.children[3]));

        assert(standalone_type != null && standalone_type.arguments.count == 1);
        standalone := B.create_function(capture.children[3], B.clang_getTranslationUnitCursor(unit));

        assert(standalone != null && standalone.type.type_of_function != null);
        assert(standalone.type.type_of_function.arguments[0] == standalone_type.arguments[0]);
        assert(standalone.type.type_of_function.arguments[0].parent == cast(*B.Declaration) standalone);
        // A first types-only lookup must not permanently erase later functions.
        namespace_shell := B.create_namespace(capture.children[0], true);

        assert(namespace_shell != null && namespace_shell.declarations.count == 3);
        space := B.create_namespace(capture.children[0]);

        assert(space == namespace_shell && space.declarations.count == 7);
        item := cast(*B.Struct) space.declarations[0];

        assert(item.declarations.count == 9);
        constructor := cast(*B.Function) item.declarations[1];
        destructor := cast(*B.Function) item.declarations[2];
        read := cast(*B.Function) item.declarations[3];
        run := cast(*B.Function) item.declarations[4];
        abstract_read := cast(*B.Function) item.declarations[5];
        with_ref := cast(*B.Function) item.declarations[6];

        assert(constructor.output_name == "constructor" && destructor.output_name == "destructor");
        assert((constructor.type.type_of_function.flags & .CONSTRUCTOR) != 0);
        assert((destructor.type.type_of_function.flags & .DESTRUCTOR) != 0);
        assert(read.type.type_of_function.arguments.count == 2);
        receiver := read.type.type_of_function.arguments[0];

        assert(receiver.name == "this" && receiver.type.pointer_to.type_of_struct == item);
        assert((receiver.type.flags & .CONST) != 0);
        assert(read.type.type_of_function.arguments[1].name == "unknown0");
        assert(run.type.type_of_function.arguments.count == 1);
        assert((abstract_read.flags & .PURE_VIRTUAL) != 0 && (abstract_read.flags & .VIRTUAL) != 0);
        assert((abstract_read.decl_flags & .NEEDS_ARGUMENT_WRAPPER) == 0);
        assert((with_ref.decl_flags & .NEEDS_ARGUMENT_WRAPPER) != 0);
        assert((item.declarations[7].decl_flags & .PRIVATE) != 0);
        assert(item.virtual_functions.count == 1 && item.virtual_functions[0] == abstract_read);
        assert((item.flags & .IS_ABSTRACT) != 0);
        factory := cast(*B.Function) item.declarations[8];
        callback := factory.type.type_of_function.return_type.pointer_to.type_of_function;

        assert(factory.type.type_of_function.arguments.count == 1);
        assert(callback != null && callback.arguments.count == 1 && callback.flags == 0);
        assert((space.declarations[3].decl_flags & .OMIT_FROM_OUTPUT) != 0);
        later := cast(*B.Struct) space.declarations[4];
        before := cast(*B.Function) space.declarations[5];
        nested := cast(*B.Namespace) space.declarations[6];

        assert(later.name == "Later" && later.declarations.count == 1);
        assert(before.type.type_of_function.arguments[0].type.pointer_to.type_of_struct == later);
        assert(nested.declarations.count == 2);
        assert(nested.declarations[0].name == "first" && nested.declarations[1].name == "second");
        // Completed canonical scopes preserve consumer changes on later lookup.
        later.output_name = "RenamedLater";
        later.declarations[0].decl_flags |= .OMIT_FROM_OUTPUT;
        assert(B.create_namespace(capture.children[2]) == space);
        assert(later.output_name == "RenamedLater");
        assert((later.declarations[0].decl_flags & .OMIT_FROM_OUTPUT) != 0);
        assert(!session.failed);
    }

    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
