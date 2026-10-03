maybe_convert_pointer_type_to_fixed_array_type :: (cur: CXCursor, type: *CType) -> *CType {
    if !type || !type.pointer_to return type;

    expression := convert_children_to_expression(cur);
    if !expression || expression.kind != .LITERAL return type;

    literal := cast(*Literal) expression;
    count: s64;

    if literal.literal_kind == .INTEGER count = literal.int_value;
    else if literal.literal_kind == .UNSIGNED_INTEGER {
        if literal.uint_value > 0x7fffffffffffffff {
            log_error("Bindings_Generator: recovered array count exceeds represented range\n");
            return type;
        }

        count = cast(s64) literal.uint_value;
    } else return type;

    if count <= 0 return type;

    if type.pointer_to.size < 0 || type.pointer_to.size > 0x7fffffffffffffff / count {
        log_error("Bindings_Generator: recovered array byte size is unrepresentable\n");
        return type;
    }

    result := binding_new_type();
    result.array_element_type = type.pointer_to;
    result.array_element_count = count;
    result.size = type.pointer_to.size * count;

    return result;
}

maybe_convert_fixed_array_type :: (type: *CType) -> *CType, changed: bool {
    return binding_normalize_array_parameter(type, 0);
}

maybe_convert_expression_zero_to_null :: (decl: *Declaration) {
    if !decl || !is_zero_int_expression(decl.expression) return;

    type := find_underlying_type(decl.type);
    if !type || !type.pointer_to || type.flags & .REFERENCE || decl.type.flags & .REFERENCE return;

    original := cast(*Literal) decl.expression;
    normalized := create_null_literal();
    normalized.location = original.location;
    normalized.raw_value = original.raw_value;
    decl.expression = cast(*Expression) normalized;
}

#scope_module

binding_new_type :: () -> *CType {
    session := context._bindings_generator_session;
    if session {
        assert(session.live && session.generator == context.generator && session.target_os == TARGET_OS);
        return GM_Sessions.new_owned(session, CType);
    }

    return New(CType);
}

binding_normalize_array_parameter :: (type: *CType, depth: int) -> *CType, bool {
    if !type || is_va_list(type) return type, false;

    if !BE_Traversal.enter(.ARRAY_PARAMETER, type) {
        log_error("Bindings_Generator: cyclic array parameter\n");
        return type, false;
    }

    defer BE_Traversal.leave();

    underlying := find_underlying_type(type);
    if !underlying || !underlying.array_element_type return type, false;

    if underlying.array_element_count >= 0 || underlying.array_element_count_expression {
        generator := gp_current_generator();

        assert(generator != null && (generator.pointer_size == 4 || generator.pointer_size == 8), "Bindings_Generator: initialized target pointer size required");
        result := binding_new_type();
        result.pointer_to = type;
        result.flags = .INSERTED;
        result.size = generator.pointer_size;
        result.alignment = generator.pointer_size;

        return result, true;
    }

    element, changed := binding_normalize_array_parameter(underlying.array_element_type, depth + 1);
    if !changed return type, false;

    result := binding_new_type();
    result.* = underlying.*;
    result.flags &= ~CType.Flags.SHARED;
    result.array_element_type = element;

    return result, true;
}
