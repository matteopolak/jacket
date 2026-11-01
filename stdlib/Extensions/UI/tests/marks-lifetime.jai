#if OS != .WASM {
    #import "Basic";
    #import "Math";
    #import "Extensions/UI";
    #import "Extensions/Testing";

    build_local_marks :: (edit: *Text_Edit) {
        marks := Text_Mark.[.{ start = 0, end = 3, color = rgb(231, 17, 44) }];
        for column(.{ grow = 1 }) text_editor(edit, .{ marks = marks });
        // The declaration is complete: later reuse of the caller's memory cannot change paint.
        marks[0].color = rgb(12, 34, 56);
    }

    editor_marks_are_retained_for_painting :: () {
        ui := headless(.{ width = 200, height = 100 });

        defer free_ui(*ui);
        edit: Text_Edit;

        defer free_text_edit(*edit);
        set_contents(*edit, "abc");
        frame(*ui, *edit, build_local_marks);
        found := false;
        for ui.draw.cmds if it.kind == .RECT && it.color == rgb(231, 17, 44) found = true;

        expect(found);
    } @test
}
