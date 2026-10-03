set_input_from_string :: (lexer: *Lexer, input: string) {
    if !lexer return;

    owned := false;

    if lexer.should_free_input {
        if input.data >= lexer.input.data && input.data < lexer.input.data + lexer.input.count {
            input = B.copy_string(input);
            owned = true;
        }

        B.free(lexer.input);
    }

    user := lexer.user_data;

    lexer.* = .{};
    lexer.user_data = user;
    lexer.input = input;
    lexer.should_free_input = owned;
    lexer.current_line_number = 1;
    lexer.current_character_index = 1;
}

set_input_from_file :: (lexer: *Lexer, file_path: string) -> bool {
    if !lexer return false;

    input, ok := F.read_entire_file(file_path);
    if !ok return false;

    set_input_from_string(lexer, input);
    lexer.should_free_input = true;

    return true;
}

peek_next_character :: (lexer: *Lexer) -> s16 {
    if !lexer || lexer.input_cursor < 0 || lexer.input_cursor >= lexer.input.count return -1;
    return lexer.input[lexer.input_cursor];
}

eat_character :: (using lexer: *Lexer) {
    c := peek_next_character(lexer);
    if c < 0 return;

    input_cursor += 1;

    if c == 10 {
        if current_line_number < 0x7fffffff current_line_number += 1;
        current_character_index = 1;
        total_lines_processed += 1;
    } else if current_character_index < 0x7fffffff current_character_index += 1;
}

unwind_one_character :: (lexer: *Lexer) {
    if !lexer || lexer.input_cursor <= 0 return;

    end := lexer.input_cursor - 1;

    lexer.input_cursor = 0;
    lexer.current_line_number = 1;
    lexer.current_character_index = 1;
    lexer.total_lines_processed = 0;
    while lexer.input_cursor < end eat_character(lexer);
}

starts_identifier :: inline (c: u8) -> bool {
    return (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || c == #char "_" || c == #char "$" || c >= 128;
}

continues_identifier :: (c: s32) -> bool {
    if c < 0 || c > 255 return false;
    return starts_identifier(cast(u8) c) || starts_number(c);
}

starts_number :: (c: s32) -> bool {
    return c >= #char "0" && c <= #char "9";
}

is_non_newline_whitespace :: (c: s16) -> bool {
    return c == 32 || c == 9 || c == 13 || c == 11 || c == 12;
}

get_decimal_digit :: (lexer: *Lexer) -> s32 {
    c := peek_next_character(lexer);
    if !starts_number(c) return -1;
    eat_character(lexer);
    return c - #char "0";
}

get_hex_digit :: (lexer: *Lexer, type: Hex_Digit_Type) -> (value: u8, success: bool) {
    digit := lex_digit(peek_next_character(lexer));
    if digit < 0 return 0, false;
    eat_character(lexer);
    return cast(u8) digit, true;
}

get_and_eat_remainder_of_line :: (lexer: *Lexer, line: s32, column: s32) -> string {
    if !lexer return "";

    if line != lexer.current_line_number || column != lexer.current_character_index {
        report_parse_error(lexer, "remainder position does not match cursor");
        return "";
    }

    start := lexer.input_cursor;

    eat_until_newline(lexer);
    result: string = .{ lexer.input_cursor - start, lexer.input.data + start };

    if peek_next_character(lexer) == 10 eat_character(lexer);

    return result;
}

eat_until_newline :: (lexer: *Lexer) {
    while peek_next_character(lexer) >= 0 && peek_next_character(lexer) != 10 eat_character(lexer);
}

set_end_of_token :: (lexer: *Lexer, token: *Token) {
    if !lexer || !token return;
    token.l1 = lexer.current_line_number;
    token.c1 = lexer.current_character_index;
}

get_unused_token :: (lexer: *Lexer) -> *Token {
    if !lexer return null;

    if lexer.num_incoming_tokens >= MAX_CONCURRENT_TOKENS {
        report_parse_error(lexer, "token lookahead capacity exceeded");
        lexer.eof_token = .{};
        return *lexer.eof_token;
    }

    index := (lexer.token_array_cursor + lexer.num_incoming_tokens) % MAX_CONCURRENT_TOKENS;
    result := *lexer.tokens[index];
    result.* = .{};
    result.l0 = lexer.current_line_number;
    result.c0 = lexer.current_character_index;
    result.preceeding_whitespace = lexer.preceeding_whitespace;
    lexer.num_incoming_tokens += 1;

    return result;
}

parse_ident :: (lexer: *Lexer, result: *Token) -> string {
    if !lexer || !result return "";

    start := lexer.input_cursor;

    result.l0 = lexer.current_line_number;
    result.c0 = lexer.current_character_index;
    while continues_identifier(peek_next_character(lexer)) eat_character(lexer);

    result.string_value = .{ lexer.input_cursor - start, lexer.input.data + start };
    result.type = .IDENT;
    if result.string_value.count == 0 || result.string_value.count > MAX_IDENTIFIER_LENGTH {
        result.type = .ERROR;
        report_parse_error(lexer, result, "invalid identifier length");
    }

    set_end_of_token(lexer, result);

    return result.string_value;
}

check_for_keyword :: (token: *Token) -> Token_Type {
    if !token return .ERROR;

    // This token vocabulary has no reserved-word variants. Recognize the
    // punctuation spellings it does expose, leaving all names as identifiers.
    if token.type == .IDENT {
        if token.string_value == "---" token.type = .TRIPLE_MINUS;
        if token.string_value == "..." token.type = .TRIPLE_DOT;
    }

    return token.type;
}

make_ident_or_keyword :: (lexer: *Lexer) -> *Token {
    token := get_unused_token(lexer);
    if !token || token == *lexer.eof_token return token;

    parse_ident(lexer, token);
    check_for_keyword(token);

    return token;
}

make_one_character_token :: (lexer: *Lexer, c: Token_Type) -> *Token {
    token := get_unused_token(lexer);
    if !token || token == *lexer.eof_token return token;

    token.type = c;
    start := lexer.input_cursor;

    eat_character(lexer);
    token.string_value = .{ lexer.input_cursor - start, lexer.input.data + start };
    set_end_of_token(lexer, token);

    return token;
}

check_for_equals :: (lexer: *Lexer, c: Token_Type, augmented: Token_Type, eat := true, subtractor: s32 = 0) -> *Token {
    token := get_unused_token(lexer);
    if !token || token == *lexer.eof_token return token;

    token.type = c;

    if peek_next_character(lexer) == #char "=" {
        code := (cast(s64) augmented) - (cast(s64) subtractor);
        if code < -32768 || code > 32767 {
            report_parse_error(lexer, "augmented token code out of range");
            token.type = .ERROR;
        } else token.type = cast(Token_Type) code;

        if eat eat_character(lexer);
    }

    set_end_of_token(lexer, token);

    return token;
}

make_string_constant :: (lexer: *Lexer, end_char: s16) -> *Token {
    token := get_unused_token(lexer);
    if !token || token == *lexer.eof_token return token;

    if peek_next_character(lexer) != end_char || (end_char != 34 && end_char != 39) {
        report_parse_error(lexer, token, "expected string delimiter");
        return token;
    }

    eat_character(lexer);

    if end_char == 34 && lex_at(lexer, 0) == 34 && lex_at(lexer, 1) == 34 {
        eat_character(lexer);
        eat_character(lexer);
        start := lexer.input_cursor;

        while peek_next_character(lexer) >= 0 {
            if lex_at(lexer, 0) == 34 && lex_at(lexer, 1) == 34 && lex_at(lexer, 2) == 34 {
                token.string_value = .{ lexer.input_cursor - start, lexer.input.data + start };
                for 0..2 eat_character(lexer);

                token.type = .STRING;
                token.value_flags = .HERE_STRING;
                set_end_of_token(lexer, token);

                return token;
            }

            eat_character(lexer);
        }

        report_parse_error(lexer, token, "unterminated here string");
        set_end_of_token(lexer, token);

        return token;
    }

    bytes: [..] u8;

    defer B.array_free(bytes);

    while peek_next_character(lexer) >= 0 {
        c := peek_next_character(lexer);

        eat_character(lexer);

        if c == end_char {
            if end_char == 39 && peek_next_character(lexer) == 39 {
                eat_character(lexer);
                B.array_add(*bytes, cast(u8) 39);
                continue;
            }

            token.type = .STRING;
            token.string_value = B.copy_temporary_string(.{ bytes.count, bytes.data });
            set_end_of_token(lexer, token);

            return token;
        }

        if c == 10 || c == 13 {
            report_parse_error(lexer, token, "newline in quoted string");
            set_end_of_token(lexer, token);
            return token;
        }

        if c == 92 && end_char == 34 {
            escape := peek_next_character(lexer);
            if escape < 0 break;

            eat_character(lexer);

            if escape == #char "n" c = 10;
            else if escape == #char "r" c = 13;
            else if escape == #char "t" c = 9;
            else if escape == #char "0" c = 0;
            else if escape == 34 || escape == 92 c = escape;
            else if escape == #char "x" || escape == #char "u" || escape == #char "U" {
                digits: s64 = 2;
                if escape == #char "u" digits = 4;
                if escape == #char "U" digits = 8;

                value: u32;

                for 0..digits - 1 {
                    digit, ok := get_hex_digit(lexer, .UNICODE_32);
                    if !ok {
                        report_parse_error(lexer, token, "invalid hexadecimal escape");
                        set_end_of_token(lexer, token);
                        return token;
                    }

                    value = (value << 4) | digit;
                }

                if escape == #char "x" B.array_add(*bytes, cast(u8) value);
                else if !lex_utf8(*bytes, value) {
                    report_parse_error(lexer, token, "invalid Unicode scalar");
                    set_end_of_token(lexer, token);
                    return token;
                }

                continue;
            } else {
                report_parse_error(lexer, token, "unknown string escape");
                set_end_of_token(lexer, token);
                return token;
            }
        }

        B.array_add(*bytes, cast(u8) c);
    }

    report_parse_error(lexer, token, "unterminated string");
    set_end_of_token(lexer, token);

    return token;
}

count_herestring_lines :: (lexer: *Lexer, token: *Token) -> s64 {
    if !token || token.value_flags & .HERE_STRING == 0 return 0;
    lines: s64 = 1;
    for token.string_value if it == 10 lines += 1;
    return lines;
}

eat_input_due_to_block_comment :: (lexer: *Lexer) {
    if lex_at(lexer, 0) != 47 || lex_at(lexer, 1) != 42 return;

    eat_character(lexer);
    eat_character(lexer);
    depth: s64 = 1;

    while peek_next_character(lexer) >= 0 {
        if lex_at(lexer, 0) == 47 && lex_at(lexer, 1) == 42 {
            depth += 1;
            eat_character(lexer);
            eat_character(lexer);
        } else if lex_at(lexer, 0) == 42 && lex_at(lexer, 1) == 47 {
            depth -= 1;
            eat_character(lexer);
            eat_character(lexer);
            if depth == 0 return;
        } else eat_character(lexer);
    }

    report_parse_error(lexer, "unterminated block comment");
}

compose_new_token :: (lexer: *Lexer) -> *Token {
    if !lexer return null;
    if lexer.num_incoming_tokens >= MAX_CONCURRENT_TOKENS return get_unused_token(lexer);

    start := lexer.input_cursor;

    while true {
        c := peek_next_character(lexer);
        if is_non_newline_whitespace(c) || c == 10 eat_character(lexer);
        else if c == 47 && lex_at(lexer, 1) == 47 eat_until_newline(lexer);
        else if c == 47 && lex_at(lexer, 1) == 42 eat_input_due_to_block_comment(lexer);
        else break;
    }

    whitespace := lexer.input_cursor - start;

    lexer.preceeding_whitespace = cast(s32) B.min(whitespace, 0x7fffffff);
    c := peek_next_character(lexer);
    if c >= 0 && lexer.previous_token_line_number != lexer.current_line_number {
        lexer.nonblank_lines_processed += 1;
        lexer.previous_token_line_number = lexer.current_line_number;
    }

    if c < 0 {
        token := get_unused_token(lexer);
        if token {
            token.type = .END_OF_INPUT;
            set_end_of_token(lexer, token);
            lexer.eof_token = token.*;
        }

        return token;
    }

    if c == 34 || c == 39 return make_string_constant(lexer, c);
    if starts_number(c) || (c == 46 && starts_number(lex_at(lexer, 1))) return lex_number(lexer);
    if starts_identifier(cast(u8) c) return make_ident_or_keyword(lexer);

    if (c == 45 || c == 46) && lex_at(lexer, 1) == c && lex_at(lexer, 2) == c {
        token := get_unused_token(lexer);
        token.type = .TRIPLE_MINUS;
        if c == 46 token.type = .TRIPLE_DOT;

        token.string_value = .{ 3, lexer.input.data + lexer.input_cursor };
        for 0..2 eat_character(lexer);

        set_end_of_token(lexer, token);

        return token;
    }

    if c < 32 {
        token := make_one_character_token(lexer, .ERROR);
        report_parse_error(lexer, token, "invalid control character");
        return token;
    }

    return make_one_character_token(lexer, cast(Token_Type) c);
}

peek_token :: (lexer: *Lexer, lookahead_index: s32) -> *Token {
    if !lexer return null;

    if lookahead_index < 0 || lookahead_index >= MAX_CONCURRENT_TOKENS {
        report_parse_error(lexer, "token lookahead index out of range");
        lexer.eof_token = .{};
        return *lexer.eof_token;
    }

    while lexer.num_incoming_tokens <= lookahead_index compose_new_token(lexer);

    return *lexer.tokens[(lexer.token_array_cursor + lookahead_index) % MAX_CONCURRENT_TOKENS];
}

peek_next_token :: (using lexer: *Lexer) -> *Token {
    return peek_token(lexer, 0);
}

eat_token :: (lexer: *Lexer) {
    if !lexer return;
    if lexer.num_incoming_tokens == 0 peek_next_token(lexer);

    if lexer.num_incoming_tokens > 0 {
        lexer.num_incoming_tokens -= 1;
        lexer.token_array_cursor = (lexer.token_array_cursor + 1) % MAX_CONCURRENT_TOKENS;
    }
}

get_last_token :: (lexer: *Lexer) -> Token {
    if !lexer return .{};
    return lexer.tokens[(lexer.token_array_cursor + MAX_CONCURRENT_TOKENS - 1) % MAX_CONCURRENT_TOKENS];
}

has_a_big_exponent :: inline (value: float64) -> bool {
    magnitude := value;
    if magnitude < 0 magnitude = -magnitude;
    return magnitude != 0 && (magnitude < 0.0001 || magnitude >= 1000000000000000.0);
}

report_parse_error :: (lexer: *Lexer, format: string, arguments: ..Any) {
    if !lexer return;
    report_parse_error(lexer, lexer.current_line_number, lexer.current_character_index, format, ..arguments);
}

report_parse_error :: (lexer: *Lexer, token: *Token, format: string, arguments: ..Any) {
    if token report_parse_error(lexer, token.l0, token.c0, format, ..arguments);
    else report_parse_error(lexer, format, ..arguments);
}

report_parse_error :: (lexer: *Lexer, line: s32, col: s32, format: string, arguments: ..Any) {
    if !lexer return;

    lexer.reported_error = true;
    lexer.current_external_file_error_report_line_number = line;
    message := B.sprint(format, ..arguments);

    defer B.free(message);
    B.print("lexer:%:%: error: %\n", line, col, message);
}

report_detail :: (lexer: *Lexer, format: string, arguments: ..Any) {
    if !lexer return;
    report_detail(lexer, lexer.current_line_number, lexer.current_character_index, format, ..arguments);
}

report_detail :: (lexer: *Lexer, line: s32, col: s32, format: string, arguments: ..Any) {
    if !lexer return;
    message := B.sprint(format, ..arguments);
    defer B.free(message);
    B.print("lexer:%:%: detail: %\n", line, col, message);
}

#scope_file
B :: #import "Basic";
F :: #import "File";

lex_at :: (lexer: *Lexer, ahead: s64) -> s16 {
    if !lexer || ahead < 0 || lexer.input_cursor < 0 || lexer.input_cursor > lexer.input.count || ahead >= lexer.input.count - lexer.input_cursor return -1;
    return lexer.input[lexer.input_cursor + ahead];
}

lex_digit :: (c: s16) -> s32 {
    if c >= 48 && c <= 57 return c - 48;
    if c >= 65 && c <= 70 return c - 65 + 10;
    if c >= 97 && c <= 102 return c - 97 + 10;
    return -1;
}

lex_utf8 :: (bytes: *[..] u8, value: u32) -> bool {
    if value > 0x10ffff || (value >= 0xd800 && value <= 0xdfff) return false;

    if value < 0x80 B.array_add(bytes, cast(u8) value);
    else {
        if value < 0x800 B.array_add(bytes, cast(u8) (0xc0 | (value >> 6)));
        else {
            if value < 0x10000 B.array_add(bytes, cast(u8) (0xe0 | (value >> 12)));
            else {
                B.array_add(bytes, cast(u8) (0xf0 | (value >> 18)));
                B.array_add(bytes, cast(u8) (0x80 | ((value >> 12) & 63)));
            }

            B.array_add(bytes, cast(u8) (0x80 | ((value >> 6) & 63)));
        }

        B.array_add(bytes, cast(u8) (0x80 | (value & 63)));
    }

    return true;
}

lex_number :: (lexer: *Lexer) -> *Token {
    token := get_unused_token(lexer);
    if !token || token == *lexer.eof_token return token;

    token.type = .IDENT;
    token.value_flags = .NUMBER;
    start := lexer.input_cursor;
    base: u64 = 10;

    if lex_at(lexer, 0) == 48 && (lex_at(lexer, 1) == #char "x" || lex_at(lexer, 1) == #char "X") {
        base = 16;
        token.value_flags |= .HEX;
        eat_character(lexer);
        eat_character(lexer);
    } else if lex_at(lexer, 0) == 48 && (lex_at(lexer, 1) == #char "b" || lex_at(lexer, 1) == #char "B") {
        base = 2;
        token.value_flags |= .BINARY;
        eat_character(lexer);
        eat_character(lexer);
    }

    value: u64;
    digits: s64;
    previous_digit := false;
    valid := true;

    while true {
        c := peek_next_character(lexer);
        digit := lex_digit(c);

        if digit >= 0 && cast(u64) digit < base {
            lane := cast(u64) digit;
            if value > (0xffffffffffffffff - lane) / base token.value_flags |= .OVERFLOWED;
            else value = value * base + lane;

            digits += 1;
            previous_digit = true;
            eat_character(lexer);
        } else if c == #char "_" {
            next := lex_digit(lex_at(lexer, 1));
            if !previous_digit || next < 0 || cast(u64) next >= base valid = false;
            previous_digit = false;
            eat_character(lexer);
        } else break;
    }

    if base == 10 && peek_next_character(lexer) == 46 && starts_number(lex_at(lexer, 1)) {
        token.value_flags |= .FLOAT;
        eat_character(lexer);

        while starts_number(peek_next_character(lexer)) {
            digits += 1;
            eat_character(lexer);
        }
    }

    if base == 10 && (peek_next_character(lexer) == #char "e" || peek_next_character(lexer) == #char "E") {
        token.value_flags |= .FLOAT;
        eat_character(lexer);
        if peek_next_character(lexer) == 43 || peek_next_character(lexer) == 45 eat_character(lexer);

        exponent_digits: s64;

        while starts_number(peek_next_character(lexer)) {
            eat_character(lexer);
            exponent_digits += 1;
        }

        if exponent_digits == 0 valid = false;
    }

    if continues_identifier(peek_next_character(lexer)) {
        valid = false;
        while continues_identifier(peek_next_character(lexer)) eat_character(lexer);
    }

    if token.value_flags & .FLOAT != 0 {
        token.value_flags |= .DEFAULTS_TO_FLOAT64;
        if digits > 15 token.value_flags |= .REQUIRES_FLOAT64_DUE_TO_SIGNIFICANT_DIGITS | .REQUIRES_FLOAT64;
    }

    if digits == 0 || !valid {
        token.type = .ERROR;
        report_parse_error(lexer, token, "malformed number");
    }

    token.string_value = .{ lexer.input_cursor - start, lexer.input.data + start };
    if token.value_flags & .FLOAT != 0 && valid {
        normalized: [..] u8;

        defer B.array_free(normalized);
        for token.string_value if it != 95 B.array_add(*normalized, it);

        float_ok, number, remainder := B.string_to_float64_new(.{ normalized.count, normalized.data });

        token.value_flags &= ~Value_Flags.OVERFLOWED;
        if !float_ok || remainder.count != 0 || number > 1.7976931348623157e308 || number != number token.value_flags |= .OVERFLOWED;
        if number > 3.402823466e38 || (number != 0 && number < 1.401298464e-45) token.value_flags |= .REQUIRES_FLOAT64;
    }

    set_end_of_token(lexer, token);

    return token;
}
