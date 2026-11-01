// Inspect the headless display list: diagnostics need a wave rather than a solid bar.
#if OS != .WASM {
    #import "Basic";
    #import "Math";
    #import "Extensions/Testing";
    #import "Extensions/UI";

    Squiggle_App :: struct {
        edit: Text_Edit;
    }

    squiggle_build :: (app: *Squiggle_App) {
        marks := Text_Mark.[.{ start = 0, end = 3, color = rgb(231, 17, 44), underline = true }];
        for column(.{ grow = 1 }) text_editor(*app.edit, .{ marks = marks });
    }

    diagnostic_underlines_draw_a_wave :: () {
        app: Squiggle_App;

        defer free_text_edit(*app.edit);
        set_contents(*app.edit, "abc");
        ui := headless(.{ width = 200, height = 100 });

        defer free_ui(*ui);
        frame(*ui, *app, squiggle_build);
        count := 0;
        low: float = 1000;
        high: float;

        for ui.draw.cmds {
            if it.kind != .RECT || it.color != rgb(231, 17, 44) continue;

            expect(it.rect.w <= 1 && it.rect.h == 1);
            low = min(low, it.rect.y);
            high = max(high, it.rect.y);
            count += 1;
        }

        expect(count > 4);
        expect_equal(high - low, 2.0);
    } @test
}
