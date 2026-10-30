// A scripted editing session against the editor with no window: the file tree, opening, typing,
// saving, undo, search, tabs, the clipboard and the configuration, then one frame drawn on the GPU
// with system fonts. Run it with `just ui-editor-smoke` (or
// `jaic examples/editor/smoke.jai -interp`). It exits non-zero on the first failed check.

#import "String";
#load "editor.jai";

failures := 0;

check :: (ok: bool, what: string) {
    if ok {
        print("ok    %\n", what);
    } else {
        print("FAIL  %\n", what);
        failures += 1;
    }
}

clipboard_text: string;

clipboard_get :: () -> string {
    return clipboard_text;
}

clipboard_set :: (text: string) {
    free(clipboard_text.data);
    clipboard_text = copy_string(text);
}

both_modifiers :: Modifiers.{ ctrl = true, super = true };

step :: (ui: *UI_State, ed: *Editor) {
    frame(ui, ed, build);
}

press :: (ui: *UI_State, ed: *Editor, key: Key_Code, mods: Modifiers = .{}) {
    send_key(ui, key, mods);
    step(ui, ed);
    step(ui, ed);
}

type :: (ui: *UI_State, ed: *Editor, text: string) {
    send_text(ui, text);
    step(ui, ed);
}

shortcut :: (ui: *UI_State, ed: *Editor, letter: u8) {
    press(ui, ed, cast(Key_Code) letter, both_modifiers);
}

read_text :: (path: string) -> string {
    text, ok := read_entire_file(path);
    if !ok return "";
    return text;
}

main :: () {
    base := environment_variable("TMPDIR");
    if !base.count base = "/tmp";

    root := tprint("%/jacket-editor-smoke-%", base, to_milliseconds(current_time_monotonic()));

    make_directory_if_it_does_not_exist(root);

    hello := tprint("%/hello.jai", root);
    notes := tprint("%/notes.txt", root);
    config := tprint("%/config.json", root);

    write_entire_file(hello, "main :: () {\n    print(\"hi\");\n}\n");
    write_entire_file(notes, "alpha beta\nalpha 42\nbeta 7\n");

    ed: Editor;

    open_editor(*ed, root);
    ed.config_file = copy_string(config);

    ui := headless(.{ width = 900, height = 600 });
    ui.hooks.clipboard_get = clipboard_get;
    ui.hooks.clipboard_set = clipboard_set;

    step(*ui, *ed);
    step(*ui, *ed);

    // The tree lists the folder, and clicking an entry opens it in a tab.
    check(find_label(*ui, "  hello.jai") != null, "tree lists hello.jai");
    check(find_label(*ui, "  notes.txt") != null, "tree lists notes.txt");
    check(find_label(*ui, "No file open") != null, "empty state before opening");

    click(*ui, find_label(*ui, "  hello.jai"));
    step(*ui, *ed);
    step(*ui, *ed);
    check(ed.docs.count == 1, "click opens a tab");
    check(find_label(*ui, "hello.jai") != null, "tab is titled hello.jai");

    // Type into the editor, then save.
    click(*ui, find_role(*ui, .TEXT_EDITOR));
    step(*ui, *ed);
    press(*ui, *ed, .HOME, .{ ctrl = true, super = true });
    type(*ui, *ed, "// header");
    press(*ui, *ed, .ENTER);
    doc := active_document(*ed);

    check(is_dirty(doc), "typing marks the document modified");
    check(begins_with(contents(*doc.edit), "// header\nmain"), "typed text reached the buffer");

    shortcut(*ui, *ed, #char "S");
    check(!is_dirty(doc), "Cmd+S clears the modified flag");
    check(begins_with(read_text(hello), "// header\nmain"), "Cmd+S wrote the file");

    // Undo and redo.
    shortcut(*ui, *ed, #char "Z");
    check(is_dirty(doc), "undo makes the saved text stale");
    check(contents(*doc.edit) != read_text(hello), "undo changed the text");
    press(*ui, *ed, cast(Key_Code) #char "Z", .{ ctrl = true, super = true, shift = true });

    // Clipboard: select all, copy, then paste at the end.
    shortcut(*ui, *ed, #char "A");
    shortcut(*ui, *ed, #char "C");
    check(begins_with(clipboard_text, "// header"), "copy fills the clipboard");
    press(*ui, *ed, .END, .{ ctrl = true, super = true });
    shortcut(*ui, *ed, #char "V");
    check(doc.edit.buffer.length > clipboard_text.count, "paste inserts the clipboard text");

    // A second document, a tab switch and the unsaved-changes guard on close.
    open_document(*ed, "notes.txt");
    step(*ui, *ed);
    step(*ui, *ed);
    check(ed.docs.count == 2 && ed.active == 1, "second tab opens and is active");

    // Search: plain text, then a regex.
    shortcut(*ui, *ed, #char "F");
    step(*ui, *ed);
    click(*ui, find_role(*ui, .TEXT_FIELD));
    step(*ui, *ed);
    type(*ui, *ed, "alpha");
    step(*ui, *ed);
    check(ed.search_marks.count == 2, "plain search finds two matches");

    ed.search_regex = true;
    free_text_edit(*ed.search_query);
    ed.search_query.single_line = true;
    set_contents(*ed.search_query, "[0-9]+");
    step(*ui, *ed);
    step(*ui, *ed);
    check(ed.search_marks.count == 2, "regex search finds the two numbers");

    set_contents(*ed.search_query, "(");
    step(*ui, *ed);
    step(*ui, *ed);
    check(ed.search_error.count > 0, "invalid regex reports an error");
    ed.search_open = false;

    // Closing the modified first tab needs a second request.
    ed.active = 0;
    close_active(*ed);
    check(ed.docs.count == 2, "closing a modified tab asks first");
    close_active(*ed);
    check(ed.docs.count == 1, "the second request closes it");

    // Files appear on disk and the tree picks them up.
    write_entire_file(tprint("%/later.jai", root), "x :: 1;\n");
    ed.tree_dirty = true;
    step(*ui, *ed);
    step(*ui, *ed);
    check(find_label(*ui, "  later.jai") != null, "new file shows in the tree");

    // The configuration survives a round trip.
    ed.config.font_size = 17;
    ed.config.tab_width = 2;
    save_config(ed.config, config);
    loaded := load_config(config);

    check(loaded.font_size == 17 && loaded.tab_width == 2, "config round trips through JSON");
    check(load_config(tprint("%/missing.json", root)).tab_width == 4, "a missing config gives the defaults");

    // One frame on the GPU with real fonts: the picture must contain more than the background.
    gpu := headless(.{ width = 900, height = 600, system_fonts = true });

    ed.active = 0;
    frame(*gpu, *ed, build);
    frame(*gpu, *ed, build);
    shot := tprint("%/frame.png", root);

    if write_screenshot(*gpu, shot) {
        bytes, ok := read_entire_file(shot);
        check(ok && bytes.count > 4000, "GPU frame rendered to a PNG");
    } else {
        print("skip  no GPU adapter, screenshot not taken\n");
    }

    free_ui(*gpu);
    free_ui(*ui);
    close_editor(*ed);
    free(clipboard_text.data);

    delete_directory_tree(root);

    if failures {
        print("\n% check(s) failed\n", failures);
        exit(1);
    }

    print("\nall checks passed\n");
}
