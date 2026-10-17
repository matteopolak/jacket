#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

errors: int;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2 || args.count == 3);
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        // Native argument names may coincide with independently chosen locals.
        receiver := B.Declaration.{ output_name = "__objc_call_0", type = generator.type_def_ptr_void };
        value := B.Declaration.{ output_name = "__objc_selector_0", type = generator.type_def_s32 };
        signature := B.Function_Type.{ flags = .OBJC_METHOD, return_type = generator.type_def_s32 };

        array_add(*signature.arguments, *receiver, *value);
        defer array_free(signature.arguments);
        type := B.CType.{ type_of_function = *signature };
        library := B.Library_Info.{ identifier = "native" };
        function := B.Function.{ kind = .FUNCTION, name = "adjust:", output_name = "adjust", type = *type, library = *library, foreign_name = "adjust:" };
        output: String_Builder;

        defer free_buffers(*output);
        append(*output, "native :: #library \"independent_objc\";\n");
        B.print_function(*output, *function, null, function.foreign_name);
        assert(!output.failed);
        signature.flags = .OBJC_CLASS_METHOD;
        function.output_name = "class_adjust";
        B.print_function(*output, *function, null, function.foreign_name);
        assert(!output.failed);
        text := builder_to_string(*output);

        defer free(text);
        assert(write_entire_file(args[1], text));

        if args.count == 3 {
            append(*output, "native :: #library \"independent_objc\";\nIndependentObjCResult :: struct { first: s64; second: s64; }\n");
            signature.return_type = generator.type_def_float64;
            function.output_name = "floating_result";
            B.print_function(*output, *function, null, function.foreign_name);
            assert(!output.failed);
            record := B.Struct.{ kind = .STRUCT, output_name = "IndependentObjCResult" };
            aggregate := B.CType.{ type_of_struct = *record, size = 16, alignment = 8 };

            record.type = *aggregate;
            signature.return_type = *aggregate;
            function.output_name = "aggregate_result";
            B.print_function(*output, *function, null, function.foreign_name);
            assert(!output.failed);
            // Force both bodies through semantic checking without executing ABI
            // paths that are not established on every Mac architecture.
            append(*output, "main :: () { floating_result(null, 32); aggregate_result(null, 32); }\n");
            variants := builder_to_string(*output);

            defer free(variants);
            assert(contains(variants, "objc_msgSend_fpret") && contains(variants, "objc_msgSend_stret"));
            assert(write_entire_file(args[2], variants));
        }

        for index: 0..1 {
            invalid: String_Builder;

            defer free_buffers(*invalid);
            append(*invalid, "preserved");
            signature.return_type = generator.type_def_s32;
            signature.arguments.count = ifx index == 0 then 0 else 2;
            signature.flags = ifx index == 0 then .OBJC_METHOD else .OBJC_METHOD | .VARIADIC;

            push_context {
                context.logger = capture_error;
                B.print_function(*invalid, *function, null, function.foreign_name);
            }

            assert(invalid.failed && builder_string_length(*invalid) == 9 && errors == (index + 1) * 2);
        }
    }

    print("ok\n");
}
