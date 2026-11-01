#import "Extensions/Testing";
// editor.jai is loaded once by native.jai in this test module.

only_committed_composition_marks_a_document_dirty :: () {
    doc: Document;

    defer free_text_edit(*doc.edit);
    set_contents(*doc.edit, "saved");
    doc.saved_version = doc.edit.buffer.version;
    ui := headless();

    defer free_ui(*ui);
    move_caret(*doc.edit, 5);
    handle_key(*ui, *doc.edit, .{ kind = .COMPOSITION, composition = "かな" }, .{}, .{}, 10);
    expect(!is_dirty(*doc));
    expect_equal(contents(*doc.edit), "saved");
    handle_key(*ui, *doc.edit, .{ kind = .COMPOSITION, composition = "仮名", commit = true }, .{}, .{}, 10);
    expect(is_dirty(*doc));
    expect_equal(contents(*doc.edit), "saved仮名");
    expect(undo_edit(*doc.edit));
    expect_equal(contents(*doc.edit), "saved");
} @test
