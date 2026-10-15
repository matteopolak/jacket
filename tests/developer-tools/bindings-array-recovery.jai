#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    function: B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;
    if cur.kind == .FunctionDecl capture.function = cur;
    return .Continue;
}

main :: () {
    generator: B.Generator;

    push_context {
        B.init_generator(*generator, .{});
        defer B.deinit_generator(*generator);
        generator.clang_index = B.clang_createIndex(0, 0);
        filename := to_c_string(tprint("%bindings-array-recovery.h", #filepath));

        defer free(filename);
        arguments := (*u8).[to_c_string("-x"), to_c_string("c")];

        defer for argument: arguments free(argument);
        parsed := B.clang_parseTranslationUnit2(generator.clang_index, filename, arguments.data, 2, null, 0, 0, *generator.translation_unit);

        assert(parsed == .Success);
        root := B.clang_getTranslationUnitCursor(generator.translation_unit);
        capture: Capture;

        B.clang_visitChildren(root, visit, *capture);
        function := B.create_function(capture.function, root);

        assert(function != null);
        type := function.type.type_of_function.arguments[0].type;

        assert(type.pointer_to != null);
        recovered := B.maybe_convert_pointer_type_to_fixed_array_type(B.clang_Cursor_getArgument(capture.function, 0), type);

        assert(recovered != type && recovered.array_element_type == type.pointer_to && recovered.array_element_count == 7 && recovered.size == 28);
        assert(type.pointer_to == generator.type_def_s32);
        plain := function.type.type_of_function.arguments[1].type;

        assert(B.maybe_convert_pointer_type_to_fixed_array_type(B.clang_Cursor_getArgument(capture.function, 1), plain) == plain);
    }

    print("ok\n");
}
