// Jai_Lexer at run time: token kinds for identifiers, keywords, every compound operator,
// numbers (bases, underscores, floats, overflow), string escapes, here-strings, notes,
// comments, backticks, positions, lookahead and the error paths.
#import "Basic";
#import "Jai_Lexer";

lex :: (source: string) -> [..] Token, Lexer {
    lexer: Lexer;
    set_input_from_string(*lexer, source);
    tokens: [..] Token;
    while true {
        token := peek_next_token(*lexer);
        if token.type == .END_OF_INPUT break;
        array_add(*tokens, token.*);
        eat_token(*lexer);
    }
    return tokens, lexer;
}

kinds :: (source: string) -> [..] Token_Type {
    tokens := lex(source);
    result: [..] Token_Type;
    for tokens array_add(*result, it.type);
    array_free(tokens);
    return result;
}

same_kinds :: (source: string, expected: [] Token_Type, loc := #caller_location) {
    got := kinds(source);
    assert(got.count == expected.count, "'%': % tokens, expected %: %", source, got.count, expected.count, got, loc = loc);
    for got assert(it == expected[it_index], "'%' token %: %, expected %", source, it_index, it, expected[it_index], loc = loc);
    array_free(got);
}

as_type :: (c: u8) -> Token_Type {
    return cast(Token_Type) c;
}

main :: () {
    // Identifiers and keywords.
    tokens, lexer := lex("x_1 for ifx then interface Some_Type _private xx");
    assert(tokens.count == 8);
    assert(tokens[0].type == .IDENT && tokens[0].ident_value.name == "x_1");
    assert(tokens[1].type == .KEYWORD_FOR && tokens[2].type == .KEYWORD_IFX && tokens[3].type == .KEYWORD_THEN && tokens[4].type == .KEYWORD_INTERFACE);
    assert(tokens[5].type == .IDENT && tokens[5].ident_value.name == "Some_Type" && tokens[6].ident_value.name == "_private" && tokens[7].type == .KEYWORD_AUTO_CAST);
    assert(tokens[0].ident_value.hash != tokens[5].ident_value.hash && tokens[0].ident_value.hash != 0);
    array_free(tokens);
    same_kinds("if else case return struct while break continue remove using defer", .[.KEYWORD_IF, .KEYWORD_ELSE, .KEYWORD_CASE, .KEYWORD_RETURN, .KEYWORD_STRUCT, .KEYWORD_WHILE, .KEYWORD_BREAK, .KEYWORD_CONTINUE, .KEYWORD_REMOVE, .KEYWORD_USING, .KEYWORD_DEFER]);
    same_kinds("size_of type_of code_of initializer_of type_info null enum true false", .[.KEYWORD_SIZE_OF, .KEYWORD_TYPE_OF, .KEYWORD_CODE_OF, .KEYWORD_INITIALIZER_OF, .KEYWORD_TYPE_INFO, .KEYWORD_NULL, .KEYWORD_ENUM, .KEYWORD_TRUE, .KEYWORD_FALSE]);
    same_kinds("inline no_inline cast context push_context operator is_constant enum_flags union", .[.KEYWORD_INLINE, .KEYWORD_NO_INLINE, .KEYWORD_CAST, .KEYWORD_CONTEXT, .KEYWORD_PUSH_CONTEXT, .KEYWORD_OPERATOR, .KEYWORD_IS_CONSTANT, .KEYWORD_ENUM_FLAGS, .KEYWORD_UNION]);
    same_kinds("format iffy forever", .[.IDENT, .IDENT, .IDENT]);

    // Operators: the longest spelling wins.
    same_kinds("<<<= >>>= &&= ||= <<= >>= === ---", .[.ROTATE_LEFT_EQUALS, .ROTATE_RIGHT_EQUALS, .LOGICAL_AND_EQUALS, .LOGICAL_OR_EQUALS, .SHIFT_LEFT_EQUALS, .SHIFT_RIGHT_EQUALS, .TRIPLE_EQUALS, .TRIPLE_MINUS]);
    same_kinds("+= -= *= /= %= == != && || <= >=", .[.PLUSEQUALS, .MINUSEQUALS, .TIMESEQUALS, .DIVEQUALS, .MODEQUALS, .ISEQUAL, .ISNOTEQUAL, .LOGICAL_AND, .LOGICAL_OR, .LESSEQUALS, .GREATEREQUALS]);
    same_kinds("-> .. $$ << >> &= |= ^= .* -- ,, .{ .[", .[.RIGHT_ARROW, .DOUBLE_DOT, .DOUBLE_DOLLAR, .POINTER_DEREFERENCE_OR_SHIFT_LEFT, .SHIFT_RIGHT, .BITWISE_AND_EQUALS, .BITWISE_OR_EQUALS, .BITWISE_XOR_EQUALS, .POSTFIX_DEREFERENCE, .DOUBLE_MINUS, .DOUBLE_COMMA, .BEGIN_STRUCT_LITERAL, .BEGIN_ARRAY_LITERAL]);
    same_kinds("<<< >>> =>", .[.ROTATE_LEFT, .ROTATE_RIGHT, .QUICK_LAMBDA]);
    same_kinds("x := a[1] ^ ~b;", .[.IDENT, as_type(#char ":"), as_type(#char "="), .IDENT, as_type(#char "["), .NUMBER, as_type(#char "]"), .BITWISE_XOR, .BITWISE_NOT, .IDENT, as_type(#char ";")]);
    same_kinds("a&b|c", .[.IDENT, .BITWISE_AND, .IDENT, .BITWISE_OR, .IDENT]);

    // Numbers.
    tokens, lexer = lex("42 0x1F 0b101 1_000_000 18446744073709551615 18446744073709551616");
    assert(tokens[0].type == .NUMBER && tokens[0].integer_value == 42 && tokens[0].value_flags == .NUMBER);
    assert(tokens[1].integer_value == 31 && (tokens[1].value_flags & .HEX) != 0);
    assert(tokens[2].integer_value == 5 && (tokens[2].value_flags & .BINARY) != 0);
    assert(tokens[3].integer_value == 1000000);
    assert(tokens[4].integer_value == 0xffff_ffff_ffff_ffff && (tokens[4].value_flags & .OVERFLOWED) == 0);
    assert((tokens[5].value_flags & .OVERFLOWED) != 0, "2^64 overflows");
    array_free(tokens);
    tokens, lexer = lex("3.25 1e3 .5 1_000.5 2.5e-3 1e300 0.1");
    assert(tokens.count == 7);
    for tokens assert(it.type == .NUMBER && (it.value_flags & .FLOAT) != 0, "float token %", it_index);
    assert(tokens[0].float64_value == 3.25 && tokens[1].float64_value == 1000 && tokens[2].float64_value == 0.5);
    assert(tokens[3].float64_value == 1000.5, "underscores in a float literal: %", tokens[3].float64_value);
    assert(tokens[4].float64_value == 0.0025 && tokens[6].float64_value == 0.1);
    assert((tokens[5].value_flags & .REQUIRES_FLOAT64) != 0 && (tokens[0].value_flags & .REQUIRES_FLOAT64) == 0);
    array_free(tokens);
    same_kinds("1..10", .[.NUMBER, .DOUBLE_DOT, .NUMBER]);
    same_kinds("a.b", .[.IDENT, as_type(#char "."), .IDENT]);

    // Strings and escapes.
    tokens, lexer = lex("\"a\\tb\\x41\\u00e9\\U0001F600\\n\\\"q\\\\\" \"\\e\\0\"");
    assert(tokens.count == 2 && tokens[0].type == .STRING);
    assert(tokens[0].string_value == "a\tbAé😀\n\"q\\", "escapes: '%'", tokens[0].string_value);
    assert(tokens[1].string_value.count == 2 && tokens[1].string_value[0] == 0x1b && tokens[1].string_value[1] == 0, "\\e and \\0: %", tokens[1].string_value.count);
    assert(!lexer.reported_error);
    array_free(tokens);
    bad_inputs := string.["\"open", "\"line\nbreak\"", "\"\\u12\"", "\"\\uD800\"", "\"\\xZZ\""];
    for bad_inputs {
        broken, broken_lexer := lex(it);
        assert(broken.count >= 1 && broken[0].type == .ERROR && broken_lexer.reported_error, "'%' is an error token", it);
        array_free(broken);
    }

    // A hexadecimal float is not decoded: one error token, and scanning resumes after it.
    hexfloat, hexfloat_lexer := lex("0x1.8p3 after");
    assert(hexfloat.count == 2 && hexfloat[0].type == .ERROR && hexfloat_lexer.reported_error, "hexfloat: %", hexfloat);
    assert(hexfloat[1].type == .IDENT && hexfloat[1].ident_value.name == "after");
    array_free(hexfloat);

    // check_for_equals picks the augmented kind when "=" follows; c1 is one past the last column.
    equals_lexer: Lexer;
    set_input_from_string(*equals_lexer, "+= -");
    plus := check_for_equals(*equals_lexer, as_type(#char "+"), .PLUSEQUALS);
    assert(plus.type == .PLUSEQUALS && plus.c0 == 1 && plus.c1 == 3, "+=: % % %", plus.type, plus.c0, plus.c1);
    eat_character(*equals_lexer);
    minus := check_for_equals(*equals_lexer, as_type(#char "-"), .MINUSEQUALS);
    assert(minus.type == as_type(#char "-") && minus.c0 == 4 && minus.c1 == 5, "-: % % %", minus.type, minus.c0, minus.c1);

    // Here-strings keep their body verbatim.
    tokens, lexer = lex("x := #string END\nline \"1\"\n  line 2\nEND\ny");
    assert(tokens.count == 5 && tokens[3].type == .STRING && (tokens[3].value_flags & .HERE_STRING) != 0, "%", tokens.count);
    assert(tokens[3].string_value == "line \"1\"\n  line 2\n", "here-string body: '%'", tokens[3].string_value);
    assert(tokens[4].type == .IDENT && tokens[4].l0 == 5);
    assert(count_herestring_lines(*lexer, *tokens[3]) == 3);
    array_free(tokens);
    unterminated, unterminated_lexer := lex("#string X\nno end");
    assert(unterminated[0].type == .ERROR && unterminated_lexer.reported_error);
    array_free(unterminated);
    same_kinds("#stringy", .[as_type(#char "#"), .IDENT]);

    // Notes, comments, backticks.
    tokens, lexer = lex("f :: () {} @Note @@System // gone\n/* outer /* inner */ still */ `caller_value");
    note := tokens[tokens.count - 3];
    system := tokens[tokens.count - 2];
    assert(note.type == .NOTE && note.ident_value.name == "Note" && note.value_flags == 0);
    assert(system.type == .NOTE && system.ident_value.name == "System" && cast(u32) system.value_flags == cast(u32) Note_Flags.IS_SYSTEM_LEVEL);
    last := tokens[tokens.count - 1];
    assert(last.type == .IDENT && last.ident_value.name == "caller_value" && last.ident_is_backticked && last.c0 == 31, "backticked identifier at column %", last.c0);
    array_free(tokens);
    open_comment, comment_lexer := lex("a /* never closed");
    assert(open_comment.count == 1 && comment_lexer.reported_error);
    array_free(open_comment);

    // Positions: lines and columns count from 1; c1 is one past the last character.
    tokens, lexer = lex("ab\n  cde += 1");
    assert(tokens[0].l0 == 1 && tokens[0].c0 == 1 && tokens[0].l1 == 1 && tokens[0].c1 == 3, "ab at % % % %", tokens[0].l0, tokens[0].c0, tokens[0].l1, tokens[0].c1);
    assert(tokens[1].l0 == 2 && tokens[1].c0 == 3 && tokens[1].c1 == 6);
    assert(tokens[2].type == .PLUSEQUALS && tokens[2].c0 == 7 && tokens[2].c1 == 9, "+= at % %", tokens[2].c0, tokens[2].c1);
    assert(lexer.total_lines_processed == 1 && lexer.nonblank_lines_processed == 2);
    array_free(tokens);

    // Lookahead and the token ring.
    ring: Lexer;
    set_input_from_string(*ring, "a b c d e f g h i j");
    third := peek_token(*ring, 2);
    assert(third.ident_value.name == "c" && peek_next_token(*ring).ident_value.name == "a");
    for 0..8 eat_token(*ring);
    assert(get_last_token(*ring).ident_value.name == "i" && peek_next_token(*ring).ident_value.name == "j");
    seventh := peek_token(*ring, 1);
    assert(seventh.type == .END_OF_INPUT && ring.eof_token.type == .END_OF_INPUT);

    // Character-level helpers.
    chars: Lexer;
    set_input_from_string(*chars, "7f\nz");
    assert(get_decimal_digit(*chars) == 7 && get_decimal_digit(*chars) == -1);
    digit, ok := get_hex_digit(*chars, .ASCII_8_BIT);
    assert(ok && digit == 15 && peek_next_character(*chars) == #char "\n");
    eat_character(*chars);
    assert(chars.current_line_number == 2 && chars.current_character_index == 1);
    unwind_one_character(*chars);
    assert(chars.current_line_number == 1 && chars.current_character_index == 3 && peek_next_character(*chars) == #char "\n", "unwind across a newline: % %", chars.current_line_number, chars.current_character_index);
    _, ok = get_hex_digit(*chars, .UNICODE_16);
    assert(!ok);
    eat_until_newline(*chars);
    eat_character(*chars);
    eat_character(*chars);
    assert(peek_next_character(*chars) == -1);
    eat_character(*chars);
    assert(chars.input_cursor == 4, "eating at the end is harmless");
    set_input_from_string(*chars, "");
    unwind_one_character(*chars);
    assert(chars.input_cursor == 0 && peek_next_token(*chars).type == .END_OF_INPUT);

    // Classification helpers.
    assert(starts_identifier(#char "_") && !starts_identifier(#char "1") && continues_identifier(#char "1") && !continues_identifier(-1) && !continues_identifier(#char "-"));
    assert(starts_number(#char "0") && !starts_number(#char "a") && has_a_big_exponent(1e39) && has_a_big_exponent(1e-39) && !has_a_big_exponent(1.5) && !has_a_big_exponent(0));

    // Reusing a lexer resets its state; a missing file is reported.
    reuse: Lexer;
    set_input_from_string(*reuse, "\"bad");
    peek_next_token(*reuse);
    assert(reuse.reported_error);
    set_input_from_string(*reuse, "fine");
    assert(!reuse.reported_error && peek_next_token(*reuse).ident_value.name == "fine" && reuse.current_line_number == 1);
    // WASI builds link no C file functions (fopen and the like), so the file path is native only.
    #if OS != .WASM assert(!set_input_from_file(*reuse, "this/file/does/not/exist.jai"));
    report_detail(*reuse, "report_detail goes to the log: %\n", 1);

    print("ok\n");
}
