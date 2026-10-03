set_input_from_string :: (lexer: *J.Lexer, input: string) {
    if lexer.should_free_input Basic.free(lexer.input);

    lexer.input = input;
    lexer.input_cursor = 0;
    lexer.should_free_input = false;
    lexer.current_line_number = 1;
    lexer.current_external_file_error_report_line_number = 1;
    lexer.current_character_index = 1;
    lexer.num_incoming_tokens = 0;
    lexer.token_array_cursor = 0;
    lexer.total_lines_processed = 0;
    lexer.nonblank_lines_processed = 0;
    lexer.previous_token_line_number = 0;
    lexer.reported_error = false;
    lexer.eof_token = .{};
}

set_input_from_file :: (lexer: *J.Lexer, file_path: string) -> bool {
    File :: #import "File";
    text, success := File.read_entire_file(file_path);

    if !success return false;

    set_input_from_string(lexer, text);
    lexer.should_free_input = true;

    return true;
}

get_unused_token :: (lexer: *J.Lexer) -> *J.Token {
    Basic.assert(lexer.num_incoming_tokens < J.MAX_CONCURRENT_TOKENS, "lexer lookahead exceeds its public token capacity");
    index := (lexer.token_array_cursor + lexer.num_incoming_tokens) % J.MAX_CONCURRENT_TOKENS;
    token := *lexer.tokens[index];
    token.* = .{};
    token.l0 = lexer.current_line_number;
    token.c0 = lexer.current_character_index;
    lexer.num_incoming_tokens += 1;

    return token;
}

peek_token :: (lexer: *J.Lexer, lookahead_index: s32) -> *J.Token {
    Basic.assert(lookahead_index >= 0 && lookahead_index < J.MAX_CONCURRENT_TOKENS, "invalid lexer lookahead index");
    while lexer.num_incoming_tokens <= lookahead_index compose_new_token(lexer);
    return *lexer.tokens[(lexer.token_array_cursor + lookahead_index) % J.MAX_CONCURRENT_TOKENS];
}

peek_next_token :: (using lexer: *J.Lexer) -> *J.Token {
    return peek_token(lexer, 0);
}

eat_token :: (lexer: *J.Lexer) {
    if lexer.num_incoming_tokens == 0 compose_new_token(lexer);
    lexer.token_array_cursor = (lexer.token_array_cursor + 1) % J.MAX_CONCURRENT_TOKENS;
    lexer.num_incoming_tokens -= 1;
}

get_last_token :: (lexer: *J.Lexer) -> J.Token {
    return lexer.tokens[(lexer.token_array_cursor + J.MAX_CONCURRENT_TOKENS - 1) % J.MAX_CONCURRENT_TOKENS];
}

parse_ident :: (lexer: *J.Lexer, result: *J.Token) -> string {
    start := lexer.input_cursor;

    while continues_identifier(look(lexer, 0)) eat_character(lexer);

    text := string.{ lexer.input_cursor - start, lexer.input.data + start };
    if text.count > J.MAX_IDENTIFIER_LENGTH report_parse_error(lexer, "identifier exceeds maximum length");

    cached_make_atom(lexer, text, result);
    set_end_of_token(lexer, result);

    return result.ident_value.name;
}

make_ident_or_keyword :: (lexer: *J.Lexer) -> *J.Token {
    result := get_unused_token(lexer);
    result.type = .IDENT;
    parse_ident(lexer, result);
    check_for_keyword(result);

    return result;
}

parse_note :: (lexer: *J.Lexer) -> string {
    start := lexer.input_cursor;
    while look(lexer, 0) > 32 eat_character(lexer);
    return Basic.copy_string(.{ lexer.input_cursor - start, lexer.input.data + start });
}

make_note :: (lexer: *J.Lexer) -> *J.Token {
    result := get_unused_token(lexer);
    result.type = .NOTE;
    if look(lexer, 0) == 64 eat_character(lexer);

    result.string_value = parse_note(lexer);
    set_end_of_token(lexer, result);

    return result;
}

make_one_character_token :: (lexer: *J.Lexer, c: J.Token_Type) -> *J.Token {
    result := get_unused_token(lexer);
    result.type = c;
    eat_character(lexer);
    set_end_of_token(lexer, result);

    return result;
}

check_for_equals :: (lexer: *J.Lexer, c: J.Token_Type, augmented: J.Token_Type, eat := true, subtractor: s32 = 0) -> *J.Token {
    result := get_unused_token(lexer);
    result.type = c;
    if eat eat_character(lexer);

    if look(lexer, 0) == 61 {
        eat_character(lexer);
        result.type = augmented;
    }

    result.c0 -= subtractor;
    set_end_of_token(lexer, result);

    return result;
}

compose_new_token :: (lexer: *J.Lexer) -> *J.Token {
    while true {
        c := look(lexer, 0);
        if c >= 0 && c <= 32 {
            eat_character(lexer);
            continue;
        }

        if c == 47 && look(lexer, 1) == 47 {
            eat_until_newline(lexer);
            continue;
        }

        if c == 47 && look(lexer, 1) == 42 {
            eat_character(lexer);
            eat_character(lexer);
            eat_input_due_to_block_comment(lexer);
            continue;
        }

        break;
    }

    c := look(lexer, 0);
    if c < 0 {
        result := get_unused_token(lexer);
        result.type = .END_OF_INPUT;
        set_end_of_token(lexer, result);
        lexer.eof_token = result.*;

        return result;
    }

    if lexer.current_line_number != lexer.previous_token_line_number {
        lexer.nonblank_lines_processed += 1;
        lexer.previous_token_line_number = lexer.current_line_number;
    }

    if starts_identifier(cast(u8) c) return make_ident_or_keyword(lexer);
    if starts_number(c) || (c == 46 && starts_number(look(lexer, 1))) return make_number(lexer);
    if c == 34 return make_string_constant(lexer);
    if c == 64 return make_note(lexer);

    if c == 96 {
        line := lexer.current_line_number;
        column := lexer.current_character_index;

        eat_character(lexer);

        if look(lexer, 0) >= 0 && starts_identifier(cast(u8) look(lexer, 0)) {
            result := make_ident_or_keyword(lexer);
            result.ident_is_backticked = true;
            result.l0 = line;
            result.c0 = column;

            return result;
        }

        report_parse_error(lexer, "backtick requires an identifier");
    }

    if c == 35 && matches_text(lexer, "#string") && !continues_identifier(look(lexer, 7)) {
        line := lexer.current_line_number;
        column := lexer.current_character_index;

        for 0..6 eat_character(lexer);

        token := parse_here_string(lexer);
        if token {
            token.l0 = line;
            token.c0 = column;
            return token;
        }

        return make_one_character_token(lexer, .ERROR);
    }

    token_type, width := punctuation(lexer);
    result := get_unused_token(lexer);
    result.type = token_type;
    for 0..width - 1 eat_character(lexer);

    set_end_of_token(lexer, result);

    return result;
}

#scope_module
matches_text :: (lexer: *J.Lexer, text: string) -> bool {
    if lexer.input_cursor + text.count > lexer.input.count return false;
    for c, i: text if lexer.input[lexer.input_cursor + i] != c return false;
    return true;
}

#scope_file
J :: #import "Jai_Lexer";
Basic :: #import "Basic";
