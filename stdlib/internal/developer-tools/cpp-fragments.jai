append_cpp_literal :: (builder: *String_Builder, lit: *Literal, target_type: *CType = null, current_scope: *Declaration = null, print_integers_as_hex := false) -> valid: bool {
    if !builder || !lit return false;

    fragment: String_Builder;

    defer free_buffers(*fragment);

    if target_type {
        if target_type.flags & .REFERENCE return false;

        if target_type.pointer_to {
            if lit.literal_kind != .NULL return false;
        } else if target_type.type_of_enum {
            if lit.literal_kind != .INTEGER && lit.literal_kind != .UNSIGNED_INTEGER return false;
        } else if target_type.number_flags || target_type.flags & .BOOL {
            if lit.literal_kind != .INTEGER && lit.literal_kind != .UNSIGNED_INTEGER && lit.literal_kind != .FLOAT return false;
        } else return false;

        append(*fragment, "static_cast<");
        if !cpp_type(*fragment, target_type, current_scope, omit_top_cv = true) return false;

        append(*fragment, ">(");
    }

    if !cpp_literal(*fragment, lit, current_scope, print_integers_as_hex) return false;
    if target_type append(*fragment, ")");
    if fragment.failed return false;

    text := builder_to_string(*fragment);

    defer free(text);
    append(builder, text);

    return !builder.failed;
}

append_cpp_expression :: (builder: *String_Builder, expression: *Expression, target_type: *CType = null, current_scope: *Declaration = null) -> valid: bool {
    if !builder || !expression return false;
    if expression.kind == .LITERAL return append_cpp_literal(builder, cast(*Literal) expression, target_type, current_scope);
    if target_type return false;

    fragment: String_Builder;

    defer free_buffers(*fragment);
    if !cpp_expression(*fragment, expression, current_scope, 0) || fragment.failed return false;

    text := builder_to_string(*fragment);

    defer free(text);
    append(builder, text);

    return !builder.failed;
}

#scope_module
#import "Basic";
#import "String";

cpp_identifier :: (name: string) -> bool {
    if !looks_like_valid_identifier(name) return false;
    for keyword: cpp_keywords if keyword == name return false;
    return true;
}

cpp_keywords :: string.["alignas", "alignof", "and", "and_eq", "asm", "auto", "bitand", "bitor", "bool", "break", "case", "catch", "char", "char16_t", "char32_t", "class", "compl", "const", "constexpr", "const_cast", "continue", "decltype", "default", "delete", "do", "double", "dynamic_cast", "else", "enum", "explicit", "export", "extern", "false", "float", "for", "friend", "goto", "if", "inline", "int", "long", "mutable", "namespace", "new", "noexcept", "not", "not_eq", "nullptr", "operator", "or", "or_eq", "private", "protected", "public", "register", "reinterpret_cast", "return", "short", "signed", "sizeof", "static", "static_assert", "static_cast", "struct", "switch", "template", "this", "thread_local", "throw", "true", "try", "typedef", "typeid", "typename", "union", "unsigned", "using", "virtual", "void", "volatile", "wchar_t", "while", "xor", "xor_eq"];

cpp_name :: (builder: *String_Builder, declaration: *Declaration, current_scope: *Declaration) -> bool {
    if !declaration || !cpp_identifier(declaration.name) return false;

    names: [..] string;
    seen: [..] *Declaration;

    defer array_free(names);
    defer array_free(seen);
    array_add(*names, declaration.name);
    parent := declaration.parent;
    qualify_global := true;

    while parent && parent != current_scope {
        if parent.kind == .FUNCTION {
            qualify_global = false;
            break;
        }

        if array_find(seen, parent) return false;

        array_add(*seen, parent);

        if parent.kind == .NAMESPACE || parent.kind == .STRUCT || parent.kind == .ENUM {
            if parent.name {
                if !cpp_identifier(parent.name) return false;
                array_add(*names, parent.name);
            } else if parent.kind != .NAMESPACE return false;
        }

        parent = parent.parent;
    }

    if parent && parent == current_scope qualify_global = false;
    if qualify_global append(builder, "::");

    for i: 0..names.count - 1 {
        if i > 0 append(builder, "::");
        append(builder, names[names.count - 1 - i]);
    }

    return true;
}

cpp_scalar_type :: (builder: *String_Builder, type: *CType, scope: *Declaration, omit_top_cv := false) -> bool {
    if !type || type.flags & .REFERENCE return false;

    if !omit_top_cv && type.flags & .CONST append(builder, "const ");
    if !omit_top_cv && type.flags & .VOLATILE append(builder, "volatile ");

    if type.flags & .VOID {
        append(builder, "void");
        return true;
    }

    if type.flags & .BOOL {
        append(builder, "bool");
        return true;
    }

    if type.type_of_enum return cpp_name(builder, cast(*Declaration) type.type_of_enum, scope);
    if type.type_of_struct return cpp_name(builder, cast(*Declaration) type.type_of_struct, scope);
    if type.type_of_typedef return cpp_name(builder, cast(*Declaration) type.type_of_typedef, scope);

    if type.number_flags & .FLOAT {
        if type.size == 4 {
            append(builder, "float");
            return true;
        }

        if type.size == 8 {
            append(builder, "double");
            return true;
        }

        return false;
    }

    if !type.number_flags return false;
    if !(type.number_flags & .SIGNED) append(builder, "unsigned ");
    else if type.size == 1 append(builder, "signed ");
    if type.size == 1 append(builder, "char");
    else if type.size == 2 append(builder, "short");
    else if type.size == 4 append(builder, "int");
    else if type.size == 8 append(builder, "long long");
    else return false;

    return true;
}

cpp_unsigned :: (builder: *String_Builder, value: u64, hexadecimal: bool) {
    if !hexadecimal {
        print_to_builder(builder, "%", value);
        return;
    }

    append(builder, "0x");
    digits := "0123456789abcdef";
    started := false;

    for i: 0..15 {
        digit := (value >> ((15 - i) * 4)) & 15;
        if digit != 0 || started || i == 15 {
            append(builder, string.{ 1, digits.data + digit });
            started = true;
        }
    }
}

cpp_literal :: (builder: *String_Builder, literal: *Literal, scope: *Declaration, hexadecimal: bool) -> bool {
    if literal.literal_kind == .RAW_CPP_TOKENS {
        append(builder, literal.raw_value);
        return !builder.failed;
    }

    if literal.literal_kind == .NULL {
        append(builder, "nullptr");
        return true;
    }

    if literal.literal_kind == .INTEGER {
        value := literal.int_value;
        if value == -9223372036854775807 - 1 {
            append(builder, "(-");
            cpp_unsigned(builder, 0x7fffffffffffffff, hexadecimal);
            append(builder, "LL - 1LL)");
            return true;
        }

        if value < 0 append(builder, "-");

        magnitude := cast(u64) (ifx value < 0 then -value else value);

        cpp_unsigned(builder, magnitude, hexadecimal);
        if magnitude > 0x7fffffff append(builder, "LL");

        return true;
    }

    if literal.literal_kind == .UNSIGNED_INTEGER {
        cpp_unsigned(builder, literal.uint_value, hexadecimal);
        append(builder, ifx literal.uint_value > 0xffffffff then "ULL" else "U");
        return true;
    }

    if literal.literal_kind == .FLOAT {
        bits := (cast(*u64) *literal.float_value).*;
        if (bits & 0x7ff0000000000000) == 0x7ff0000000000000 return false;

        text := tprint("%", FormatFloat.{ value = literal.float_value, mode = .SHORTEST });

        append(builder, text);
        if !contains(text, ".") && !contains(text, "e") && !contains(text, "E") append(builder, ".0");

        return true;
    }

    if literal.literal_kind == .STRING {
        if literal.string_value.count < 0 || (literal.string_value.count > 0 && !literal.string_value.data) return false;

        append(builder, "\"");
        octal := "01234567";

        for byte: literal.string_value {
            if byte == 34 || byte == 92 {
                append(builder, "\\");
                append(builder, string.{ 1, *byte });
            } else if byte >= 32 && byte <= 126 append(builder, string.{ 1, *byte });
            else {
                append(builder, "\\");
                append(builder, string.{ 1, octal.data + (byte >> 6) });
                append(builder, string.{ 1, octal.data + ((byte >> 3) & 7) });
                append(builder, string.{ 1, octal.data + (byte & 7) });
            }
        }

        append(builder, "\"");

        return true;
    }

    if literal.literal_kind == .IDENTIFIER return cpp_name(builder, literal.identifier, scope);

    if literal.literal_kind == .ENUM {
        if !literal.enum_type || !literal.enum_value return false;
        if !cpp_name(builder, cast(*Declaration) literal.enum_type, scope) || !cpp_identifier(literal.enum_value.name) return false;
        print_to_builder(builder, "::%", literal.enum_value.name);
        return true;
    }

    return false;
}

cpp_expression :: (builder: *String_Builder, expression: *Expression, scope: *Declaration, depth: s64) -> bool {
    if !expression || builder.failed return false;
    if !BE_Traversal.enter(.CPP_EXPRESSION, expression) return false;

    defer BE_Traversal.leave();
    if expression.kind == .LITERAL return cpp_literal(builder, cast(*Literal) expression, scope, false);

    if expression.kind == .STRUCT_LITERAL {
        node := cast(*Struct_Literal) expression;
        if node.values.count < 0 || (node.values.count > 0 && !node.values.data) return false;

        append(builder, node.type_name);
        append(builder, "{");

        for value, index: node.values {
            if index > 0 append(builder, ", ");
            if !cpp_expression(builder, value, scope, depth + 1) return false;
        }

        append(builder, "}");

        return !builder.failed;
    }

    if expression.kind == .PAREN_EXPRESSION {
        node := cast(*Paren_Expression) expression;

        append(builder, "(");
        if !cpp_expression(builder, node.child, scope, depth + 1) return false;

        append(builder, ")");

        return true;
    }

    if expression.kind == .UNARY_OPERATOR {
        node := cast(*Unary_Operator) expression;
        if node.op == "sizeof" {
            append(builder, "sizeof(");

            if node.child && node.child.kind == .TYPE_REFERENCE {
                if !cpp_type(builder, (cast(*Type_Reference) node.child).type, scope) return false;
            } else if !cpp_expression(builder, node.child, scope, depth + 1) return false;
        } else {
            if node.op != "+" && node.op != "-" && node.op != "~" && node.op != "!" return false;
            print_to_builder(builder, "%(", node.op);
            if !cpp_expression(builder, node.child, scope, depth + 1) return false;
        }

        append(builder, ")");

        return true;
    }

    if expression.kind == .BINARY_OPERATOR {
        node := cast(*Binary_Operator) expression;
        valid := false;

        for operator: string.["+", "-", "*", "/", "%", "<<", ">>", "&", "|", "^", "&&", "||", "==", "!=", "<", ">", "<=", ">="] if node.op == operator valid = true;
        if !valid return false;

        append(builder, "(");
        if !cpp_expression(builder, node.lhs, scope, depth + 1) return false;

        print_to_builder(builder, " % ", node.op);
        if !cpp_expression(builder, node.rhs, scope, depth + 1) return false;

        append(builder, ")");

        return true;
    }

    return false;
}
