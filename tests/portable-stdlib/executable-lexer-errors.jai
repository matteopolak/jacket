#import "Basic";
E :: #import "executable_formats";

main :: () {
    lexer: E.Lexer;

    for input: string.["\"unterminated", "\"\\q\"", "\"\\uD800\"", "0x", "0b2", "1e+", "/* never closed"] {
        E.set_input_from_string(*lexer, input);
        token := E.peek_next_token(*lexer);

        assert(lexer.reported_error && (token.type == .ERROR || token.type == .END_OF_INPUT));
        assert(lexer.input_cursor > 0 && lexer.input_cursor <= input.count);
        assert(E.peek_next_token(*lexer) == token); // Diagnostics are not repeated by peeking.
    }

    E.set_input_from_string(*lexer, "a b c d e f g h i");
    E.peek_token(*lexer, 7);
    cursor := lexer.input_cursor;
    overflow := E.compose_new_token(*lexer);

    assert(overflow.type == .ERROR && lexer.num_incoming_tokens == 8 && lexer.input_cursor == cursor);
    assert(E.peek_token(*lexer, 8).type == .ERROR && lexer.num_incoming_tokens == 8);
    assert(E.peek_token(*lexer, -1).type == .ERROR && lexer.num_incoming_tokens == 8);
    E.eat_token(*lexer);
    assert(E.peek_token(*lexer, 7).string_value == "i");
    E.set_input_from_string(*lexer, "");
    E.report_parse_error(*lexer, "value %", 7);
    E.report_detail(*lexer, "detail %", 8);
    E.report_parse_error(*lexer, 3, 4, "position");
    token: E.Token;
    token.l0 = 5;
    token.c0 = 6;
    E.report_parse_error(*lexer, *token, "token");
    E.report_detail(*lexer, 7, 8, "tail");
    assert(lexer.reported_error && lexer.current_external_file_error_report_line_number == 5);
    E.set_input_from_string(*lexer, "18446744073709551616");
    assert(E.peek_next_token(*lexer).value_flags & .OVERFLOWED != 0);
    letters: [513] u8;
    for letters letters[it_index] = #char "a";

    E.set_input_from_string(*lexer, .{ 513, letters.data });
    assert(E.peek_next_token(*lexer).type == .ERROR && lexer.reported_error && lexer.input_cursor == 513);
    print("ok\n");
}
