#import "Basic";
L :: #import "Jai_Lexer";

main :: () {
    lexer: L.Lexer;
    L.set_input_from_string(*lexer, "g");
    digit, success := L.get_hex_digit(*lexer, .ASCII_8_BIT);
    assert(!success && digit == 0 && lexer.reported_error && lexer.input_cursor == 0);
    assert(L.peek_next_character(*lexer) == #char "g");
    L.set_input_from_string(*lexer, "18446744073709551616");
    token := L.peek_next_token(*lexer);
    assert(token.type == .NUMBER && token.integer_value == 0);
    assert(lexer.reported_error && (token.value_flags & .OVERFLOWED) != 0);
    L.set_input_from_string(*lexer, "0b102");
    token = L.peek_next_token(*lexer);
    assert(token.integer_value == 2 && lexer.reported_error && lexer.input_cursor == 4);
    L.eat_token(*lexer);
    assert(L.peek_next_token(*lexer).integer_value == 2);
    L.set_input_from_string(*lexer, "1.2.3");
    token = L.peek_next_token(*lexer);
    assert(lexer.reported_error && lexer.input_cursor == 4);
    L.eat_token(*lexer);
    assert(L.peek_next_token(*lexer).integer_value == 3);
    L.set_input_from_string(*lexer, "1e+");
    token = L.peek_next_token(*lexer);
    assert(lexer.reported_error && token.type == .NUMBER);
    L.set_input_from_string(*lexer, "/* unfinished");
    assert(L.peek_next_token(*lexer).type == .END_OF_INPUT && lexer.reported_error);
    L.set_input_from_string(*lexer, "0h123");
    token = L.peek_next_token(*lexer);
    assert(lexer.reported_error);
    assert(!L.has_a_big_exponent(0h0000000000000001));
    assert(!L.has_a_big_exponent(0h7ff0000000000000));
    assert(!L.has_a_big_exponent(0h7ff8000000000000));
    print("ok\n");
}
