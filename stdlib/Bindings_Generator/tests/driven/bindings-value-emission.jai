// Original values/expressions exercise generated Jai, not exact formatting.
#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

main :: () {
    source: String_Builder;

    defer free_buffers(*source);
    literal: B.Literal;
    unsigned := B.CType.{ size = 1, number_flags = ._8BIT };
    signed := B.CType.{ size = 1, number_flags = ._8BIT | .SIGNED };
    real := B.CType.{ size = 4, number_flags = ._32BIT | .FLOAT };
    boolean := B.CType.{ size = 1, flags = .BOOL };

    append(*source, "minimum :: ");
    B.append_integer(*source, -9223372036854775807 - 1, true);
    append(*source, ";\nmaximum :: ");
    literal.literal_kind = .UNSIGNED_INTEGER;
    literal.uint_value = 0xffffffffffffffff;
    assert(!B.print_literal_to_builder(*source, *literal));
    append(*source, ";\nunsigned_byte :: cast(u8) ");
    B.append_as_unsigned_integer_hex(*source, -1, *unsigned);
    append(*source, ";\nsigned_byte :: cast(s8) ");
    B.append_as_signed_integer_hex(*source, 0xff, *signed);
    append(*source, ";\nconverted_unsigned :: ");
    literal.literal_kind = .INTEGER;
    literal.int_value = -1;
    assert(!B.print_literal_to_builder(*source, *literal, *unsigned));
    append(*source, ";\nconverted_signed :: ");
    literal.literal_kind = .UNSIGNED_INTEGER;
    literal.uint_value = 255;
    assert(!B.print_literal_to_builder(*source, *literal, *signed));
    append(*source, ";\nconverted_bool :: ");
    assert(!B.print_literal_to_builder(*source, *literal, *boolean));
    append(*source, ";\nconverted_float :: ");
    literal.literal_kind = .FLOAT;
    literal.float_value = 0.10000000000000002;
    assert(!B.print_literal_to_builder(*source, *literal, *real));
    append(*source, ";\nnegative_zero :: ");
    raw_negative_zero: u64 = 0x8000000000000000;

    literal.float_value = (cast(*float64) *raw_negative_zero).*;
    assert(!B.print_literal_to_builder(*source, *literal));
    append(*source, ";\nbytes :: ");
    bytes := u8.[0, 1, 10, 13, 34, 92, 255, 65, 55];

    literal.literal_kind = .STRING;
    literal.string_value = .{ bytes.count, bytes.data };
    assert(!B.print_literal_to_builder(*source, *literal));
    append(*source, ";\nMode :: enum s32 { VALUE :: 7; }\nmode :: ");
    enumeration := B.Enum.{ kind = .ENUM, name = "Original_Mode", output_name = "Mode" };
    value := B.Declaration.{ name = "ORIGINAL_VALUE", output_name = "VALUE" };

    literal.literal_kind = .ENUM;
    literal.enum_type = *enumeration;
    literal.enum_value = *value;
    assert(B.print_literal_to_builder(*source, *literal));
    append(*source, ";\nactual_value :: 42;\nidentifier :: ");
    identifier := B.Declaration.{ output_name = "actual_value" };

    literal.literal_kind = .IDENTIFIER;
    literal.identifier = *identifier;
    assert(!B.print_literal_to_builder(*source, *literal));
    left: B.Literal;
    right: B.Literal;

    left.literal_kind = .INTEGER;
    left.int_value = 2;
    right.literal_kind = .INTEGER;
    right.int_value = 5;
    addition := B.Binary_Operator.{ kind = .BINARY_OPERATOR, op = "+", lhs = cast(*B.Expression) *left, rhs = cast(*B.Expression) *right };
    multiplier: B.Literal;
    multiplier.literal_kind = .INTEGER;
    multiplier.int_value = 3;
    multiply := B.Binary_Operator.{ kind = .BINARY_OPERATOR, op = "*", lhs = cast(*B.Expression) *addition, rhs = cast(*B.Expression) *multiplier };

    append(*source, ";\nprecedence :: ");
    assert(!B.print_expression_to_builder(*source, cast(*B.Expression) *multiply));
    append(*source, ";\nleft_shift :: ");
    B.append_maybe_left_shift(*source, "3", 4);
    append(*source, ";\nright_shift :: ");
    B.append_maybe_right_shift(*source, "80", 3);
    append(*source, ";\nzero_shift :: ");
    B.append_maybe_left_shift(*source, "19", 0);
    append(*source, ";\ncomplement :: ");
    right.literal_kind = .UNSIGNED_INTEGER;
    right.uint_value = 5;
    complement := B.Unary_Operator.{ kind = .UNARY_OPERATOR, op = "~", child = cast(*B.Expression) *right };

    assert(!B.print_expression_to_builder(*source, cast(*B.Expression) *complement));
    append(*source, ";\nnative_size :: ");
    integer_type := B.CType.{ size = 4, number_flags = ._32BIT | .SIGNED };
    reference := B.Type_Reference.{ kind = .TYPE_REFERENCE, type = *integer_type };
    size := B.Unary_Operator.{ kind = .UNARY_OPERATOR, op = "sizeof", child = cast(*B.Expression) *reference };

    assert(!B.print_expression_to_builder(*source, cast(*B.Expression) *size));
    append(*source, ";\n");
    assert(!source.failed);
    text := builder_to_string(*source);

    defer free(text);
    assert(write_entire_file(tprint("%../../../../target/developer-tools-value-emission-generated.jai", #filepath), text));
    print("ok\n");
}
