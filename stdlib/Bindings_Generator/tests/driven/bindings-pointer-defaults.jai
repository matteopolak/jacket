// Public zero-to-null normalization must not mutate a shared source literal.
#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

main :: () {
    integer := B.CType.{ size = 4, number_flags = ._32BIT | .SIGNED };
    pointer := B.CType.{ pointer_to = *integer };
    alias := B.Typedef.{ kind = .TYPEDEF, name = "IntPointer", output_name = "IntPointer", type = *pointer };
    aliased := B.CType.{ type_of_typedef = *alias };
    zero := B.Literal.{ literal_kind = .INTEGER, int_value = 0, raw_value = "0", location = .{ line = 7, column = 3 } };
    argument := B.Declaration.{ output_name = "value", type = *aliased, expression = cast(*B.Expression) *zero };

    B.maybe_convert_expression_zero_to_null(*argument);
    assert(argument.expression != cast(*B.Expression) *zero);
    assert((cast(*B.Literal) argument.expression).literal_kind == .NULL);
    assert((cast(*B.Literal) argument.expression).location.line == 7);
    assert((cast(*B.Literal) argument.expression).raw_value == "0");
    assert(zero.literal_kind == .INTEGER && zero.int_value == 0);
    retained := argument.expression;

    defer free(retained);
    B.maybe_convert_expression_zero_to_null(*argument);
    assert(argument.expression == retained);

    ordinary := B.Declaration.{ type = *integer, expression = cast(*B.Expression) *zero };

    B.maybe_convert_expression_zero_to_null(*ordinary);
    assert(ordinary.expression == cast(*B.Expression) *zero);
    nonzero := B.Literal.{ literal_kind = .INTEGER, int_value = 1 };

    ordinary.type = *pointer;
    ordinary.expression = cast(*B.Expression) *nonzero;
    B.maybe_convert_expression_zero_to_null(*ordinary);
    assert(ordinary.expression == cast(*B.Expression) *nonzero);

    unsigned_zero := B.Literal.{ literal_kind = .UNSIGNED_INTEGER, uint_value = 0 };

    ordinary.expression = cast(*B.Expression) *unsigned_zero;
    B.maybe_convert_expression_zero_to_null(*ordinary);
    assert((cast(*B.Literal) ordinary.expression).literal_kind == .NULL);
    free(ordinary.expression);
    real_zero := B.Literal.{ literal_kind = .FLOAT, float_value = 0 };

    ordinary.expression = cast(*B.Expression) *real_zero;
    B.maybe_convert_expression_zero_to_null(*ordinary);
    assert(ordinary.expression == cast(*B.Expression) *real_zero);
    reference := pointer;
    reference.flags = .REFERENCE;
    ordinary.type = *reference;
    ordinary.expression = cast(*B.Expression) *zero;
    B.maybe_convert_expression_zero_to_null(*ordinary);
    assert(ordinary.expression == cast(*B.Expression) *zero);

    // The public signature printer consumes an unnormalized shared default.
    argument.type = *pointer;
    argument.expression = cast(*B.Expression) *zero;
    function := B.Function_Type.{ return_type = *integer };

    array_add(*function.arguments, *argument);
    defer array_free(function.arguments);
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "native :: #library \"developer_tools_pointer_defaults\";\npointer_default :: ");
    B.print_function_type_to_builder(*builder, *function, null);
    assert(!builder.failed);
    assert(argument.expression == cast(*B.Expression) *zero && zero.literal_kind == .INTEGER);
    append(*builder, " #foreign native \"pointer_default\";\n");
    text := builder_to_string(*builder);

    defer free(text);
    assert(write_entire_file(tprint("%../../../../target/developer-tools-pointer-defaults-generated.jai", #filepath), text));
    print("ok\n");
}
