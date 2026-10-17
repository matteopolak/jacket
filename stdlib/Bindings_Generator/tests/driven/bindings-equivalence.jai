// Reviewed limited expression/literal equivalence; original public-model values.
#import "Basic";
B :: #import "Bindings_Generator";

errors: s64;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    signed := B.get_int_type(4, true);

    defer free(signed);
    same_integer := signed.*;
    same_integer.flags = .CONST | .VOLATILE;
    same_integer.number_flags = .SIGNED | ._64BIT; // Numeric size, not redundant width flags, is compared.
    assert(B.is_same_type(signed, *same_integer));
    same_integer.size = 8;
    assert(!B.is_same_type(signed, *same_integer));
    same_integer = signed.*;
    same_integer.number_flags &= ~B.Number_Flags.SIGNED;
    assert(!B.is_same_type(signed, *same_integer));
    same_integer = signed.*;
    alias: B.Typedef;
    alias.type = signed;
    alias_type: B.CType;
    alias_type.type_of_typedef = *alias;
    assert(B.is_same_type(*alias_type, *same_integer));
    pointer, reference: B.CType;
    pointer.pointer_to = signed;
    reference.pointer_to = *same_integer;
    reference.flags = .REFERENCE | .CONST;
    assert(B.is_same_type(*pointer, *reference));
    same_integer.size = 8;
    assert(!B.is_same_type(*pointer, *reference));
    same_integer = signed.*;
    record, another_record: B.Struct;
    named, another_named: B.CType;
    named.type_of_struct = *record;
    another_named.type_of_struct = *record;
    assert(B.is_same_type(*named, *another_named));
    another_named.type_of_struct = *another_record;
    assert(!B.is_same_type(*named, *another_named));
    function, other_function: B.Function_Type;
    argument, other_argument: B.Declaration;
    argument.type = signed;
    other_argument.type = *same_integer;
    function.return_type = signed;
    other_function.return_type = *same_integer;
    other_function.flags = .INLINE | .CPP_METHOD;
    array_add(*function.arguments, *argument);
    array_add(*other_function.arguments, *other_argument);
    defer array_free(function.arguments);
    defer array_free(other_function.arguments);
    named = .{};
    another_named = .{};
    named.type_of_function = *function;
    another_named.type_of_function = *other_function;
    assert(B.is_same_type(*named, *another_named));
    same_integer.size = 8;
    assert(!B.is_same_type(*named, *another_named));
    same_integer = signed.*;
    assert(B.is_same_type(null, null) && !B.is_same_type(signed, null));
    left, right: B.Literal;

    assert(B.is_same_literal(*left, *right));
    assert(B.is_same_literal(null, null));
    assert(!B.is_same_literal(*left, null));
    left.literal_kind = .INTEGER;
    left.int_value = -9223372036854775807 - 1;
    right = left;
    right.raw_value = "different original spelling";
    right.location.line = 9;
    assert(B.is_same_literal(*left, *right));
    right.int_value += 1;
    assert(!B.is_same_literal(*left, *right));
    left.literal_kind = .UNSIGNED_INTEGER;
    left.uint_value = 0xffffffffffffffff;
    right = left;
    assert(B.is_same_literal(*left, *right));
    right.literal_kind = .INTEGER;
    assert(!B.is_same_literal(*left, *right));
    left.literal_kind = .FLOAT;
    left.float_value = 0.0;
    right = left;
    negative_zero: u64 = 0x8000000000000000;

    right.float_value = (cast(*float64) *negative_zero).*;
    assert(B.is_same_literal(*left, *right));
    nan: u64 = 0x7ff8000000000001;

    left.float_value = (cast(*float64) *nan).*;
    right = left;
    assert(!B.is_same_literal(*left, *right));
    assert(B.is_same_literal(*left, *left));
    left.literal_kind = .STRING;
    left.string_value = "one\0two";
    right = left;
    copied := copy_string(left.string_value);

    defer free(copied);
    right.string_value = copied;
    assert(left.string_value.data != right.string_value.data);
    assert(B.is_same_literal(*left, *right));
    right.string_value.count -= 1;
    assert(!B.is_same_literal(*left, *right));
    enumeration, other_enum: B.Enum;
    first_member, same_member: B.Declaration;
    first_member.name = "MEMBER";
    same_member.name = "MEMBER";
    left.literal_kind = .ENUM;
    left.enum_type = *enumeration;
    left.enum_value = *first_member;
    right = left;
    right.enum_value = *same_member;
    assert(B.is_same_literal(*left, *right));
    right.enum_type = *other_enum;
    assert(!B.is_same_literal(*left, *right));
    right.enum_type = *enumeration;
    same_member.name = "OTHER_MEMBER";
    assert(!B.is_same_literal(*left, *right));
    left.literal_kind = .IDENTIFIER;
    left.identifier = *first_member;
    right = left;
    assert(B.is_same_literal(*left, *right));
    right.identifier = *same_member;
    assert(!B.is_same_literal(*left, *right));
    left.literal_kind = .MACRO;
    right = left;
    assert(!B.is_same_literal(*left, *right));
    assert(B.is_same_literal(*left, *left));
    expression, same_syntax: B.Binary_Operator;
    expression.kind = .BINARY_OPERATOR;
    expression.op = "+";
    same_syntax = expression;
    assert(B.is_same_expression(cast(*B.Expression) *expression, cast(*B.Expression) *expression));
    assert(!B.is_same_expression(cast(*B.Expression) *expression, cast(*B.Expression) *same_syntax));
    left.literal_kind = .INTEGER;
    left.int_value = 42;
    right = left;
    assert(B.is_same_expression(cast(*B.Expression) *left, cast(*B.Expression) *right));
    right.int_value += 1;
    assert(!B.is_same_expression(cast(*B.Expression) *left, cast(*B.Expression) *right));
    assert(!B.is_same_expression(cast(*B.Expression) *left, cast(*B.Expression) *expression));
    assert(B.is_same_expression(null, null));
    assert(!B.is_same_expression(null, cast(*B.Expression) *left));
    // Raw-token payload member identity is still a precise reviewed question.
    left.literal_kind = .RAW_CPP_TOKENS;
    right = left;

    push_context {
        context.logger = capture;
        assert(!B.is_same_literal(*left, *right));
    }

    assert(errors == 1);
    alias.type = *alias_type;

    push_context {
        context.logger = capture;
        assert(!B.is_same_type(*alias_type, signed));
    }

    assert(errors == 2);
    print("ok\n");
}
