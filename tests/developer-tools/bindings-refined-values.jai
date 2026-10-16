// Reviewed token text is passthrough, not C++ translation or source formatting.
#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

errors: int;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    literal: B.Literal;
    literal.literal_kind = .RAW_CPP_TOKENS;
    literal.raw_value = "  (42 /* retained bytes */)\n";
    B.print_literal_to_builder(*builder, *literal);
    assert(!builder.failed);
    text := builder_to_string(*builder);

    defer free(text);
    assert(text == literal.raw_value);
    B.print_expression_to_builder(*builder, cast(*B.Expression) *literal);
    assert(!builder.failed);
    expression := builder_to_string(*builder);

    defer free(expression);
    assert(expression == literal.raw_value);
    assert(B.append_cpp_literal(*builder, *literal));
    cpp_text := builder_to_string(*builder);

    defer free(cpp_text);
    assert(cpp_text == literal.raw_value);

    signed := B.CType.{ size = 16, alignment = 16, number_flags = .SIGNED };
    unsigned := B.CType.{ size = 16, alignment = 16 };

    B.print_type_to_builder(*builder, *signed, null);
    assert(!builder.failed);
    name := builder_to_string(*builder);

    defer free(name);
    assert(name == "S128");
    B.print_type_to_builder(*builder, *unsigned, null);
    assert(!builder.failed);
    unsigned_name := builder_to_string(*builder);

    defer free(unsigned_name);
    assert(unsigned_name == "U128");
    literal.literal_kind = .INTEGER;
    literal.int_value = -42;
    B.print_literal_to_builder(*builder, *literal, *signed);
    assert(!builder.failed);
    signed_value := builder_to_string(*builder);

    defer free(signed_value);
    assert(signed_value == "-42");
    literal.literal_kind = .UNSIGNED_INTEGER;
    literal.uint_value = 0xffffffffffffffff;
    B.print_literal_to_builder(*builder, *literal, *unsigned);
    assert(!builder.failed);
    unsigned_value := builder_to_string(*builder);

    defer free(unsigned_value);
    assert(unsigned_value == "18446744073709551615");
    record := B.Struct.{ kind = .STRUCT, output_name = "NotAnInteger" };
    aggregate := B.CType.{ size = 16, type_of_struct = *record };

    record.type = *aggregate;
    invalid: String_Builder;

    defer free_buffers(*invalid);
    append(*invalid, "preserved");

    push_context {
        context.logger = capture_error;
        B.print_literal_to_builder(*invalid, *literal, *aggregate);
    }

    assert(invalid.failed && errors == 1 && builder_string_length(*invalid) == 9);

    for width, index: int.[1, 2, 4, 8] {
        target := B.CType.{ size = width };

        B.append_as_signed_integer_hex(*builder, 0xffffffffffffffff, *target);
        signed_hex := builder_to_string(*builder);

        defer free(signed_hex);
        assert(signed_hex == "-0x1");
        B.append_as_unsigned_integer_hex(*builder, -1, *target);
        unsigned_hex := builder_to_string(*builder);

        defer free(unsigned_hex);
        assert(unsigned_hex == string.["0xff", "0xffff", "0xffffffff", "0xffffffffffffffff"][index]);
    }

    arguments := get_command_line_arguments();

    defer array_free(arguments);

    if arguments.count >= 2 {
        consumer := tprint("#import \"Basic\";\nvalue :: %;\nSigned :: %;\nUnsigned :: %;\nsigned_payload :: %;\nunsigned_payload :: cast(u64) (%);\nmain :: () { assert(value == 42 && size_of(Signed) == 16 && size_of(Unsigned) == 16 && signed_payload == -42 && unsigned_payload == 0xffffffffffffffff); print(\"ok\\n\"); }\n", text, name, unsigned_name, signed_value, unsigned_value);
        assert(write_entire_file(arguments[1], consumer));
    }

    if arguments.count == 3 assert(write_entire_file(arguments[2], tprint("constexpr int value = %;\nstatic_assert(value == 42);\nint main() { return value == 42 ? 0 : 1; }\n", cpp_text)));

    print("ok\n");
}
