#import "Basic";
E :: #import "executable_formats";

main :: () {
    lexer: E.Lexer;

    E.set_input_from_string(*lexer, "  alpha _b2 $symbol 0xff 0b101 12.5e+2 --- ... :\n\"line\\n\\u03bb\" /* outer /* inner */ end */ 'it''s' // comment\nlast");
    assert(lexer.current_line_number == 1 && lexer.current_character_index == 1 && !lexer.should_free_input);
    first := E.peek_next_token(*lexer);

    assert(first.type == .IDENT && first.string_value == "alpha" && first.l0 == 1 && first.c0 == 3);
    assert(first.preceeding_whitespace == 2 && lexer.num_incoming_tokens == 1);
    assert(E.peek_next_token(*lexer) == first);
    for 0..7 assert(E.peek_token(*lexer, cast(s32) it).type != .ERROR);

    assert(lexer.num_incoming_tokens == 8);
    E.eat_token(*lexer);
    assert(E.get_last_token(*lexer).string_value == "alpha");
    expect(*lexer, .IDENT, "_b2");
    expect(*lexer, .IDENT, "$symbol");
    token := E.peek_next_token(*lexer);

    assert(token.value_flags & .HEX != 0 && token.value_flags & .NUMBER != 0);
    expect(*lexer, .IDENT, "0xff");
    token = E.peek_next_token(*lexer);
    assert(token.value_flags & .BINARY != 0);
    expect(*lexer, .IDENT, "0b101");
    token = E.peek_next_token(*lexer);
    assert(token.value_flags & .FLOAT != 0);
    expect(*lexer, .IDENT, "12.5e+2");
    expect(*lexer, .TRIPLE_MINUS, "---");
    expect(*lexer, .TRIPLE_DOT, "...");
    expect(*lexer, cast(E.Token_Type) #char ":", ":");
    expect(*lexer, .STRING, "line\nλ");
    expect(*lexer, .STRING, "it's");
    expect(*lexer, .IDENT, "last");
    assert(E.peek_next_token(*lexer).type == .END_OF_INPUT && !lexer.reported_error);
    assert(lexer.nonblank_lines_processed == 3 && lexer.total_lines_processed == 2);
    E.set_input_from_string(*lexer, "\"\"\"one\ntwo\"\"\"");
    token = E.peek_next_token(*lexer);
    assert(token.type == .STRING && token.value_flags & .HERE_STRING != 0);
    assert(token.string_value == "one\ntwo" && E.count_herestring_lines(*lexer, token) == 2);
    assert(E.check_for_keyword(token) == .STRING);
    E.set_input_from_string(*lexer, "9AfZ\nrest");
    assert(E.get_decimal_digit(*lexer) == 9);
    hex, hex_ok := E.get_hex_digit(*lexer, .UNICODE_16);

    assert(hex_ok && hex == 10);
    hex, hex_ok = E.get_hex_digit(*lexer, .ASCII_8_BIT);
    assert(hex_ok && hex == 15);
    assert(E.get_decimal_digit(*lexer) == -1 && E.peek_next_character(*lexer) == #char "Z");
    E.eat_character(*lexer);
    E.eat_character(*lexer);
    assert(lexer.current_line_number == 2 && lexer.current_character_index == 1);
    E.unwind_one_character(*lexer);
    assert(lexer.current_line_number == 1 && lexer.current_character_index == 5);
    assert(E.get_and_eat_remainder_of_line(*lexer, 1, 5) == "");
    assert(E.get_and_eat_remainder_of_line(*lexer, 2, 1) == "rest");
    assert(E.starts_identifier(#char "_") && !E.starts_identifier(#char "7"));
    assert(E.continues_identifier(#char "7") && !E.continues_identifier(-1));
    assert(E.starts_number(#char "7") && !E.starts_number(#char "a"));
    assert(E.is_non_newline_whitespace(13) && !E.is_non_newline_whitespace(10));
    assert(E.has_a_big_exponent(1e20) && E.has_a_big_exponent(1e-8));
    assert(!E.has_a_big_exponent(0) && !E.has_a_big_exponent(42));
    E.set_input_from_string(*lexer, "=");
    token = E.check_for_equals(*lexer, cast(E.Token_Type) #char "+", .TRIPLE_MINUS);
    assert(token.type == .TRIPLE_MINUS && lexer.input_cursor == 1);
    E.set_input_from_string(*lexer, "word");
    direct: E.Token;

    assert(E.parse_ident(*lexer, *direct) == "word" && direct.l1 == 1 && direct.c1 == 5);
    assert(E.set_input_from_file(*lexer, #file));
    assert(lexer.should_free_input && E.peek_next_token(*lexer).type == cast(E.Token_Type) #char "#");
    E.set_input_from_string(*lexer, ""); // Releases the path-input buffer.
    assert(!lexer.should_free_input && lexer.input.count == 0);
    E.set_input_from_string(*lexer, "\"\\xFF\" \"\\U0001F600\"");
    token = E.peek_next_token(*lexer);
    assert(token.type == .STRING && token.string_value.count == 1 && token.string_value[0] == 255);
    E.eat_token(*lexer);
    token = E.peek_next_token(*lexer);
    assert(token.type == .STRING && token.string_value.count == 4 && token.string_value[0] == 0xf0 && token.string_value[3] == 0x80);
    E.set_input_from_string(*lexer, "1e-309 1e309 1_000.5 .25");
    token = E.peek_next_token(*lexer);
    assert(token.value_flags & .FLOAT != 0 && token.value_flags & .OVERFLOWED == 0);
    E.eat_token(*lexer);
    assert(E.peek_next_token(*lexer).value_flags & .OVERFLOWED != 0);
    E.eat_token(*lexer);
    assert(E.peek_next_token(*lexer).value_flags & .OVERFLOWED == 0);
    E.eat_token(*lexer);
    expect(*lexer, .IDENT, ".25");
    letters: [512] u8;
    for letters letters[it_index] = #char "a";

    E.set_input_from_string(*lexer, .{ 512, letters.data });
    assert(E.peek_next_token(*lexer).type == .IDENT);
    assert(E.set_input_from_file(*lexer, #file));
    prior := lexer.input.data;
    subview: string = .{ 4, lexer.input.data + 1 };

    E.set_input_from_string(*lexer, subview);
    assert(lexer.should_free_input && lexer.input == "impo" && lexer.input.data != prior + 1);
    E.set_input_from_string(*lexer, "");
    print("ok\n");
}

expect :: (lexer: *E.Lexer, kind: E.Token_Type, value: string) {
    token := E.peek_next_token(lexer);
    assert(token.type == kind && token.string_value == value, "unexpected token %, %", token.type, token.string_value);
    assert(token.l1 >= token.l0);
    E.eat_token(lexer);
}
