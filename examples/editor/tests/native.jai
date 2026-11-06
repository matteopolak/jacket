#import "Basic";
#import "Extensions/Testing";
#load "../editor.jai";

menu_model_has_unique_commands_and_shortcuts :: (t: *Test) {
    expect(valid_menu(editor_menus));
    expect_equal(editor_menus.count, 13);
    expect_equal(editor_menus[0].menu, "File");
    expect_equal(editor_menus[5].menu, "Edit");
    expect_equal(editor_menus[11].menu, "View");
} @test

cancel_does_not_open_fallback :: (t: *Test) {
    ed: Editor;
    ed.quick_open_open = true;
    apply_dialog_result(*ed, .SAVE, .{ status = .CANCELLED });
    expect(!ed.quick_open_open);
    expect_equal(ed.docs.count, 0);
} @test

unsupported_and_failed_dialogs_use_the_right_prompt :: (t: *Test) {
    ed: Editor;
    ed.quick_open_path.single_line = true;
    defer free_text_edit(*ed.quick_open_path);
    apply_dialog_result(*ed, .SAVE, .{ status = .UNSUPPORTED });
    expect(ed.quick_open_open && ed.prompt_kind == .SAVE);
    apply_dialog_result(*ed, .FOLDER, .{ status = .FAILED });
    expect(ed.quick_open_open && ed.prompt_kind == .FOLDER);
} @test

save_as_only_changes_identity_after_success :: (t: *Test) {
    // Documents keep canonical paths, and the temporary folder may sit behind a symlink.
    root, root_ok := get_absolute_path(temp_dir(t));

    defer free(root);
    expect(root_ok);
    original := tprint("%/original.txt", root);
    renamed := tprint("%/renamed.txt", root);

    write_entire_file(original, "original");
    ed: Editor;
    ed.root = root;
    defer free(ed.status);
    doc := open_document(*ed, original);

    defer {
        close_document_data(doc);
        array_free(ed.docs);
    }

    set_contents(*doc.edit, "changed");
    expect(!save_document_as(*ed, tprint("%/missing/fail.txt", root)));
    expect_equal(doc.path, original);
    expect(is_dirty(doc));
    expect(save_document_as(*ed, renamed));
    expect_equal(doc.path, renamed);
    expect_equal(doc.name, "renamed.txt");
    expect(!is_dirty(doc));
    text, ok := read_entire_file(renamed);

    defer free(text);
    expect(ok);
    expect_equal(text, "changed");
    old, old_ok := read_entire_file(original);

    defer free(old);
    expect(old_ok);
    expect_equal(old, "original");
} @test

unsaved_paths_keep_their_canonical_identity_after_creation :: (t: *Test) {
    root, root_ok := get_absolute_path(temp_dir(t));

    defer free(root);
    expect(root_ok);
    spelled := tprint("%/./unsaved.txt", root);
    expected := tprint("%/unsaved.txt", root);

    expect_equal(absolute_path(spelled), expected);
    ed: Editor;
    ed.root = root;
    defer free(ed.status);
    doc := open_document(*ed, spelled);

    defer {
        for opened: ed.docs close_document_data(opened);
        array_free(ed.docs);
    }

    expect_equal(doc.path, expected);
    set_contents(*doc.edit, "created");
    expect(save_document(*ed));
    canonical, ok := get_absolute_path(spelled);

    defer free(canonical);
    expect(ok);
    expect_equal(doc.path, canonical);
    expect(open_document(*ed, spelled) == doc);
    expect_equal(ed.docs.count, 1);
} @test

parent_directories_preserve_absolute_drive_roots :: (t: *Test) {
    expect_equal(parent_directory("C:\\name"), "C:\\");
    expect_equal(parent_directory("C:/name"), "C:/");
    expect_equal(parent_directory("/name"), "/");
} @test

closing_editor_keeps_configuration_strings_independently_owned :: (t: *Test) {
    root := temp_dir(t);
    path := tprint("%/close-config.json", root);
    ed: Editor;
    ed.root = copy_string(root);
    ed.config_file = copy_string(path);
    ed.config.last_root = copy_string("previous-root");

    close_editor(*ed);
    saved := load_config(path);

    defer free_json(*saved);
    expect_equal(saved.last_root, root);
} @test
