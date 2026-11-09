// Shared by the compile-time halves of Args, Csv, Json and Toml: renaming rules for keys, the
// splitting of `@json(...)`-style note options, and quoting strings in and out of generated
// source. `#load`ed into each module, so it adds no public names.

#scope_module

// ---- renaming ----

Rename_Rule :: enum u8 {
    NONE;
    LOWERCASE;
    UPPERCASE;
    CAMEL;
    PASCAL;
    SNAKE;
    SCREAMING_SNAKE;
    KEBAB;
    SCREAMING_KEBAB;
}

parse_rule :: (text: string) -> Rename_Rule, bool {
    if text == "lowercase" return .LOWERCASE, true;
    if text == "UPPERCASE" return .UPPERCASE, true;
    if text == "camelCase" return .CAMEL, true;
    if text == "PascalCase" return .PASCAL, true;
    if text == "snake_case" return .SNAKE, true;
    if text == "SCREAMING_SNAKE_CASE" return .SCREAMING_SNAKE, true;
    if text == "kebab-case" return .KEBAB, true;
    if text == "SCREAMING-KEBAB-CASE" return .SCREAMING_KEBAB, true;

    return .NONE, false;
}

is_lower_byte :: (c: u8) -> bool {
    return c >= #char "a" && c <= #char "z";
}

is_upper_byte :: (c: u8) -> bool {
    return c >= #char "A" && c <= #char "Z";
}

lower_byte :: (c: u8) -> u8 {
    return ifx is_upper_byte(c) then c + 32 else c;
}

upper_byte :: (c: u8) -> u8 {
    return ifx is_lower_byte(c) then c - 32 else c;
}

// Splits an identifier into words: at `_` and `-`, at a lowercase letter followed by an
// uppercase one, and before the last capital of an acronym (`HTTPServer` is `HTTP`, `Server`).
split_words :: (name: string) -> [..] string {
    words: [..] string;
    start := -1;

    for i: 0..name.count {
        c: u8 = 0;
        if i < name.count c = name[i];

        separator := i == name.count || c == #char "_" || c == #char "-";
        boundary := separator;

        if !boundary && start >= 0 && i > 0 {
            before := name[i - 1];

            if is_upper_byte(c) && (is_lower_byte(before) || (before >= #char "0" && before <= #char "9")) boundary = true;
            else if is_upper_byte(c) && is_upper_byte(before) && i + 1 < name.count && is_lower_byte(name[i + 1]) boundary = true;
        }

        if boundary && start >= 0 {
            array_add(*words, slice(name, start, i - start));
            start = -1;
        }

        if !separator && start < 0 start = i;
    }

    return words;
}

apply_rule :: (name: string, rule: Rename_Rule) -> string {
    if rule == .NONE return name;

    builder: String_Builder;

    if rule == .LOWERCASE || rule == .UPPERCASE {
        for name append(*builder, ifx rule == .LOWERCASE then lower_byte(it) else upper_byte(it));

        return builder_to_string(*builder,, temp);
    }

    for word, word_index: split_words(name) {
        if word_index > 0 {
            if rule == .SNAKE || rule == .SCREAMING_SNAKE append(*builder, #char "_");
            else if rule == .KEBAB || rule == .SCREAMING_KEBAB append(*builder, #char "-");
        }

        for c, i: word {
            upper := false;

            if rule == .SCREAMING_SNAKE || rule == .SCREAMING_KEBAB upper = true;
            else if i == 0 && (rule == .PASCAL || (rule == .CAMEL && word_index > 0)) upper = true;

            append(*builder, ifx upper then upper_byte(c) else lower_byte(c));
        }
    }

    return builder_to_string(*builder,, temp);
}

// Splits `a, b=c, d="x, y"` at the commas outside quotes.
split_note_option :: (text: string) -> bool, string, string {
    quoted := false;
    escaped := false;

    for text {
        if escaped {
            escaped = false;
        } else if quoted && it == #char "\\" {
            escaped = true;
        } else if it == #char "\"" {
            quoted = !quoted;
        } else if !quoted && it == #char "," {
            return true, slice(text, 0, it_index), slice(text, it_index + 1, text.count - it_index - 1);
        }
    }

    return false, text, "";
}

// ---- quoting ----

// A quoted note value without its quotes, with `\\`, `\"`, `\n`, `\r`, `\t` and `\uXXXX` (a
// code point that is not a surrogate) resolved; text that is not quoted comes back as it is.
// False for any other escape or a trailing backslash.
//
//     unquote("\"a\\\"b\"")  // a"b, true
//     unquote("\"\\u00e9\"")   // \xc3\xa9, true
unquote :: (text: string) -> string, bool {
    if text.count < 2 || text[0] != #char "\"" || text[text.count - 1] != #char "\"" return text, true;

    builder: String_Builder;
    escaped := false;
    // Hex digits of a `\u` escape still to read, and what they spell so far.
    digits_left := 0;
    code_point: u32;

    for slice(text, 1, text.count - 2) {
        if digits_left {
            digit := hex_digit_value(it);

            if digit < 0 return "", false;

            code_point = code_point * 16 + cast(u32) digit;
            digits_left -= 1;

            if !digits_left {
                if code_point >= 0xd800 && code_point <= 0xdfff return "", false;

                append_code_point(*builder, code_point);
            }
        } else if escaped {
            if it == {
                case #char "\\"; #through;
                case #char "\""; append(*builder, it);
                case #char "n"; append(*builder, #char "\n");
                case #char "r"; append(*builder, #char "\r");
                case #char "t"; append(*builder, #char "\t");
                case #char "u";
                    digits_left = 4;
                    code_point = 0;
                case; return "", false;
            }

            escaped = false;
        } else if it == #char "\\" {
            escaped = true;
        } else {
            append(*builder, it);
        }
    }

    if escaped || digits_left return "", false;

    return builder_to_string(*builder,, temp), true;
}

hex_digit_value :: (c: u8) -> s32 {
    if c >= #char "0" && c <= #char "9" return c - #char "0";
    if c >= #char "a" && c <= #char "f" return c - #char "a" + 10;
    if c >= #char "A" && c <= #char "F" return c - #char "A" + 10;

    return -1;
}

// `code_point` (at most 0xffff) as UTF-8.
append_code_point :: (builder: *String_Builder, code_point: u32) {
    if code_point < 0x80 {
        append(builder, cast(u8) code_point);
    } else if code_point < 0x800 {
        append(builder, cast(u8) (0xc0 | (code_point >> 6)));
        append(builder, cast(u8) (0x80 | (code_point & 0x3f)));
    } else {
        append(builder, cast(u8) (0xe0 | (code_point >> 12)));
        append(builder, cast(u8) (0x80 | ((code_point >> 6) & 0x3f)));
        append(builder, cast(u8) (0x80 | (code_point & 0x3f)));
    }
}

// `text` as a Jai string literal, for generated source.
//
//     jai_quote("say \"hi\"\n")  // "say \"hi\"\n" with the quotes and newline escaped
jai_quote :: (text: string) -> string {
    builder: String_Builder;

    append(*builder, "\"");

    for text {
        if it == #char "\"" append(*builder, "\\\"");
        else if it == #char "\\" append(*builder, "\\\\");
        else if it == #char "\n" append(*builder, "\\n");
        else if it == #char "\r" append(*builder, "\\r");
        else if it == #char "\t" append(*builder, "\\t");
        else append(*builder, it);
    }

    append(*builder, "\"");

    return builder_to_string(*builder,, temp);
}
