// The formatter: re-emits the token stream in one canonical layout.
//
// Two passes. `plan_lines` decides where lines break: the source's own line breaks, plus the
// canonical ones (one statement per line, block bodies on their own lines, a `{` or `else`
// joined to the line before it). The emit loop then writes each line with computed indentation,
// and exactly zero or one space between two tokens on a line (`spacing_rule`).
// String literals, here-strings, `#asm` bodies and `// jaifmt: off` regions are copied verbatim.
// After formatting, the result is lexed again and must produce the same tokens (see
// `check_equivalent`); otherwise no output is returned.

#scope_module

Format_Result :: struct {
    output: string;
    ok: bool;
    error: string;
    long_lines: int; // lines over config.max_width in the output
}

format_text :: (src: string, config: Format_Config) -> Format_Result {
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

// Lexes both texts and compares them: identical tokens (kind and text) in the same order, the
// same comments up to whitespace, the same adjacency inside directive flags, and a line break
// before the same tokens, except where the parser never looks at line breaks
// (`break_change_allowed`).
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
        if break_a != break_b && !break_change_allowed(before, a.tokens, prev_a, ia, break_a) {
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

// The parser looks at line breaks in a few places only: a line starting with `*`, `-`, `+` or
// `<<` right after a `}` begins a new statement, and a directive on a new line ends a
// declaration's flags. A break may therefore be added or removed after a block's `{`, after `;`
// (not before a note, which attaches to the declaration), before `}`, between `}` and a word
// (`} else`, a statement after a block) and after a note; and removed before a `{` (joining it
// to its header). `current` and `previous` are code tokens of `src`; `removed`: `src` has the break.
break_change_allowed :: (src: string, tokens: [] Tok, previous: int, current: int, removed: bool) -> bool {
    tok := tokens[current];
    if tok.kind == .PUNCT {
        text := token_text(src, tok);
        if text == "}" return true;
        if text == "{" && removed return true;
    }
    if previous < 0 return false;
    prev := tokens[previous];
    if prev.kind == .NOTE return true;
    if prev.kind != .PUNCT return false;
    text := token_text(src, prev);
    if text == "{" return true;
    if text == ";" return tok.kind != .NOTE;
    if text == "}" return tok.kind == .IDENT;
    return false;
}

// Tokens after which the parser looks at exact adjacency: a directive and the flags written
// against it (`#library,system`, `#type, distinct`, `#insert,scope(x)`, `#location(x)`). The
// formatter keeps whether these touch (see `Spacing.PRESENCE`).
directive_zone :: (tokens: [] Tok, src: string) -> [..] bool {
    zone: [..] bool;
    array_resize(*zone, tokens.count);
    // Set before the loop: the flags of a directive that starts the file are read from zone[0].
    if tokens.count && tokens[0].kind == .DIRECTIVE zone[0] = true;
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


Frame :: struct {
    open: string; // "{", "(", "[", ".{", ".["; "" for the file itself
    block: bool; // a `{` that ends its line: its lines are indented one level
    switch_block: bool; // `if x == {`: `case` lines and their statements
    seen_case: bool;
    is_cast: bool; // `cast(T)`: what follows the `)` is an operand
    is_index: bool; // `a[i]` rather than an array type `[N] T`
    colons: int; // `:` seen in the current declaration (the second one is spaced)
    line_new: int; // output indentation / source column of the line holding the opener
    line_orig: int;
    open_new: int; // output / source column of the opener itself
    open_orig: int;
    base: int; // output indentation of the closing `}` of a block
    content: int; // output indentation of a statement inside a block
    stmt_new: int; // the current statement's first line, output / source indentation
    stmt_orig: int;
}

Context_Kind :: enum u8 {
    OPERATOR;
    OPERAND;
    UNKNOWN;
}

Formatter :: struct {
    src: string;
    tokens: [] Tok;
    config: Format_Config;
    case_indent: int;
    case_body_indent: int;
    newline: string;
    out: String_Builder;

    frames: [..] Frame;
    verbatim: [..] bool;
    zone: [..] bool; // see directive_zone
    breaks: [..] s32; // line breaks to write before each token (plan_lines)
    columns: [..] s32; // each token's column in the source (tabs count 4)
    last_on_line: [..] bool; // in the output
    literal_brace: [..] bool; // a `{` that opens a struct literal written without the dot
    operand_end: [..] Context_Kind; // what an operator after token i would follow
    prev_sig := -1; // previous non-comment token
    line_new: int; // indentation of the output line being written
    line_orig: int; // source column of that line's first token
    column: int; // output column after the text written so far
    emitted_any := false;
    operator_decl := false; // between `operator` and `::`: spacing copied verbatim
}

KEYWORDS_BEFORE_OPERAND :: string.[
    "if", "ifx", "then", "else", "case", "return", "while", "for", "xx", "cast", "using", "defer",
    "inline", "no_inline", "push_context", "struct", "union", "enum", "enum_flags", "operator",
    "remove", "break", "continue", "interface", "size_of", "type_of", "type_info", "initializer_of",
    "is_constant", "code_of", "auto_cast",
];

// Keywords that take a space before `(` or `[` (`if (x)`, `xx (a + b)`); other words are calls,
// indexing or type constructors (`foo(x)`, `cast(T)`, `size_of(T)`, `struct(N: int)`).
KEYWORDS_BEFORE_PAREN :: string.[
    "if", "ifx", "while", "for", "return", "then", "else", "case", "defer", "xx", "auto_cast",
    "using", "push_context", "inline", "no_inline", "remove",
];

// Directives that are values themselves (`#line - 1`); after any other directive an operator
// is a prefix (`#type *int`, `#run -x`).
OPERAND_DIRECTIVES :: string.["line", "file", "filepath", "this", "caller_location", "caller_code", "compile_time"];

// Assignment, declaration, comparison and logic operators always get a space on both sides.
SPACED_OPERATORS :: string.[
    "=", ":=", "::", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "%=", "&=", "|=",
    "^=", "<<=", ">>=", "<<<=", ">>>=", "&&=", "||=", "->", "=>",
];

// Operators that are always binary, spaced on both sides.
BINARY_OPERATORS :: string.["/", "%", "|", "^", ">>", "<<<", ">>>"];

// Binary after an operand (spaced), prefix otherwise (attached to their operand). `<` and `>`
// are only prefix as the `for <` flag.
AMBIGUOUS_OPERATORS :: string.["-", "+", "*", "&", "<<", "<", ">"];

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

is_opener :: inline (text: string) -> bool {
    return text == "{" || text == "(" || text == "[" || text == ".{" || text == ".[";
}

one_of :: (text: string, list: [] string) -> bool {
    for list if it == text return true;
    return false;
}

directive_name :: (f: *Formatter, i: int) -> string {
    text := text_of(f, i);
    at := 1;
    while at < text.count && (text[at] == #char " " || text[at] == #char "\t") at += 1;
    return slice(text, at, text.count - at);
}

uses_crlf :: (src: string) -> bool {
    crlf := 0;
    lf := 0;
    for i: 0..src.count - 1 {
        if src[i] == #char "\n" {
            lf += 1;
            if i > 0 && src[i - 1] == #char "\r" crlf += 1;
        }
    }
    return lf > 0 && crlf * 2 > lf;
}

contains_newline :: (src: string, from: s64, to: s64) -> bool {
    for i: from..to - 1 if src[i] == #char "\n" return true;
    return false;
}

same_ignoring_whitespace :: (a: string, b: string) -> bool {
    i := 0;
    j := 0;
    while true {
        while i < a.count && is_blank(a[i]) i += 1;
        while j < b.count && is_blank(b[j]) j += 1;
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
            if width > max_width count += 1;
            width = 0;
        } else if c != #char "\r" && (c & 0xC0) != 0x80 {
            width += 1; // counts code points, not bytes
        }
    }
    if width > max_width count += 1;
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
            while j < n && f.tokens[j].kind != .EOF && !is_punct(f, j, "{") && !is_punct(f, j, ";") j += 1;
            if is_punct(f, j, "{") {
                depth := 0;
                k := j;
                while k < n && f.tokens[k].kind != .EOF {
                    if is_punct(f, k, "{") || is_punct(f, k, ".{") depth += 1;
                    if is_punct(f, k, "}") {
                        depth -= 1;
                        if depth == 0 break;
                    }
                    if k > j f.verbatim[k] = true;
                    k += 1;
                }
                i = k + 1;
                continue;
            }
        }
        i += 1;
    }
}

// The source column of every token, counted like the lexer counts indentation.
mark_columns :: (f: *Formatter) {
    array_resize(*f.columns, f.tokens.count);
    at: s64 = 0;
    if f.src.count >= 3 && f.src[0] == 0xEF && f.src[1] == 0xBB && f.src[2] == 0xBF at = 3;
    column: s32 = 0;
    for tok, i: f.tokens {
        while at < tok.start {
            c := f.src[at];
            if c == #char "\n" {
                column = 0;
            } else if c == #char "\t" {
                column += 4;
            } else if c != #char "\r" && c != 0x0b && c != 0x0c && (c & 0xC0) != 0x80 {
                column += 1;
            }
            at += 1;
        }
        f.columns[i] = column;
    }
}

// Is token i the last code on its output line (only comments may follow it)?
mark_line_ends :: (f: *Formatter) {
    n := f.tokens.count;
    array_resize(*f.last_on_line, n);
    for i: 0..n - 2 {
        j := i + 1;
        while f.tokens[j].kind == .BLOCK_COMMENT && f.breaks[j] == 0
            && !contains_newline(f.src, f.tokens[j].start, f.tokens[j].end) j += 1;
        t := f.tokens[j];
        f.last_on_line[i] = f.breaks[j] > 0 || t.kind == .EOF || t.kind == .LINE_COMMENT
            || (t.kind == .BLOCK_COMMENT && f.breaks[j] == 0);
    }
}

// Decides the line structure of the output: `f.breaks[i]` is the number of line breaks before
// token i (0: same line). It starts as the source's and then:
// - with brace_style = "same_line", a `{` that opens a block moves up to its header, and `else`
//   moves up to the `}` before it;
// - every non-empty `{ ... }` block gets its body on lines of its own (`{}` stays, `{\n}`
//   becomes `{}`), and a statement after its `}` on the same line moves down;
// - statements separated by `;` in a block go on lines of their own, except that a `case`
//   label keeps one simple statement after it (`case .A; return 1;`, `case 2; #through;`).
// Trailing comments stay where they are. Unbalanced input is left to the emit pass to report.
plan_lines :: (f: *Formatter) {
    n := f.tokens.count;
    array_resize(*f.breaks, n);
    for i: 0..n - 1 f.breaks[i] = f.tokens[i].newlines;

    partner: [..] int; // the matching bracket
    enclosing: [..] int; // the innermost open bracket around each token, -1 at the top level
    ends_case: [..] bool; // a `;` that ends a `case` label
    defer array_free(partner);
    defer array_free(enclosing);
    defer array_free(ends_case);
    array_resize(*partner, n);
    array_resize(*enclosing, n);
    array_resize(*ends_case, n);

    open: [..] int;
    statement: [..] int; // first token of the current statement, per bracket level
    defer array_free(open);
    defer array_free(statement);
    array_add(*statement, -1);
    for i: 0..n - 1 {
        partner[i] = -1;
        enclosing[i] = ifx open.count then open[open.count - 1] else -1;
        tok := f.tokens[i];
        if tok.kind == .EOF break;
        if is_comment(f, i) continue;
        level := statement.count - 1;
        if statement[level] < 0 statement[level] = i;
        if tok.kind != .PUNCT continue;
        text := text_of(f, i);
        if is_opener(text) {
            array_add(*open, i);
            array_add(*statement, -1);
        } else if text == "}" || text == ")" || text == "]" {
            if !open.count || !closes(text_of(f, open[open.count - 1]), text) return;
            o := pop(*open);
            pop(*statement);
            partner[o] = i;
            partner[i] = o;
            if text_of(f, o) == "{" statement[statement.count - 1] = -1;
        } else if text == ";" {
            ends_case[i] = is_ident(f, statement[level], "case");
            statement[level] = -1;
        }
    }
    if open.count return;

    array_resize(*f.literal_brace, n);
    for o: 0..n - 1 {
        if is_punct(f, o, "{") && !f.verbatim[o] && partner[o] > o {
            f.literal_brace[o] = opens_dotless_literal(f, o, partner[o], enclosing);
        }
    }

    if f.config.brace_style == .SAME_LINE {
        for i: 1..n - 1 {
            if f.breaks[i] == 0 || f.verbatim[i] || f.verbatim[i - 1] || is_comment(f, i - 1) continue;
            if is_punct(f, i, "{") && joins_header(f, i - 1) f.breaks[i] = 0;
            if is_ident(f, i, "else") && is_punct(f, i - 1, "}") f.breaks[i] = 0;
        }
    }

    for o: 0..n - 1 {
        if !is_punct(f, o, "{") || f.verbatim[o] || f.verbatim[o + 1] continue; // `#asm { ... }` stays
        if f.literal_brace[o] continue; // a value, laid out like `.{`
        c := partner[o];
        if c == o + 1 {
            f.breaks[c] = 0;
            continue;
        }
        if !trailing_comment(f, o + 1) f.breaks[o + 1] = max(f.breaks[o + 1], 1);
        if !f.verbatim[c] f.breaks[c] = max(f.breaks[c], 1);
        next := c + 1;
        if f.tokens[next].kind == .IDENT && f.breaks[next] == 0 && !f.verbatim[next]
            && !is_ident(f, next, "else") && !is_ident(f, next, "then") {
            f.breaks[next] = 1;
        }
    }

    for s: 0..n - 1 {
        if !is_punct(f, s, ";") || f.verbatim[s] continue;
        e := enclosing[s];
        if e >= 0 && !is_punct(f, e, "{") continue;
        j := s + 1;
        while f.tokens[j].kind == .NOTE && f.breaks[j] == 0 j += 1;
        t := f.tokens[j];
        if t.kind == .EOF || f.breaks[j] > 0 || f.verbatim[j] || trailing_comment(f, j) continue;
        if is_punct(f, j, ";") || is_punct(f, j, "}") continue;
        if ends_case[s] && case_keeps_statement(f, j) continue;
        f.breaks[j] = 1;
    }

    if f.config.blank_lines_between_items separate_items(f, partner, enclosing, 0, n, -1);
}

// A file-level declaration or directive (`blank_lines_between_items`): its first and last code
// tokens.
Item :: struct {
    start := -1;
    end: int;
    is_import: bool; // holds a `#import` or `#load` of its own (`#import "A";`, `A :: #import "A";`)
    multi_line: bool;
}

// `blank_lines_between_items`: walks the items whose innermost bracket is `level` (-1: the file;
// otherwise the `{` of a file-level `#if` block, whose items count as file-level too) in tokens
// `from` to `to - 1`, and puts a blank line between two neighbours when either spans lines, or
// when one is an import and the other is not. A `#scope_*` line is not an item: like a comment,
// it belongs to the item below it. Only existing line breaks are deepened, so the token check
// is unaffected.
separate_items :: (f: *Formatter, partner: [] int, enclosing: [] int, from: int, to: int, level: int) {
    previous: Item;
    item: Item;
    k := from;
    while k < to && f.tokens[k].kind != .EOF {
        tok := f.tokens[k];
        if is_comment(f, k) || (item.start < 0 && tok.kind == .DIRECTIVE && begins_with(directive_name(f, k), "scope_")) {
            k += 1;
            continue;
        }
        if item.start < 0 item.start = k;
        done := false;
        if tok.kind == .PUNCT {
            text := text_of(f, k);
            if is_opener(text) && partner[k] > k {
                o := k;
                if text == "{" && is_item_container(f, partner, item.start, o) {
                    separate_items(f, partner, enclosing, o + 1, partner[o], o);
                }
                k = partner[o];
                if text == "{" && !f.literal_brace[o] done = block_ends_item(f, k);
            } else if text == ";" {
                done = true;
            }
        }
        item.end = k;
        if done {
            finish_item(f, *item, enclosing, level);
            if previous.start >= 0 separate_pair(f, previous, item);
            previous = item;
            item = .{};
        }
        k += 1;
    }
    if item.start >= 0 {
        finish_item(f, *item, enclosing, level);
        if previous.start >= 0 separate_pair(f, previous, item);
    }
}

finish_item :: (f: *Formatter, item: *Item, enclosing: [] int, level: int) {
    for k: item.start..item.end {
        t := f.tokens[k];
        if k > item.start && f.breaks[k] > 0 item.multi_line = true;
        if (t.kind == .HERE_STRING || t.kind == .STRING || t.kind == .BLOCK_COMMENT) && contains_newline(f.src, t.start, t.end) {
            item.multi_line = true;
        }
        if t.kind == .DIRECTIVE && enclosing[k] == level {
            name := directive_name(f, k);
            if name == "import" || name == "load" item.is_import = true;
        }
    }
}

// Ensures a blank line between items `a` and `b` where the rules ask for one. It goes before
// the first line after `a`, so comments and `#scope_*` lines directly above `b` stay with it.
separate_pair :: (f: *Formatter, a: Item, b: Item) {
    if a.is_import && b.is_import return;
    if !a.multi_line && !b.multi_line && a.is_import == b.is_import return;
    t := a.end + 1;
    while t <= b.start && f.breaks[t] == 0 t += 1;
    if t > b.start || f.verbatim[t] return; // on one line, or inside `// jaifmt: off`
    for j: t..b.start if f.breaks[j] > 1 return;
    f.breaks[t] = 2;
}

// Does the block closed at `close` end its item? Not when `else`, a note or punctuation
// (`;`, `,`) continues it, or when more code follows on the same line.
block_ends_item :: (f: *Formatter, close: int) -> bool {
    next := close + 1;
    broken := false;
    while is_comment(f, next) {
        if f.breaks[next] > 0 broken = true;
        next += 1;
    }
    t := f.tokens[next];
    if t.kind == .EOF return true;
    if !broken && f.breaks[next] == 0 return false;
    if t.kind == .NOTE || is_ident(f, next, "else") || is_ident(f, next, "then") return false;
    return !is_punct(f, next, ";") && !is_punct(f, next, ",");
}

// Is the `{` at `o`, in the item starting at `start`, the body of a file-level `#if` (or its
// `else`), whose contents are file-level items? Not a `#if x == {` switch, and not the body of
// a declaration written under a braceless `#if` (`#if A f :: () {`).
is_item_container :: (f: *Formatter, partner: [] int, start: int, o: int) -> bool {
    if f.tokens[start].kind != .DIRECTIVE || directive_name(f, start) != "if" || f.literal_brace[o] return false;
    p := previous_code(f, o);
    if is_punct(f, p, "==") return false;
    while p >= start {
        t := f.tokens[p];
        if is_comment(f, p) {
            p -= 1;
            continue;
        }
        if t.kind == .PUNCT {
            text := text_of(f, p);
            if (text == ")" || text == "]") && partner[p] >= 0 && partner[p] < p {
                p = partner[p] - 1;
                continue;
            }
            if one_of(text, string.["::", ":=", ":", "=", "->", ";", "{", "}"]) return false;
        } else if t.kind == .IDENT {
            text := text_of(f, p);
            if text == "else" return true;
            if one_of(text, string.["struct", "union", "enum", "enum_flags"]) return false;
        } else if t.kind == .DIRECTIVE {
            name := directive_name(f, p);
            if name == "if" return true;
            if !is_punct(f, p + 1, "(") && !one_of(name, OPERAND_DIRECTIVES) return false;
        }
        p -= 1;
    }
    return false;
}

// `f({1, 2})`, `x = {w, h}`, `return {a, b};`: a brace where a value goes, holding no
// statements, is a struct literal without its dot rather than a block.
opens_dotless_literal :: (f: *Formatter, o: int, close: int, enclosing: [] int) -> bool {
    p := previous_code(f, o);
    value_position := is_ident(f, p, "return");
    for text: string.["(", ",", "=", ":=", "["] if is_punct(f, p, text) value_position = true;
    if !value_position return false;
    for j: o + 1..close - 1 {
        if enclosing[j] == o && is_punct(f, j, ";") return false;
    }
    return true;
}

// The `{` at `previous + 1`, alone on its line, belongs to the code before it (a procedure,
// struct, `if x`, `if x ==` switch, `else`, directive ...), not a new block statement.
joins_header :: (f: *Formatter, previous: int) -> bool {
    t := f.tokens[previous];
    if t.kind == .PUNCT return !one_of(text_of(f, previous), string.[";", "{", "}", ",", "(", "[", ".{", ".["]);
    if t.kind == .DIRECTIVE {
        name := directive_name(f, previous);
        return !begins_with(name, "scope_") && name != "through" && name != "load" && name != "import";
    }
    if t.kind == .NOTE {
        // `} @Note` and `x: int; @Note` end a declaration.
        q := previous;
        while q >= 0 && (f.tokens[q].kind == .NOTE || is_comment(f, q)) q -= 1;
        return q >= 0 && !is_punct(f, q, "}") && !is_punct(f, q, ";");
    }
    return true;
}

// Token i starts comments that end their line (`x := 1; // note`), or i is the line's end.
trailing_comment :: (f: *Formatter, i: int) -> bool {
    if !is_comment(f, i) || f.breaks[i] > 0 return false;
    j := i;
    while is_comment(f, j) && f.breaks[j] == 0 {
        if f.tokens[j].kind == .LINE_COMMENT return true;
        j += 1;
    }
    return f.breaks[j] > 0 || f.tokens[j].kind == .EOF;
}

// After `case X;`, does the rest of the source line hold exactly one simple statement (no
// block), so that it can stay next to its label?
case_keeps_statement :: (f: *Formatter, first: int) -> bool {
    depth := 0;
    j := first;
    while f.tokens[j].kind != .EOF && (j == first || f.breaks[j] == 0) {
        if is_comment(f, j) {
            j += 1;
            continue;
        }
        text := ifx f.tokens[j].kind == .PUNCT then text_of(f, j) else "";
        if text == "{" return false;
        if is_opener(text) {
            depth += 1;
        } else if text == "}" || text == ")" || text == "]" {
            depth -= 1;
            if depth < 0 return false;
        } else if text == ";" && depth == 0 {
            k := j + 1;
            return f.breaks[k] > 0 || f.tokens[k].kind == .EOF || trailing_comment(f, k) || is_punct(f, k, "}");
        }
        j += 1;
    }
    return false;
}

// Appends text to the output and keeps track of the output column.
put :: (f: *Formatter, text: string) {
    append(*f.out, text);
    start := 0;
    for < i: 0..text.count - 1 {
        if text[i] == #char "\n" {
            start = i + 1;
            f.column = 0;
            break;
        }
    }
    for i: start..text.count - 1 if (text[i] & 0xC0) != 0x80 f.column += 1;
}

run :: (f: *Formatter) -> bool, string {
    mark_verbatim(f);
    mark_columns(f);
    plan_lines(f);
    mark_line_ends(f);
    array_resize(*f.operand_end, f.tokens.count);
    f.zone = directive_zone(f.tokens, f.src);

    root: Frame;
    root.block = true;
    array_add(*f.frames, root);
    defer array_free(f.frames);
    defer array_free(f.verbatim);
    defer array_free(f.columns);
    defer array_free(f.breaks);
    defer array_free(f.last_on_line);
    defer array_free(f.literal_brace);
    defer array_free(f.operand_end);
    defer array_free(f.zone);

    // A byte order mark stays in front.
    if f.src.count >= 3 && f.src[0] == 0xEF && f.src[1] == 0xBB && f.src[2] == 0xBF append(*f.out, slice(f.src, 0, 3));

    for i: 0..f.tokens.count - 1 {
        tok := f.tokens[i];
        if tok.kind == .EOF break;

        if f.verbatim[i] {
            put(f, slice(f.src, tok.ws_start, tok.end - tok.ws_start));
            if tok.newlines > 0 {
                f.line_new = tok.column;
                f.line_orig = tok.column;
            }
        } else if f.breaks[i] > 0 || !f.emitted_any {
            ok, message := start_line(f, i);
            if !ok return false, message;
        } else {
            put(f, spacing(f, i));
            put(f, text_of(f, i));
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
    if f.emitted_any put(f, f.newline);
    return true, "";
}

// Writes the line break(s) and indentation before token i, which starts a line.
start_line :: (f: *Formatter, i: int) -> bool, string {
    tok := f.tokens[i];
    if f.emitted_any {
        blank := min(cast(int) f.breaks[i] - 1, f.config.max_blank_lines);
        for 0..blank put(f, f.newline);
    }

    indent, ok, message := line_indent(f, i);
    if !ok return false, message;
    if indent < 0 indent = 0;
    for 1..indent put(f, " ");

    if tok.kind == .BLOCK_COMMENT {
        append_block_comment(f, i, indent - f.columns[i]);
    } else {
        put(f, text_of(f, i));
    }
    f.line_new = indent;
    f.line_orig = f.columns[i];
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
            while spaces < line.count && line[spaces] == #char " " spaces += 1;
            if spaces < -delta {
                delta = 0;
                break;
            }
        }
    }
    for line, index: lines {
        // Every line of the comment ends with the file's newline: drop a `\r` of its own.
        line = trim_right(line);
        if index > 0 {
            put(f, f.newline);
            if line.count {
                if delta > 0 {
                    for 1..delta put(f, " ");
                } else if delta < 0 {
                    line = slice(line, -delta, line.count + delta);
                }
            }
        }
        put(f, line);
    }
}

relative :: inline (base_new: int, base_orig: int, column: int) -> int {
    return max(0, base_new + column - base_orig);
}

// A continuation line inside `(`, `[` or a one-line `{`: aligned after the opener, it stays
// aligned with it; otherwise it keeps its offset from the line that holds the opener.
continuation_indent :: (f: *Formatter, frame: *Frame, i: int) -> int {
    column := f.columns[i];
    if column > frame.open_orig return frame.open_new + column - frame.open_orig;
    return relative(frame.line_new, frame.line_orig, column);
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
            return continuation_indent(f, top, i), true, "";
        }
    }
    if !top.block return continuation_indent(f, top, i), true, "";

    // A line continuing a statement keeps its offset from the statement's first line.
    if !starts_statement(f) return max(top.stmt_new, relative(top.stmt_new, top.stmt_orig, f.columns[i])), true, "";

    // The code this line is about: itself, or what a comment line precedes.
    code := i;
    while is_comment(f, code) code += 1;
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
        top.stmt_orig = f.columns[i];
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
        while q >= 0 && (f.tokens[q].kind == .NOTE || is_comment(f, q)) q -= 1;
        return q < 0 || is_punct(f, q, "}") || is_punct(f, q, ";");
    }
    return false;
}

// Updates brackets, statement state and the operand classification after emitting token i.
track :: (f: *Formatter, i: int) -> bool, string {
    tok := f.tokens[i];
    f.operand_end[i] = classify(f, i);
    top := *f.frames[f.frames.count - 1];

    if tok.kind == .IDENT {
        text := text_of(f, i);
        if text == "operator" f.operator_decl = true;
        if text == "case" && top.switch_block top.seen_case = true;
        return true, "";
    }
    if tok.kind != .PUNCT return true, "";
    text := text_of(f, i);
    if text == "::" f.operator_decl = false;
    if text == ":" top.colons += 1;
    if text == ";" || text == "," top.colons = 0;

    if is_opener(text) {
        frame: Frame;
        frame.open = ifx text == "{" && i < f.literal_brace.count && f.literal_brace[i] then ".{" else text;
        frame.line_new = f.line_new;
        frame.line_orig = f.line_orig;
        frame.open_new = f.column - text.count;
        frame.open_orig = f.columns[i];
        if (text == "{" || text == ".{") && f.last_on_line[i] {
            frame.block = true;
            // A header continued over several lines still indents its body by one level.
            frame.base = ifx top.block then top.stmt_new else f.line_new;
            frame.switch_block = text == "{" && opens_switch(f, i);
            frame.content = frame.base + ifx frame.switch_block then f.case_indent else f.config.indent_width;
            frame.stmt_new = frame.content;
            frame.stmt_orig = 0;
        }
        if text == "{" top.colons = 0;
        if text == "(" {
            frame.is_cast = is_cast_paren(f, i);
        }
        if text == "[" {
            frame.is_index = f.prev_sig >= 0 && f.operand_end[f.prev_sig] == .OPERAND && f.breaks[i] == 0;
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
        if text == "}" {
            // A struct literal is a value; after a block a new statement begins.
            f.operand_end[i] = ifx frame.open == ".{" then Context_Kind.OPERAND else .OPERATOR;
            if frame.open == "{" f.frames[f.frames.count - 1].colons = 0;
        }
    }
    return true, "";
}

opens_switch :: (f: *Formatter, i: int) -> bool {
    p := f.prev_sig;
    if p >= 0 && f.tokens[p].kind == .DIRECTIVE && directive_name(f, p) == "complete" p = previous_code(f, p);
    return is_punct(f, p, "==");
}

previous_code :: (f: *Formatter, i: int) -> int {
    j := i - 1;
    while j >= 0 && is_comment(f, j) j -= 1;
    return j;
}

// No whitespace between token i and the one before it.
is_tight :: inline (f: *Formatter, i: int) -> bool {
    return f.tokens[i].ws_start == f.tokens[i].start;
}

// `cast(T)` and `cast,no_check(T)`: the parenthesized type is not an operand.
is_cast_paren :: (f: *Formatter, i: int) -> bool {
    p := f.prev_sig;
    while !is_ident(f, p, "cast") {
        if p < 2 || f.tokens[p].kind != .IDENT || !is_punct(f, p - 1, ",") || !is_tight(f, p) || !is_tight(f, p - 1) return false;
        p -= 2;
    }
    return true;
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
        case .DIRECTIVE;
            if one_of(directive_name(f, i), OPERAND_DIRECTIVES) return .OPERAND;
            return .OPERATOR;
        case .PUNCT;
            text := text_of(f, i);
            if text == ".*" || text == "---" || text == ")" || text == "]" || text == "}" return .OPERAND; // refined by `track`
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
// flags never get here: they are handled by `directive_zone`.)
is_flag_comma :: (f: *Formatter, comma: int) -> bool {
    next := comma + 1;
    if f.tokens[next].kind != .IDENT || f.tokens[next].newlines > 0 return false;
    if f.tokens[next].ws_start != f.tokens[next].start || f.tokens[comma].ws_start != f.tokens[comma].start return false;
    p := comma - 1;
    while p >= 2 && f.tokens[p].kind == .IDENT && is_punct(f, p - 1, ",")
        && f.tokens[p].ws_start == f.tokens[p].start && f.tokens[p - 1].ws_start == f.tokens[p - 1].start p -= 2;
    if p < 0 return false;
    return f.tokens[p].kind == .IDENT && one_of(text_of(f, p), string.["cast", "xx", "using"]);
}

// `$T/Interface`: the `/` of a polymorphic constraint is written tight.
is_constraint_slash :: (f: *Formatter, slash: int) -> bool {
    return slash >= 2 && f.tokens[slash - 1].kind == .IDENT && (is_punct(f, slash - 2, "$") || is_punct(f, slash - 2, "$$"));
}

// NONE and SPACE are the canonical choices; PRESENCE keeps whether the source had any
// whitespace there (one space if it did), for spots where the parser looks at adjacency.
Spacing :: enum u8 {
    NONE;
    SPACE;
    PRESENCE;
}

// The whitespace to write between token i and the token before it on the same line.
spacing :: (f: *Formatter, i: int) -> string {
    rule := spacing_rule(f, i);
    if rule == .PRESENCE rule = ifx is_tight(f, i) then Spacing.NONE else .SPACE;
    if rule == .NONE && would_merge(f, i - 1, i) rule = .SPACE;
    return ifx rule == .SPACE then " " else "";
}

spacing_rule :: (f: *Formatter, i: int) -> Spacing {
    p := i - 1;
    pt := f.tokens[p];
    nt := f.tokens[i];
    ptext := ifx pt.kind == .PUNCT then text_of(f, p) else "";
    ntext := ifx nt.kind == .PUNCT then text_of(f, i) else "";

    if nt.kind == .LINE_COMMENT return .SPACE;
    if nt.kind == .BLOCK_COMMENT || pt.kind == .BLOCK_COMMENT {
        if ptext == "(" || ptext == "[" || ptext == ".[" return .NONE;
        if ntext == ")" || ntext == "]" || ntext == "," || ntext == ";" return .NONE;
        return .SPACE;
    }
    if f.operator_decl || adjacency_matters(f.zone, f.tokens, f.src, p, i) return .PRESENCE;

    // Separators and brackets.
    if ntext == "," || ntext == ";" return .NONE;
    if ptext == "," {
        if is_flag_comma(f, p) || ntext == ")" || ntext == "]" return .NONE;
        return .SPACE;
    }
    if ptext == ";" return .SPACE;
    if ptext == "(" || ptext == "[" || ptext == ".[" return .NONE;
    if ntext == ")" || ntext == "]" return .NONE;
    if ptext == "{" || ptext == ".{" return ifx ntext == "}" then .NONE else .SPACE;
    if ntext == "{" || ntext == "}" return .SPACE;

    // `x: int`, `N: int : 5`.
    if ntext == ":" return ifx f.frames[f.frames.count - 1].colons > 0 then Spacing.SPACE else .NONE;
    if ptext == ":" return .SPACE;

    if one_of(ntext, SPACED_OPERATORS) || one_of(ptext, SPACED_OPERATORS) return .SPACE;

    before := context_before(f, i);
    if ntext == ".*" return .NONE;
    if ntext == "." || ntext == ".{" || ntext == ".[" || ntext == ".." return ifx before == .OPERAND then Spacing.NONE else .SPACE;
    if ptext == "." || ptext == ".." return .NONE;
    if ptext == "$" || ptext == "$$" || ptext == "`" || ptext == "!" || ptext == "~" return .NONE;

    if (ntext == "/" && is_constraint_slash(f, i)) || (ptext == "/" && is_constraint_slash(f, p)) return .NONE;
    if one_of(ntext, BINARY_OPERATORS) || one_of(ptext, BINARY_OPERATORS) return .SPACE;
    if one_of(ntext, AMBIGUOUS_OPERATORS) {
        if before == .UNKNOWN return .PRESENCE;
        if before == .OPERAND return .SPACE;
        // A prefix operator right after another one attaches to it (`**T`, `-*p`).
        if one_of(ptext, AMBIGUOUS_OPERATORS) && context_before(f, p) == .OPERATOR && ptext != "<" && ptext != ">" return .NONE;
        return .SPACE;
    }
    if one_of(ptext, AMBIGUOUS_OPERATORS) {
        side := context_before(f, p);
        if side == .UNKNOWN return .PRESENCE;
        if side == .OPERAND || ptext == "<" || ptext == ">" return .SPACE;
        return .NONE;
    }

    if ntext == "(" || ntext == "[" {
        if pt.kind == .IDENT return ifx one_of(text_of(f, p), KEYWORDS_BEFORE_PAREN) then Spacing.SPACE else .NONE;
        if ptext == "]" && ntext == "[" return .NONE;
        if before == .OPERAND return .NONE;
        return .SPACE;
    }
    return .SPACE;
}

// Would writing tokens p and i with nothing between them lex differently?
would_merge :: (f: *Formatter, p: int, i: int) -> bool {
    pt := f.tokens[p];
    nt := f.tokens[i];
    a := text_of(f, p);
    b := text_of(f, i);
    if !a.count || !b.count return false;
    if is_ident_char(a[a.count - 1]) && (is_ident_char(b[0]) || b[0] == #char "\\") return true;
    if pt.kind == .NOTE return !(nt.kind == .PUNCT && one_of(b, string.[";", ",", ")", "]", "}"]));
    if pt.kind == .NUMBER && b[0] == #char "." && !begins_with(b, "..") return true;
    // `1.2.3` is `1.2`, `.`, `3`; once the rule above spaces `1.2 .`, a tight `.3` would lex as a
    // float, so the digits after a lone `.` get a space too.
    if pt.kind == .PUNCT && a == "." && nt.kind == .NUMBER && is_digit(b[0]) return true;
    if a[a.count - 1] == #char "/" && (b[0] == #char "/" || b[0] == #char "*") return true;
    if pt.kind == .PUNCT && nt.kind == .PUNCT {
        for PUNCTUATION {
            if it.count > a.count && begins_with(it, a) && it[a.count] == b[0] return true;
        }
    }
    return false;
}
