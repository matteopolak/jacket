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

run_smoke :: (gpu_frame: bool = false) {
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
    if !ed.language.running check(Str.contains(ed.status, "not found"), "missing server leaves an editor status message");

    free(ed.config_file.data);
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
    saved_file := read_text(hello);

    defer free(saved_file);
    check(begins_with(saved_file, "// header\nmain"), "Cmd+S wrote the file");

    // IME preedit is transient; only the committed text enters the buffer and undo history.
    saved := copy_string(contents(*doc.edit));

    send_composition(*ui, "か");
    step(*ui, *ed);
    check(doc.edit.composing && doc.edit.composition == "か", "marked text reaches the editor");
    check(!is_dirty(doc) && contents(*doc.edit) == saved, "preedit leaves saved contents unchanged");
    send_composition(*ui, "かな");
    step(*ui, *ed);
    check(doc.edit.composition == "かな", "marked text updates across frames");
    send_composition(*ui, "仮名", commit = true);
    step(*ui, *ed);
    check(!doc.edit.composing && is_dirty(doc), "IME commit modifies the document");
    check(undo_edit(*doc.edit) && contents(*doc.edit) == saved, "one undo removes the complete IME commit");
    send_composition(*ui, "discard");
    step(*ui, *ed);
    send_composition(*ui, "", cancel = true);
    step(*ui, *ed);
    check(!doc.edit.composing && contents(*doc.edit) == saved, "cancel discards marked text");
    free(saved.data);

    // Undo and redo.
    shortcut(*ui, *ed, #char "Z");
    check(is_dirty(doc), "undo makes the saved text stale");
    after_undo := read_text(hello);

    defer free(after_undo);
    check(contents(*doc.edit) != after_undo, "undo changed the text");
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

    // Native menu models and their routing are tested without opening a menu or dialog.
    check(valid_menu(editor_menus), "native menu model has unique IDs and shortcuts");
    before := ed.config.line_numbers;

    send_menu_action(*ui, editor_menus, 13);
    step(*ui, *ed);
    check(ed.config.line_numbers != before, "View menu action follows the shortcut path once");
    apply_dialog_result(*ed, .SAVE, .{ status = .CANCELLED });
    check(!ed.quick_open_open, "cancelled dialog does not open the fallback");
    apply_dialog_result(*ed, .SAVE, .{ status = .UNSUPPORTED });
    check(ed.quick_open_open && ed.prompt_kind == .SAVE, "unsupported save dialog opens Save As prompt");
    ed.quick_open_open = false;
    renamed := tprint("%/renamed.txt", root);

    check(save_document_as(*ed, renamed), "Save As writes a new path");
    canonical_saved_path, canonical_saved_ok := get_absolute_path(renamed);

    defer free(canonical_saved_path);
    check(canonical_saved_ok && active_document(*ed).path == canonical_saved_path && file_exists(renamed), "Save As updates the canonical tab identity");

    if gpu_frame {
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
    } else {
        print("skip  GPU frame (set EDITOR_SMOKE_GPU=1 to enable)\n");
    }

    // Real font metrics exercise the shared kerning/caret path without needing a GPU.
    metrics := headless();

    free_text_system(*metrics.text);

    #if OS == .MACOS {
        loaded_font := load_font(*metrics.text, "/System/Library/Fonts/Supplemental/Arial.ttf");
        check(loaded_font, "proportional kerning font loads");
    } else load_system_fonts(*metrics.text);

    style := Text_Style.{ size = 48 };
    adjustment := pair_kerning(*metrics.text, #char "A", #char "V", style);

    #if OS == .MACOS check(adjustment < 0, "AV pair has negative kerning");
    width := text_width(*metrics.text, "AV", style);

    check(width == glyph_advance(*metrics.text, #char "A", style) + glyph_advance(*metrics.text, #char "V", style) + adjustment, "kerning is included in measured width");
    check(draw_text(*metrics, 0, 0, "AV", style) == width && index_at_x(*metrics.text, "AV", style, width) == 2, "kerning drawing and caret hit testing agree");
    style.font = .MONO;
    check(pair_kerning(*metrics.text, #char "A", #char "V", style) == 0, "monospace excludes pair kerning");
    free_ui(*metrics);

    language_smoke(*ui, *ed, root);

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

// Live server coverage uses exactly the same frame pump and request handlers as the window.
language_smoke :: (ui: *UI_State, ed: *Editor, root: string) {
    if !ed.language.running {
        check(environment_variable("EDITOR_REQUIRE_LSP") != "1", "jailsp absent (optional language smoke skipped)");
        return;
    }

    source := "identity :: (value: int) -> int { return value; }\nmain :: () {\n    result := identity(7);\n    bad: int = \"wrong\";\n}\n";
    path := tprint("%/language.jai", root);

    write_entire_file(path, source);
    doc := open_document(ed, path);

    check(open_document(ed, tprint("%/./language.jai", root)) == doc, "canonical paths share a document and LSP URI");

    for 0..1499 {
        step(ui, ed);
        if doc.diagnostics.items.count || ed.language.failed break;
        sleep_milliseconds(10);
    }

    check(ed.language.ready && !ed.language.failed, "jailsp initializes over stdio");
    type_errors := 0;
    for *diagnostic: doc.diagnostics.items if Lsp.number(Lsp.get(diagnostic, "severity")) == 1 type_errors += 1;

    check(type_errors > 0 && doc.diagnostic_marks.count > 0, "type error arrives as diagnostics and squiggles");
    doc.edit.cursor = find_index_from_left(source, "identity(7)") + 2;
    doc.edit.anchor = doc.edit.cursor;
    language_request(ed, "textDocument/hover");
    wait_language(ui, ed);
    check(contains(ed.hover, "identity"), "hover text arrives from jailsp");
    origin := doc.edit.cursor;

    language_request(ed, "textDocument/definition");
    wait_language(ui, ed);
    check(active_document(ed) == doc && doc.edit.cursor == 0, "definition selects the declaration");
    canonical_origin, canonical_origin_ok := get_absolute_path(path);

    defer free(canonical_origin);
    check(canonical_origin_ok && ed.back_path == canonical_origin && ed.back_cursor == origin, "definition records the canonical return location");
    press(ui, ed, .ARROW_LEFT, .{ alt = true });
    check(doc.edit.cursor == origin, "Alt+Left returns from definition");
    next := ed.language.next_id;

    language_request(ed, "textDocument/hover");
    language_request(ed, "textDocument/completion");
    check(ed.language.pending.count == 1 && ed.language.next_id == next + 2, "new request cancels its predecessor and uses a fresh id");
    wait_language(ui, ed);
    check(ed.completions.items.count > 0, "completion items arrive from jailsp");
    set_contents(*ed.completion_filter, "identity");
    step(ui, ed);
    item := find_label(ui, "identity");

    check(item != null, "completion filter shows the matching item");

    if item {
        click(ui, item);
        step(ui, ed);
        check(contents(*doc.edit) == source && doc.edit.cursor == find_index_from_left(source, "identity(7)") + 8, "completion click replaces the whole identifier using its textEdit");
        undo_edit(*doc.edit);
    }

    // A late response from an older version must never change the hover.
    clear_language_popup(ed);
    array_add(*ed.language.pending, .{ id = 999999, method = copy_string("textDocument/hover"), uri = copy_string(Lsp.uri(doc.path)), version = doc.edit.buffer.version - 1, cursor = doc.edit.cursor });
    stale, _ := from_json(Json_Value, "{\"id\":999999,\"result\":{\"contents\":\"stale hover\"}}");

    receive_language(ed, *stale);
    free_json(*stale);
    check(ed.hover.count == 0, "late response for an old document version is discarded");
    // Unsaved incremental replacement removes the error, including shifted UTF-16 columns.
    select_all(*doc.edit);
    insert_text(*doc.edit, "// é😀\nidentity :: (value: int) -> int { return value; }\nmain :: () { result := identity(7); }\n");

    for 0..1499 {
        step(ui, ed);
        if doc.diagnostic_version == doc.edit.buffer.version break;
        sleep_milliseconds(10);
    }

    errors := 0;
    for *diagnostic: doc.diagnostics.items if Lsp.number(Lsp.get(diagnostic, "severity")) == 1 errors += 1;

    check(doc.diagnostic_version == doc.edit.buffer.version && errors == 0, "unsaved incremental edit clears the error");
    before := copy_string(contents(*doc.edit));

    format_document(ed);
    check(Str.contains(ed.status, "formatted"), "format document uses the shared formatter");
    undo_edit(*doc.edit);
    check(contents(*doc.edit) == before, "format document is one undoable edit");
    free(before);
    renamed := tprint("%/renamed-language.jai", root);

    check(save_document_as(ed, renamed), "Save As succeeds for an LSP document");
    canonical_renamed, canonical_ok := get_absolute_path(renamed);

    defer free(canonical_renamed);
    check(canonical_ok && doc.path == canonical_renamed && doc.lsp_version == doc.edit.buffer.version && ed.language.pending.count == 0, "Save As opens the new canonical LSP identity and retires old requests");
    check(open_document(ed, renamed) == doc, "reopening Save As through an aliased root shares the document");
    cross_file_language_smoke(ui, ed, root);
    watched_dependency_language_smoke(ui, ed, root);
}

// Cross-file definition and return use filesystem identity, including aliased TMPDIR roots.
cross_file_language_smoke :: (ui: *UI_State, ed: *Editor, root: string) {
    target_path := tprint("%/navigation-target.jai", root);
    source_path := tprint("%/navigation-source.jai", root);

    write_entire_file(target_path, "destination :: () {}\n");
    source := "// é😀\n#load \"navigation-target.jai\";\nmain :: () { /* é😀 */ destination(); }\n";

    write_entire_file(source_path, source);
    doc := open_document(ed, source_path);

    for 0..1499 {
        step(ui, ed);
        if doc.diagnostic_version == doc.edit.buffer.version || ed.language.failed break;
        sleep_milliseconds(10);
    }

    origin := find_index_from_left(source, "destination();") + 2;

    doc.edit.cursor = origin;
    doc.edit.anchor = origin;
    expected_source, source_ok := get_absolute_path(source_path);

    defer free(expected_source);
    expected_target, target_ok := get_absolute_path(target_path);

    defer free(expected_target);
    language_request(ed, "textDocument/definition");
    wait_language(ui, ed);
    target := active_document(ed);

    check(source_ok && target_ok && target && target.path == expected_target && target.edit.cursor == 0, "cross-file definition opens the canonical target");
    check(ed.back_path == expected_source && ed.back_cursor == origin, "cross-file definition records the exact origin");
    press(ui, ed, .ARROW_LEFT, .{ alt = true });
    check(active_document(ed) == doc && doc.edit.cursor == origin, "cross-file Alt+Left returns to the original document and UTF-16 cursor");

    unsaved_path := tprint("%/unsaved-navigation.jai", root);
    unsaved := open_document(ed, unsaved_path);

    insert_text(*unsaved.edit, "main :: () {}\n");
    check(save_document(ed), "an unsaved document below an aliased root saves");
    check(open_document(ed, unsaved_path) == unsaved, "saving an unsaved document preserves its canonical identity");
    check(parent_directory("C:\\name") == "C:\\", "Windows drive root remains absolute when finding a parent");
}

wait_language :: (ui: *UI_State, ed: *Editor) {
    for 0..1499 {
        step(ui, ed);
        if ed.language.pending.count == 0 || ed.language.failed break;
        sleep_milliseconds(10);
    }
}

// Root watcher events must refresh a dependent open document without opening the changed file.
watched_dependency_language_smoke :: (ui: *UI_State, ed: *Editor, root: string) {
    dependency := tprint("%/unopened-dependency.jai", root);
    source_path := tprint("%/watched-source.jai", root);

    write_entire_file(dependency, "unopened :: () {}\n");
    write_entire_file(source_path, "#load \"unopened-dependency.jai\"; main :: () { unopened(); }\n");
    doc := open_document(ed, source_path);

    for 0..1499 {
        step(ui, ed);
        if doc.diagnostic_version == doc.edit.buffer.version || ed.language.failed break;
        sleep_milliseconds(10);
    }

    check(doc.diagnostic_version == doc.edit.buffer.version && doc.diagnostics.items.count == 0, "unopened dependency starts checked without diagnostics");
    write_entire_file(dependency, "replacement :: () {}\n");

    for 0..1499 {
        step(ui, ed);
        if doc.diagnostics.items.count > 0 || ed.language.failed break;
        sleep_milliseconds(10);
    }

    check(doc.diagnostics.items.count > 0, "root watcher proactively refreshes diagnostics for an unopened dependency");
}
