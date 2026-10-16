#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    generator: B.Generator;
    options: B.Generate_Bindings_Options;

    push_context {
        B.init_generator(*generator, options);
        defer B.deinit_generator(*generator);
        record := B.Struct.{ kind = .STRUCT, output_name = "IndependentGeneric", decl_flags = .TEMPLATE };
        record_type := B.CType.{ type_of_struct = *record };

        record.type = *record_type;
        metatype := B.CType.{ hardcoded_jai_string = "Type" };
        parameter := B.Declaration.{ kind = .DECLARATION, output_name = "T", type = *metatype, parent = cast(*B.Declaration) *record };
        integer := B.CType.{ size = 4, number_flags = .SIGNED | ._32BIT };
        default_type := B.Type_Reference.{ kind = .TYPE_REFERENCE, type = *integer };

        parameter.expression = cast(*B.Expression) *default_type;
        number := B.Declaration.{ kind = .DECLARATION, output_name = "count", type = *integer, parent = cast(*B.Declaration) *record };
        literal := B.Literal.{ literal_kind = .INTEGER, int_value = 3 };

        number.expression = cast(*B.Expression) *literal;
        array_add(*record.template_type_params, *parameter, *number);
        defer array_free(record.template_type_params);
        element := B.CType.{ type_of_declaration = *parameter };
        bound := B.Literal.{ literal_kind = .IDENTIFIER, identifier = *number };
        array := B.CType.{ array_element_type = *element, array_element_count_expression = cast(*B.Expression) *bound };
        field := B.Declaration.{ kind = .DECLARATION, output_name = "data", type = *array };

        array_add(*record.declarations, *field);
        defer array_free(record.declarations);
        builder: String_Builder;

        defer free_buffers(*builder);
        B.print_struct(*builder, *record, null);
        assert(!builder.failed);
        text := builder_to_string(*builder);

        defer free(text);
        consumer := tprint("#import \"Basic\";\n%\nmain :: () { a: IndependentGeneric(); b: IndependentGeneric(u8, 7); a.data[2] = 42; b.data[6] = 17; assert(a.data.count == 3 && a.data[2] == 42 && b.data.count == 7 && b.data[6] == 17); print(\"ok\\n\"); }\n", text);

        assert(write_entire_file(args[1], consumer));
    }

    print("ok\n");
}
