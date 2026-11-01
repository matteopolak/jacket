#if OS != .WASM {
    #import "Basic";
    #import "Extensions/UI";
    #import "Extensions/Testing";

    pair_width_drawing_and_hit_testing_agree :: () {
        ui := headless();

        defer free_ui(*ui);
        free_text_system(*ui.text);

        #if OS == .MACOS {
            expect(load_font(*ui.text, "/System/Library/Fonts/Supplemental/Arial.ttf"));
            expect(load_font(*ui.text, "/System/Library/Fonts/Menlo.ttc"));
        } else load_system_fonts(*ui.text);

        style := Text_Style.{ size = 48 };
        adjustment := pair_kerning(*ui.text, #char "A", #char "V", style);

        // Pin a face whose AV pair FreeType exposes; the default Helvetica returns zero.
        #if OS == .MACOS expect(adjustment < 0);
        width := text_width(*ui.text, "AV", style);

        expect_equal(width, glyph_advance(*ui.text, #char "A", style) + glyph_advance(*ui.text, #char "V", style) + adjustment);
        expect_equal(draw_text(*ui, 0, 0, "AV", style), width);
        a := text_offset_x(*ui.text, "AV", 1, style);

        expect_equal(a, glyph_advance(*ui.text, #char "A", style) + adjustment);

        expect_equal(index_at_x(*ui.text, "AV", style, a), 1);
        expect_equal(index_at_x(*ui.text, "AV", style, width), 2);
        expect_equal(pair_kerning(*ui.text, #char "A", #char "V", style), adjustment);
        style.font = .MONO;
        expect_equal(pair_kerning(*ui.text, #char "A", #char "V", style), 0.0);
        expect_equal(text_width(*ui.text, "AV", style), glyph_advance(*ui.text, #char "A", style) + glyph_advance(*ui.text, #char "V", style));
    } @test
}
