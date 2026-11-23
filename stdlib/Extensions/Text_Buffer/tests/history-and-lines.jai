// Text_Buffer line lookups at line ends for LF and CRLF text, and undo and redo for typed runs, deleted runs and groups.
#import "Basic";
#import "Extensions/Text_Buffer";

show :: (buffer: *Text_Buffer) -> string {
    return buffer_to_string(buffer,, temp);
}

main :: () {
    // Line lookups. Offsets: a0 b1 \r2 \n3 c4 d5 \n6 \n7 l8 a9 s10 t11, length 12.
    lines: Text_Buffer;

    defer free_buffer(*lines);
    set_text(*lines, "ab\r\ncd\n\nlast");
    print("lines=%\n", line_count(*lines));
    assert(line_count(*lines) == 4);

    for 0..3 print("line % start=% end=% text=[%]\n", it, line_start(*lines, it), line_end(*lines, it), line_text(*lines, it));

    assert(line_start(*lines, 0) == 0 && line_end(*lines, 0) == 2);
    assert(line_start(*lines, 1) == 4 && line_end(*lines, 1) == 6);
    assert(line_start(*lines, 2) == 7 && line_end(*lines, 2) == 7);
    assert(line_start(*lines, 3) == 8 && line_end(*lines, 3) == 12);

    // Past the last line clamps to the last line, and its end is the buffer length.
    assert(line_start(*lines, 99) == 8 && line_end(*lines, 99) == 12);

    // An offset on a line break belongs to the line before it.
    print("offset 2 line=% offset 3 line=% offset 7 line=% offset 12 line=%\n", line_of_offset(*lines, 2), line_of_offset(*lines, 3), line_of_offset(*lines, 7), line_of_offset(*lines, 12));
    assert(line_of_offset(*lines, 2) == 0 && line_of_offset(*lines, 3) == 0);
    assert(line_of_offset(*lines, 6) == 1 && line_of_offset(*lines, 7) == 2 && line_of_offset(*lines, 12) == 3);

    // Typed runs merge into one undo step; a cursor move (seal_undo) starts a new one.
    typed: Text_Buffer;

    defer free_buffer(*typed);
    insert(*typed, 0, "a");
    insert(*typed, 1, "b");
    insert(*typed, 2, "c");
    seal_undo(*typed);
    insert(*typed, 3, "!");
    print("typed=%\n", show(*typed));
    assert(show(*typed) == "abc!");

    ok, cursor := undo(*typed);

    print("undo1 ok=% cursor=% text=%\n", ok, cursor, show(*typed));
    assert(ok && cursor == 3 && show(*typed) == "abc");

    ok, cursor = undo(*typed);
    print("undo2 ok=% cursor=% text=[%] can_undo=%\n", ok, cursor, show(*typed), can_undo(*typed));
    assert(ok && cursor == 0 && show(*typed) == "" && !can_undo(*typed));

    ok, cursor = redo(*typed);
    assert(ok && show(*typed) == "abc");
    ok, cursor = redo(*typed);
    print("redo2 ok=% cursor=% text=%\n", ok, cursor, show(*typed));
    assert(ok && cursor == 4 && show(*typed) == "abc!" && !can_redo(*typed));

    // Backspace runs merge: erasing "f" then "e" is one step that restores "ef".
    deleted: Text_Buffer;

    defer free_buffer(*deleted);
    set_text(*deleted, "abcdef");
    erase(*deleted, 5, 1);
    erase(*deleted, 4, 1);
    print("after erases=%\n", show(*deleted));
    assert(show(*deleted) == "abcd");
    ok, cursor = undo(*deleted);
    print("undo erases ok=% cursor=% text=%\n", ok, cursor, show(*deleted));
    assert(ok && show(*deleted) == "abcdef" && cursor == 6);

    // A group is one undo step however many edits it holds.
    grouped: Text_Buffer;

    defer free_buffer(*grouped);
    set_text(*grouped, "base");
    begin_group(*grouped);
    insert(*grouped, 0, "x");
    insert(*grouped, 0, "y");
    erase(*grouped, 4, 2);
    end_group(*grouped);
    print("grouped=%\n", show(*grouped));
    assert(show(*grouped) == "yxba");
    undo(*grouped);
    print("grouped undone=%\n", show(*grouped));
    assert(show(*grouped) == "base");
    print("ok\n");
}
