#import "Basic";
L :: #import "Jai_Lexer";

main :: () {
    lexer: L.Lexer;

    // The name may follow line ends and comments; positions stay right across the skipped lines.
    L.set_input_from_string(*lexer, "#string /* a /* nested */ b */ // note\n\n  TAG\nab\nTAG, 1\n");
    token := L.peek_token(*lexer, 0);

    assert(token.type == .STRING && token.string_value == "ab\n");
    assert(token.l0 == 1 && token.c0 == 1);
    assert(token.l1 == 5 && token.c1 == 4);
    assert(!lexer.reported_error);

    // Text may follow the name on the closing line.
    next := L.peek_token(*lexer, 1);

    assert(next.type == xx #char "," && next.l0 == 5 && next.c0 == 4);

    // Flags may be split from the name by comments and line ends, and `\%` is a flag.
    L.set_input_from_string(*lexer, "#string // a\n , // b\n cr /* c */\n , \\%\nEND\na\\%b\nEND\n");
    token = L.peek_token(*lexer, 0);
    assert(token.type == .STRING && token.string_value == "a\x1fb\r\n");
    assert(!lexer.reported_error);

    // A comment after the name is an error.
    L.set_input_from_string(*lexer, "#string END // no\nx\nEND\n");
    L.peek_token(*lexer, 0);
    assert(lexer.reported_error);

    print("ok\n");
}
