// Reviewed public Enum model and print_enum/print_enum_body workflows.
#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

errors: int;

capture :: (_message: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2, "supply a private output directory");
    generator: B.Generator;
    options: B.Generate_Bindings_Options;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);

    push_context {
        S.activate(*generator, *options, *session);

        integer: B.CType;
        integer.size = 4;
        integer.alignment = 4;
        integer.number_flags = .SIGNED | ._32BIT;
        negative: B.Literal;
        negative.literal_kind = .INTEGER;
        negative.int_value = -7;
        positive := negative;
        positive.int_value = 5;
        hidden := negative;
        hidden.int_value = 99;
        first, second, omitted: B.Declaration;
        first.name = "NATIVE_FAILED";
        first.output_name = "Failed";
        first.expression = cast(*B.Expression) *negative;
        second.name = "NATIVE_READY";
        second.output_name = "Ready";
        second.expression = cast(*B.Expression) *positive;
        omitted.name = "OMITTED_NATIVE_MEMBER";
        omitted.output_name = omitted.name;
        omitted.expression = cast(*B.Expression) *hidden;
        omitted.decl_flags = .OMIT_FROM_OUTPUT;
        enumeration: B.Enum;
        enumeration.kind = .ENUM;
        enumeration.name = "NativeStatus";
        enumeration.output_name = "Status";
        enumeration.type = *integer;
        enumeration.flags = .ALIAS_ORIGINAL_NAMES | .VALUES_IN_HEX;
        first.parent = cast(*B.Declaration) *enumeration;
        second.parent = first.parent;
        omitted.parent = first.parent;
        array_add(*enumeration.enumerates, *first, *omitted, *second);
        defer array_free(enumeration.enumerates);
        output: String_Builder;

        defer free_buffers(*output);
        B.print_enum(*output, *enumeration);
        assert(!output.failed);
        append(*output, "BodyOnly :: ");
        B.print_enum_body(*output, *enumeration);
        append(*output, "\n");
        append(*output, "native :: #foreign_library \"enum_printers\";\ncheck_status :: ");
        status_type: B.CType;
        status_type.type_of_enum = *enumeration;
        status_type.size = 4;
        status_type.alignment = 4;
        argument: B.Declaration;
        argument.output_name = "status";
        argument.type = *status_type;
        argument.expression = cast(*B.Expression) *positive;
        signature: B.Function_Type;
        signature.return_type = *integer;
        array_add(*signature.arguments, *argument);
        defer array_free(signature.arguments);
        B.print_function_type_to_builder(*output, *signature, null);
        append(*output, " #foreign native \"original_enum_value\";\n");
        enumeration.output_name = "Mask";
        enumeration.flags = .IS_ENUM_FLAGS | .VALUES_IN_HEX;
        options.alias_original_enum_names = false;
        positive.int_value = 4;
        negative.int_value = 1;
        B.print_enum(*output, *enumeration);
        assert(!output.failed);
        enumeration.output_name = "Default";
        enumeration.flags = 0;
        enumeration.type = null;
        B.print_enum(*output, *enumeration);
        assert(!output.failed && enumeration.type == null);
        assert(generator.counts.enums == 4);
        text := builder_to_string(*output);

        defer free(text);
        assert(!contains(text, "OMITTED_NATIVE_MEMBER"));
        assert(contains(text, "Default :: enum {"));
        assert(write_entire_file(tprint("%/developer-tools-enums-generated.jai", args[1]), text));

        append(*output, "preserve");
        second.output_name = "bad name";

        push_context {
            context.logger = capture;
            B.print_enum(*output, *enumeration);
        }

        assert(output.failed && errors == 1);
        assert(builder_string_length(*output) == "preserve".count);
        assert(enumeration.enumerates.count == 3 && first.expression == cast(*B.Expression) *negative);
        print("ok\n");
    }
}
