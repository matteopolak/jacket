#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

check_cpp :: (type: *B.CType, flags: B.Print_Decl_Flags, expected: string) {
    builder: String_Builder;

    defer free_buffers(*builder);
    B.append_cpp_type(*builder, type, null, flags);
    assert(!builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(text == expected);
}

main :: () {
    integer := B.CType.{ size = 4, alignment = 4, number_flags = .SIGNED | ._32BIT };
    array := B.CType.{ array_element_type = *integer, array_element_count = 3 };
    pointer := B.CType.{ pointer_to = *array };
    inserted := B.CType.{ pointer_to = *array, flags = .INSERTED };
    reference := B.CType.{ pointer_to = *integer, flags = .REFERENCE };

    check_cpp(*array, 0, "int *");
    check_cpp(*pointer, 0, "int **");
    check_cpp(*inserted, 0, "int *");
    check_cpp(*reference, .IN_FUNCTION_ARGS, "int &");
    check_cpp(*reference, .IN_FUNCTION_RETURN, "int *");
    incomplete := B.CType.{ array_element_type = *integer, array_element_count = -1 };
    text := B.type_to_string(*incomplete, null);

    defer free(text);
    assert(text == "*s32");
    dimension: B.Literal;
    dimension.literal_kind = .INTEGER;
    dimension.int_value = 7;
    dependent := incomplete;
    dependent.array_element_count_expression = cast(*B.Expression) *dimension;
    bound := B.type_to_string(*dependent, null);

    defer free(bound);
    assert(bound == "[7] s32");
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        S.activate(*generator, *options, context._bindings_generator_session);
        alias := B.Typedef.{ kind = .TYPEDEF, name = "va_list", output_name = "NativeVaList", type = *integer };
        list_type := B.CType.{ type_of_typedef = *alias };
        argument := B.Declaration.{ kind = .DECLARATION, output_name = "arguments", type = *list_type };
        function: B.Function_Type;
        function.return_type = *integer;
        array_add(*function.arguments, *argument);
        defer array_free(function.arguments);
        function_type := B.CType.{ type_of_function = *function };
        opaque := B.type_to_string(*function_type, null);

        defer free(opaque);
        assert(opaque.count > 5 && opaque.data[0] == #char "*" && opaque.data[1] == #char "v");
    }

    print("ok\n");
}
