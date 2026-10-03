make_string_constant :: (lexer: *J.Lexer) -> *J.Token {
    result := get_unused_token(lexer);
    result.type = .STRING;
    builder: Basic.String_Builder;

    Basic.init_string_builder(*builder);
    eat_character(lexer);
    closed := false;

    while look(lexer, 0) >= 0 {
        c := look(lexer, 0);

        eat_character(lexer);

        if c == 34 {
            closed = true;
            break;
        }

        if c != 92 {
            Basic.append(*builder, cast(u8) c);
            continue;
        }

        c = look(lexer, 0);
        if c < 0 break;

        eat_character(lexer);

        if c == 110 Basic.append(*builder, cast(u8) 10);
        else if c == 114 Basic.append(*builder, cast(u8) 13);
        else if c == 116 Basic.append(*builder, cast(u8) 9);
        else if c == 48 Basic.append(*builder, cast(u8) 0);
        else if c == 92 || c == 34 Basic.append(*builder, cast(u8) c);
        else if c == 120 || c == 117 || c == 85 {
            digits: s64 = 2;
            kind := J.Hex_Digit_Type.ASCII_8_BIT;

            if c == 117 {
                digits = 4;
                kind = .UNICODE_16;
            }

            if c == 85 {
                digits = 8;
                kind = .UNICODE_32;
            }

            value: u32;
            valid := true;

            for 0..digits - 1 {
                digit, ok := get_hex_digit(lexer, kind);
                if !ok {
                    valid = false;
                    break;
                }

                value = (value << 4) | cast(u32) digit;
            }

            if valid {
                if c == 120 Basic.append(*builder, cast(u8) value);
                else append_unicode(lexer, *builder, value);
            }
        } else if c == 100 {
            value: s32;
            valid := true;

            for 0..2 {
                digit := get_decimal_digit(lexer);
                if digit < 0 {
                    valid = false;
                    break;
                }

                value = value * 10 + digit;
            }

            if valid && value <= 255 Basic.append(*builder, cast(u8) value);
            else if valid report_parse_error(lexer, "decimal escape exceeds one byte");
        } else {
            report_parse_error(lexer, "unknown string escape");
            Basic.append(*builder, cast(u8) c);
        }
    }

    if !closed report_parse_error(lexer, "unterminated string literal");

    result.string_value = Basic.builder_to_string(*builder);
    Basic.free_buffers(*builder);
    set_end_of_token(lexer, result);

    return result;
}

parse_here_string :: (lexer: *J.Lexer) -> *J.Token {
    Basic.assert(lexer.num_incoming_tokens == 0, "here-string parsing requires empty buffered lookahead");
    line := lexer.current_line_number;
    column := lexer.current_character_index;

    while look(lexer, 0) == 32 || look(lexer, 0) == 9 eat_character(lexer);

    cr := false;

    if look(lexer, 0) == 44 {
        eat_character(lexer);
        while look(lexer, 0) == 32 || look(lexer, 0) == 9 eat_character(lexer);

        if !matches_text(lexer, "cr") || continues_identifier(look(lexer, 2)) {
            report_parse_error(lexer, "unknown here-string modifier");
            return null;
        }

        eat_character(lexer);
        eat_character(lexer);
        cr = true;
        while look(lexer, 0) == 32 || look(lexer, 0) == 9 eat_character(lexer);
    }

    if look(lexer, 0) < 0 || !starts_identifier(cast(u8) look(lexer, 0)) {
        report_parse_error(lexer, "here-string requires a terminator identifier");
        return null;
    }

    ident_start := lexer.input_cursor;

    while continues_identifier(look(lexer, 0)) eat_character(lexer);

    delimiter := string.{ lexer.input_cursor - ident_start, lexer.input.data + ident_start };

    while look(lexer, 0) == 32 || look(lexer, 0) == 9 || look(lexer, 0) == 13 eat_character(lexer);

    if look(lexer, 0) != 10 {
        report_parse_error(lexer, "here-string header has non-whitespace or no content line");
        return null;
    }

    eat_character(lexer);
    start := lexer.input_cursor;

    while look(lexer, 0) >= 0 {
        line_start := lexer.input_cursor;

        while look(lexer, 0) == 32 || look(lexer, 0) == 9 eat_character(lexer);

        if matches_text(lexer, delimiter) && !continues_identifier(look(lexer, delimiter.count)) {
            content := string.{ line_start - start, lexer.input.data + start };

            eat_until_newline(lexer);
            if look(lexer, 0) == 10 eat_character(lexer);

            builder: Basic.String_Builder;

            Basic.init_string_builder(*builder);

            for c, i: content {
                if c == 13 && !cr && i + 1 < content.count && content[i + 1] == 10 continue;
                if c == 10 && cr && (i == 0 || content[i - 1] != 13) Basic.append(*builder, cast(u8) 13);
                Basic.append(*builder, c);
            }

            result := get_unused_token(lexer);
            result.type = .STRING;
            result.value_flags = .HERE_STRING;
            result.l0 = line;
            result.c0 = column;
            result.string_value = Basic.builder_to_string(*builder);
            Basic.free_buffers(*builder);
            set_end_of_token(lexer, result);

            return result;
        }

        eat_until_newline(lexer);
        if look(lexer, 0) == 10 eat_character(lexer);
    }

    report_parse_error(lexer, "unterminated here-string");

    return null;
}

count_herestring_lines :: (lexer: *J.Lexer, token: *J.Token) -> s64 {
    if !token return 0;
    lines: s64;
    for c: token.string_value if c == 10 lines += 1;
    return lines;
}

#scope_module
append_unicode :: (lexer: *J.Lexer, builder: *Basic.String_Builder, value: u32) {
    if value > 0x10ffff || (value >= 0xd800 && value <= 0xdfff) {
        report_parse_error(lexer, "invalid Unicode scalar in string escape");
        return;
    }

    if value < 0x80 Basic.append(builder, cast(u8) value);
    else if value < 0x800 {
        Basic.append(builder, cast(u8) (0xc0 | (value >> 6)));
        Basic.append(builder, cast(u8) (0x80 | (value & 0x3f)));
    } else if value < 0x10000 {
        Basic.append(builder, cast(u8) (0xe0 | (value >> 12)));
        Basic.append(builder, cast(u8) (0x80 | ((value >> 6) & 0x3f)));
        Basic.append(builder, cast(u8) (0x80 | (value & 0x3f)));
    } else {
        Basic.append(builder, cast(u8) (0xf0 | (value >> 18)));
        Basic.append(builder, cast(u8) (0x80 | ((value >> 12) & 0x3f)));
        Basic.append(builder, cast(u8) (0x80 | ((value >> 6) & 0x3f)));
        Basic.append(builder, cast(u8) (0x80 | (value & 0x3f)));
    }
}

#scope_file
J :: #import "Jai_Lexer";
Basic :: #import "Basic";
