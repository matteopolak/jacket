#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

Capture :: struct {
    saved: #Context;
    cursor: B.CXCursor;
}

collect :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        name := B.clang_getCursorSpelling(cur);
        defer B.clang_disposeString(name);
        if to_string(B.clang_getCString(name)) == "DispatchDerived" capture.cursor = cur;
    }

    return .Continue;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    generator: B.Generator;
    options: B.Generate_Bindings_Options;
    options.generate_vtable_helpers = false;
    options.generate_c_style_api_for_vtable = false;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        S :: type_of(context._bindings_generator_session.*);

        S.activate(*generator, *options, context._bindings_generator_session);
        generator.clang_index = B.clang_createIndex(0, 0);
        filename := to_c_string(tprint("%bindings-vtable-abi.hpp", #filepath));

        defer free(filename);
        assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, null, 0, null, 0, 0, *generator.translation_unit) == .Success);
        capture := Capture.{ saved = context };

        B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), collect, *capture);
        derived := B.create_struct(capture.cursor);

        assert(derived != null);
        info := B.find_or_create_extra_info(derived.type, cast(*B.Declaration) derived);

        assert(info && info.vtable_struct);
        output: String_Builder;

        defer free_buffers(*output);

        // The native test only needs the object pointer and callable table, not direct method symbols.
        for function, index: info.vtable {
            print("% %\n", index, function.name);
        }

        // The derived table reaches the base's slots through its first member, so the base's table goes first.
        if info.vtable_parent_info B.print_struct(*output, info.vtable_parent_info.vtable_struct, null);

        B.print_struct(*output, info.vtable_struct, null);
        assert(!output.failed);
        text := builder_to_string(*output);

        defer free(text);
        assert(write_entire_file(args[1], text));
    }
}
