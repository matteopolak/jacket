// Where a byte of a string literal's value sits in the source. A jaic extension,
// not official Jai: other Jai compilers have no such module, and it may change between jaic
// releases. See docs/extensions/string-literal.md.
//
// Compile-time code that checks a string it was given (a query, a pattern, a format) can report
// a problem at the exact character instead of at the whole literal:
//
//     check_sql :: (code: Code) {
//         literal := cast(*Code_Literal) compiler_get_nodes(code);
//         error_at := find_problem(literal._string);
//         location := String_Literal.location_of_byte(literal, error_at);
//         compiler_report("syntax error", location);
//     }
//
// Both quoted literals (with every escape) and here-strings (`#string`, `#string,cr`,
// `#string,\%`) are mapped. The source is read through the compiler's file system, so the
// editor maps against the unsaved buffer.

#import "Basic";
#import "Compiler";
#import "File";
#import "String";

// The location of byte `byte` of the value of `literal` (a string literal node from
// `compiler_get_nodes`); `byte == value.count` is the end of the value. `ok` is false when the
// literal's source cannot be read again, or does not decode to the node's value (a string the
// compiler built rather than read): the location is then where the literal starts.
location_of_byte :: (literal: *Code_Literal, byte: int) -> Source_Code_Location, ok: bool {
    start: Source_Code_Location;
    start.line_number = literal.l0;
    start.character_number = literal.c0;
    if !literal.enclosing_load || literal.value_type != .STRING return start, false;

    start.fully_pathed_filename = literal.enclosing_load.fully_pathed_filename;
    if byte < 0 || byte > literal._string.count return start, false;

    text, read := read_entire_file(start.fully_pathed_filename,, temp);
    if !read return start, false;

    at := offset_of(text, literal.l0, literal.c0);
    if at < 0 return start, false;

    value, sources, decoded := decode(text, at);
    if !decoded || value != literal._string return start, false;

    offset := ifx byte < sources.count then sources[byte] else end_offset(text, at, sources);
    line, column := line_and_column(text, offset);
    location := start;
    location.line_number = line;
    location.character_number = column;

    return location, true;
}

#scope_module

// The byte offset of a 1-based line and (byte) column, or -1.
offset_of :: (text: string, line: int, column: int) -> int {
    at := 0;

    for 1..line - 1 {
        newline := find_index_from_left(advance(text, at), #char "\n");
        if newline < 0 return -1;
        at += newline + 1;
    }

    at += column - 1;

    return ifx at >= 0 && at < text.count then at else -1;
}

line_and_column :: (text: string, offset: int) -> line: int, column: int {
    line, line_start := 1, 0;

    for i: 0..offset - 1 {
        if text[i] == #char "\n" {
            line += 1;
            line_start = i + 1;
        }
    }

    return line, offset - line_start + 1;
}

// Where the value ends: just past the source of its last byte (for a quoted literal, the
// closing quote).
end_offset :: (text: string, at: int, sources: [] int) -> int {
    if text[at] == #char "\"" {
        i := at + 1;
        while i < text.count && text[i] != #char "\"" {
            i += ifx text[i] == #char "\\" then 2 else 1;
        }

        return i;
    }

    if sources.count == 0 return at;

    return sources[sources.count - 1] + 1;
}

// The value of the literal starting at `at`, and for each of its bytes the offset of the source
// that produced it.
decode :: (text: string, at: int) -> value: string, sources: [] int, ok: bool {
    if text[at] == #char "\"" return decode_quoted(text, at + 1);
    if text[at] == #char "#" return decode_here(text, at + 1);
    return "", .[], false;
}

decode_quoted :: (text: string, at: int) -> value: string, sources: [] int, ok: bool {
    out: [..] u8;
    sources: [..] int;

    out.allocator = temp;
    sources.allocator = temp;
    i := at;
    while i < text.count {
        c := text[i];
        if c == #char "\"" {
            return xx out, sources, true;
        }

        if c != #char "\\" {
            push(*out, *sources, c, i);
            i += 1;
            continue;
        }

        if i + 1 >= text.count break;

        from := i;
        e := text[i + 1];

        i += 2;

        if e == {
            case #char "n"; push(*out, *sources, #char "\n", from);
            case #char "r"; push(*out, *sources, #char "\r", from);
            case #char "t"; push(*out, *sources, #char "\t", from);
            case #char "0"; push(*out, *sources, 0, from);
            case #char "e"; push(*out, *sources, 0x1b, from);
            case #char "a"; push(*out, *sources, 0x07, from);
            case #char "b"; push(*out, *sources, 0x08, from);
            case #char "f"; push(*out, *sources, 0x0c, from);
            case #char "v"; push(*out, *sources, 0x0b, from);
            case #char "%"; push(*out, *sources, 0x1f, from);
            case #char "\\"; #through;
            case #char "\""; #through;
            case #char "'"; push(*out, *sources, e, from);
            case #char "\n"; // a line continuation: nothing
            case #char "x";
                value, ok := digits(text, i, 2, 16);
                if !ok return "", .[], false;

                push(*out, *sources, cast(u8) value, from);
                i += 2;
            case #char "d";
                value, ok := digits(text, i, 3, 10);
                if !ok return "", .[], false;

                push(*out, *sources, cast(u8) value, from);
                i += 3;
            case #char "u"; #through;
            case #char "U";
                count := ifx e == #char "u" then 4 else 8;
                value, ok := digits(text, i, count, 16);

                if !ok return "", .[], false;

                encoded := utf8(cast(u32) value);
                for encoded push(*out, *sources, it, from);

                i += count;
            case;
                return "", .[], false;
        }
    }

    return "", .[], false;
}

push :: (out: *[..] u8, sources: *[..] int, byte: u8, from: int) {
    array_add(out, byte);
    array_add(sources, from);
}

digits :: (text: string, at: int, count: int, base: int) -> int, bool {
    value := 0;

    for 0..count - 1 {
        if at + it >= text.count return 0, false;

        c := text[at + it];
        d := -1;

        if c >= #char "0" && c <= #char "9" d = c - #char "0";
        else if base == 16 && c >= #char "a" && c <= #char "f" d = c - #char "a" + 10;
        else if base == 16 && c >= #char "A" && c <= #char "F" d = c - #char "A" + 10;
        if d < 0 || d >= base return 0, false;

        value = value * base + d;
    }

    return value, true;
}

utf8 :: (code: u32) -> [] u8 {
    out: [..] u8;
    out.allocator = temp;

    if code < 0x80 {
        array_add(*out, cast(u8) code);
    } else if code < 0x800 {
        array_add(*out, cast(u8) (0xc0 | (code >> 6)), cast(u8) (0x80 | (code & 0x3f)));
    } else if code < 0x10000 {
        array_add(*out, cast(u8) (0xe0 | (code >> 12)), cast(u8) (0x80 | ((code >> 6) & 0x3f)), cast(u8) (0x80 | (code & 0x3f)));
    } else {
        array_add(*out, cast(u8) (0xf0 | (code >> 18)), cast(u8) (0x80 | ((code >> 12) & 0x3f)), cast(u8) (0x80 | ((code >> 6) & 0x3f)), cast(u8) (0x80 | (code & 0x3f)));
    }

    return out;
}

is_blank :: (c: u8) -> bool {
    return c == #char " " || c == #char "\t";
}

is_word :: (c: u8) -> bool {
    return c == #char "_" || (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || (c >= #char "0" && c <= #char "9") || c >= 0x80;
}

// `#string,cr,\% TAG` then a body up to a line whose first word is TAG. `at` is just past `#`.
decode_here :: (text: string, at: int) -> value: string, sources: [] int, ok: bool {
    i := at;
    while i < text.count && is_blank(text[i]) i += 1;
    if !begins_with(advance(text, i), "string") return "", .[], false;

    i += "string".count;
    cr, escape_percent := false, false;

    while true {
        while i < text.count && is_blank(text[i]) i += 1;
        if i >= text.count || text[i] != #char "," break;

        i += 1;
        while i < text.count && is_blank(text[i]) i += 1;

        if i < text.count && text[i] == #char "\\" {
            escape_percent ||= i + 1 < text.count && text[i + 1] == #char "%";
            i += 2;
        } else {
            word_start := i;
            while i < text.count && is_word(text[i]) i += 1;
            cr ||= slice(text, word_start, i - word_start) == "cr";
        }
    }

    tag_start := i;

    while i < text.count && is_word(text[i]) i += 1;

    tag := slice(text, tag_start, i - tag_start);
    if tag.count == 0 return "", .[], false;
    while i < text.count && text[i] != #char "\n" i += 1;

    i += 1;

    out: [..] u8;
    sources: [..] int;

    out.allocator = temp;
    sources.allocator = temp;

    while i < text.count {
        line_end := i;
        while line_end < text.count && text[line_end] != #char "\n" line_end += 1;

        indent := i;
        while indent < line_end && is_blank(text[indent]) indent += 1;

        rest := slice(text, indent, line_end - indent);
        if begins_with(rest, tag) && (rest.count == tag.count || !is_word(rest[tag.count])) {
            return xx out, sources, true;
        }

        j := i;
        while j <= line_end && j < text.count {
            c := text[j];
            if c == #char "\r" {
                j += 1;
                continue;
            }

            if c == #char "\n" && cr {
                array_add(*out, #char "\r");
                array_add(*sources, j);
            }

            if escape_percent && c == #char "\\" && j + 1 < line_end && text[j + 1] == #char "%" {
                array_add(*out, 0x1f);
                array_add(*sources, j);
                j += 2;
                continue;
            }

            array_add(*out, c);
            array_add(*sources, j);
            j += 1;
        }

        i = line_end + 1;
    }

    return "", .[], false;
}
