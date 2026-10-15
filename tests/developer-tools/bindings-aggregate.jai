// Reviewed Struct_Literal stores only supplied type_name and expression values.
#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

errors: s64;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 3, "supply private output directory and native library path");
    one: B.Literal;
    one.kind = .LITERAL;
    one.literal_kind = .INTEGER;
    one.int_value = 11;
    two := one;
    two.int_value = 22;
    values := (*B.Expression).[cast(*B.Expression) *one, cast(*B.Expression) *two];
    aggregate: B.Struct_Literal;
    aggregate.kind = .STRUCT_LITERAL;
    aggregate.type_name = "Packet";
    aggregate.values = values;
    packet: B.Struct;
    packet.kind = .STRUCT;
    packet.name = "Packet";
    packet.output_name = "Packet";
    packet_type: B.CType;
    packet_type.type_of_struct = *packet;
    packet_type.size = 8;
    packet_type.alignment = 4;
    integer: B.CType;
    integer.number_flags = .SIGNED | ._32BIT;
    integer.size = 4;
    integer.alignment = 4;
    argument: B.Declaration;
    argument.type = *packet_type;
    argument.output_name = "packet";
    argument.expression = cast(*B.Expression) *aggregate;
    function: B.Function_Type;
    function.return_type = *integer;
    array_add(*function.arguments, *argument);
    defer array_free(function.arguments);
    jai: String_Builder;

    defer free_buffers(*jai);
    print_to_builder(*jai, "Packet :: struct { first: s32; second: s32; }\nnative :: #foreign_library \"%\";\ndefault_packet :: ", args[2]);
    B.print_function_type_to_builder(*jai, *function, null);
    append(*jai, " #foreign native \"original_packet\";\nmake_packet :: () -> Packet { return ");
    B.print_expression_to_builder(*jai, cast(*B.Expression) *aggregate, *packet_type);
    append(*jai, "; }\n");
    assert(!jai.failed);
    text := builder_to_string(*jai);

    defer free(text);
    assert(write_entire_file(tprint("%/developer-tools-aggregate-generated.jai", args[1]), text));
    cpp: String_Builder;

    defer free_buffers(*cpp);
    append(*cpp, "#include \"bindings-aggregate.h\"\nint main() { Packet value = ");
    aggregate.type_name = "Native::Packet";
    assert(B.append_cpp_expression(*cpp, cast(*B.Expression) *aggregate));
    append(*cpp, "; return original_packet(value) == 33 ? 0 : 1; }\n");
    cpp_text := builder_to_string(*cpp);

    defer free(cpp_text);
    assert(write_entire_file(tprint("%/developer-tools-aggregate-consumer.cpp", args[1]), cpp_text));
    reset(*cpp);
    append(*cpp, "preserve");
    aggregate.values = .{ 1, values.data };
    values[0] = cast(*B.Expression) *aggregate;
    assert(!B.append_cpp_expression(*cpp, cast(*B.Expression) *aggregate));
    // Observing the destination must not consume it before the failure check.
    unchanged := builder_to_string(*cpp, do_reset = false);

    defer free(unchanged);
    assert(unchanged == "preserve");

    push_context {
        context.logger = capture;
        B.print_expression_to_builder(*cpp, cast(*B.Expression) *aggregate);
    }

    assert(cpp.failed && errors == 1 && builder_string_length(*cpp) == "preserve".count);
    print("ok\n");
}
