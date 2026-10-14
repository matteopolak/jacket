// Public lexer interface: input, token categories, names, values, and EOF.
#import "Basic";
Jai_Lexer :: #import "Jai_Lexer";

main :: () {
    lexer: Jai_Lexer.Lexer;
    Jai_Lexer.set_input_from_string(*lexer, "orchid 23");

    token := Jai_Lexer.peek_next_token(*lexer);
    assert(token.type == .IDENT && token.ident_value.name == "orchid");
    Jai_Lexer.eat_token(*lexer);

    token = Jai_Lexer.peek_next_token(*lexer);
    assert(token.type == .NUMBER && token.integer_value == 23);
    Jai_Lexer.eat_token(*lexer);

    token = Jai_Lexer.peek_next_token(*lexer);
    assert(token.type == .END_OF_INPUT);
    assert(Jai_Lexer.starts_identifier(cast(u8) #char "Q"));
    assert(!Jai_Lexer.starts_identifier(cast(u8) #char "7"));

    Jai_Lexer.set_input_from_string(*lexer, "8 <<< 3");
    token = Jai_Lexer.peek_next_token(*lexer);
    assert(token.type == .NUMBER && token.integer_value == 8);
    Jai_Lexer.eat_token(*lexer);
    token = Jai_Lexer.peek_next_token(*lexer);
    assert(token.type == .ROTATE_LEFT);
    print("ok\n");
}
