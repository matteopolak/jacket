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

append_as_unsigned_integer_hex :: (builder: *String_Builder, value: s64, target_type: *CType) {
    bits := (cast(*u64) *value).*;
    emit_integer_bits(builder, bits, integer_emission_bytes(target_type), false, true);
}

append_as_signed_integer_hex :: (builder: *String_Builder, value: u64, target_type: *CType) {
    emit_integer_bits(builder, value, integer_emission_bytes(target_type), true, true);
}

append_maybe_left_shift :: (builder: *String_Builder, value: string, shift: int) {
    assert(shift >= 0, "negative generated shift");
    if shift == 0 append(builder, value);
    else print_to_builder(builder, "(% << %)", value, shift);
}

append_maybe_right_shift :: (builder: *String_Builder, value: string, shift: int) {
    assert(shift >= 0, "negative generated shift");
    if shift == 0 append(builder, value);
    else print_to_builder(builder, "(% >> %)", value, shift);
}

print_literal_to_builder :: (builder: *String_Builder, lit: *Literal, target_type: *CType = null, current_scope: *Declaration = null, print_integers_as_hex := false) -> did_print_enum: bool {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    did_print_enum := false;
    valid := jai_literal_fragment(*fragment, lit, target_type, current_scope, print_integers_as_hex, *did_print_enum);

    commit_value_fragment(builder, *fragment, valid);

    return valid && did_print_enum;
}

print_expression_to_builder :: (builder: *String_Builder, expression: *Expression, target_type: *CType = null, current_scope: *Declaration = null, print_integers_as_hex := false) -> did_print_enum: bool {
    fragment: String_Builder;

    defer free_buffers(*fragment);
    did_print_enum := false;
    valid := jai_target_expression_fragment(*fragment, expression, target_type, current_scope, print_integers_as_hex, *did_print_enum, 0);

    commit_value_fragment(builder, *fragment, valid);

    return valid && did_print_enum;
}

maybe_append_cast :: (builder: *String_Builder, tokens: [] CXToken, start_index: int, _enum: *Enum) -> bool, end_index: int {
    return binding_append_token_cast(builder, tokens, start_index, _enum);
}

#load "../internal/developer-tools/binding-value-emission.jai";
