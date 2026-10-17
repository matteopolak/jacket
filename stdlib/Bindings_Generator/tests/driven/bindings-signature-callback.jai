#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

selected: [1] *B.Declaration;
expected_type: *B.Function_Type;
expected_func: *B.Function;
calls: int;

arguments :: (func: *B.Function, type: *B.Function_Type, mode: B.Arg_Print_Mode) -> [] *B.Declaration {
    assert(func == expected_func && type == expected_type && mode == .FOREIGN_FUNC_DECLARATION);
    calls += 1;
    return selected;
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
        original, replacement: B.Declaration;
        original.output_name = "original";
        original.type = generator.type_def_float64;
        replacement.output_name = "value";
        replacement.type = generator.type_def_s32;
        literal: B.Literal;
        literal.literal_kind = .INTEGER;
        literal.int_value = 41;
        replacement.expression = cast(*B.Expression) *literal;
        type: B.Function_Type;
        type.return_type = generator.type_def_s32;
        array_add(*type.arguments, *original);
        defer array_free(type.arguments);
        function: B.Function;

        expected_type = *type;
        expected_func = *function;
        selected[0] = *replacement;
        output: String_Builder;

        defer free_buffers(*output);
        append(*output, "native :: #foreign_library \"signature_callback\";\ncall :: ");
        B.print_function_type_to_builder(*output, *type, null, *function);
        append(*output, " #foreign native \"original_signature\";\n");
        assert(!output.failed && calls == 1 && type.arguments[0] == *original);
        text := builder_to_string(*output);

        defer free(text);
        assert(write_entire_file(tprint("%/developer-tools-signature-generated.jai", args[1]), text));
        B.print_function_type_to_builder(*output, *type, null);
        assert(!output.failed && calls == 1);
    }

    print("ok\n");
}
