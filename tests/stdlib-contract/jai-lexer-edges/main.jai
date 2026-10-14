#import "Basic";
L :: #import "Jai_Lexer";

main :: () {
    lexer: L.Lexer;
    L.set_input_from_string(*lexer, "\n \torchid + 23");
    first := L.peek_token(*lexer, 0);
    assert(first.type == .IDENT && first.ident_value.name == "orchid");
    assert(first.l0 == 2 && first.c0 == 3 && first.l1 == 2 && first.c1 == 9);
    assert(L.peek_token(*lexer, 2).integer_value == 23);
    L.eat_token(*lexer);
    assert(L.get_last_token(*lexer).ident_value.name == "orchid");
    assert(L.peek_next_token(*lexer).type == cast(L.Token_Type) #char "+");

    L.set_input_from_string(*lexer, "for if context <<< >>> <<<= >>>= .{ .[ .* --- -- ,,");
    expected := L.Token_Type.[.KEYWORD_FOR, .KEYWORD_IF, .KEYWORD_CONTEXT, .ROTATE_LEFT, .ROTATE_RIGHT, .ROTATE_LEFT_EQUALS, .ROTATE_RIGHT_EQUALS, .BEGIN_STRUCT_LITERAL, .BEGIN_ARRAY_LITERAL, .POSTFIX_DEREFERENCE, .TRIPLE_MINUS, .DOUBLE_MINUS, .DOUBLE_COMMA];
    for expected { assert(L.peek_next_token(*lexer).type == it); L.eat_token(*lexer); }
    assert(L.peek_next_token(*lexer).type == .END_OF_INPUT);

    L.set_input_from_string(*lexer, "/* a /* b */ c */orchid //comment\npetal");
    assert(L.peek_next_token(*lexer).ident_value.name == "orchid"); L.eat_token(*lexer);
    assert(L.peek_next_token(*lexer).ident_value.name == "petal");
    assert(!lexer.reported_error);

    L.set_input_from_string(*lexer, "0.12345678 0.123456789 16777216.0 0h3f800000 0h3ff0000000000000");
    token := L.peek_next_token(*lexer);
    assert((token.value_flags & .DEFAULTS_TO_FLOAT64) != 0 && (token.value_flags & .REQUIRES_FLOAT64) == 0);
    L.eat_token(*lexer); token = L.peek_next_token(*lexer);
    assert((token.value_flags & .REQUIRES_FLOAT64) != 0 && (token.value_flags & .REQUIRES_FLOAT64_DUE_TO_SIGNIFICANT_DIGITS) != 0);
    L.eat_token(*lexer); token = L.peek_next_token(*lexer);
    assert((token.value_flags & (.DEFAULTS_TO_FLOAT64 | .REQUIRES_FLOAT64)) == 0);
    L.eat_token(*lexer); token = L.peek_next_token(*lexer);
    assert(token.float64_value == 1.0);
    L.eat_token(*lexer); token = L.peek_next_token(*lexer);
    assert(token.float64_value == 1.0);
    assert(!L.has_a_big_exponent(0.0) && !L.has_a_big_exponent(1.0));
    assert(L.has_a_big_exponent(1.0e39) && L.has_a_big_exponent(1.0e-40));

    L.set_input_from_string(*lexer, "\"a\\n\\x42\\u03bb\\d067\"");
    assert(L.peek_next_token(*lexer).string_value == "a\nBλC");
    assert(!lexer.reported_error);

    L.set_input_from_string(*lexer, "#string , cr END\n a\n  END_SUFFIX\n END\n");
    token = L.peek_next_token(*lexer);
    assert(token.type == .STRING && token.string_value == " a\r\n  END_SUFFIX\r\n");
    assert(L.count_herestring_lines(*lexer, token) == 2);
    assert(!lexer.reported_error);
    print("ok\n");
}
