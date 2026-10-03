// Generated keyword spellings and documented input operators.
check_for_keyword :: (token: *J.Token) -> J.Token_Type {
    if token.ident_value.name == "auto_cast" {
        token.type = .KEYWORD_AUTO_CAST;
        return token.type;
    }

    if token.ident_value.name == "break" {
        token.type = .KEYWORD_BREAK;
        return token.type;
    }

    if token.ident_value.name == "case" {
        token.type = .KEYWORD_CASE;
        return token.type;
    }

    if token.ident_value.name == "cast" {
        token.type = .KEYWORD_CAST;
        return token.type;
    }

    if token.ident_value.name == "code_of" {
        token.type = .KEYWORD_CODE_OF;
        return token.type;
    }

    if token.ident_value.name == "context" {
        token.type = .KEYWORD_CONTEXT;
        return token.type;
    }

    if token.ident_value.name == "continue" {
        token.type = .KEYWORD_CONTINUE;
        return token.type;
    }

    if token.ident_value.name == "defer" {
        token.type = .KEYWORD_DEFER;
        return token.type;
    }

    if token.ident_value.name == "else" {
        token.type = .KEYWORD_ELSE;
        return token.type;
    }

    if token.ident_value.name == "enum" {
        token.type = .KEYWORD_ENUM;
        return token.type;
    }

    if token.ident_value.name == "enum_flags" {
        token.type = .KEYWORD_ENUM_FLAGS;
        return token.type;
    }

    if token.ident_value.name == "false" {
        token.type = .KEYWORD_FALSE;
        return token.type;
    }

    if token.ident_value.name == "for" {
        token.type = .KEYWORD_FOR;
        return token.type;
    }

    if token.ident_value.name == "if" {
        token.type = .KEYWORD_IF;
        return token.type;
    }

    if token.ident_value.name == "ifx" {
        token.type = .KEYWORD_IFX;
        return token.type;
    }

    if token.ident_value.name == "initializer_of" {
        token.type = .KEYWORD_INITIALIZER_OF;
        return token.type;
    }

    if token.ident_value.name == "inline" {
        token.type = .KEYWORD_INLINE;
        return token.type;
    }

    if token.ident_value.name == "interface" {
        token.type = .KEYWORD_INTERFACE;
        return token.type;
    }

    if token.ident_value.name == "is_constant" {
        token.type = .KEYWORD_IS_CONSTANT;
        return token.type;
    }

    if token.ident_value.name == "no_inline" {
        token.type = .KEYWORD_NO_INLINE;
        return token.type;
    }

    if token.ident_value.name == "null" {
        token.type = .KEYWORD_NULL;
        return token.type;
    }

    if token.ident_value.name == "operator" {
        token.type = .KEYWORD_OPERATOR;
        return token.type;
    }

    if token.ident_value.name == "push_context" {
        token.type = .KEYWORD_PUSH_CONTEXT;
        return token.type;
    }

    if token.ident_value.name == "remove" {
        token.type = .KEYWORD_REMOVE;
        return token.type;
    }

    if token.ident_value.name == "return" {
        token.type = .KEYWORD_RETURN;
        return token.type;
    }

    if token.ident_value.name == "size_of" {
        token.type = .KEYWORD_SIZE_OF;
        return token.type;
    }

    if token.ident_value.name == "struct" {
        token.type = .KEYWORD_STRUCT;
        return token.type;
    }

    if token.ident_value.name == "then" {
        token.type = .KEYWORD_THEN;
        return token.type;
    }

    if token.ident_value.name == "true" {
        token.type = .KEYWORD_TRUE;
        return token.type;
    }

    if token.ident_value.name == "type_info" {
        token.type = .KEYWORD_TYPE_INFO;
        return token.type;
    }

    if token.ident_value.name == "type_of" {
        token.type = .KEYWORD_TYPE_OF;
        return token.type;
    }

    if token.ident_value.name == "union" {
        token.type = .KEYWORD_UNION;
        return token.type;
    }

    if token.ident_value.name == "using" {
        token.type = .KEYWORD_USING;
        return token.type;
    }

    if token.ident_value.name == "while" {
        token.type = .KEYWORD_WHILE;
        return token.type;
    }

    return token.type;
}

#scope_module
punctuation :: (lexer: *J.Lexer) -> (type: J.Token_Type, width: s64) {
    if matches_text(lexer, "<<<=") return .ROTATE_LEFT_EQUALS, 4;
    if matches_text(lexer, ">>>=") return .ROTATE_RIGHT_EQUALS, 4;
    if matches_text(lexer, "&&=") return .LOGICAL_AND_EQUALS, 3;
    if matches_text(lexer, "---") return .TRIPLE_MINUS, 3;
    if matches_text(lexer, "<<<") return .ROTATE_LEFT, 3;
    if matches_text(lexer, "<<=") return .SHIFT_LEFT_EQUALS, 3;
    if matches_text(lexer, "===") return .TRIPLE_EQUALS, 3;
    if matches_text(lexer, ">>=") return .SHIFT_RIGHT_EQUALS, 3;
    if matches_text(lexer, ">>>") return .ROTATE_RIGHT, 3;
    if matches_text(lexer, "||=") return .LOGICAL_OR_EQUALS, 3;
    if matches_text(lexer, "!=") return .ISNOTEQUAL, 2;
    if matches_text(lexer, "$$") return .DOUBLE_DOLLAR, 2;
    if matches_text(lexer, "%=") return .MODEQUALS, 2;
    if matches_text(lexer, "&&") return .LOGICAL_AND, 2;
    if matches_text(lexer, "&=") return .BITWISE_AND_EQUALS, 2;
    if matches_text(lexer, "*=") return .TIMESEQUALS, 2;
    if matches_text(lexer, "+=") return .PLUSEQUALS, 2;
    if matches_text(lexer, ",,") return .DOUBLE_COMMA, 2;
    if matches_text(lexer, "--") return .DOUBLE_MINUS, 2;
    if matches_text(lexer, "-=") return .MINUSEQUALS, 2;
    if matches_text(lexer, "->") return .RIGHT_ARROW, 2;
    if matches_text(lexer, ".*") return .POSTFIX_DEREFERENCE, 2;
    if matches_text(lexer, "..") return .DOUBLE_DOT, 2;
    if matches_text(lexer, ".[") return .BEGIN_ARRAY_LITERAL, 2;
    if matches_text(lexer, ".{") return .BEGIN_STRUCT_LITERAL, 2;
    if matches_text(lexer, "/=") return .DIVEQUALS, 2;
    if matches_text(lexer, "<<") return .SHIFT_LEFT, 2;
    if matches_text(lexer, "<=") return .LESSEQUALS, 2;
    if matches_text(lexer, "==") return .ISEQUAL, 2;
    if matches_text(lexer, "=>") return .QUICK_LAMBDA, 2;
    if matches_text(lexer, ">=") return .GREATEREQUALS, 2;
    if matches_text(lexer, ">>") return .SHIFT_RIGHT, 2;
    if matches_text(lexer, "^=") return .BITWISE_XOR_EQUALS, 2;
    if matches_text(lexer, "|=") return .BITWISE_OR_EQUALS, 2;
    if matches_text(lexer, "||") return .LOGICAL_OR, 2;

    return cast(J.Token_Type) look(lexer, 0), 1;
}

#scope_file
J :: #import "Jai_Lexer";
