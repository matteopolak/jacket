#import "Basic";
#import "String";
#import "Extensions/Testing";
#import,dir "../vt";

cursor_erase_and_attributes :: () {
    g := make_grid(8, 3);

    defer free_grid(*g);
    feed(*g, "abcdef\x1b[1;3H\x1b[31;1;3;4;7mZ\x1b[K");
    text := row_text(*g, 0);

    defer free(text);
    expect_equal(text, "abZ     ");
    c := g.cells[2];

    expect_equal(c.attr.fg, palette(1));
    expect(c.attr.bold && c.attr.italic && c.attr.underline && c.attr.inverse);
    feed(*g, "\x1b[0m\x1b[38;5;196;48;2;1;2;3m!");
    expect_equal(g.cells[3].attr.fg, cast(u32) 0xff0000);
    expect_equal(g.cells[3].attr.bg, cast(u32) 0x010203);
} @test

scroll_regions_and_history :: () {
    g := make_grid(4, 4);

    defer free_grid(*g);
    feed(*g, "1111\r\n2222\r\n3333\r\n4444\x1b[2;3r\x1b[3;1H\n");
    expect_equal(g.cells[0].code, cast(u32) #char "1");
    expect_equal(g.cells[4].code, cast(u32) #char "3");
    expect_equal(g.cells[8].code, cast(u32) 32);
    expect_equal(g.cells[12].code, cast(u32) #char "4");
    expect_equal(g.history.count, 0);
    feed(*g, "\x1b[r\x1b[4;1H\n");
    expect_equal(g.history.count, 4);
    expect_equal(g.history[0].code, cast(u32) #char "1");
} @test

alternate_title_and_split_utf8 :: () {
    g := make_grid(8, 2);

    defer free_grid(*g);
    feed(*g, "abc\x1b]2;session\x1b");
    feed(*g, "\\\x1b[?2004h\x1b[?1049h\xe7\x95");
    feed(*g, "\x8c!");
    expect_equal(g.title, "session");
    expect(g.bracketed && g.alternate);
    expect_equal(g.cells[0].code, cast(u32) 0x754c);
    expect_equal(g.cells[0].width, 2);
    expect_equal(g.cells[1].width, 0);
    feed(*g, "\x1b[?1049lZ");
    text := row_text(*g, 0);

    defer free(text);
    expect_equal(text, "abcZ    ");
} @test

insert_delete_and_wide_overwrite :: () {
    g := make_grid(8, 3);

    defer free_grid(*g);
    feed(*g, "abcdef\r\x1b[2C\x1b[2@XY\x1b[P");
    text := row_text(*g, 0);

    defer free(text);
    expect_equal(text, "abXYdef ");
    feed(*g, "\r界\x1b[1;2HZ");
    expect_equal(g.cells[0].code, cast(u32) 32);
    expect_equal(g.cells[1].code, cast(u32) #char "Z");
    feed(*g, "\x1b[2;1Hline2\x1b[3;1Hline3\x1b[2;1H\x1b[L");
    expect_equal(g.cells[8].code, cast(u32) 32);
    expect_equal(g.cells[16].code, cast(u32) #char "l");
    feed(*g, "\x1b[H\x1b[M");
    expect_equal(g.history.count, 0);
} @test

malformed_and_bounded_sequences :: () {
    g := make_grid(4, 2);

    defer free_grid(*g);
    feed(*g, "\xf0\x80\x80\x80x\xc2y");
    expect_equal(g.cells[0].code, cast(u32) 0xfffd);
    expect_equal(g.cells[2].code, cast(u32) 0xfffd);
    feed(*g, "\x1b[999999999999999999999999H");
    expect_equal(g.y, 1);
    resize_grid(*g, 1, 1);
    feed(*g, "界");
    expect_equal(g.cells[0].width, 1);
} @test

stream_chunk_boundaries_do_not_change_the_snapshot :: () {
    bytes := "ab\x1b[2;3H\x1b[38;2;90;80;70m界é\x1b]0;title\x07\r\nlast";
    whole := make_grid(12, 4);

    defer free_grid(*whole);
    feed(*whole, bytes);

    for split: 0..bytes.count {
        chunked := make_grid(12, 4);

        defer free_grid(*chunked);
        feed(*chunked, slice(bytes, 0, split));
        feed(*chunked, slice(bytes, split, bytes.count - split));

        for i: 0..whole.cells.count - 1 {
            expect_equal(chunked.cells[i].code, whole.cells[i].code);
            expect_equal(chunked.cells[i].attr.fg, whole.cells[i].attr.fg);
            expect_equal(chunked.cells[i].width, whole.cells[i].width);
        }

        expect_equal(chunked.title, whole.title);
        expect_equal(chunked.x, whole.x);
        expect_equal(chunked.y, whole.y);
    }
} @test

history_is_bounded_and_alternate_resize_keeps_primary :: () {
    g := make_grid(4, 2);

    defer free_grid(*g);
    g.history_limit = 2;
    feed(*g, "one\r\ntwo\r\ntri\r\nfour\r\n");
    expect_equal(g.history.count, 8);
    expect_equal(history_cell(*g, 0).code, cast(u32) #char "t");
    visible := g.cells[0].code;

    feed(*g, "\x1b[3J");
    expect_equal(g.history.count, 0);
    expect_equal(g.cells[0].code, visible);
    feed(*g, "\x1b[?1049halt");
    resize_grid(*g, 3, 3);
    feed(*g, "\x1b[?1049l");
    expect_equal(g.cells[0].code, cast(u32) #char "f");
    expect_equal(g.history.count, 0);
    expect(!g.alternate);
} @test

cursor_queries_and_delayed_wrap :: () {
    g := make_grid(4, 2);

    defer free_grid(*g);
    feed(*g, "abcd\x1b[31me\x1b[6n\x1b[5n\x1b[?1h");
    expect_equal(g.cells[4].code, cast(u32) #char "e");
    expect_equal(string.{ g.replies.count, g.replies.data }, "\x1b[2;2R\x1b[0n");
    expect(g.application_cursor);
} @test
