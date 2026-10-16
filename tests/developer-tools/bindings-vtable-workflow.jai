#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);
errors: int;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

Capture :: struct {
    saved: #Context;
    cursor: B.CXCursor;
}

collect :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        if cur.kind == .StructDecl capture.cursor = cur;
    }

    return .Continue;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2 || args.count == 4);
    generator: B.Generator;
    options: B.Generate_Bindings_Options;
    options.generate_vtable_helpers = false;
    options.generate_c_style_api_for_vtable = false;

    if args.count == 4 {
        options.generate_c_style_api_for_vtable = args[2] == "true";
        options.generate_vtable_helpers = args[3] == "true";
    }

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        S.activate(*generator, *options, context._bindings_generator_session);
        generator.clang_index = B.clang_createIndex(0, 0);
        filename := to_c_string(tprint("%bindings-vtable-workflow.hpp", #filepath));

        defer free(filename);
        native_args := (*u8).[to_c_string("-x"), to_c_string("c++")];

        defer for argument: native_args free(argument);
        assert(B.clang_parseTranslationUnit2(generator.clang_index, filename, native_args.data, 2, null, 0, 0, *generator.translation_unit) == .Success);
        capture := Capture.{ saved = context };

        B.clang_visitChildren(B.clang_getTranslationUnitCursor(generator.translation_unit), collect, *capture);
        record := B.create_struct(capture.cursor);

        assert(record && record.virtual_functions.count == 2);
        // Exclude direct foreign method declarations while retaining canonical slots.
        count: int;

        for member: record.declarations {
            if member.kind == .FUNCTION continue;
            record.declarations[count] = member;
            count += 1;
        }

        record.declarations.count = count;
        record.virtual_functions[1].decl_flags |= .OMIT_FROM_OUTPUT;
        info, _created := B.find_or_create_extra_info(record.type, cast(*B.Declaration) record);

        assert(info && info.vtable_struct && info.vtable_struct.declarations.count == 2);
        output: String_Builder;

        defer free_buffers(*output);
        B.print_struct(*output, record, null);
        assert(!output.failed);
        text := builder_to_string(*output);

        defer free(text);
        assert(contains(text, "IndependentDispatch_VTable :: struct"));
        assert(contains(text, "slot_0") && contains(text, "slot_1: *void"));
        assert(contains(text, "IndependentDispatch_slot_0 ::") == options.generate_c_style_api_for_vtable);
        assert(contains(text, "vtable ::") == options.generate_vtable_helpers);
        assert(!contains(text, "IndependentDispatch_slot_1 ::"));
        assert(write_entire_file(args[1], text));
        // Metadata's variadic slot still prints; only its automatic wrapper skips.
        slot := info.vtable_struct.declarations[0].type.type_of_function;
        flags := slot.flags;

        slot.flags |= .VARIADIC;
        saved_wrappers := options.generate_c_style_api_for_vtable;

        options.generate_c_style_api_for_vtable = true;
        skipped: String_Builder;

        defer free_buffers(*skipped);

        push_context {
            context.logger = capture_error;
            B.print_struct_vtable(*skipped, record, info.*);
        }

        assert(!skipped.failed && errors == 1);
        skipped_text := builder_to_string(*skipped);

        defer free(skipped_text);
        assert(contains(skipped_text, "slot_0") && contains(skipped_text, "slot_1"));
        assert(!contains(skipped_text, "IndependentDispatch_slot_0 ::"));
        slot.flags = flags;
        options.generate_c_style_api_for_vtable = saved_wrappers;
    }

    print("ok\n");
}
