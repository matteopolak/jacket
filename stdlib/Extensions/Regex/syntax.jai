// The pattern parser. It turns pattern text into a tree of Nodes with every flag already
// applied, so later stages never see flags: a case-insensitive letter is a class, `.` is a class,
// `^` and `$` are looks for the mode they were written in, and `(?x)` whitespace is gone.

Look :: enum u8 {
    START_TEXT; // \A, or ^ without (?m)
    END_TEXT; // \z, or $ without (?m)
    START_LINE; // ^ with (?m)
    END_LINE; // $ with (?m)
    START_LINE_CRLF; // ^ with (?mR)
    END_LINE_CRLF; // $ with (?mR)
    WORD; // \b
    NOT_WORD; // \B
    WORD_START; // \b{start}, \<
    WORD_END; // \b{end}, \>
    WORD_START_HALF; // \b{start-half}
    WORD_END_HALF; // \b{end-half}
    WORD_ASCII; // the same six with (?-u): word characters are [0-9A-Za-z_]
    NOT_WORD_ASCII;
    WORD_START_ASCII;
    WORD_END_ASCII;
    WORD_START_HALF_ASCII;
    WORD_END_HALF_ASCII;
}

Node_Kind :: enum u8 {
    EMPTY;
    LITERAL;
    CLASS;
    LOOK;
    REPEAT;
    CAPTURE;
    CONCAT;
    ALTERNATE;
}

Node :: struct {
    kind: Node_Kind;
    literal: string; // LITERAL: the bytes to match
    ranges: [..] Char_Range; // CLASS: sorted and merged; code points, or bytes when `bytes`
    bytes: bool;
    look: Look;
    min: int; // REPEAT: max < 0 means no upper bound
    max: int;
    greedy: bool;
    index: int; // CAPTURE: the group number, from 1
    name: string; // CAPTURE: "" for an unnamed group
    children: [..] *Node; // REPEAT and CAPTURE have exactly one
}

Parsed :: struct {
    root: *Node;
    groups: int; // capture groups, not counting the whole match
    names: [..] string; // names[i] is group i's name; names[0] is ""
    ok: bool;
    error: string;
    offset: int; // where in the pattern the error is
}

MAX_REPEAT :: 0xFFFF_FFFF; // as in Rust; big counts still fail as too large a program
MAX_NESTING :: 250;

parse_pattern :: (pattern: string, flags: Options) -> Parsed {
    p: Parser;
    p.pattern = pattern;
    p.flags = flags;
    array_add(*p.out.names, "");
    p.out.ok = true;
    root := parse_alternation(*p);

    if p.out.ok && p.pos < pattern.count {
        // parse_alternation only stops early at a `)` with no group open.
        fail(*p, p.pos, "unopened group: this `)` has no `(`");
    }

    if p.out.ok p.out.root = root;

    return p.out;
}

#scope_file

Parser :: struct {
    pattern: string;
    pos: int;
    flags: Options;
    depth: int;
    out: Parsed;
}

fail :: (p: *Parser, offset: int, message: string) -> *Node {
    if p.out.ok {
        p.out.ok = false;
        p.out.error = message;
        p.out.offset = offset;
    }

    return null;
}

make :: (kind: Node_Kind) -> *Node {
    node := New(Node);
    node.kind = kind;
    return node;
}

at_end :: (p: *Parser) -> bool {
    return p.pos >= p.pattern.count;
}

peek :: (p: *Parser) -> u8 {
    if p.pos >= p.pattern.count return 0;
    return p.pattern[p.pos];
}

peek_is :: (p: *Parser, text: string) -> bool {
    if p.pos + text.count > p.pattern.count return false;
    for 0..text.count - 1 if p.pattern[p.pos + it] != text[it] return false;
    return true;
}

// The code point at `p.pos`; advances past it. Patterns are UTF-8; a stray byte is taken as
// itself so that (?-u) patterns can still name bytes directly.
next_char :: (p: *Parser) -> u32 {
    c, size := decode_utf8(p.pattern, p.pos);
    p.pos += size;
    return c;
}

// In (?x) mode, skips whitespace and `#` comments that run to the end of the line.
skip_space :: (p: *Parser) {
    if !p.flags.ignore_space return;

    while !at_end(p) {
        c := peek(p);
        if c == #char " " || c == #char "\t" || c == #char "\n" || c == #char "\r" || c == 0x0b || c == 0x0c {
            p.pos += 1;
        } else if c == #char "#" {
            while !at_end(p) && peek(p) != #char "\n" p.pos += 1;
        } else {
            break;
        }
    }
}

parse_alternation :: (p: *Parser) -> *Node {
    branches: [..] *Node;

    while true {
        branch := parse_concat(p);

        if !p.out.ok return null;

        array_add(*branches, branch);
        if peek(p) != #char "|" break;

        p.pos += 1;
    }

    if branches.count == 1 return branches[0];

    node := make(.ALTERNATE);
    node.children = branches;

    return node;
}

parse_concat :: (p: *Parser) -> *Node {
    items: [..] *Node;

    while true {
        skip_space(p);
        if at_end(p) break;

        c := peek(p);
        if c == #char "|" || c == #char ")" break;

        atom := parse_atom(p);

        if !p.out.ok return null;
        if !atom continue; // a flag group like (?i): nothing to match

        while true {
            skip_space(p);
            repeated := parse_repetition(p, atom);

            if !p.out.ok return null;
            if !repeated break;

            atom = repeated;
        }

        array_add(*items, atom);
    }

    if items.count == 0 return make(.EMPTY);
    if items.count == 1 return items[0];

    node := make(.CONCAT);
    node.children = items;

    return node;
}

// One of * + ? {n} {n,} {n,m} {,m} after `atom`, with an optional lazy `?`. Null when the next
// thing is not a repetition.
parse_repetition :: (p: *Parser, atom: *Node) -> *Node {
    if at_end(p) return null;

    c := peek(p);
    start := p.pos;
    min, max: int;

    if c == #char "*" {
        min, max = 0, -1;
        p.pos += 1;
    } else if c == #char "+" {
        min, max = 1, -1;
        p.pos += 1;
    } else if c == #char "?" {
        min, max = 0, 1;
        p.pos += 1;
    } else if c == #char "{" {
        p.pos += 1;
        skip_space(p);
        if !is_digit(peek(p)) return fail(p, start, "repetition quantifier expects a count, like `{2}` or `{2,5}`");

        min = parse_count(p);
        if !p.out.ok return null;

        skip_space(p);

        if peek(p) == #char "," {
            p.pos += 1;
            skip_space(p);

            if is_digit(peek(p)) {
                max = parse_count(p);
                if !p.out.ok return null;
            } else {
                max = -1;
            }
        } else {
            max = min;
        }

        skip_space(p);
        if peek(p) != #char "}" return fail(p, start, "unclosed repetition quantifier: expected `}`");

        p.pos += 1;
        if max >= 0 && min > max return fail(p, start, "invalid repetition: the minimum is larger than the maximum");
    } else {
        return null;
    }

    greedy := true;

    if peek(p) == #char "?" {
        greedy = false;
        p.pos += 1;
    }

    if p.flags.swap_greed greedy = !greedy;

    node := make(.REPEAT);
    node.min = min;
    node.max = max;
    node.greedy = greedy;
    array_add(*node.children, atom);

    return node;
}

parse_count :: (p: *Parser) -> int {
    start := p.pos;
    value := 0;

    while is_digit(peek(p)) {
        value = value * 10 + cast(int) (peek(p) - #char "0");
        p.pos += 1;

        if value > MAX_REPEAT {
            fail(p, start, tprint("repetition count is too large (the limit is %)", MAX_REPEAT));
            return 0;
        }
    }

    return value;
}

is_digit :: (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
}

is_hex :: (c: u8) -> bool {
    return is_digit(c) || (c >= #char "a" && c <= #char "f") || (c >= #char "A" && c <= #char "F");
}

hex_value :: (c: u8) -> u32 {
    if is_digit(c) return cast(u32) (c - #char "0");
    if c >= #char "a" return cast(u32) (c - #char "a" + 10);
    return cast(u32) (c - #char "A" + 10);
}

is_ascii_letter :: (c: u8) -> bool {
    return (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z");
}

parse_atom :: (p: *Parser) -> *Node {
    start := p.pos;
    c := peek(p);

    if c == {
        case #char "(";
            return parse_group(p);
        case #char "[";
            ranges, ok := parse_class(p);
            if !ok return null;

            return class_node(p, ranges);
        case #char ".";
            p.pos += 1;
            node := make(.CLASS);
            node.bytes = !p.flags.unicode;
            top: u32 = ifx p.flags.unicode then cast(u32) 0x10FFFF else 0xFF;

            if p.flags.dot_all {
                array_add(*node.ranges, .{ 0, top });
            } else if p.flags.crlf {
                array_add(*node.ranges, .{ 0, 9 });
                array_add(*node.ranges, .{ 11, 12 });
                array_add(*node.ranges, .{ 14, top });
            } else {
                array_add(*node.ranges, .{ 0, 9 });
                array_add(*node.ranges, .{ 11, top });
            }

            return node;
        case #char "^";
            p.pos += 1;
            return look_node(ifx !p.flags.multi_line then .START_TEXT else ifx p.flags.crlf then .START_LINE_CRLF else .START_LINE);
        case #char "$";
            p.pos += 1;
            return look_node(ifx !p.flags.multi_line then .END_TEXT else ifx p.flags.crlf then .END_LINE_CRLF else .END_LINE);
        case #char "\\";
            return parse_escape(p);
        case #char "*"; #through;
        case #char "+"; #through;
        case #char "?";
            return fail(p, start, "repetition operator missing expression");
        case #char "{";
            return fail(p, start, "repetition quantifier missing expression; write `\\{` for a literal brace");
    }

    ch := next_char(p);

    return literal_node(p, ch);
}

look_node :: (look: Look) -> *Node {
    node := make(.LOOK);
    node.look = look;
    return node;
}

// A literal code point, or under (?i) the class of it and its case variants.
literal_node :: (p: *Parser, c: u32) -> *Node {
    if !p.flags.unicode && c > 0x7F {
        // Without Unicode, the pattern still names a code point; it matches its UTF-8 bytes.
        // Case folding of non-ASCII text is a Unicode feature, so it is not applied.
        node := make(.LITERAL);
        node.literal = utf8_encode_string(c);

        return node;
    }

    if p.flags.case_insensitive {
        ranges: [..] Char_Range;

        array_add(*ranges, .{ c, c });

        if p.flags.unicode {
            for unicode_simple_fold(c) array_add(*ranges, .{ it, it });
        } else {
            if c >= #char "a" && c <= #char "z" array_add(*ranges, .{ c - 32, c - 32 });
            if c >= #char "A" && c <= #char "Z" array_add(*ranges, .{ c + 32, c + 32 });
        }

        if ranges.count > 1 {
            ranges_normalize(*ranges);
            node := make(.CLASS);
            node.ranges = ranges;
            node.bytes = !p.flags.unicode;

            return node;
        }
    }

    node := make(.LITERAL);
    node.literal = utf8_encode_string(c);

    return node;
}

byte_literal_node :: (b: u32) -> *Node {
    node := make(.LITERAL);
    text := alloc_string(1);
    text[0] = cast(u8) b;
    node.literal = text;

    return node;
}

// The finished class: folded under (?i), as code points or as bytes for the mode.
class_node :: (p: *Parser, ranges: [..] Char_Range) -> *Node {
    node := make(.CLASS);
    node.ranges = ranges;
    node.bytes = !p.flags.unicode;
    return node;
}

parse_group :: (p: *Parser) -> *Node {
    open := p.pos;

    p.pos += 1;
    if p.depth >= MAX_NESTING return fail(p, open, "pattern nests groups too deeply");

    name := "";
    capturing := true;
    saved := p.flags;

    if peek(p) == #char "?" {
        p.pos += 1;
        if peek_is(p, "<=") || peek_is(p, "<!") return fail(p, open, "look-around is not supported: matching runs in linear time");

        if peek_is(p, "P<") || peek(p) == #char "<" {
            if peek(p) == #char "P" p.pos += 1;

            p.pos += 1;
            name_start := p.pos;

            while !at_end(p) && peek(p) != #char ">" {
                c := peek(p);
                first := p.pos == name_start;
                valid := is_ascii_letter(c) || c == #char "_" || (!first && (is_digit(c) || c == #char "." || c == #char "[" || c == #char "]"));

                if !valid return fail(p, p.pos, "invalid capture group name: names start with a letter or `_` and continue with letters, digits, `_`, `.`, `[` or `]`");

                p.pos += 1;
            }

            if at_end(p) return fail(p, open, "unclosed capture group name: expected `>`");
            if p.pos == name_start return fail(p, p.pos, "empty capture group name");

            name = slice(p.pattern, name_start, p.pos - name_start);
            p.pos += 1;
            for p.out.names if it == name return fail(p, name_start, tprint("duplicate capture group name `%`", name));
        } else {
            // Flags: (?flags) changes them for the rest of the enclosing group, (?flags:...)
            // only inside.
            flags := p.flags;
            negated := false;
            any := false;
            dash := -1;

            while true {
                if at_end(p) return fail(p, open, "unclosed group");

                c := peek(p);
                if c == #char ")" || c == #char ":" break;

                value := !negated;

                if c == {
                    case #char "i"; flags.case_insensitive = value;
                    case #char "m"; flags.multi_line = value;
                    case #char "s"; flags.dot_all = value;
                    case #char "U"; flags.swap_greed = value;
                    case #char "x"; flags.ignore_space = value;
                    case #char "R"; flags.crlf = value;
                    case #char "u"; flags.unicode = value;
                    case #char "-";
                        if negated return fail(p, p.pos, "repeated negation in flags");

                        negated = true;
                        dash = p.pos;
                        p.pos += 1;
                        continue;
                    case;
                        if c == #char "=" || c == #char "!" return fail(p, open, "look-around is not supported: matching runs in linear time");
                        if c == #char "P" && p.pos + 1 < p.pattern.count && (p.pattern[p.pos + 1] == #char "=" || p.pattern[p.pos + 1] == #char ">") return fail(p, p.pos, "backreferences are not supported");

                        return fail(p, p.pos, tprint("unrecognized flag `%`", slice(p.pattern, p.pos, 1)));
                }

                any = true;
                if dash >= 0 && dash == p.pos - 1 dash = -2; // something followed the dash

                p.pos += 1;
            }

            if dash >= 0 return fail(p, dash, "a `-` in flags must be followed by a flag");

            if peek(p) == #char ")" {
                if !any && !negated return fail(p, open, "empty flag group `(?)`");
                p.pos += 1;
                p.flags = flags;
                return null; // applies to the rest of the enclosing group
            }

            p.pos += 1; // ':'
            p.flags = flags;
            capturing = false;
        }
    }

    index := 0;

    if capturing {
        p.out.groups += 1;
        index = p.out.groups;
        array_add(*p.out.names, name);
    }

    p.depth += 1;
    inner := parse_alternation(p);

    p.depth -= 1;
    if !p.out.ok return null;
    if peek(p) != #char ")" return fail(p, open, "unclosed group: expected `)`");

    p.pos += 1;
    p.flags = saved;
    if !capturing return inner;

    node := make(.CAPTURE);
    node.index = index;
    node.name = name;
    array_add(*node.children, inner);

    return node;
}

// Escapes that stand for one code point (or one byte under (?-u)). `ok` is false and nothing is
// consumed when the escape is something else.
parse_char_escape :: (p: *Parser) -> value: u32, is_byte: bool, ok: bool {
    start := p.pos;
    c := p.pattern[p.pos + 1];
    simple: u32 = 0xFFFF_FFFF;

    if c == {
        case #char "a"; simple = 0x07;
        case #char "f"; simple = 0x0C;
        case #char "t"; simple = 0x09;
        case #char "n"; simple = 0x0A;
        case #char "r"; simple = 0x0D;
        case #char "v"; simple = 0x0B;
    }

    if simple != 0xFFFF_FFFF {
        p.pos += 2;
        return simple, false, true;
    }

    if c == #char "x" || c == #char "u" || c == #char "U" {
        p.pos += 2;
        digits := ifx c == #char "x" then 2 else ifx c == #char "u" then 4 else 8;
        value: u32 = 0;

        if peek(p) == #char "{" {
            p.pos += 1;
            count := 0;

            while !at_end(p) && peek(p) != #char "}" {
                if !is_hex(peek(p)) || count >= 8 {
                    fail(p, start, "invalid hexadecimal escape: expected hex digits and `}`");
                    return 0, false, false;
                }

                value = value * 16 + hex_value(peek(p));
                count += 1;
                p.pos += 1;
            }

            if at_end(p) || count == 0 {
                fail(p, start, "invalid hexadecimal escape: expected hex digits and `}`");
                return 0, false, false;
            }

            p.pos += 1;
        } else {
            for 1..digits {
                if !is_hex(peek(p)) {
                    fail(p, start, tprint("invalid hexadecimal escape: expected % hex digits", digits));
                    return 0, false, false;
                }

                value = value * 16 + hex_value(peek(p));
                p.pos += 1;
            }
        }

        if !p.flags.unicode && c == #char "x" && value <= 0xFF return value, value > 0x7F, true;

        if value > 0x10FFFF || (value >= 0xD800 && value <= 0xDFFF) {
            fail(p, start, "escape is not a Unicode scalar value");
            return 0, false, false;
        }

        return value, false, true;
    }

    if c < 0x80 && !is_ascii_letter(c) && !is_digit(c) && c != #char "<" && c != #char ">" {
        p.pos += 2;
        return c, false, true;
    }

    return 0, false, false;
}

parse_escape :: (p: *Parser) -> *Node {
    start := p.pos;
    if p.pos + 1 >= p.pattern.count return fail(p, start, "incomplete escape sequence at the end of the pattern");

    c := p.pattern[p.pos + 1];
    if c == {
        case #char "A";
            p.pos += 2;
            return look_node(.START_TEXT);
        case #char "z";
            p.pos += 2;
            return look_node(.END_TEXT);
        case #char "<";
            p.pos += 2;
            return look_node(ifx p.flags.unicode then Look.WORD_START else .WORD_START_ASCII);
        case #char ">";
            p.pos += 2;
            return look_node(ifx p.flags.unicode then Look.WORD_END else .WORD_END_ASCII);
        case #char "B";
            p.pos += 2;
            return look_node(ifx p.flags.unicode then Look.NOT_WORD else .NOT_WORD_ASCII);
        case #char "b";
            p.pos += 2;
            u := p.flags.unicode;

            if peek(p) == #char "{" {
                // \b{start} and friends; anything else after \b is a repetition of \b.
                end := p.pos + 1;
                while end < p.pattern.count && (is_ascii_letter(p.pattern[end]) || p.pattern[end] == #char "-") end += 1;

                if end > p.pos + 1 && end < p.pattern.count && p.pattern[end] == #char "}" {
                    name := slice(p.pattern, p.pos + 1, end - p.pos - 1);
                    look: Look;

                    if name == "start" look = ifx u then Look.WORD_START else .WORD_START_ASCII;
                    else if name == "end" look = ifx u then Look.WORD_END else .WORD_END_ASCII;
                    else if name == "start-half" look = ifx u then Look.WORD_START_HALF else .WORD_START_HALF_ASCII;
                    else if name == "end-half" look = ifx u then Look.WORD_END_HALF else .WORD_END_HALF_ASCII;
                    else return fail(p, start, tprint("unrecognized word boundary `\\b{%}`; expected start, end, start-half or end-half", name));

                    p.pos = end + 1;

                    return look_node(look);
                }
            }

            return look_node(ifx u then Look.WORD else .WORD_ASCII);
        case #char "Z";
            return fail(p, start, "`\\Z` is not supported; use `\\z` for the end of the text");
    }

    if is_digit(c) {
        if c == #char "0" return fail(p, start, "octal escapes are not supported; use `\\x00`");
        return fail(p, start, "backreferences are not supported");
    }

    if c == #char "d" || c == #char "D" || c == #char "s" || c == #char "S" || c == #char "w" || c == #char "W" || c == #char "p" || c == #char "P" {
        ranges, ok := parse_class_escape(p);
        if !ok return null;
        return class_node(p, ranges);
    }

    value, is_byte, ok := parse_char_escape(p);

    if !p.out.ok return null;
    if !ok return fail(p, start, tprint("unrecognized escape sequence `\\%`", slice(p.pattern, p.pos + 1, 1)));
    if is_byte return byte_literal_node(value);

    return literal_node(p, value);
}

// \d \s \w \p{..} and their negations, already folded for (?i). Positioned at the backslash.
parse_class_escape :: (p: *Parser) -> [..] Char_Range, bool {
    start := p.pos;
    c := p.pattern[p.pos + 1];

    p.pos += 2;
    negate := c == #char "D" || c == #char "S" || c == #char "W" || c == #char "P";
    ranges: [..] Char_Range;

    if c == #char "d" || c == #char "D" {
        if p.flags.unicode ranges = unicode_perl_digit();
        else array_add(*ranges, .{ #char "0", #char "9" });
    } else if c == #char "s" || c == #char "S" {
        if p.flags.unicode ranges = unicode_perl_space();
        else {
            array_add(*ranges, .{ 9, 13 });
            array_add(*ranges, .{ 32, 32 });
        }
    } else if c == #char "w" || c == #char "W" {
        if p.flags.unicode ranges = unicode_perl_word();
        else ascii_word(*ranges);
    } else {
        if !p.flags.unicode {
            fail(p, start, "Unicode properties need Unicode mode; remove (?-u)");
            return ranges, false;
        }

        spec: string;

        if peek(p) == #char "{" {
            close := p.pos;
            while close < p.pattern.count && p.pattern[close] != #char "}" close += 1;

            if close >= p.pattern.count {
                fail(p, start, "unclosed Unicode class: expected `}`");
                return ranges, false;
            }

            spec = slice(p.pattern, p.pos + 1, close - p.pos - 1);
            p.pos = close + 1;
        } else {
            if at_end(p) {
                fail(p, start, "incomplete Unicode class: expected a name after `\\p`");
                return ranges, false;
            }

            spec_start := p.pos;

            next_char(p);
            spec = slice(p.pattern, spec_start, p.pos - spec_start);
        }

        if spec.count && spec[0] == #char "^" {
            negate = !negate;
            spec = slice(spec, 1, spec.count - 1);
        }

        found, ok, message := unicode_property(spec);
        if !ok {
            fail(p, start, message);
            return ranges, false;
        }

        ranges = found;
    }

    if p.flags.case_insensitive ranges = fold_ranges(p, ranges);
    if negate ranges = negate_ranges(p, ranges);

    return ranges, true;
}

ascii_word :: (ranges: *[..] Char_Range) {
    array_add(ranges, .{ #char "0", #char "9" });
    array_add(ranges, .{ #char "A", #char "Z" });
    array_add(ranges, .{ #char "_", #char "_" });
    array_add(ranges, .{ #char "a", #char "z" });
}

fold_ranges :: (p: *Parser, ranges: [] Char_Range) -> [..] Char_Range {
    if p.flags.unicode return unicode_case_fold_ranges(ranges);

    out: [..] Char_Range;

    for ranges {
        array_add(*out, it);
        lo := max(it.lo, #char "a");
        hi := min(it.hi, #char "z");

        if lo <= hi array_add(*out, .{ lo - 32, hi - 32 });

        lo = max(it.lo, #char "A");
        hi = min(it.hi, #char "Z");
        if lo <= hi array_add(*out, .{ lo + 32, hi + 32 });
    }

    ranges_normalize(*out);

    return out;
}

negate_ranges :: (p: *Parser, ranges: [] Char_Range) -> [..] Char_Range {
    if p.flags.unicode return ranges_negate(ranges);

    out: [..] Char_Range;
    next: u32 = 0;

    for ranges {
        if it.lo > 0xFF break;
        if it.lo > next array_add(*out, .{ next, it.lo - 1 });
        next = it.hi + 1;
    }

    if next <= 0xFF array_add(*out, .{ next, 0xFF });

    return out;
}

// A bracketed class `[...]`, positioned at `[`. Set operations bind loosest and run left to
// right; ranges and plain unions bind tightest; a leading `^` negates the whole.
parse_class :: (p: *Parser) -> [..] Char_Range, bool {
    open := p.pos;

    p.pos += 1;
    p.depth += 1;
    defer p.depth -= 1;
    empty: [..] Char_Range;

    if p.depth >= MAX_NESTING {
        fail(p, open, "pattern nests classes too deeply");
        return empty, false;
    }

    negate := false;

    if peek(p) == #char "^" {
        negate = true;
        p.pos += 1;
    }

    result, ok := parse_class_union(p, open, true);
    if !ok return empty, false;

    while true {
        op := 0;
        if peek_is(p, "&&") op = 1;
        else if peek_is(p, "--") op = 2;
        else if peek_is(p, "~~") op = 3;
        else break;

        p.pos += 2;
        right, right_ok := parse_class_union(p, open, false);
        if !right_ok return empty, false;
        if op == 1 result = ranges_intersect(result, right);
        else if op == 2 result = ranges_intersect(result, negate_ranges(p, right));
        else result = ranges_union(ranges_intersect(result, negate_ranges(p, right)), ranges_intersect(right, negate_ranges(p, result)));
    }

    if peek(p) != #char "]" {
        fail(p, open, "unclosed character class: expected `]`");
        return empty, false;
    }

    p.pos += 1;
    if p.flags.case_insensitive result = fold_ranges(p, result);
    if negate result = negate_ranges(p, result);

    return result, true;
}

// Items up to `]` or a set operator. A `]` first in the class is a literal, and so is a `-`
// at either end.
parse_class_union :: (p: *Parser, open: int, first_in_class: bool) -> [..] Char_Range, bool {
    ranges: [..] Char_Range;
    first := true;

    while true {
        skip_space(p);

        if at_end(p) {
            fail(p, open, "unclosed character class: expected `]`");
            return ranges, false;
        }

        c := peek(p);
        if c == #char "]" && !(first && first_in_class) break;

        if peek_is(p, "&&") || peek_is(p, "--") || peek_is(p, "~~") {
            if first {
                fail(p, p.pos, "set operator needs a class on both sides");
                return ranges, false;
            }

            break;
        }

        first = false;

        if c == #char "[" {
            if peek_is(p, "[:") {
                posix, is_posix, posix_ok := parse_posix_class(p);
                if !posix_ok return ranges, false;

                if is_posix {
                    for posix array_add(*ranges, it);
                    continue;
                }
            }

            nested, ok := parse_class(p);
            if !ok return ranges, false;
            for nested array_add(*ranges, it);

            continue;
        }

        item_start := p.pos;
        lo, is_class, class_ranges, ok := parse_class_atom(p);

        if !ok return ranges, false;

        if is_class {
            for class_ranges array_add(*ranges, it);
            continue;
        }

        // A range `a-z`, unless the `-` ends the class or starts a `--` operator.
        skip_space(p);

        if peek(p) == #char "-" && !peek_is(p, "--") && p.pos + 1 < p.pattern.count && p.pattern[p.pos + 1] != #char "]" {
            p.pos += 1;
            skip_space(p);

            if at_end(p) {
                fail(p, open, "unclosed character class: expected `]`");
                return ranges, false;
            }

            hi, hi_is_class, _, hi_ok := parse_class_atom(p);
            if !hi_ok return ranges, false;

            if hi_is_class {
                fail(p, item_start, "invalid range: a range's ends must be single characters");
                return ranges, false;
            }

            if hi < lo {
                fail(p, item_start, "invalid range: the start is greater than the end");
                return ranges, false;
            }

            array_add(*ranges, .{ lo, hi });
        } else {
            array_add(*ranges, .{ lo, lo });
        }
    }

    if !p.flags.unicode {
        // Without Unicode, a class is over bytes: a non-ASCII character could only mean its
        // UTF-8 bytes one at a time, which is never what was meant.
        for ranges if it.hi > 0xFF {
            fail(p, open, "a class in (?-u) mode can't hold non-ASCII characters; use `\\xNN` bytes");
            return ranges, false;
        }
    }

    ranges_normalize(*ranges);

    return ranges, true;
}

// One character (literal or escape) or a class escape inside `[...]`.
parse_class_atom :: (p: *Parser) -> value: u32, is_class: bool, ranges: [..] Char_Range, ok: bool {
    empty: [..] Char_Range;

    if peek(p) == #char "\\" {
        start := p.pos;

        if p.pos + 1 >= p.pattern.count {
            fail(p, start, "incomplete escape sequence at the end of the pattern");
            return 0, false, empty, false;
        }

        c := p.pattern[p.pos + 1];
        if c == #char "d" || c == #char "D" || c == #char "s" || c == #char "S" || c == #char "w" || c == #char "W" || c == #char "p" || c == #char "P" {
            // Fold later with the whole class; parse_class_escape would fold now, which is
            // harmless because folding twice changes nothing.
            ranges, ok := parse_class_escape(p);
            return 0, true, ranges, ok;
        }

        value, _, ok := parse_char_escape(p);

        if !p.out.ok return 0, false, empty, false;

        if !ok {
            fail(p, start, tprint("unrecognized escape sequence `\\%` in a class", slice(p.pattern, p.pos + 1, 1)));
            return 0, false, empty, false;
        }

        return value, false, empty, true;
    }

    return next_char(p), false, empty, true;
}

POSIX_NAMES :: string.["alnum", "alpha", "ascii", "blank", "cntrl", "digit", "graph", "lower", "print", "punct", "space", "upper", "word", "xdigit"];

// `[:name:]` or `[:^name:]` (ASCII only). `is_posix` is false when the text only looks like one,
// e.g. `[:a]`, in which case nothing is consumed and it is a nested class.
parse_posix_class :: (p: *Parser) -> [..] Char_Range, is_posix: bool, ok: bool {
    ranges: [..] Char_Range;
    start := p.pos;
    i := p.pos + 2;
    negate := false;

    if i < p.pattern.count && p.pattern[i] == #char "^" {
        negate = true;
        i += 1;
    }

    name_start := i;

    while i < p.pattern.count && (is_ascii_letter(p.pattern[i])) i += 1;
    if i + 1 >= p.pattern.count || p.pattern[i] != #char ":" || p.pattern[i + 1] != #char "]" return ranges, false, true;

    name := slice(p.pattern, name_start, i - name_start);

    add :: (lo: u32, hi: u32) #expand {
        array_add(*ranges, .{ lo, hi });
    }

    if name == {
        case "alnum";
            add(#char "0", #char "9");
            add(#char "A", #char "Z");
            add(#char "a", #char "z");
        case "alpha";
            add(#char "A", #char "Z");
            add(#char "a", #char "z");
        case "ascii"; add(0, 0x7F);
        case "blank";
            add(9, 9);
            add(32, 32);
        case "cntrl";
            add(0, 0x1F);
            add(0x7F, 0x7F);
        case "digit"; add(#char "0", #char "9");
        case "graph"; add(0x21, 0x7E);
        case "lower"; add(#char "a", #char "z");
        case "print"; add(0x20, 0x7E);
        case "punct";
            add(0x21, 0x2F);
            add(0x3A, 0x40);
            add(0x5B, 0x60);
            add(0x7B, 0x7E);
        case "space";
            add(9, 13);
            add(32, 32);
        case "upper"; add(#char "A", #char "Z");
        case "word"; ascii_word(*ranges);
        case "xdigit";
            add(#char "0", #char "9");
            add(#char "A", #char "F");
            add(#char "a", #char "f");
        case;
            fail(p, start, tprint("unrecognized POSIX class `[:%:]`", name));
            return ranges, false, false;
    }

    p.pos = i + 2;
    ranges_normalize(*ranges);
    if negate ranges = negate_ranges(p, ranges);

    return ranges, true, true;
}

#scope_module

// Both inputs sorted and merged.
ranges_intersect :: (a: [] Char_Range, b: [] Char_Range) -> [..] Char_Range {
    out: [..] Char_Range;
    i, j := 0, 0;

    while i < a.count && j < b.count {
        lo := max(a[i].lo, b[j].lo);
        hi := min(a[i].hi, b[j].hi);

        if lo <= hi array_add(*out, .{ lo, hi });
        if a[i].hi < b[j].hi i += 1;
        else j += 1;
    }

    return out;
}

ranges_union :: (a: [] Char_Range, b: [] Char_Range) -> [..] Char_Range {
    out: [..] Char_Range;
    for a array_add(*out, it);
    for b array_add(*out, it);

    ranges_normalize(*out);

    return out;
}

// The code point starting at `index` and its length in bytes. Invalid UTF-8 decodes as the
// single byte, so every byte of any string is reachable.
decode_utf8 :: (s: string, index: int) -> u32, int {
    b0 := cast(u32) s[index];
    if b0 < 0x80 return b0, 1;

    count := 0;
    value: u32;

    if b0 >= 0xC2 && b0 <= 0xDF {
        count = 1;
        value = b0 & 0x1F;
    } else if b0 >= 0xE0 && b0 <= 0xEF {
        count = 2;
        value = b0 & 0x0F;
    } else if b0 >= 0xF0 && b0 <= 0xF4 {
        count = 3;
        value = b0 & 0x07;
    } else {
        return b0, 1;
    }

    if index + count >= s.count return b0, 1;

    for 1..count {
        b := cast(u32) s[index + it];
        if (b & 0xC0) != 0x80 return b0, 1;
        value = (value << 6) | (b & 0x3F);
    }

    // Overlong forms, surrogates and values past U+10FFFF are not valid.
    if count == 2 && (value < 0x800 || (value >= 0xD800 && value <= 0xDFFF)) return b0, 1;
    if count == 3 && (value < 0x10000 || value > 0x10FFFF) return b0, 1;

    return value, count + 1;
}

utf8_encode_string :: (c: u32) -> string {
    buffer: [4] u8;
    count := utf8_encode(c, *buffer);
    text := alloc_string(count);

    for 0..count - 1 text[it] = buffer[it];

    return text;
}

utf8_encode :: (c: u32, out: *[4] u8) -> int {
    if c < 0x80 {
        (out.*)[0] = cast(u8) c;
        return 1;
    }

    if c < 0x800 {
        (out.*)[0] = cast(u8) (0xC0 | (c >> 6));
        (out.*)[1] = cast(u8) (0x80 | (c & 0x3F));
        return 2;
    }

    if c < 0x10000 {
        (out.*)[0] = cast(u8) (0xE0 | (c >> 12));
        (out.*)[1] = cast(u8) (0x80 | ((c >> 6) & 0x3F));
        (out.*)[2] = cast(u8) (0x80 | (c & 0x3F));
        return 3;
    }

    (out.*)[0] = cast(u8) (0xF0 | (c >> 18));
    (out.*)[1] = cast(u8) (0x80 | ((c >> 12) & 0x3F));
    (out.*)[2] = cast(u8) (0x80 | ((c >> 6) & 0x3F));
    (out.*)[3] = cast(u8) (0x80 | (c & 0x3F));

    return 4;
}
