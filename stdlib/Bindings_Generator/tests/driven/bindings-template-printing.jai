#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    integer: B.CType;
    integer.size = 4;
    integer.number_flags = .SIGNED | ._32BIT;
    integer64 := integer;
    integer64.size = 8;
    integer64.number_flags = .SIGNED | ._64BIT;
    metatype: B.CType;
    metatype.hardcoded_jai_string = "Type";
    formal_type, formal_count: B.Declaration;
    formal_type.type = *metatype;
    formal_count.type = *integer64;
    formals := (*B.Declaration).[*formal_type, *formal_count];
    reference: B.Type_Reference;
    reference.kind = .TYPE_REFERENCE;
    reference.type = *integer;
    count: B.Literal;
    count.literal_kind = .INTEGER;
    count.int_value = 3;
    params := (*B.Expression).[cast(*B.Expression) *reference, cast(*B.Expression) *count];
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "Box :: struct(T: Type, count: s64) { data: [count] T; }\nExplicit :: Box");
    B.append_template_instantiation_params(*builder, params, formals, null);
    append(*builder, ";\n");
    record: B.Struct;
    record.kind = .STRUCT;
    record.output_name = "Box";
    array_add(*record.template_type_params, *formal_type, *formal_count);
    defer array_free(record.template_type_params);
    application: B.CType;
    application.type_of_struct = *record;
    array_add(*application.template_instantiation_params, params[0], params[1]);
    defer array_free(application.template_instantiation_params);
    append(*builder, "Selected :: ");
    B.print_type_to_builder(*builder, *application, null);
    append(*builder, ";\n");
    alias: B.Typedef;
    alias.kind = .TYPEDEF;
    alias.output_name = "Plain";
    alias.type = *integer;
    B.print_typedef(*builder, *alias);
    alias.output_name = "Distinct";
    alias.flags = .DISTINCT;
    B.print_typedef(*builder, *alias);
    alias.output_name = "Inherited";
    alias.flags = .ISA;
    B.print_typedef(*builder, *alias);
    assert(!builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(write_entire_file(tprint("%/developer-tools-templates-generated.jai", args[1]), text));
    before := builder_string_length(*builder);

    B.append_template_instantiation_params(*builder, .{}, .{}, null);
    assert(builder_string_length(*builder) == before);
    assert(reference.type == *integer && application.template_instantiation_params[0] == cast(*B.Expression) *reference);
    print("ok\n");
}
