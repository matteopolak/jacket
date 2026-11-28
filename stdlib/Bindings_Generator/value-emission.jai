// Appends the integer `value` to `builder` in decimal, or with `as_hex` in hexadecimal.
//
// Hexadecimal is written as `0x` and lowercase digits without padding, with a leading `-` for a negative
// signed value. Asserts that `T` is an integer type of at most 8 bytes.
append_integer :: (builder: *String_Builder, value: $T, as_hex: bool) {
    info := type_info(T);

    assert(info.type == .INTEGER && info.runtime_size <= 8, "integer emitter requires a supported integer value");

    if !as_hex {
        print_to_builder(builder, "%", value);
        return;
    }

    if (cast(*Type_Info_Integer) info).signed {
        signed := cast(s64) value;
        if signed < 0 {
            append(builder, "-");
            magnitude := cast(u64) (-(signed + 1)) + 1;
            cpp_unsigned(builder, magnitude, true);
            return;
        }
    }

    cpp_unsigned(builder, cast(u64) value, true);
}

// Appends the bits of `value` as unsigned hexadecimal, cut to the byte size of `target_type`.
//
// `target_type` may be a typedef or enum; without a size, 8 bytes are used. Asserts on a size other than 1,
// 2, 4 or 8.
append_as_unsigned_integer_hex :: (builder: *String_Builder, value: s64, target_type: *CType) {
    bits := (cast(*u64) *value).*;
    emit_integer_bits(builder, bits, integer_emission_bytes(target_type), false, true);
}

// Appends `value`, cut to the byte size of `target_type`, as signed hexadecimal.
//
// A value whose top bit is set within that size is written as `-` and the hexadecimal magnitude. Asserts on a
// size other than 1, 2, 4 or 8.
append_as_signed_integer_hex :: (builder: *String_Builder, value: u64, target_type: *CType) {
    emit_integer_bits(builder, value, integer_emission_bytes(target_type), true, true);
}

// Appends `value`, or `(value << shift)` when `shift` is not zero. Asserts that `shift` is not negative.
append_maybe_left_shift :: (builder: *String_Builder, value: string, shift: int) {
    assert(shift >= 0, "negative generated shift");
    if shift == 0 append(builder, value);
    else print_to_builder(builder, "(% << %)", value, shift);
}

// Appends `value`, or `(value >> shift)` when `shift` is not zero. Asserts that `shift` is not negative.
append_maybe_right_shift :: (builder: *String_Builder, value: string, shift: int) {
    assert(shift >= 0, "negative generated shift");
    if shift == 0 append(builder, value);
    else print_to_builder(builder, "(% >> %)", value, shift);
}

// Appends the Jai source for the literal `lit` to `builder`, and returns whether it names an enum value.
//
// With `target_type`, the value is converted to that type, usually written as `cast(T) (...)`;
// `current_scope` is the scope that names are written relative to, and `print_integers_as_hex` selects
// hexadecimal integers. When the literal cannot be represented, an error is logged, `builder.failed` is set
// and nothing is appended.
print_literal_to_builder :: (builder: *String_Builder, lit: *Literal, target_type: *CType = null, current_scope: *Declaration = null, print_integers_as_hex := false) -> did_print_enum: bool {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    did_print_enum := false;
    valid := jai_literal_fragment(*fragment, lit, target_type, current_scope, print_integers_as_hex, *did_print_enum);

    commit_value_fragment(builder, *fragment, valid);

    return valid && did_print_enum;
}

// Appends the Jai source for `expression` to `builder`, and returns whether it names an enum value.
//
// Handles literals, struct literals, type references, parentheses and unary and binary operators. With
// `target_type`, the value is converted to that type. `current_scope` and `print_integers_as_hex` work as in
// [print_literal_to_builder]. When the expression cannot be represented, an error is logged, `builder.failed`
// is set and nothing is appended.
print_expression_to_builder :: (builder: *String_Builder, expression: *Expression, target_type: *CType = null, current_scope: *Declaration = null, print_integers_as_hex := false) -> did_print_enum: bool {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    did_print_enum := false;
    valid := jai_target_expression_fragment(*fragment, expression, target_type, current_scope, print_integers_as_hex, *did_print_enum, 0);

    commit_value_fragment(builder, *fragment, valid);

    return valid && did_print_enum;
}

// Appends the Jai `cast(...)` for a C cast that starts at the `(` token `tokens[start_index]` of a macro.
//
// Returns true and the index of the closing `)` token when the tokens are a cast to a known type: a builtin
// type, a typedef, struct or enum, with any number of `*` or a function pointer. Returns false and -1
// otherwise, with nothing appended. A cast to the type of `_enum` itself is consumed without appending
// anything. A cast to an integer type followed by `-` or `~` is written as `cast,trunc`.
maybe_append_cast :: (builder: *String_Builder, tokens: [] CXToken, start_index: int, _enum: *Enum) -> bool, end_index: int {
    return binding_append_token_cast(builder, tokens, start_index, _enum);
}

#load "../internal/developer-tools/binding-value-emission.jai";
