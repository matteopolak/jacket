#scope_module
#import "Basic";

jai_target_expression_fragment :: (builder: *String_Builder, expression: *Expression, target_type: *CType, scope: *Declaration, hexadecimal: bool, enum_printed: *bool, depth: int) -> bool {
    if !expression return false;
    if !BE_Traversal.enter(.TARGET_EXPRESSION, expression) return false;

    defer BE_Traversal.leave();
    if expression.kind == .LITERAL return jai_literal_fragment(builder, cast(*Literal) expression, target_type, scope, hexadecimal, enum_printed);

    if expression.kind == .TYPE_REFERENCE {
        if target_type && target_type.hardcoded_jai_string != "Type" return false;
        return jai_model_type(builder, (cast(*Type_Reference) expression).type, scope, 0, depth + 1);
    }

    if expression.kind == .STRUCT_LITERAL {
        node := cast(*Struct_Literal) expression;
        target := strip_type(target_type, typedefs = true);
        valid := target_type == null;
        omit_name := false;

        if target && target.type_of_struct && !target.pointer_to && !target.array_element_type && !(target.flags & .REFERENCE) {
            record := target.type_of_struct;
            valid = !node.type_name || node.type_name == record.name || node.type_name == record.output_name;
            omit_name = valid;
        }

        return valid && jai_struct_literal_fragment(builder, node, scope, hexadecimal, enum_printed, depth, omit_name);
    }

    if target_type {
        append(builder, "cast(");
        if !jai_model_type(builder, target_type, scope, 0, depth + 1) return false;
        append(builder, ") (");
    }

    if !jai_expression_fragment(builder, expression, scope, hexadecimal, enum_printed, depth + 1) return false;
    if target_type append(builder, ")");

    return !builder.failed;
}

binding_append_token_cast :: (builder: *String_Builder, tokens: [] CXToken, start: int, enumeration: *Enum) -> bool, int {
    if !builder || !gm_tokens_valid(tokens) || start < 0 || start + 2 >= tokens.count return false, -1;
    if !is_token(tokens[start], "(") return false, -1;

    unit := gm_token_unit();
    if !unit return false, -1;

    native := clang_getTokenSpelling(unit, tokens[start + 1]);

    defer clang_disposeString(native);
    identifier := to_string(clang_getCString(native));
    known_type := identifier == "int" || identifier == "void";
    generator := gp_current_generator();

    if !generator return false, -1;

    builtin: *CType;
    if identifier == "char" builtin = generator.type_def_char;
    if identifier == "bool" builtin = generator.type_def_bool;
    if identifier == "float" builtin = generator.type_def_float32;
    if identifier == "double" builtin = generator.type_def_float64;
    if builtin known_type = true;

    alias_found, alias := GM_Table.table_find(*generator.unwrapped_typedefs, identifier);
    if alias_found && alias known_type = true;

    for declaration: generator.global_scope.members {
        if !declaration || (declaration.name != identifier && declaration.output_name != identifier) continue;
        if declaration.kind == .TYPEDEF || declaration.kind == .STRUCT || declaration.kind == .ENUM known_type = true;
    }

    if !known_type return false, -1;

    found, _name, type, parent_enum := find_and_check_macro_identifier(identifier, enumeration, true);

    if builtin {
        found = true;
        type = builtin;
    }

    if !found || !type || parent_enum return false, -1;

    index := start + 2;
    stars := 0;

    while index < tokens.count && is_token(tokens[index], "*") {
        stars += 1;
        index += 1;
    }

    function_pointer := false;

    if index < tokens.count && is_token(tokens[index], "(") {
        index += 1;
        function_stars := 0;

        while index < tokens.count && is_token(tokens[index], "*") {
            function_stars += 1;
            index += 1;
        }

        if !function_stars || index + 1 >= tokens.count || !is_token(tokens[index], ")") || !is_token(tokens[index + 1], "(") return false, -1;

        index = find_closing_paren_token_index(tokens, index + 2);
        if index < 0 return false, -1;

        index += 1;
        function_pointer = true;
    }

    if index >= tokens.count || !is_token(tokens[index], ")") return false, -1;
    if !stars && !function_pointer && enumeration && is_same_type(type, enumeration.type) return true, index;

    fragment: String_Builder;

    defer free_buffers(*fragment);
    underlying := strip_type(type, typedefs = true, enums = true);
    truncate := !stars && !function_pointer && underlying && underlying.number_flags != 0 && !(underlying.number_flags & .FLOAT) && index + 1 < tokens.count && (is_token(tokens[index + 1], "-") || is_token(tokens[index + 1], "~"));

    append(*fragment, ifx truncate then "cast,truncate(" else "cast(");

    if function_pointer {
        if !jai_model_type(*fragment, generator.type_def_ptr_void, null, 0, 0) return false, -1;
    } else {
        for 0..stars - 1 append(*fragment, "*");
        if !jai_model_type(*fragment, type, null, 0, 0) return false, -1;
    }

    append(*fragment, ") ");
    if fragment.failed return false, -1;

    text := builder_to_string(*fragment);

    defer free(text);
    append(builder, text);

    return !builder.failed, index;
}

commit_value_fragment :: (destination: *String_Builder, fragment: *String_Builder, valid: bool) {
    if !destination || !valid || fragment.failed {
        log_error("Bindings_Generator: unsupported or malformed literal/expression model\n");
        if destination destination.failed = true;
        return;
    }

    text := builder_to_string(fragment);

    defer free(text);
    append(destination, text);
}

integer_emission_bytes :: (target: *CType) -> s64 {
    base := strip_type(target, typedefs = true, enums = true);
    bytes: s64 = 8;

    if base bytes = base.size;

    assert(bytes == 1 || bytes == 2 || bytes == 4 || bytes == 8, "unsupported integer emission width");

    return bytes;
}

emit_integer_bits :: (builder: *String_Builder, value: u64, bytes: s64, signed: bool, hexadecimal: bool) {
    mask: u64 = 0xffffffffffffffff;
    if bytes < 8 mask = (cast(u64) 1 << (bytes * 8)) - 1;

    bits := value & mask;
    if signed && (bits & (cast(u64) 1 << (bytes * 8 - 1))) != 0 {
        append(builder, "-");
        magnitude := (~bits & mask) + 1;
        cpp_unsigned(builder, magnitude, hexadecimal);
    } else cpp_unsigned(builder, bits, hexadecimal);
}

jai_literal_fragment :: (builder: *String_Builder, literal: *Literal, target: *CType, scope: *Declaration, hexadecimal: bool, enum_printed: *bool) -> bool {
    if !literal return false;
    if !BE_Traversal.enter(.LITERAL, literal) return false;

    defer BE_Traversal.leave();

    if literal.literal_kind == .RAW_CPP_TOKENS {
        append(builder, literal.raw_value);
        return !builder.failed;
    }

    numeric := literal.literal_kind == .INTEGER || literal.literal_kind == .UNSIGNED_INTEGER || literal.literal_kind == .FLOAT;
    base := strip_type(target, typedefs = true, enums = true);
    nominal := strip_type(target, typedefs = true);

    if literal.literal_kind == .MACRO {
        enumeration: *Enum;
        if nominal enumeration = nominal.type_of_enum;
        if !enumeration && scope && scope.kind == .ENUM enumeration = cast(*Enum) scope;

        text, valid, _reason := gme_macro_text(literal.macro_value, enumeration, "", true);
        if !valid return false;

        defer free(text);

        if target {
            append(builder, "cast(");
            if !jai_model_type(builder, target, scope, 0, 0) return false;
            append(builder, ") (");
        }

        append(builder, text);
        if target append(builder, ")");
        if target && nominal && nominal.type_of_enum enum_printed.* = true;

        return !builder.failed;
    }

    if literal.literal_kind == .FLOAT {
        bits := (cast(*u64) *literal.float_value).*;
        if (bits & 0x7ff0000000000000) == 0x7ff0000000000000 return false;
    }

    if base && base.flags & .BOOL && numeric && (!nominal || !nominal.type_of_enum) {
        truth: bool;
        if literal.literal_kind == .INTEGER truth = literal.int_value != 0;
        if literal.literal_kind == .UNSIGNED_INTEGER truth = literal.uint_value != 0;
        if literal.literal_kind == .FLOAT truth = literal.float_value != 0;

        append(builder, ifx truth then "true" else "false");

        return true;
    }

    if numeric && binding_wide_integer_type(base) {
        if literal.literal_kind == .FLOAT return false;

        // Wide targets retain the represented 64-bit decimal payload. The
        // replacement limb type is not a scalar-cast construction contract.
        if hexadecimal {
            bits := literal.uint_value;
            if literal.literal_kind == .INTEGER bits = (cast(*u64) *literal.int_value).*;
            emit_integer_bits(builder, bits, integer_emission_bytes(base), (base.number_flags & .SIGNED) != 0, true);
        } else if literal.literal_kind == .INTEGER append_integer(builder, literal.int_value, false);
        else append_integer(builder, literal.uint_value, false);

        return !builder.failed;
    }

    if target {
        if base && base.pointer_to {
            if literal.literal_kind != .NULL return false;
        } else if base && (base.number_flags || binding_wide_integer_type(base)) {
            if !numeric && literal.literal_kind != .ENUM return false;
            if literal.literal_kind == .FLOAT && !(base.number_flags & .FLOAT) return false;
        } else if base && base.hardcoded_jai_string == "string" {
            if literal.literal_kind != .STRING return false;
        } else return false;

        append(builder, "cast(");
        if !jai_model_type(builder, target, scope, 0, 0) return false;

        append(builder, ") (");
    } else if literal.literal_kind == .UNSIGNED_INTEGER append(builder, "cast(u64) (");

    if numeric && base && base.number_flags && !(base.number_flags & .FLOAT) {
        bits: u64;
        if literal.literal_kind == .INTEGER bits = (cast(*u64) *literal.int_value).*;
        else bits = literal.uint_value;
        emit_integer_bits(builder, bits, integer_emission_bytes(base), (base.number_flags & .SIGNED) != 0, hexadecimal);
    } else if literal.literal_kind == .NULL append(builder, "null");
    else if literal.literal_kind == .INTEGER append_integer(builder, literal.int_value, hexadecimal);
    else if literal.literal_kind == .UNSIGNED_INTEGER append_integer(builder, literal.uint_value, hexadecimal);
    else if literal.literal_kind == .FLOAT {
        text := tprint("%", FormatFloat.{ value = literal.float_value, mode = .SHORTEST });
        append(builder, text);
        if !contains(text, ".") && !contains(text, "e") && !contains(text, "E") append(builder, ".0");
    } else if literal.literal_kind == .STRING emit_model_quoted(builder, literal.string_value);
    else if literal.literal_kind == .IDENTIFIER {
        if !jai_named_type(builder, literal.identifier, scope, 0) return false;
        if !binding_template_arguments(builder, literal.template_instantiation_params, .[], scope, 0) return false;
    } else if literal.literal_kind == .ENUM {
        if !literal.enum_type || !literal.enum_value || !looks_like_valid_identifier(literal.enum_value.output_name) return false;
        if !jai_named_type(builder, cast(*Declaration) literal.enum_type, scope, 0) return false;
        print_to_builder(builder, ".%", literal.enum_value.output_name);
        enum_printed.* = true;
    } else return false;

    if target || literal.literal_kind == .UNSIGNED_INTEGER append(builder, ")");

    return !builder.failed;
}

jai_expression_fragment :: (builder: *String_Builder, expression: *Expression, scope: *Declaration, hexadecimal: bool, enum_printed: *bool, depth: int) -> bool {
    if !expression return false;
    if !BE_Traversal.enter(.EXPRESSION, expression) return false;

    defer BE_Traversal.leave();
    if expression.kind == .LITERAL return jai_literal_fragment(builder, cast(*Literal) expression, null, scope, hexadecimal, enum_printed);
    if expression.kind == .TYPE_REFERENCE return jai_model_type(builder, (cast(*Type_Reference) expression).type, scope, 0, depth + 1);
    if expression.kind == .STRUCT_LITERAL return jai_struct_literal_fragment(builder, cast(*Struct_Literal) expression, scope, hexadecimal, enum_printed, depth, false);

    if expression.kind == .PAREN_EXPRESSION {
        append(builder, "(");
        if !jai_expression_fragment(builder, (cast(*Paren_Expression) expression).child, scope, hexadecimal, enum_printed, depth + 1) return false;
        append(builder, ")");
        return true;
    }

    if expression.kind == .UNARY_OPERATOR {
        node := cast(*Unary_Operator) expression;
        if node.op == "sizeof" {
            if !node.child || node.child.kind != .TYPE_REFERENCE return false;
            append(builder, "size_of(");
        } else {
            if node.op != "+" && node.op != "-" && node.op != "~" && node.op != "!" return false;
            print_to_builder(builder, "%(", node.op);
        }

        if !jai_expression_fragment(builder, node.child, scope, hexadecimal, enum_printed, depth + 1) return false;

        append(builder, ")");

        return true;
    }

    if expression.kind == .BINARY_OPERATOR {
        node := cast(*Binary_Operator) expression;
        valid := false;

        for operator: string.["+", "-", "*", "/", "%", "<<", ">>", "&", "|", "^", "&&", "||", "==", "!=", "<", ">", "<=", ">="] if node.op == operator valid = true;
        if !valid return false;

        append(builder, "(");
        if !jai_expression_fragment(builder, node.lhs, scope, hexadecimal, enum_printed, depth + 1) return false;

        print_to_builder(builder, " % ", node.op);
        if !jai_expression_fragment(builder, node.rhs, scope, hexadecimal, enum_printed, depth + 1) return false;

        append(builder, ")");

        return true;
    }

    return false;
}

jai_struct_literal_fragment :: (builder: *String_Builder, node: *Struct_Literal, scope: *Declaration, hexadecimal: bool, enum_printed: *bool, depth: int, omit_name: bool) -> bool {
    if !node || node.values.count < 0 || (node.values.count > 0 && !node.values.data) return false;
    if !BE_Traversal.enter(.STRUCT_LITERAL, node) return false;

    defer BE_Traversal.leave();
    if !omit_name append(builder, node.type_name);

    append(builder, ".{");

    for value, index: node.values {
        if index > 0 append(builder, ", ");
        if !jai_expression_fragment(builder, value, scope, hexadecimal, enum_printed, depth + 1) return false;
    }

    append(builder, "}");

    return !builder.failed;
}
