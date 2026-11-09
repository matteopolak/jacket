// An editable UTF-8 text buffer with undo, for editors.
//
// The text sits in a piece table, so inserting or deleting in a large file costs the number of edits
// made so far, not the size of the file. Lines are found by binary search. Offsets are byte offsets;
// move the cursor on character boundaries with `next_grapheme` and `previous_grapheme`.
//
// ```jai
// #import "Extensions/Text_Buffer";
//
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "hello\nworld");
// insert(*buffer, 5, ", there");
// erase(*buffer, 0, 1);
// assert(line_text(*buffer, 0) == "ello, there");
// undo(*buffer);
// assert(line_text(*buffer, 0) == "hello, there");
// ```
// @experimental

#import "Basic";

using,only(next_grapheme, previous_grapheme, grapheme_count) #import "Extensions/Unicode_Text";

#load "graphemes.jai";
#load "history.jai";

// A run of bytes in one of the buffer's two backing arrays. The text is the pieces in order.
Piece :: struct {
    from_added: bool; // false: the original text, true: the added text
    start: int;
    length: int;
}

// The text being edited. A zero value is an empty buffer; release it with `free_buffer`. Change it
// only through the procedures here. `version` goes up with every edit, so a cache keyed on the
// text can tell when to refresh.
Text_Buffer :: struct {
    original: [..] u8;
    added: [..] u8;
    pieces: [..] Piece;
    length: int; // bytes in the whole text
    line_starts: [..] int; // offset of the start of each line; line 0 starts at 0 (empty until first use)
    version: int; // counts edits, so caches keyed on the text know when it changed
    history: History;
}

// Frees everything the buffer owns and leaves it empty.
free_buffer :: (buffer: *Text_Buffer) {
    array_free(buffer.original);
    array_free(buffer.added);
    array_free(buffer.pieces);
    array_free(buffer.line_starts);
    free_history(*buffer.history);
    buffer.* = .{};
}

// Replaces the whole text with a copy of `text` and forgets the undo history.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "one\ntwo");
// assert(line_count(*buffer) == 2);
// ```
set_text :: (buffer: *Text_Buffer, text: string) {
    array_reset(*buffer.original);
    array_reset(*buffer.added);
    array_reset(*buffer.pieces);
    free_history(*buffer.history);

    array_resize(*buffer.original, text.count);
    if text.count memcpy(buffer.original.data, text.data, text.count);

    if text.count array_add(*buffer.pieces, .{ false, 0, text.count });

    buffer.length = text.count;
    buffer.version += 1;
    rebuild_lines(buffer);
}

// The number of lines: one more than the newlines, so an empty buffer has one empty line.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// assert(line_count(*buffer) == 1);
// set_text(*buffer, "a\nb\nc");
// assert(line_count(*buffer) == 3);
// ```
line_count :: (buffer: *Text_Buffer) -> int {
    ensure_lines(buffer);
    return buffer.line_starts.count;
}

// The offset of the first byte of `line`. A line past the end gives the last line.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "ab\ncd");
// assert(line_start(*buffer, 1) == 3);
// ```
line_start :: (buffer: *Text_Buffer, line: int) -> int {
    ensure_lines(buffer);
    return buffer.line_starts[clamp(line, 0, buffer.line_starts.count - 1)];
}

// The offset just past the last byte of `line`, before its line break (`\n` or `\r\n`).
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "ab\r\ncd");
// assert(line_end(*buffer, 0) == 2);
// assert(line_end(*buffer, 1) == 6);
// ```
line_end :: (buffer: *Text_Buffer, line: int) -> int {
    ensure_lines(buffer);

    l := clamp(line, 0, buffer.line_starts.count - 1);
    if l + 1 >= buffer.line_starts.count return buffer.length;

    end := buffer.line_starts[l + 1] - 1;
    // A "\r\n" ending belongs to the line break, not the text.
    if end > buffer.line_starts[l] && byte_at(buffer, end - 1) == #char "\r" end -= 1;

    return end;
}

// The line that holds `offset`, or the last line for an offset at the end.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "ab\ncd");
// assert(line_of_offset(*buffer, 4) == 1);
// ```
line_of_offset :: (buffer: *Text_Buffer, offset: int) -> int {
    ensure_lines(buffer);

    lo := 0;
    hi := buffer.line_starts.count - 1;

    while lo < hi {
        mid := (lo + hi + 1) / 2;
        if buffer.line_starts[mid] <= offset lo = mid;
        else hi = mid - 1;
    }

    return lo;
}

// The byte at `offset`, or 0 outside the text.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "abc");
// assert(byte_at(*buffer, 1) == #char "b");
// assert(byte_at(*buffer, 9) == 0);
// ```
byte_at :: (buffer: *Text_Buffer, offset: int) -> u8 {
    if offset < 0 || offset >= buffer.length return 0;

    base := 0;

    for piece: buffer.pieces {
        if offset < base + piece.length return piece_data(buffer, piece)[offset - base];
        base += piece.length;
    }

    return 0;
}

// The text of `line` without its line break. It points into the buffer when the line sits in one
// piece and is a temporary copy otherwise; either way it is valid only until the next edit.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "first\nsecond");
// assert(line_text(*buffer, 1) == "second");
// ```
line_text :: (buffer: *Text_Buffer, line: int) -> string {
    return text_range(buffer, line_start(buffer, line), line_end(buffer, line));
}

// The bytes from `start` up to `end`. Like `line_text`, it is a view or a temporary copy, valid until
// the next edit. Use `copy_range` to keep it.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "hello world");
// assert(text_range(*buffer, 6, 11) == "world");
// ```
text_range :: (buffer: *Text_Buffer, start: int, end: int) -> string {
    s := clamp(start, 0, buffer.length);
    e := clamp(end, s, buffer.length);

    if s == e return "";

    base := 0;

    for piece: buffer.pieces {
        if s >= base + piece.length {
            base += piece.length;
            continue;
        }

        if e <= base + piece.length {
            data := piece_data(buffer, piece);
            return string.{ e - s, data + (s - base) };
        }

        break;
    }

    return copy_range(buffer, s, e, temp);
}

// A copy of the bytes from `start` up to `end`, made with `allocator`.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "hello world");
// word := copy_range(*buffer, 0, 5, temp);
// insert(*buffer, 0, ">> ");
// assert(word == "hello");
// ```
copy_range :: (buffer: *Text_Buffer, start: int, end: int, allocator: Allocator) -> string {
    s := clamp(start, 0, buffer.length);
    e := clamp(end, s, buffer.length);
    result: string;
    result.count = e - s;
    result.data = alloc(max(e - s, 1),, allocator);

    base := 0;
    written := 0;

    for piece: buffer.pieces {
        if base >= e break;

        piece_end := base + piece.length;

        if piece_end > s {
            from := max(s, base);
            to := min(e, piece_end);
            memcpy(result.data + written, piece_data(buffer, piece) + (from - base), to - from);
            written += to - from;
        }

        base = piece_end;
    }

    return result;
}

// The whole text as a new string, made with `allocator`. Free it unless the allocator is temporary.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "abc");
// insert(*buffer, 3, "def");
// assert(buffer_to_string(*buffer, temp) == "abcdef");
// ```
buffer_to_string :: (buffer: *Text_Buffer, allocator: Allocator = context.allocator) -> string {
    return copy_range(buffer, 0, buffer.length, allocator);
}

// Inserts `text` at `offset`. Undo can take it back; typing one character after another merges into
// a single undo step.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "held");
// insert(*buffer, 2, "llo wor");
// assert(buffer_to_string(*buffer, temp) == "hello world");
// ```
insert :: (buffer: *Text_Buffer, offset: int, text: string) {
    if !text.count return;

    at := clamp(offset, 0, buffer.length);
    record_edit(buffer, at, "", text);
    apply_insert(buffer, at, text);
}

// Deletes `count` bytes from `offset`. Undo can take it back.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "hello world");
// erase(*buffer, 5, 6);
// assert(buffer_to_string(*buffer, temp) == "hello");
// ```
erase :: (buffer: *Text_Buffer, offset: int, count: int) {
    start := clamp(offset, 0, buffer.length);
    end := clamp(offset + count, start, buffer.length);

    if start == end return;

    removed := copy_range(buffer, start, end, context.allocator);

    record_edit(buffer, start, removed, "");
    free(removed.data);
    apply_erase(buffer, start, end - start);
}

// Replaces `count` bytes at `offset` with `text`, as one undo step.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "hello world");
// replace(*buffer, 6, 5, "there");
// assert(buffer_to_string(*buffer, temp) == "hello there");
// undo(*buffer);
// assert(buffer_to_string(*buffer, temp) == "hello world");
// ```
replace :: (buffer: *Text_Buffer, offset: int, count: int, text: string) {
    begin_group(buffer);
    erase(buffer, offset, count);
    insert(buffer, offset, text);
    end_group(buffer);
}

// The offset of the first `needle` at or after `from`, or -1.
//
// ```jai
// buffer: Text_Buffer;
// defer free_buffer(*buffer);
// set_text(*buffer, "to be or not to be");
// assert(find_text(*buffer, "be") == 3);
// assert(find_text(*buffer, "be", 4) == 16);
// assert(find_text(*buffer, "maybe") == -1);
// ```
find_text :: (buffer: *Text_Buffer, needle: string, from: int = 0) -> int {
    if !needle.count return -1;

    whole := text_range(buffer, 0, buffer.length);
    start := clamp(from, 0, buffer.length);

    if whole.count - start < needle.count return -1;

    i := start;
    last := whole.count - needle.count;

    while i <= last {
        if whole.data[i] == needle.data[0] {
            j := 1;
            while j < needle.count && whole.data[i + j] == needle.data[j] j += 1;
            if j == needle.count return i;
        }

        i += 1;
    }

    return -1;
}

#scope_module

piece_data :: (buffer: *Text_Buffer, piece: Piece) -> *u8 {
    if piece.from_added return buffer.added.data + piece.start;
    return buffer.original.data + piece.start;
}

ensure_lines :: (buffer: *Text_Buffer) {
    if !buffer.line_starts.count rebuild_lines(buffer);
}

rebuild_lines :: (buffer: *Text_Buffer) {
    array_reset(*buffer.line_starts);
    array_add(*buffer.line_starts, 0);

    base := 0;

    for piece: buffer.pieces {
        data := piece_data(buffer, piece);

        for i: 0..piece.length - 1 {
            if data[i] == #char "\n" array_add(*buffer.line_starts, base + i + 1);
        }

        base += piece.length;
    }
}

// Index of the piece containing `offset` and the offset of that piece's start; splits a piece when
// `offset` falls inside it so the result starts exactly at `offset`. Returns pieces.count at the end.
split_at :: (buffer: *Text_Buffer, offset: int) -> int {
    base := 0;

    for piece, i: buffer.pieces {
        if offset == base return i;

        if offset < base + piece.length {
            inside := offset - base;
            right := Piece.{ piece.from_added, piece.start + inside, piece.length - inside };

            buffer.pieces[i].length = inside;
            array_insert_at(*buffer.pieces, right, i + 1);

            return i + 1;
        }

        base += piece.length;
    }

    return buffer.pieces.count;
}

apply_insert :: (buffer: *Text_Buffer, offset: int, text: string) {
    ensure_lines(buffer);

    added_at := buffer.added.count;

    array_resize(*buffer.added, added_at + text.count);
    memcpy(buffer.added.data + added_at, text.data, text.count);

    index := split_at(buffer, offset);

    // Typing continues the previous added piece instead of making a new one.
    if index > 0 {
        previous := *buffer.pieces[index - 1];

        if previous.from_added && previous.start + previous.length == added_at {
            previous.length += text.count;
            index = -1;
        }
    }

    if index >= 0 array_insert_at(*buffer.pieces, .{ true, added_at, text.count }, index);

    // Line starts: shift everything after the insertion, then add the new breaks.
    line := line_of_offset(buffer, offset);
    for i: line + 1..buffer.line_starts.count - 1 buffer.line_starts[i] += text.count;

    at := line + 1;

    for i: 0..text.count - 1 {
        if text.data[i] == #char "\n" {
            array_insert_at(*buffer.line_starts, offset + i + 1, at);
            at += 1;
        }
    }

    buffer.length += text.count;
    buffer.version += 1;
}

apply_erase :: (buffer: *Text_Buffer, offset: int, count: int) {
    ensure_lines(buffer);

    first := split_at(buffer, offset);
    last := split_at(buffer, offset + count);

    // Remove the pieces in [first, last).
    removed := last - first;

    for i: first..buffer.pieces.count - 1 - removed buffer.pieces[i] = buffer.pieces[i + removed];

    buffer.pieces.count -= removed;

    // Line starts inside (offset, offset + count] disappear; later ones shift down.
    keep := 0;

    for start: buffer.line_starts {
        if start > offset && start <= offset + count continue;

        buffer.line_starts[keep] = ifx start > offset then start - count else start;
        keep += 1;
    }

    buffer.line_starts.count = keep;
    buffer.length -= count;
    buffer.version += 1;
}
