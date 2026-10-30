// Text_Buffer against a plain byte array: a property test over random edit sequences, then the
// line index, undo and redo (including merged typing and groups), and grapheme/word movement.
#import "Basic";
#import "Extensions/Testing";
#import "Extensions/Text_Buffer";

Op :: struct {
    kind: u8;
    at: u8;
    count: u8;
    text: string;
}

apply_to_model :: (model: *[..] u8, op: Op) {
    at := min(cast(int) op.at, model.count);

    if op.kind % 2 == 0 {
        for 0..op.text.count - 1 array_insert_at(model, op.text.data[it], at + it);
    } else {
        n := min(cast(int) op.count, model.count - at);
        for 0..n - 1 array_ordered_remove_by_index(model, at);
    }
}

matches_model :: (buffer: *Text_Buffer, model: [] u8) -> bool {
    text := buffer_to_string(buffer,, temp);
    if text.count != model.count return false;

    for model if text.data[it_index] != it return false;
    return true;
}

edits_match_a_byte_array :: (ops: [] Op) {
    buffer: Text_Buffer;

    defer free_buffer(*buffer);
    set_text(*buffer, "alpha\nbeta\n\ngamma");

    model: [..] u8;

    defer array_free(model);
    for "alpha\nbeta\n\ngamma" array_add(*model, it);

    for op: ops {
        at := min(cast(int) op.at, model.count);

        if op.kind % 2 == 0 {
            insert(*buffer, at, op.text);
        } else {
            erase(*buffer, at, cast(int) op.count);
        }

        apply_to_model(*model, op);
        require(matches_model(*buffer, model));
    }

    // The line index agrees with a scan of the model.
    lines := 1;
    for model if it == #char "\n" lines += 1;

    expect_equal(line_count(*buffer), lines);

    // Undoing every step restores the starting text.
    while can_undo(*buffer) undo(*buffer);

    expect_equal(buffer_to_string(*buffer,, temp), "alpha\nbeta\n\ngamma");
} @test @property(300)

lines_are_found_by_offset :: () {
    buffer: Text_Buffer;

    defer free_buffer(*buffer);
    set_text(*buffer, "one\ntwo\r\nthree");

    expect_equal(line_count(*buffer), 3);
    expect_equal(line_text(*buffer, 0), "one");
    expect_equal(line_text(*buffer, 1), "two");
    expect_equal(line_text(*buffer, 2), "three");
    expect_equal(line_of_offset(*buffer, 4), 1);
    expect_equal(line_of_offset(*buffer, 14), 2);

    insert(*buffer, 3, "\nx");
    expect_equal(line_count(*buffer), 4);
    expect_equal(line_text(*buffer, 1), "x");
    expect_equal(line_text(*buffer, 2), "two");

    erase(*buffer, 3, 2);
    expect_equal(line_count(*buffer), 3);
    expect_equal(line_text(*buffer, 1), "two");
} @test

an_empty_buffer_has_one_line :: () {
    buffer: Text_Buffer;

    defer free_buffer(*buffer);

    expect_equal(line_count(*buffer), 1);
    expect_equal(line_text(*buffer, 0), "");
    expect_equal(buffer.length, 0);
} @test

typing_merges_into_one_undo_step :: () {
    buffer: Text_Buffer;

    defer free_buffer(*buffer);

    for #char "a"..#char "e" {
        c: u8 = cast(u8) it;
        insert(*buffer, buffer.length, string.{ 1, *c });
    }

    expect_equal(buffer_to_string(*buffer,, temp), "abcde");

    ok, cursor := undo(*buffer);

    expect(ok);
    expect_equal(cursor, 0);
    expect_equal(buffer.length, 0);
    expect(!can_undo(*buffer));

    ok, cursor = redo(*buffer);
    expect(ok);
    expect_equal(cursor, 5);
    expect_equal(buffer_to_string(*buffer,, temp), "abcde");
} @test

sealing_and_groups_split_undo_steps :: () {
    buffer: Text_Buffer;

    defer free_buffer(*buffer);

    insert(*buffer, 0, "a");
    seal_undo(*buffer);
    insert(*buffer, 1, "b");
    expect_equal(buffer_to_string(*buffer,, temp), "ab");

    undo(*buffer);
    expect_equal(buffer_to_string(*buffer,, temp), "a");

    replace(*buffer, 0, 1, "xyz");
    expect_equal(buffer_to_string(*buffer,, temp), "xyz");
    undo(*buffer);
    expect_equal(buffer_to_string(*buffer,, temp), "a");
    redo(*buffer);
    expect_equal(buffer_to_string(*buffer,, temp), "xyz");
} @test

backspacing_merges :: () {
    buffer: Text_Buffer;

    defer free_buffer(*buffer);
    set_text(*buffer, "hello");

    erase(*buffer, 4, 1);
    erase(*buffer, 3, 1);
    erase(*buffer, 2, 1);
    expect_equal(buffer_to_string(*buffer,, temp), "he");

    undo(*buffer);
    expect_equal(buffer_to_string(*buffer,, temp), "hello");
} @test

finds_text_across_pieces :: () {
    buffer: Text_Buffer;

    defer free_buffer(*buffer);
    set_text(*buffer, "hello world");
    insert(*buffer, 5, ",");
    insert(*buffer, 0, ">> ");

    expect_equal(find_text(*buffer, "o, w"), 7);
    expect_equal(find_text(*buffer, "world", 8), 10);
    expect_equal(find_text(*buffer, "nope"), -1);
} @test

graphemes_stay_together :: () {
    // e + combining acute, a flag (two regional indicators), a family built with joiners, CR LF.
    text := "é|🇩🇪|👨‍👩‍👧|\r\n|x";

    expect_equal(next_grapheme(text, 0), 3);
    expect_equal(next_grapheme(text, 3), 4);
    expect_equal(next_grapheme(text, 4), 12);
    expect_equal(previous_grapheme(text, 12), 4);

    family := 13;
    after := next_grapheme(text, family);

    expect_equal(text.data[after], #char "|");
    expect_equal(previous_grapheme(text, after), family);

    crlf := after + 1;

    expect_equal(next_grapheme(text, crlf), crlf + 2);
    expect_equal(previous_grapheme(text, crlf + 2), crlf);

    expect_equal(grapheme_count("ab"), 2);
    expect_equal(grapheme_count("é"), 1);
} @test

words_move_by_class :: () {
    text := "foo_bar  += baz";

    expect_equal(next_word(text, 0), 7);
    expect_equal(next_word(text, 7), 11);
    expect_equal(next_word(text, 11), 15);
    expect_equal(previous_word(text, 15), 12);
    expect_equal(previous_word(text, 12), 9);
    expect_equal(previous_word(text, 9), 0);
} @test
