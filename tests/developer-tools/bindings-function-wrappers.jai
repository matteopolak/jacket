#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    saved: #Context;
    cursors: [..] B.CXCursor;
}

wrapped: *B.Function;
foreign_argument, outer_argument, inner_argument: B.Declaration;
shared_view: [1] *B.Declaration;
modes: [3] int;
errors: int;

arguments :: (func: *B.Function, type: *B.Function_Type, mode: B.Arg_Print_Mode) -> [] *B.Declaration {
    if func != wrapped return type.arguments;

    modes[cast(int) mode] += 1;
    if mode == .FOREIGN_FUNC_DECLARATION shared_view[0] = *foreign_argument;
    else if mode == .OUTER_WRAPPER shared_view[0] = *outer_argument;
    else shared_view[0] = *inner_argument;

    return shared_view;
}

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        array_add(*capture.cursors, cur);
    }

    return .Continue;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    generator: B.Generator;
    options: B.Generate_Bindings_Options;
    options.get_func_args_for_printing = arguments;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        S.activate(*generator, *options, context._bindings_generator_session);
        generator.clang_index = B.clang_createIndex(0, 0);
        filename := to_c_string(tprint("%bindings-function-wrappers.h", #filepath));

        defer free(filename);
        native_args := (*u8).[to_c_string("-x"), to_c_string("c")];

        defer for argument: native_args free(argument);
        status := B.clang_parseTranslationUnit2(generator.clang_index, filename, native_args.data, 2, null, 0, 0, *generator.translation_unit);

        assert(status == .Success);
        capture: Capture;
        capture.saved = context;
        root := B.clang_getTranslationUnitCursor(generator.translation_unit);

        B.clang_visitChildren(root, visit, *capture);
        defer array_free(capture.cursors);
        record := B.create_struct(capture.cursors[0]);

        assert(record != null);
        wrapped = B.create_function(capture.cursors[1], root);
        formatting := B.create_function(capture.cursors[2], root);

        assert(wrapped != null && formatting != null);
        library: B.Library_Info;
        library.identifier = "native";
        wrapped.library = *library;
        formatting.library = *library;
        wrapped.output_name = "pair_sum";
        wrapped.decl_flags |= .NEEDS_ARGUMENT_WRAPPER;
        foreign_argument = wrapped.type.type_of_function.arguments[0].*;
        foreign_argument.output_name = "wire_pair";
        outer_argument = foreign_argument;
        outer_argument.output_name = "visible";
        outer_argument.decl_flags |= .POINTER_TO_VALUE;
        inner_argument = outer_argument;
        formatting.output_name = "format_message";
        formatting.decl_flags |= .NEEDS_PRINTF_WRAPPER;
        output: String_Builder;

        defer free_buffers(*output);
        append(*output, "#import \"Basic\";\nnative :: #foreign_library \"function_wrappers\";\n");
        B.print_struct(*output, record, null);
        B.print_function(*output, wrapped, null, "original_pointer_pair");
        B.print_function(*output, formatting, null, "original_format_adapter");
        assert(!output.failed && generator.generated_printf_wrapper);
        assert(modes[0] == 1 && modes[1] == 1 && modes[2] == 1);
        assert(wrapped.type.type_of_function.arguments[0].output_name == "pair");
        text := builder_to_string(*output);

        defer free(text);
        assert(contains(text, "format_message_CFormat") && !contains(text, "pair_sum_raw"));
        assert(contains(text, "wire_pair: *OriginalWrapperPair"));
        assert(write_entire_file(tprint("%/developer-tools-wrappers-generated.jai", args[1]), text));
        formatting.decl_flags &= ~B.Declaration_Flags.NEEDS_PRINTF_WRAPPER;
        formatting.decl_flags |= .NEEDS_ARGUMENT_WRAPPER;
        append(*output, "preserve");

        push_context {
            context.logger = capture_error;
            B.print_function(*output, formatting, null, "original_printf");
        }

        assert(output.failed && builder_string_length(*output) == "preserve".count);
        assert(errors > 0);
    }

    print("ok\n");
}
