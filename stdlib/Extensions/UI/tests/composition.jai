#if OS != .WASM {
    #import "Basic";
    #import "Input";
    #import "Extensions/UI";
    #import "Extensions/Testing";

    build_composition :: (edit: *Text_Edit) {
        for column() text_editor(edit);
    }

    marked_updates_commit_once_and_undo_restores_the_selection :: () {
        ui := headless();

        defer free_ui(*ui);
        edit: Text_Edit;

        defer free_text_edit(*edit);
        set_contents(*edit, "before selected after");
        frame(*ui, *edit, build_composition);
        click(*ui, find_role(*ui, .TEXT_EDITOR));
        frame(*ui, *edit, build_composition);
        move_caret(*edit, 7);
        move_caret(*edit, 15, extend = true);
        version := edit.buffer.version;

        send_composition(*ui, "か");
        frame(*ui, *edit, build_composition);
        expect(edit.composing);
        expect_equal(edit.composition, "か");
        expect_equal(edit.buffer.version, version);
        expect_equal(contents(*edit), "before selected after");
        expect(ui.ime_active);
        expect(ui.ime_rect.h > 0);
        send_composition(*ui, "かな", cursor = 3);
        frame(*ui, *edit, build_composition);
        expect_equal(edit.composition, "かな");
        expect_equal(edit.composition_cursor, 3);
        expect_equal(edit.buffer.version, version);
        send_composition(*ui, "仮名", commit = true);
        frame(*ui, *edit, build_composition);
        expect(!edit.composing);
        expect_equal(contents(*edit), "before 仮名 after");
        expect(undo_edit(*edit));
        expect_equal(contents(*edit), "before selected after");
    } @test

    cancelled_preedit_does_not_modify_the_document :: () {
        ui := headless();

        defer free_ui(*ui);
        edit: Text_Edit;

        defer free_text_edit(*edit);
        set_contents(*edit, "unchanged");
        options: Editor_Options;
        style: Text_Style;

        handle_key(*ui, *edit, .{ kind = .COMPOSITION, composition = "你好" }, style, options, 10);
        expect(edit.composing);
        handle_key(*ui, *edit, .{ kind = .COMPOSITION, cancel = true }, style, options, 10);
        expect(!edit.composing);
        expect_equal(contents(*edit), "unchanged");
        expect(!undo_edit(*edit));
        handle_key(*ui, *edit, .{ kind = .COMPOSITION, composition = "marked" }, style, options, 10);
        handle_key(*ui, *edit, .{ kind = .COMPOSITION }, style, options, 10);
        expect(!edit.composing);
        options.read_only = true;
        handle_key(*ui, *edit, .{ kind = .COMPOSITION, composition = "no", commit = true }, style, options, 10);
        expect_equal(contents(*edit), "unchanged");
    } @test
    composition_does_not_merge_with_adjacent_typing :: () {
        ui := headless();

        defer free_ui(*ui);
        edit: Text_Edit;

        defer free_text_edit(*edit);
        handle_key(*ui, *edit, .{ kind = .TEXT, character = #char "a" }, .{}, .{}, 10);
        handle_key(*ui, *edit, .{ kind = .COMPOSITION, composition = "e" }, .{}, .{}, 10);
        handle_key(*ui, *edit, .{ kind = .COMPOSITION, composition = "é", commit = true }, .{}, .{}, 10);
        handle_key(*ui, *edit, .{ kind = .TEXT, character = #char "b" }, .{}, .{}, 10);
        expect_equal(contents(*edit), "aéb");
        expect(undo_edit(*edit));
        expect_equal(contents(*edit), "aé");
        expect(undo_edit(*edit));
        expect_equal(contents(*edit), "a");
        expect(undo_edit(*edit));
        expect_equal(contents(*edit), "");
    } @test

}
