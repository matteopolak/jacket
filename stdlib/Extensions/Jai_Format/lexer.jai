// Trivia-preserving Jai tokenizer for the formatter.
//
// It mirrors the token boundaries of jaic's lexer (crates/jaic/src/lexer.rs): the same
// punctuation table (longest match first), `#string` here-strings, `@` notes, `\` identifier
// separators and `.5` float literals. Unlike the compiler it keeps comments as tokens and
// records the whitespace before every token, which is all the formatter needs.

#scope_module

Tok_Kind :: enum u8 {
    IDENT;
    NUMBER;
    STRING; // "..." (may span lines)
    HERE_STRING; // #string TAG ... TAG, from the `#` to the end of the closing tag
    DIRECTIVE; // #name
    NOTE; // @name, @name(args), @"text"
    PUNCT;
    LINE_COMMENT;
    BLOCK_COMMENT; // /* ... */, nested
    EOF;
}

Tok :: struct {
    kind: Tok_Kind;
    start: s64; // byte range in the source
    end: s64;
    newlines: s32; // line breaks in the trivia before this token
    column: s32; // original column when the token starts a line (tabs count 4)
    ws_start: s64; // where the trivia before this token starts (end of the previous token)
}

Lex_Result :: struct {
    tokens: [..] Tok;
    error: string; // empty on success
    error_offset: s64;
}

// Longest first, exactly as in the compiler.
PUNCTUATION :: string.[
    "<<<=", ">>>=",
    "---", "&&=", "||=", "<<=", ">>=", "<<<", ">>>",
    "::", ":=", "->", "=>", "..", ".{", ".[", ".*", "<<", ">>", "<=", ">=", "==", "!=", "&&", "||",
    "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "$$",
    "{", "}", "[", "]", "(", ")", ";", ",", ":", ".", "$", "`", "+", "-", "*", "/", "%", "=", "!",
    "<", ">", "|", "&", "^", "~", "?",
];

is_ident_start :: inline (c: u8) -> bool {
    return (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || c == #char "_" || c >= 0x80;
}

is_ident_char :: inline (c: u8) -> bool {
    return is_ident_start(c) || is_digit(c);
}

is_digit :: inline (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
}

is_hex_digit :: inline (c: u8) -> bool {
    return is_digit(c) || (c >= #char "a" && c <= #char "f") || (c >= #char "A" && c <= #char "F");
}

lex :: (src: string) -> Lex_Result {
    result: Lex_Result;
    array_reserve(*result.tokens, src.count / 4 + 16);

    at: s64 = 0;
    // A UTF-8 byte order mark is not a token; the formatter copies it through.
    if src.count >= 3 && src[0] == 0xEF && src[1] == 0xBB && src[2] == 0xBF at = 3;

    fail :: (message: string, offset: s64) #expand {
        `result.error = message;
        `result.error_offset = offset;
        `return `result;
    }

    peek :: (n: s64) -> u8 #expand {
        if `at + n < `src.count return `src[`at + n];
        return 0;
    }

    while true {
        // Trivia: whitespace only. Comments are tokens.
        ws_start := at;
        newlines: s32 = 0;
        column: s32 = 0;
        while at < src.count {
            c := src[at];
            if c == #char "\n" {
                newlines += 1;
                column = 0;
                at += 1;
            } else if c == #char " " || c == #char "\r" || c == 0x0c || c == 0x0b {
                if c == #char " " column += 1;
                at += 1;
            } else if c == #char "\t" {
                column += 4;
                at += 1;
            } else if c == 0xc2 && peek(1) == 0xa0 {
                column += 1;
                at += 2;
            } else {
                break;
            }
        }

        tok: Tok;
        tok.start = at;
        tok.newlines = newlines;
        tok.column = column;
        tok.ws_start = ws_start;

        if at >= src.count {
            tok.kind = .EOF;
            tok.end = at;
            array_add(*result.tokens, tok);
            return result;
        }

        c := src[at];
        if c == #char "/" && peek(1) == #char "/" {
            while at < src.count && src[at] != #char "\n" at += 1;
            // A carriage return belongs to the line break, not the comment.
            end := at;
            while end > tok.start && (src[end - 1] == #char "\r" || src[end - 1] == #char " " || src[end - 1] == #char "\t") end -= 1;
            tok.kind = .LINE_COMMENT;
            tok.end = end;
            at = end;
        } else if c == #char "/" && peek(1) == #char "*" {
            at += 2;
            depth := 1;
            while depth > 0 {
                if at >= src.count fail("unterminated block comment", tok.start);
                if src[at] == #char "/" && peek(1) == #char "*" {
                    depth += 1;
                    at += 2;
                } else if src[at] == #char "*" && peek(1) == #char "/" {
                    depth -= 1;
                    at += 2;
                } else {
                    at += 1;
                }
            }
            tok.kind = .BLOCK_COMMENT;
            tok.end = at;
        } else if is_ident_start(c) {
            lex_ident(src, *at);
            // `group\_fraction`: a backslash (optionally followed by blanks) joins identifier parts.
            while at < src.count && src[at] == #char "\\" {
                skip := 1;
                while at + skip < src.count && (src[at + skip] == #char " " || src[at + skip] == #char "\t") skip += 1;
                if at + skip >= src.count || !is_ident_char(src[at + skip]) break;
                at += skip;
                lex_ident(src, *at);
            }
            tok.kind = .IDENT;
            tok.end = at;
        } else if is_digit(c) || at_leading_dot_float(src, at) {
            lex_number(src, *at);
            tok.kind = .NUMBER;
            tok.end = at;
        } else if c == #char "\"" {
            at += 1;
            while true {
                if at >= src.count fail("unterminated string", tok.start);
                d := src[at];
                if d == #char "\"" {
                    at += 1;
                    break;
                }
                if d == #char "\\" {
                    at += 2;
                    continue;
                }
                at += 1;
            }
            if at > src.count fail("unterminated string", tok.start);
            tok.kind = .STRING;
            tok.end = at;
        } else if c == #char "#" && directive_follows(src, at) {
            at += 1;
            while at < src.count && (src[at] == #char " " || src[at] == #char "\t") at += 1;
            name_start := at;
            lex_ident(src, *at);
            name := slice(src, name_start, at - name_start);
            if name == "string" {
                ok := lex_here_string(src, *at);
                if !ok fail("unterminated here-string", tok.start);
                tok.kind = .HERE_STRING;
            } else {
                tok.kind = .DIRECTIVE;
            }
            tok.end = at;
        } else if c == #char "@" {
            at += 1;
            if at < src.count && src[at] == #char "\"" {
                at += 1;
                while true {
                    if at >= src.count fail("unterminated note", tok.start);
                    d := src[at];
                    if d == #char "\"" {
                        at += 1;
                        break;
                    }
                    if d == #char "\\" {
                        at += 2;
                        continue;
                    }
                    at += 1;
                }
            } else {
                depth := 0;
                while at < src.count {
                    d := src[at];
                    if d == #char "(" {
                        depth += 1;
                    } else if d == #char ")" {
                        if depth == 0 break;
                        depth -= 1;
                    } else if depth == 0 && !(is_ident_char(d) || d == #char "." || d == #char "-" || d == #char "/" || d == #char ":" || d == #char "=") {
                        break;
                    }
                    at += 1;
                }
                // An open `(` swallows the rest of the file, trailing whitespace included, which
                // the formatter must trim: refuse it like an unbalanced bracket.
                if depth > 0 fail("unterminated note", tok.start);
            }
            tok.kind = .NOTE;
            tok.end = at;
        } else {
            matched := false;
            for p: PUNCTUATION {
                if at + p.count <= src.count && memcmp(src.data + at, p.data, p.count) == 0 {
                    at += p.count;
                    matched = true;
                    break;
                }
            }
            if !matched fail("unexpected character", at);
            tok.kind = .PUNCT;
            tok.end = at;
        }
        array_add(*result.tokens, tok);
    }
    return result;
}

#scope_file

lex_ident :: (src: string, at: *s64) {
    while at.* < src.count && is_ident_char(src[at.*]) at.* += 1;
}

directive_follows :: (src: string, at: s64) -> bool {
    skip := at + 1;
    while skip < src.count && (src[skip] == #char " " || src[skip] == #char "\t") skip += 1;
    return skip < src.count && is_ident_start(src[skip]);
}

// `.5` is a float literal unless the dot continues an expression (`x.5`, `1..5`).
at_leading_dot_float :: (src: string, at: s64) -> bool {
    if src[at] != #char "." || at + 1 >= src.count || !is_digit(src[at + 1]) return false;
    if at == 0 return true;
    prev := src[at - 1];
    return !(is_ident_char(prev) || prev == #char ")" || prev == #char "]" || prev == #char "." || prev == #char "\"");
}

lex_number :: (src: string, at: *s64) {
    i := at.*;
    peek :: (n: s64) -> u8 #expand {
        if `i + n < `src.count return `src[`i + n];
        return 0;
    }
    if src[i] == #char "0" {
        p := peek(1);
        if p == #char "x" || p == #char "X" || p == #char "b" || p == #char "B" || p == #char "h" || p == #char "H" {
            i += 2;
            while i < src.count && (is_hex_digit(src[i]) || src[i] == #char "_") i += 1;
            at.* = i;
            return;
        }
    }
    is_float := false;
    while i < src.count {
        c := src[i];
        if is_digit(c) || c == #char "_" {
            i += 1;
        } else if c == #char "." && !is_float && is_digit(peek(1)) {
            is_float = true;
            i += 1;
        } else if c == #char "." && !is_float && peek(1) != #char "." && !is_ident_start(peek(1)) {
            is_float = true;
            i += 1;
        } else if (c == #char "e" || c == #char "E") && (is_digit(peek(1)) || ((peek(1) == #char "-" || peek(1) == #char "+") && is_digit(peek(2)))) {
            is_float = true;
            i += 2;
        } else {
            break;
        }
    }
    at.* = i;
}

// After `#string`: optional `,cr` / `,\%` flags, the tag, then the body from the next line up
// to a line whose first non-blank text is the tag. Leaves `at` after the closing tag.
lex_here_string :: (src: string, at: *s64) -> bool {
    i := at.*;
    while true {
        while i < src.count && (src[i] == #char " " || src[i] == #char "\t") i += 1;
        if i >= src.count || src[i] != #char "," break;
        i += 1;
        while i < src.count && (src[i] == #char " " || src[i] == #char "\t") i += 1;
        if i < src.count && src[i] == #char "\\" {
            i += 2;
        } else {
            while i < src.count && is_ident_char(src[i]) i += 1;
        }
    }
    tag_start := i;
    while i < src.count && is_ident_char(src[i]) i += 1;
    tag := slice(src, tag_start, i - tag_start);
    if !tag.count return false;
    while i < src.count && src[i] != #char "\n" i += 1;
    i += 1;
    while i < src.count {
        line_end := i;
        while line_end < src.count && src[line_end] != #char "\n" line_end += 1;
        indent := i;
        while indent < line_end && (src[indent] == #char " " || src[indent] == #char "\t") indent += 1;
        rest := slice(src, indent, line_end - indent);
        if begins_with(rest, tag) && (rest.count == tag.count || !is_ident_char(rest[tag.count])) {
            at.* = indent + tag.count;
            return true;
        }
        i = line_end + 1;
    }
    return false;
}
