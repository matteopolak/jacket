// Scanning SQL text for typed queries: turns each `%` hole into `$n`, and maps positions in the
// rewritten statement (what the server sees) back to the source string (what the user wrote).
// The scanner skips everything that is text rather than SQL: quotes, `E''` strings, dollar
// quotes, quoted identifiers and comments.

// One place where the rewritten text differs from the source: a `%` that became `$n` (`hole` is
// n), or a `%%` that became `%` (`hole` is 0).
Rewrite_Edit :: struct {
    source_offset: s64;
    source_length: s64;
    rewritten_offset: s64;
    rewritten_length: s64;
    hole: s64;
}

Rewrite :: struct {
    sql: string; // the statement the server sees
    edits: [..] Rewrite_Edit;
    holes: s64; // how many `%` holes the source has
    error: string; // empty when the source scanned cleanly
    error_offset: s64; // where in the source the error is
}

// Rewrites `%` to `$1`, `$2`, ... in order and `%%` to `%`. A `$n` of Postgres' own in the source
// is an error: one style only. Text inside quotes and comments is copied as it is.
//
// - source: the statement as the user wrote it, with `%` holes.
rewrite_placeholders :: (source: string) -> Rewrite {
    result: Rewrite;
    out: [..] u8;

    out.allocator = temp;
    result.edits.allocator = temp;

    i := 0;

    while i < source.count {
        c := source[i];

        if c == #char "'" {
            // E'..' strings give a backslash its meaning.
            escapes := i > 0 && (source[i - 1] == #char "E" || source[i - 1] == #char "e") && (i < 2 || !scan_is_word(source[i - 2]));
            end := scan_skip_quoted(source, i, #char "'", escapes);

            scan_append(*out, source, i, end);
            i = end;
        } else if c == #char "\"" {
            end := scan_skip_quoted(source, i, #char "\"", false);
            scan_append(*out, source, i, end);
            i = end;
        } else if c == #char "-" && i + 1 < source.count && source[i + 1] == #char "-" {
            end := i;
            while end < source.count && source[end] != #char "\n" end += 1;
            scan_append(*out, source, i, end);
            i = end;
        } else if c == #char "/" && i + 1 < source.count && source[i + 1] == #char "*" {
            end := scan_skip_comment(source, i);
            scan_append(*out, source, i, end);
            i = end;
        } else if c == #char "$" {
            if i > 0 && scan_is_word(source[i - 1]) {
                // `a$1` is one identifier.
                array_add(*out, c);
                i += 1;
            } else if i + 1 < source.count && scan_is_digit(source[i + 1]) {
                result.error = "Postgres' own $n placeholders are not used here; write % and pass the value as an argument";
                result.error_offset = i;
                return result;
            } else {
                end := scan_skip_dollar(source, i);
                scan_append(*out, source, i, ifx end > i then end else i + 1);
                i = ifx end > i then end else i + 1;
            }
        } else if c == #char "%" {
            edit: Rewrite_Edit;
            edit.source_offset = i;
            edit.rewritten_offset = out.count;

            if i + 1 < source.count && source[i + 1] == #char "%" {
                edit.source_length = 2;
                edit.rewritten_length = 1;
                array_add(*out, #char "%");
                i += 2;
            } else {
                result.holes += 1;
                text := tprint("$%", result.holes);

                edit.hole = result.holes;
                edit.source_length = 1;
                edit.rewritten_length = text.count;
                scan_append(*out, text, 0, text.count);
                i += 1;
            }

            array_add(*result.edits, edit);
        } else {
            array_add(*out, c);
            i += 1;
        }
    }

    result.sql = copy_string(cast(string) out);

    return result;
}

// The offset in the source of a byte of the rewritten statement. Inside a replaced `%` it is
// that `%`.
//
// - rewrite: the result of `rewrite_placeholders`.
// - rewritten_offset: a byte offset in `rewrite.sql`.
source_offset_of :: (rewrite: *Rewrite, rewritten_offset: s64) -> s64 {
    shift := 0; // rewritten minus source, after the edits passed so far

    for rewrite.edits {
        if rewritten_offset < it.rewritten_offset break;
        if rewritten_offset < it.rewritten_offset + it.rewritten_length return it.source_offset;

        shift = it.rewritten_offset + it.rewritten_length - (it.source_offset + it.source_length);
    }

    return rewritten_offset - shift;
}

// The byte offset in `text` of a 1-based character position (what an ErrorResponse's `P` field
// counts, in characters, not bytes). Past the end, the end.
//
// - text: the statement.
// - character: the position, counted from 1.
byte_of_character :: (text: string, character: s64) -> s64 {
    seen := 0;

    for text {
        if (it & 0xc0) == 0x80 continue; // a continuation byte
        if seen == character - 1 return it_index;

        seen += 1;
    }

    return text.count;
}

// A 64-bit statement id: the first eight bytes of SHA-256 of the rewritten statement.
//
// - rewritten_sql: the statement with `$n` in place of `%`.
statement_id :: (rewritten_sql: string) -> u64 {
    digest := Sha.sha256(rewritten_sql);
    id: u64;

    for 0..7 id = (id << 8) | digest[it];

    return id;
}

// The full SHA-256 of a text, as lower-case hex.
//
// - text: the bytes to hash.
statement_digest_hex :: (text: string) -> string {
    digest := Sha.sha256(text);

    return scan_hex_text(digest.data, digest.count);
}

// The 16 lower-case hex digits of a statement id.
//
// - id: a statement id.
id_hex :: (id: u64) -> string {
    bytes: [8] u8;
    for 0..7 bytes[it] = cast,no_check(u8) (id >> ((7 - it) * 8));

    return scan_hex_text(bytes.data, 8);
}

// Lower-case hex of `count` bytes at `data`, twice as long.
//
// - data: the first byte.
// - count: how many bytes to show.
scan_hex_text :: (data: *u8, count: s64) -> string {
    DIGITS :: "0123456789abcdef";
    out := alloc_string(count * 2);

    for 0..count - 1 {
        out[it * 2] = DIGITS[data[it] >> 4];
        out[it * 2 + 1] = DIGITS[data[it] & 0xf];
    }

    return out;
}

#scope_module

scan_is_digit :: (c: u8) -> bool {
    return c >= #char "0" && c <= #char "9";
}

// A character that can be inside an SQL identifier (or start one: bytes above 127 are letters).
scan_is_word :: (c: u8) -> bool {
    return c == #char "_" || c == #char "$" || scan_is_digit(c) || (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || c >= 0x80;
}

scan_append :: (out: *[..] u8, text: string, from: s64, to: s64) {
    for from..to - 1 array_add(out, text[it]);
}

// Just past the quote that closes the one at `start`; the end of the text if it never closes.
scan_skip_quoted :: (text: string, start: s64, quote: u8, backslash_escapes: bool) -> s64 {
    i := start + 1;

    while i < text.count {
        if backslash_escapes && text[i] == #char "\\" {
            i += 2;
        } else if text[i] == quote {
            if i + 1 < text.count && text[i + 1] == quote {
                i += 2;
            } else {
                return i + 1;
            }
        } else {
            i += 1;
        }
    }

    return text.count;
}

// Just past the `*/` that closes the comment at `start`, allowing for nesting.
scan_skip_comment :: (text: string, start: s64) -> s64 {
    depth := 1;
    i := start + 2;

    while i < text.count {
        if text[i] == #char "/" && i + 1 < text.count && text[i + 1] == #char "*" {
            depth += 1;
            i += 2;
        } else if text[i] == #char "*" && i + 1 < text.count && text[i + 1] == #char "/" {
            depth -= 1;
            i += 2;
            if depth == 0 return i;
        } else {
            i += 1;
        }
    }

    return text.count;
}

// If a dollar quote `$tag$` opens at `start`, just past its closing tag (the end of the text if
// it never closes); else `start`.
scan_skip_dollar :: (text: string, start: s64) -> s64 {
    i := start + 1;

    while i < text.count && text[i] != #char "$" {
        c := text[i];
        if !scan_is_word(c) || c == #char "$" || scan_is_digit(text[start + 1]) return start;

        i += 1;
    }

    if i >= text.count return start;

    tag := slice(text, start, i - start + 1); // "$tag$"
    rest := advance(text, i + 1);
    close := find_index_from_left(rest, tag);

    if close < 0 return text.count;

    return i + 1 + close + tag.count;
}
