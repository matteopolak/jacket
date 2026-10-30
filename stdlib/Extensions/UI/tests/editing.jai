// Text editing through the model and through the widget: typing, selection, movement by character,
// word and line, Unicode, undo, the clipboard and scrolling to keep the caret visible.
#import "Basic";
#import "Input";
#import "Extensions/Testing";
#import "Extensions/UI";

Editor_App :: struct {
    edit: Text_Edit;
    changed: bool;
}

editor_build :: (app: *Editor_App) {
    for column() {
        if text_editor(*app.edit, .{ line_numbers = true }) app.changed = true;
    }
}

type :: (ui: *UI_State, app: *Editor_App, text: string) {
    send_text(ui, text);
    frame(ui, app, editor_build);
}

press :: (ui: *UI_State, app: *Editor_App, key: Key_Code, mods: Modifiers = .{}) {
    send_key(ui, key, mods);
    frame(ui, app, editor_build);
}

focused_editor :: (ui: *UI_State, app: *Editor_App) {
    frame(ui, app, editor_build);
    click(ui, find_role(ui, .TEXT_EDITOR));
    frame(ui, app, editor_build);
}

typing_edits_the_focused_editor :: () {
    app: Editor_App;

    defer free_text_edit(*app.edit);
    ui := headless(.{ width = 400, height = 300 });

    defer free_ui(*ui);

    // Without focus, typing does nothing.
    type(*ui, *app, "ignored");
    expect_equal(app.edit.buffer.length, 0);

    focused_editor(*ui, *app);
    type(*ui, *app, "hello");
    press(*ui, *app, .ENTER);
    type(*ui, *app, "world");

    expect_equal(contents(*app.edit), "hello\nworld");
    expect(app.changed);
    line, column := caret_position(*app.edit);

    expect_equal(line, 1);
    expect_equal(column, 5);
} @test

movement_by_character_word_and_line :: () {
    app: Editor_App;

    defer free_text_edit(*app.edit);
    ui := headless(.{ width = 400, height = 300 });

    defer free_ui(*ui);

    set_contents(*app.edit, "alpha beta\n  gamma delta");
    focused_editor(*ui, *app);

    // Clicking placed the caret somewhere; start from the top.
    move_caret(*app.edit, 0);

    press(*ui, *app, .ARROW_RIGHT);
    expect_equal(app.edit.cursor, 1);

    word := Modifiers.{ ctrl = true, alt = true };

    press(*ui, *app, .ARROW_RIGHT, word);
    expect_equal(app.edit.cursor, 5);
    press(*ui, *app, .ARROW_RIGHT, word);
    expect_equal(app.edit.cursor, 10);

    // Down keeps the column; End and Home go to the ends of the line.
    press(*ui, *app, .ARROW_DOWN);
    line, _ := caret_position(*app.edit);

    expect_equal(line, 1);
    press(*ui, *app, .END);
    expect_equal(app.edit.cursor, app.edit.buffer.length);
    press(*ui, *app, .HOME);
    expect_equal(app.edit.cursor, 13); // after the leading blanks first
    press(*ui, *app, .HOME);
    expect_equal(app.edit.cursor, 11);

    // Right at the end of a line wraps to the next one.
    move_caret(*app.edit, 10);
    press(*ui, *app, .ARROW_RIGHT);
    expect_equal(app.edit.cursor, 11);
} @test

selection_and_replacement :: () {
    app: Editor_App;

    defer free_text_edit(*app.edit);
    ui := headless(.{ width = 400, height = 300 });

    defer free_ui(*ui);

    set_contents(*app.edit, "one two three");
    focused_editor(*ui, *app);
    move_caret(*app.edit, 4);

    shift := Modifiers.{ shift = true };

    press(*ui, *app, .ARROW_RIGHT, shift);
    press(*ui, *app, .ARROW_RIGHT, shift);
    press(*ui, *app, .ARROW_RIGHT, shift);
    expect_equal(selected_text(*app.edit), "two");

    type(*ui, *app, "2");
    expect_equal(contents(*app.edit), "one 2 three");

    press(*ui, *app, .BACKSPACE);
    expect_equal(contents(*app.edit), "one  three");

    // Cmd or Ctrl+A selects everything and typing replaces it.
    command := Modifiers.{ ctrl = true, super = true };

    press(*ui, *app, cast(Key_Code) #char "A", command);
    expect_equal(selected_text(*app.edit), "one  three");
    type(*ui, *app, "x");
    expect_equal(contents(*app.edit), "x");
} @test

unicode_is_moved_and_deleted_by_grapheme :: () {
    app: Editor_App;

    defer free_text_edit(*app.edit);
    ui := headless(.{ width = 400, height = 300 });

    defer free_ui(*ui);

    // a, e with a combining accent, a flag, b.
    set_contents(*app.edit, "aé🇩🇪b");
    focused_editor(*ui, *app);
    move_caret(*app.edit, 0);

    press(*ui, *app, .ARROW_RIGHT);
    expect_equal(app.edit.cursor, 1);
    press(*ui, *app, .ARROW_RIGHT);
    expect_equal(app.edit.cursor, 4);
    press(*ui, *app, .ARROW_RIGHT);
    expect_equal(app.edit.cursor, 12);

    press(*ui, *app, .BACKSPACE);
    expect_equal(contents(*app.edit), "aéb");
    press(*ui, *app, .BACKSPACE);
    expect_equal(contents(*app.edit), "ab");
} @test

undo_and_redo_follow_the_typing :: () {
    app: Editor_App;

    defer free_text_edit(*app.edit);
    ui := headless(.{ width = 400, height = 300 });

    defer free_ui(*ui);

    focused_editor(*ui, *app);
    type(*ui, *app, "abc");
    press(*ui, *app, .ARROW_LEFT);
    type(*ui, *app, "X");
    expect_equal(contents(*app.edit), "abXc");

    command := Modifiers.{ ctrl = true, super = true };
    z := cast(Key_Code) #char "Z";

    press(*ui, *app, z, command);
    expect_equal(contents(*app.edit), "abc");
    press(*ui, *app, z, command);
    expect_equal(contents(*app.edit), "");

    press(*ui, *app, z, .{ ctrl = true, super = true, shift = true });
    expect_equal(contents(*app.edit), "abc");
} @test

enter_keeps_indentation :: () {
    app: Editor_App;

    defer free_text_edit(*app.edit);
    ui := headless(.{ width = 400, height = 300 });

    defer free_ui(*ui);

    set_contents(*app.edit, "    indented");
    focused_editor(*ui, *app);
    move_caret(*app.edit, 12);
    press(*ui, *app, .ENTER);
    type(*ui, *app, "next");
    expect_equal(contents(*app.edit), "    indented\n    next");

    press(*ui, *app, .TAB);
    expect_equal(contents(*app.edit), "    indented\n    next    ");
} @test

clipboard_copy_cut_and_paste :: () {
    app: Editor_App;

    defer free_text_edit(*app.edit);
    ui := headless(.{ width = 400, height = 300 });

    defer free_ui(*ui);

    ui.hooks.clipboard_set = test_clipboard_set;
    ui.hooks.clipboard_get = test_clipboard_get;

    defer {
        free(test_clipboard.data);
        test_clipboard = "";
    }

    set_contents(*app.edit, "copy me");
    focused_editor(*ui, *app);
    select_all(*app.edit);

    command := Modifiers.{ ctrl = true, super = true };

    press(*ui, *app, cast(Key_Code) #char "X", command);
    expect_equal(test_clipboard, "copy me");
    expect_equal(contents(*app.edit), "");

    press(*ui, *app, cast(Key_Code) #char "V", command);
    press(*ui, *app, cast(Key_Code) #char "V", command);
    expect_equal(contents(*app.edit), "copy mecopy me");
} @test

the_view_scrolls_to_keep_the_caret_visible :: () {
    app: Editor_App;

    defer free_text_edit(*app.edit);
    ui := headless(.{ width = 400, height = 100 });

    defer free_ui(*ui);

    builder: String_Builder;
    for 0..99 print_to_builder(*builder, "line %\n", it);

    set_contents(*app.edit, builder_to_string(*builder,, temp));

    focused_editor(*ui, *app);
    move_caret(*app.edit, 0);
    frame(*ui, *app, editor_build);
    expect_equal(app.edit.scroll_y, 0.0);

    press(*ui, *app, .ARROW_DOWN, .{ super = true });
    press(*ui, *app, .PAGE_DOWN);
    press(*ui, *app, .PAGE_DOWN);

    // The caret's line is inside the 100 point viewport (13 point code at 1.25 line height).
    line, _ := caret_position(*app.edit);
    top := line * 16.25;

    expect(app.edit.scroll_y > 0);
    expect(top >= app.edit.scroll_y - 0.01);
    expect(top + 16.25 <= app.edit.scroll_y + 100.01);
} @test

test_clipboard: string;

test_clipboard_set :: (text: string) {
    free(test_clipboard.data);
    test_clipboard = copy_string(text);
}

test_clipboard_get :: () -> string {
    return test_clipboard;
}
