// The formatter: re-emits the token stream with normalized indentation and spacing.
//
// Line structure is kept: the formatter never splits a line and only joins two in one case
// (brace_style = "same_line": a `{` alone on its line after a header, and `else` after `}`).
// Spacing between two tokens is either normalized (one space, or none) or copied from the
// source, and runs of several spaces between tokens are kept as authored alignment.
// String literals, here-strings, `#asm` bodies and `// jaifmt: off` regions are copied verbatim.
// After formatting, the result is lexed again and must produce the same tokens (see
// `check_equivalent`); otherwise the caller refuses to use it.

Format_Result :: struct {
    output: string;
    ok: bool;
    error: string;
    long_lines: int;   // lines over config.max_width in the output
}

format_source :: (src: string, config: Config) -> Format_Result {
    result: Format_Result;
    lexed := lex(src);
    defer array_free(lexed.tokens);
    if lexed.error.count {
        line, column := line_and_column(src, lexed.error_offset);
        result.error = tprint("%:%: %", line, column, lexed.error);
        return result;
    }

    f: Formatter;
    f.src = src;
    f.tokens = lexed.tokens;
    f.config = config;
    f.case_indent = resolved_case_indent(config);
    f.case_body_indent = resolved_case_body_indent(config);
    f.newline = ifx uses_crlf(src) then "\r\n" else "\n";
    init_string_builder(*f.out, src.count + src.count / 8 + 64);
    defer free_buffers(*f.out);

    ok, message := run(*f);
    if !ok {
        result.error = message;
        return result;
    }
    result.output = builder_to_string(*f.out);

    equivalent, why := check_equivalent(src, result.output);
    if !equivalent {
        result.error = tprint("internal error: formatting would change the token stream (%); file left unchanged", why);
        free(result.output);
        result.output = "";
        return result;
    }
    result.long_lines = count_long_lines(result.output, config.max_width);
    result.ok = true;
    return result;
}

// Lexes both texts and compares them: identical tokens (kind and text) in the same order, a
// line break before the same tokens (the parser is sensitive to some), and the same comments
// up to whitespace. The only allowed line-break difference is the one brace joining makes:
// before a `{` or an `else`.
check_equivalent :: (before: string, after: string) -> bool, string {
    a := lex(before);
    defer array_free(a.tokens);
    b := lex(after);
    defer array_free(b.tokens);
    if a.error.count return false, tprint("input does not lex: %", a.error);
    if b.error.count {
        line, column := line_and_column(after, b.error_offset);
        return false, tprint("output does not lex at %:%: %", line, column, b.error);
    }

    zone := directive_zone(a.tokens, before);
    defer array_free(zone);

    ia := 0;
    ib := 0;
    end_a: s64 = 0;
    end_b: s64 = 0;
    first := true;
    prev_a := -1;
    prev_b := -1;
    while true {
        // Comments, in order, must survive (whitespace inside them may change).
        while a.tokens[ia].kind == .LINE_COMMENT || a.tokens[ia].kind == .BLOCK_COMMENT {
            ta := a.tokens[ia];
            tb := b.tokens[ib];
            if tb.kind != ta.kind || !same_ignoring_whitespace(token_text(before, ta), token_text(after, tb)) {
                line, column := line_and_column(before, ta.start);
                return false, tprint("comment at %:% changed", line, column);
            }
            ia += 1;
            ib += 1;
        }
        if b.tokens[ib].kind == .LINE_COMMENT || b.tokens[ib].kind == .BLOCK_COMMENT {
            line, column := line_and_column(after, b.tokens[ib].start);
            return false, tprint("unexpected comment at output %:%", line, column);
        }
        ta := a.tokens[ia];
        tb := b.tokens[ib];
        text_a := token_text(before, ta);
        text_b := token_text(after, tb);
        if ta.kind != tb.kind || text_a != text_b {
            line, column := line_and_column(before, ta.start);
            return false, tprint("token `%` at %:% became `%`", text_a, line, column, text_b);
        }
        if ta.kind == .EOF break;
        // Like the compiler's lexer, the first token always counts as starting a line.
        break_a := first || contains_newline(before, end_a, ta.start);
        break_b := first || contains_newline(after, end_b, tb.start);
        first = false;
        if break_a != break_b && !(break_a && (text_a == "{" || text_a == "else")) {
            line, column := line_and_column(before, ta.start);
            return false, tprint("line break before `%` at %:% changed", text_a, line, column);
        }
        if prev_a >= 0 && adjacency_matters(zone, a.tokens, before, prev_a, ia) {
            tight_a := prev_a == ia - 1 && ta.ws_start == ta.start;
            tight_b := prev_b == ib - 1 && tb.ws_start == tb.start;
            if tight_a != tight_b {
                line, column := line_and_column(before, ta.start);
                return false, tprint("spacing before `%` at %:% (after a directive) changed", text_a, line, column);
            }
        }
        prev_a = ia;
        prev_b = ib;
        end_a = ta.end;
        end_b = tb.end;
        ia += 1;
        ib += 1;
    }
    return true, "";
}

// Tokens after which the parser looks at exact adjacency: a directive and the flags written
// against it (`#library,system`, `#type, distinct`, `#insert,scope(x)`, `#location(x)`). The
// formatter copies the whitespace after these tokens unchanged.
directive_zone :: (tokens: [] Tok, src: string) -> [..] bool {
    zone: [..] bool;
    array_resize(*zone, tokens.count);
    for i: 1..tokens.count - 1 {
        t := tokens[i];
        if t.kind == .DIRECTIVE {
            zone[i] = true;
        } else if zone[i - 1] {
            text := token_text(src, t);
            previous := tokens[i - 1];
            if t.kind == .PUNCT && (text == "," || text == "(") && t.ws_start == t.start {
                zone[i] = true;
            } else if t.kind == .IDENT && previous.kind == .PUNCT && token_text(src, previous) == "," {
                zone[i] = true;
            }
        }
    }
    if tokens.count && tokens[0].kind == .DIRECTIVE  zone[0] = true;
    return zone;
}

// Within a directive's flags, whether `previous` and `current` touch is meaningful: before a
// `,` or `(`, and after a flag `,`. (`#c_call ;` may still become `#c_call;`.)
adjacency_matters :: (zone: [] bool, tokens: [] Tok, src: string, previous: int, current: int) -> bool {
    if !zone[previous] return false;
    if tokens[current].kind == .PUNCT {
        text := token_text(src, tokens[current]);
        if text == "," || text == "(" return true;
    }
    return tokens[previous].kind == .PUNCT && token_text(src, tokens[previous]) == ",";
}

line_and_column :: (src: string, offset: s64) -> line: int, column: int {
    line := 1;
    column := 1;
    for i: 0..min(offset, src.count) - 1 {
        if src[i] == #char "\n" {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    return line, column;
}

token_text :: inline (src: string, tok: Tok) -> string {
    return slice(src, tok.start, tok.end - tok.start);
}

#scope_file

Frame :: struct {
    open: string;          // "{", "(", "[", ".{", ".["; "" for the file itself
    block: bool;           // a `{` that ends its line: its lines are indented one level
    switch_block: bool;    // `if x == {`: `case` lines and their statements
    seen_case: bool;
    is_cast: bool;         // `cast(T)`: what follows the `)` is an operand
    is_index: bool;        // `a[i]` rather than an array type `[N] T`
    line_new: int;         // output / source indentation of the line holding the opener
    line_orig: int;
    base: int;             // output indentation of the closing `}` of a block
    content: int;          // output indentation of a statement inside a block
    stmt_new: int;         // the current statement's first line, output / source indentation
    stmt_orig: int;
}

Context_Kind :: enum u8 { OPERATOR; OPERAND; UNKNOWN; }

Formatter :: struct {
    src: string;
    tokens: [] Tok;
    config: Config;
    case_indent: int;
    case_body_indent: int;
    newline: string;
    out: String_Builder;

    frames: [..] Frame;
    verbatim: [..] bool;
    zone: [..] bool;                 // see directive_zone
    last_on_line: [..] bool;
    operand_end: [..] Context_Kind;   // what an operator after token i would follow
    prev_sig := -1;                  // previous non-comment token
    line_new: int;                   // indentation of the output line being written
    line_orig: int;
    emitted_any := false;
    operator_decl := false;          // between `operator` and `::`: spacing copied verbatim
}

KEYWORDS_BEFORE_OPERAND :: string.[
    "if", "ifx", "then", "else", "case", "return", "while", "for", "xx", "cast", "using", "defer",
    "inline", "no_inline", "push_context", "struct", "union", "enum", "enum_flags", "operator",
    "remove", "break", "continue", "interface", "size_of", "type_of", "type_info", "initializer_of",
    "is_constant", "code_of", "auto_cast",
];

// Assignment, declaration, comparison and logic operators always get a space on both sides.
SPACED_OPERATORS :: string.[
    "=", ":=", "::", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "%=", "&=", "|=",
    "^=", "<<=", ">>=", "<<<=", ">>>=", "&&=", "||=", "->", "=>",
];

// Arithmetic and bit operators that are always binary. Written tight on both sides (`n-1`,
// `a*b + c`) they stay tight, since that spacing often shows precedence; otherwise they get one
// space on each side.
ARITHMETIC_OPERATORS :: string.["/", "%", "|", "^", ">>", "<<<", ">>>"];

// Binary after an operand, prefix otherwise (or, for `<` and `>`, a `for` flag).
AMBIGUOUS_OPERATORS :: string.["-", "+", "*", "&", "<<", "<", ">"];

// Prefix operators whose operand is written attached: `-x`, `*T`, `<<p`, `!ok`.
PREFIX_OPERATORS :: string.["-", "+", "*", "<<", "!"];

KEYWORDS_BEFORE_PAREN :: string.["if", "ifx", "while", "for", "return", "then", "else", "case", "defer"];

text_of :: inline (f: *Formatter, i: int) -> string {
    t := f.tokens[i];
    return slice(f.src, t.start, t.end - t.start);
}

is_punct :: inline (f: *Formatter, i: int, text: string) -> bool {
    return i >= 0 && f.tokens[i].kind == .PUNCT && text_of(f, i) == text;
}

is_ident :: inline (f: *Formatter, i: int, text: string) -> bool {
    return i >= 0 && f.tokens[i].kind == .IDENT && text_of(f, i) == text;
}

is_comment :: inline (f: *Formatter, i: int) -> bool {
    k := f.tokens[i].kind;
    return k == .LINE_COMMENT || k == .BLOCK_COMMENT;
}

one_of :: (text: string, list: [] string) -> bool {
    for list if it == text return true;
    return false;
}

directive_name :: (f: *Formatter, i: int) -> string {
    text := text_of(f, i);
    at := 1;
    while at < text.count && (text[at] == #char " " || text[at] == #char "\t")  at += 1;
    return slice(text, at, text.count - at);
}

uses_crlf :: (src: string) -> bool {
    crlf := 0;
    lf := 0;
    for i: 0..src.count - 1 {
        if src[i] == #char "\n" {
            lf += 1;
            if i > 0 && src[i - 1] == #char "\r"  crlf += 1;
        }
    }
    return lf > 0 && crlf * 2 > lf;
}

contains_newline :: (src: string, from: s64, to: s64) -> bool {
    for i: from..to - 1  if src[i] == #char "\n" return true;
    return false;
}

same_ignoring_whitespace :: (a: string, b: string) -> bool {
    i := 0;
    j := 0;
    while true {
        while i < a.count && is_blank(a[i])  i += 1;
        while j < b.count && is_blank(b[j])  j += 1;
        if i >= a.count || j >= b.count return i >= a.count && j >= b.count;
        if a[i] != b[j] return false;
        i += 1;
        j += 1;
    }
    return false;
}

is_blank :: inline (c: u8) -> bool {
    return c == #char " " || c == #char "\t" || c == #char "\n" || c == #char "\r";
}

count_long_lines :: (text: string, max_width: int) -> int {
    count := 0;
    width := 0;
    for i: 0..text.count - 1 {
        c := text[i];
        if c == #char "\n" {
            if width > max_width  count += 1;
            width = 0;
        } else if c != #char "\r" && (c & 0xC0) != 0x80 {
            width += 1;   // counts code points, not bytes
        }
    }
    if width > max_width  count += 1;
    return count;
}

comment_marker :: (f: *Formatter, i: int, word: string) -> bool {
    if f.tokens[i].kind != .LINE_COMMENT return false;
    text := trim(slice(text_of(f, i), 2, text_of(f, i).count - 2));
    if !begins_with(text, "jaifmt:") return false;
    return trim(slice(text, 7, text.count - 7)) == word;
}

// Marks tokens copied byte for byte: `// jaifmt: off` ... `// jaifmt: on` and `#asm` bodies.
mark_verbatim :: (f: *Formatter) {
    n := f.tokens.count;
    array_resize(*f.verbatim, n);
    i := 0;
    while i < n {
        if comment_marker(f, i, "off") {
            j := i + 1;
            while j < n && f.tokens[j].kind != .EOF {
                f.verbatim[j] = true;
                if comment_marker(f, j, "on") break;
                j += 1;
            }
            i = j + 1;
            continue;
        }
        if f.tokens[i].kind == .DIRECTIVE && directive_name(f, i) == "asm" {
            j := i + 1;
            while j < n && f.tokens[j].kind != .EOF && !is_punct(f, j, "{") && !is_punct(f, j, ";")  j += 1;
            if is_punct(f, j, "{") {
                depth := 0;
                k := j;
                while k < n && f.tokens[k].kind != .EOF {
                    if is_punct(f, k, "{") || is_punct(f, k, ".{")  depth += 1;
                    if is_punct(f, k, "}") {
                        depth -= 1;
                        if depth == 0 break;
                    }
                    if k > j  f.verbatim[k] = true;
                    k += 1;
                }
                i = k + 1;
                continue;
            }
        }
        i += 1;
    }
}

// Is token i the last code on its line (only comments may follow it)?
mark_line_ends :: (f: *Formatter) {
    n := f.tokens.count;
    array_resize(*f.last_on_line, n);
    for i: 0..n - 2 {
        j := i + 1;
        while f.tokens[j].kind == .BLOCK_COMMENT && f.tokens[j].newlines == 0
            && !contains_newline(f.src, f.tokens[j].start, f.tokens[j].end)  j += 1;
        t := f.tokens[j];
        f.last_on_line[i] = t.newlines > 0 || t.kind == .EOF || t.kind == .LINE_COMMENT
            || (t.kind == .BLOCK_COMMENT && f.tokens[j].newlines == 0);
    }
}

run :: (f: *Formatter) -> bool, string {
    mark_verbatim(f);
    mark_line_ends(f);
    array_resize(*f.operand_end, f.tokens.count);
    f.zone = directive_zone(f.tokens, f.src);

    root: Frame;
    root.block = true;
    array_add(*f.frames, root);
    defer array_free(f.frames);
    defer array_free(f.verbatim);
    defer array_free(f.last_on_line);
    defer array_free(f.operand_end);
    defer array_free(f.zone);

    // A byte order mark stays in front.
    if f.src.count >= 3 && f.src[0] == 0xEF && f.src[1] == 0xBB && f.src[2] == 0xBF  append(*f.out, slice(f.src, 0, 3));

    for i: 0..f.tokens.count - 1 {
        tok := f.tokens[i];
        if tok.kind == .EOF break;

        if f.verbatim[i] {
            append(*f.out, slice(f.src, tok.ws_start, tok.end - tok.ws_start));
            if tok.newlines > 0 {
                f.line_new = tok.column;
                f.line_orig = tok.column;
            }
        } else if tok.newlines > 0 || !f.emitted_any {
            if joins_previous_line(f, i) {
                append(*f.out, " ");
                append(*f.out, text_of(f, i));
            } else {
                ok, message := start_line(f, i);
                if !ok return false, message;
            }
        } else {
            append(*f.out, spacing(f, i));
            append(*f.out, text_of(f, i));
        }
        f.emitted_any = true;

        if !is_comment(f, i) {
            ok, message := track(f, i);
            if !ok return false, message;
            f.prev_sig = i;
        }
    }

    if f.frames.count > 1 {
        top := f.frames[f.frames.count - 1];
        return false, tprint("unclosed `%`", top.open);
    }
    if f.emitted_any  append(*f.out, f.newline);
    return true, "";
}

// brace_style = "same_line": `if x\n{` -> `if x {`, and `}\nelse` -> `} else`.
joins_previous_line :: (f: *Formatter, i: int) -> bool {
    if f.config.brace_style != .SAME_LINE return false;
    tok := f.tokens[i];
    if tok.newlines != 1 || i == 0 || f.prev_sig != i - 1 || f.verbatim[i - 1] return false;
    p := i - 1;
    pk := f.tokens[p].kind;
    if is_punct(f, i, "{") {
        if pk == .IDENT || pk == .NUMBER return true;
        if is_punct(f, p, ")") || is_punct(f, p, "]") return true;
        if pk == .DIRECTIVE {
            name := directive_name(f, p);
            return !begins_with(name, "scope_") && name != "through" && name != "load" && name != "import";
        }
        return false;
    }
    if is_ident(f, i, "else") && is_punct(f, p, "}") {
        // Only after a `}` alone on its line: a one-line `if x { ... }` keeps its `else` below.
        return f.tokens[p].newlines > 0;
    }
    return false;
}

// Writes the line break(s) and indentation before token i, which starts a line.
start_line :: (f: *Formatter, i: int) -> bool, string {
    tok := f.tokens[i];
    if f.emitted_any {
        blank := min(cast(int) tok.newlines - 1, f.config.max_blank_lines);
        for 0..blank  append(*f.out, f.newline);
    }

    indent, ok, message := line_indent(f, i);
    if !ok return false, message;
    if indent < 0  indent = 0;
    for 1..indent  append(*f.out, " ");

    if tok.kind == .BLOCK_COMMENT {
        append_block_comment(f, i, indent - tok.column);
    } else {
        append(*f.out, text_of(f, i));
    }
    f.line_new = indent;
    f.line_orig = tok.column;
    return true, "";
}

// Shifts the continuation lines of a block comment along with its first line.
append_block_comment :: (f: *Formatter, i: int, delta: int) {
    text := text_of(f, i);
    lines := split(text, "\n");
    if delta < 0 {
        // Only dedent when every continuation line has the room.
        for 1..lines.count - 1 {
            line := lines[it];
            if !trim(line).count continue;
            spaces := 0;
            while spaces < line.count && line[spaces] == #char " "  spaces += 1;
            if spaces < -delta { delta = 0; break; }
        }
    }
    for line, index: lines {
        if index > 0 {
            append(*f.out, f.newline);
            line = trim_right(line);
            if line.count {
                if delta > 0 {
                    for 1..delta  append(*f.out, " ");
                } else if delta < 0 {
                    line = slice(line, -delta, line.count + delta);
                }
            }
        }
        append(*f.out, line);
    }
}

relative :: inline (base_new: int, base_orig: int, column: int) -> int {
    return max(0, base_new + column - base_orig);
}

line_indent :: (f: *Formatter, i: int) -> indent: int, ok: bool, message: string {
    tok := f.tokens[i];
    top := *f.frames[f.frames.count - 1];

    if tok.kind == .PUNCT {
        text := text_of(f, i);
        if text == "}" || text == ")" || text == "]" {
            if f.frames.count <= 1 || !closes(top.open, text) {
                line, column := line_and_column(f.src, tok.start);
                return 0, false, tprint("%:%: unbalanced `%`", line, column, text);
            }
            if top.block return top.base, true, "";
            return relative(top.line_new, top.line_orig, tok.column), true, "";
        }
    }
    if !top.block return relative(top.line_new, top.line_orig, tok.column), true, "";

    if !starts_statement(f) return relative(top.stmt_new, top.stmt_orig, tok.column), true, "";

    // The code this line is about: itself, or what a comment line precedes.
    code := i;
    while is_comment(f, code)  code += 1;
    indent := top.content;
    if top.switch_block {
        if is_ident(f, code, "case") {
            indent = top.base + f.case_indent;
        } else if top.seen_case || (code != i && is_punct(f, code, "}")) {
            indent = top.base + f.case_indent + ifx top.seen_case then f.case_body_indent else 0;
        }
    }
    if code == i {
        top.stmt_new = indent;
        top.stmt_orig = tok.column;
    }
    return indent, true, "";
}

closes :: (open: string, close: string) -> bool {
    if close == "}" return open == "{" || open == ".{";
    if close == ")" return open == "(";
    if close == "]" return open == "[" || open == ".[";
    return false;
}

// Does the next code token begin a statement (rather than continue the previous line's)?
starts_statement :: (f: *Formatter) -> bool {
    p := f.prev_sig;
    if p < 0 return true;
    t := f.tokens[p];
    if t.kind == .PUNCT {
        text := text_of(f, p);
        return text == ";" || text == "{" || text == "}" || text == ".{";
    }
    if t.kind == .DIRECTIVE {
        name := directive_name(f, p);
        return name == "scope_file" || name == "scope_export" || name == "scope_module";
    }
    if t.kind == .NOTE {
        // `} @Note` / `x: int; @Note` end a declaration.
        q := p;
        while q >= 0 && (f.tokens[q].kind == .NOTE || is_comment(f, q))  q -= 1;
        return q < 0 || is_punct(f, q, "}") || is_punct(f, q, ";");
    }
    return false;
}

// Updates brackets, statement state and the operand classification after emitting token i.
track :: (f: *Formatter, i: int) -> bool, string {
    tok := f.tokens[i];
    f.operand_end[i] = classify(f, i);

    if tok.kind == .IDENT {
        text := text_of(f, i);
        if text == "operator" f.operator_decl = true;
        if text == "case" {
            top := *f.frames[f.frames.count - 1];
            if top.switch_block  top.seen_case = true;
        }
        return true, "";
    }
    if tok.kind != .PUNCT return true, "";
    text := text_of(f, i);
    if text == "::" f.operator_decl = false;

    if text == "{" || text == "(" || text == "[" || text == ".{" || text == ".[" {
        top := *f.frames[f.frames.count - 1];
        frame: Frame;
        frame.open = text;
        frame.line_new = f.line_new;
        frame.line_orig = f.line_orig;
        if (text == "{" || text == ".{") && f.last_on_line[i] {
            frame.block = true;
            // A header continued over several lines still indents its body by one level.
            frame.base = ifx top.block then top.stmt_new else f.line_new;
            frame.switch_block = text == "{" && opens_switch(f, i);
            frame.content = frame.base + ifx frame.switch_block then f.case_indent else f.config.indent_width;
            frame.stmt_new = frame.content;
            frame.stmt_orig = 0;
        }
        if text == "(" {
            frame.is_cast = is_cast_paren(f, i);
        }
        if text == "[" {
            frame.is_index = f.prev_sig >= 0 && f.operand_end[f.prev_sig] == .OPERAND && f.tokens[i].newlines == 0;
        }
        array_add(*f.frames, frame);
        return true, "";
    }
    if text == "}" || text == ")" || text == "]" {
        if f.frames.count <= 1 || !closes(f.frames[f.frames.count - 1].open, text) {
            line, column := line_and_column(f.src, tok.start);
            return false, tprint("%:%: unbalanced `%`", line, column, text);
        }
        frame := pop(*f.frames);
        if text == ")" f.operand_end[i] = ifx frame.is_cast then Context_Kind.OPERATOR else .OPERAND;
        if text == "]" f.operand_end[i] = ifx frame.is_index || frame.open == ".[" then Context_Kind.OPERAND else .OPERATOR;
    }
    return true, "";
}

opens_switch :: (f: *Formatter, i: int) -> bool {
    p := f.prev_sig;
    if p >= 0 && f.tokens[p].kind == .DIRECTIVE && directive_name(f, p) == "complete"  p = previous_code(f, p);
    return is_punct(f, p, "==");
}

previous_code :: (f: *Formatter, i: int) -> int {
    j := i - 1;
    while j >= 0 && is_comment(f, j)  j -= 1;
    return j;
}

// No whitespace between token i and the one before it.
is_tight :: inline (f: *Formatter, i: int) -> bool {
    return f.tokens[i].ws_start == f.tokens[i].start;
}

// `cast(T)` and `cast,no_check(T)`: the parenthesized type is not an operand.
is_cast_paren :: (f: *Formatter, i: int) -> bool {
    p := f.prev_sig;
    while p >= 2 && f.tokens[p].kind == .IDENT && is_punct(f, p - 1, ",") && is_tight(f, p) && is_tight(f, p - 1)  p -= 2;
    return is_ident(f, p, "cast");
}

classify :: (f: *Formatter, i: int) -> Context_Kind {
    t := f.tokens[i];
    if t.kind == {
        case .IDENT;
            if one_of(text_of(f, i), KEYWORDS_BEFORE_OPERAND) return .OPERATOR;
            return .OPERAND;
        case .NUMBER; #through;
        case .STRING; #through;
        case .HERE_STRING;
            return .OPERAND;
        case .PUNCT;
            text := text_of(f, i);
            if text == ".*" || text == "---" || text == ")" || text == "]" return .OPERAND;   // `)`/`]` refined by `track`
            if text == "}" return .UNKNOWN;
            return .OPERATOR;
    }
    return .UNKNOWN;
}

context_before :: (f: *Formatter, i: int) -> Context_Kind {
    p := previous_code(f, i);
    if p < 0 return .OPERATOR;
    return f.operand_end[p];
}

// A comma that attaches a cast flag: `cast,no_check`, `xx,force`, `using,except(x)`. (Directive
// flags never get here: spacing after a directive is copied, see `directive_zone`.)
is_flag_comma :: (f: *Formatter, comma: int) -> bool {
    next := comma + 1;
    if f.tokens[next].kind != .IDENT || f.tokens[next].newlines > 0 return false;
    if f.tokens[next].ws_start != f.tokens[next].start || f.tokens[comma].ws_start != f.tokens[comma].start return false;
    p := comma - 1;
    while p >= 2 && f.tokens[p].kind == .IDENT && is_punct(f, p - 1, ",")
        && f.tokens[p].ws_start == f.tokens[p].start && f.tokens[p - 1].ws_start == f.tokens[p - 1].start  p -= 2;
    if p < 0 return false;
    return f.tokens[p].kind == .IDENT && one_of(text_of(f, p), string.["cast", "xx", "using"]);
}

// `x: int`: the declared names sit against the colon.
colon_attaches :: (f: *Formatter, colon: int) -> bool {
    p := colon - 1;
    if f.tokens[p].kind != .IDENT return false;
    if f.tokens[p].newlines > 0 return true;
    q := p - 1;
    if q < 0 return true;
    if f.tokens[q].kind == .IDENT  return one_of(text_of(f, q), string.["for", "using"]);
    if f.tokens[q].kind != .PUNCT return false;
    return one_of(text_of(f, q), string.[",", "(", "{", ";", "}", "$", "$$"]);
}

Spacing :: enum u8 { KEEP; NONE; SPACE; }

// The whitespace to write between token i and the token before it on the same line.
spacing :: (f: *Formatter, i: int) -> string {
    tok := f.tokens[i];
    original := slice(f.src, tok.ws_start, tok.start - tok.ws_start);
    rule := spacing_rule(f, i);
    if rule == {
        case .NONE;
            return "";
        case .SPACE;
            // Runs of spaces are kept as alignment.
            if original.count >= 2 return original;
            return " ";
    }
    return original;
}

spacing_rule :: (f: *Formatter, i: int) -> Spacing {
    p := i - 1;
    pt := f.tokens[p];
    nt := f.tokens[i];
    if nt.kind == .LINE_COMMENT return .SPACE;
    if nt.kind == .BLOCK_COMMENT || pt.kind == .BLOCK_COMMENT return .KEEP;
    if f.operator_decl || adjacency_matters(f.zone, f.tokens, f.src, p, i) return .KEEP;

    ptext := ifx pt.kind == .PUNCT then text_of(f, p) else "";
    ntext := ifx nt.kind == .PUNCT then text_of(f, i) else "";

    if ntext == "," || ntext == ";" {
        if ptext == "," || ptext == ";" return .KEEP;
        return .NONE;
    }
    if ptext == "," {
        if ntext == ")" || ntext == "]" || ntext == "}" return .KEEP;
        if is_flag_comma(f, p) return .NONE;
        return .SPACE;
    }
    if ptext == "{" || ptext == ".{" || ntext == "}" return .KEEP;
    if ptext == ";" return .SPACE;
    if ntext == ")" || ntext == "]" return .NONE;
    if ptext == "(" || ptext == "[" || ptext == ".[" return .NONE;
    if ntext == "{" return .SPACE;

    if ntext == ":" {
        if colon_attaches(f, i) return .NONE;
        return .KEEP;
    }
    if ptext == ":" {
        if ntext == "=" || ntext == ":" return .KEEP;
        return .SPACE;
    }

    if ntext.count && one_of(ntext, SPACED_OPERATORS) return .SPACE;
    if ptext.count && one_of(ptext, SPACED_OPERATORS) return .SPACE;
    if ntext.count && one_of(ntext, ARITHMETIC_OPERATORS) return binary_spacing(f, i);
    if ptext.count && one_of(ptext, ARITHMETIC_OPERATORS) return binary_spacing(f, p);

    if ntext.count && one_of(ntext, AMBIGUOUS_OPERATORS) {
        if context_before(f, i) == .OPERAND return binary_spacing(f, i);
        return .KEEP;
    }
    if ptext.count && one_of(ptext, AMBIGUOUS_OPERATORS) {
        side := context_before(f, p);
        if side == .OPERAND return binary_spacing(f, p);
        if side == .OPERATOR && one_of(ptext, PREFIX_OPERATORS) && attaches_to_prefix(f, i) return .NONE;
        return .KEEP;
    }
    if ptext == "!" && attaches_to_prefix(f, i) return .NONE;

    if pt.kind == .IDENT && ntext == "(" && one_of(text_of(f, p), KEYWORDS_BEFORE_PAREN) return .SPACE;
    return .KEEP;
}

// Binary operator at `op`: tight on both sides stays tight, anything else becomes spaced.
// Comparisons are always spaced.
binary_spacing :: (f: *Formatter, op: int) -> Spacing {
    text := text_of(f, op);
    if text == "<" || text == ">" return .SPACE;
    tight_before := f.tokens[op].ws_start == f.tokens[op].start;
    after := f.tokens[op + 1];
    tight_after := after.ws_start == after.start;
    if tight_before && tight_after return .KEEP;
    return .SPACE;
}

attaches_to_prefix :: (f: *Formatter, i: int) -> bool {
    t := f.tokens[i];
    return t.kind == .IDENT || t.kind == .NUMBER || is_punct(f, i, "(");
}
