#import "Basic";
S :: #import "Simp";
F :: #import "File";
FT :: #import "freetype";

Case :: struct {
    effects: S.Font_Effects;
    first: u32;
    second: u32;
}

main :: () {
    path := sprint("%fixtures/feature-font.ttf", #filepath);

    defer free(path);
    bytes, ok := F.read_entire_file(path);

    assert(ok, "original feature font must be generated before this test");
    defer free(bytes);
    font := S.get_font_at_size(.{ bytes.count, bytes.data }, 32);

    assert(font != null && font.feature_smallcaps != null && font.feature_lining_numbers != null);
    assert(FT.FT_Get_Char_Index(font.private1, #char "a") == 2);
    assert(FT.FT_Get_Char_Index(font.private1, #char "1") == 4);
    cases := Case.[.{ 0, 2, 4 }, .{ .SMALLCAPS, 8, 6 }, .{ .LINING_FIGURES, 2, 5 }, .{ .SMALLCAPS | .LINING_FIGURES, 8, 5 }];
    for item: cases {
        width := S.prepare_text(font, "a1", item.effects);

        assert(font.temporary_glyphs.count == 2);
        assert(font.temporary_glyphs[0].glyph_index_within_font == item.first);
        assert(font.temporary_glyphs[1].glyph_index_within_font == item.second);
        native_width: s64;

        for index: u32.[item.first, item.second] {
            assert(FT.FT_Load_Glyph(font.private1, index, FT.FT_LOAD_DEFAULT) == 0);
            native_width += font.private1.glyph.advance.x / 64;
        }

        assert(width == native_width);
        S.generate_quads_for_prepared_text(font, 4, 20, item.effects);
        assert(font.current_quads.count == 2 && font.current_quads[0].glyph == font.temporary_glyphs[0]);
    }

    assert(font.feature_smallcaps.expanded && font.feature_lining_numbers.expanded);
    assert(font.feature_smallcaps.lookup_tables.count == 2);
    assert(font.feature_smallcaps.lookup_tables[0].lookup_type == 7);
    assert(font.feature_smallcaps.lookup_tables[1].lookup_type == 1);
    assert(font.feature_lining_numbers.lookup_tables[0].lookup_type == 3);
    path2 := sprint("%fixtures/filtered-feature-font.ttf", #filepath);

    defer free(path2);
    filtered := S.load_font("", path2, 32);

    assert(filtered != null);
    S.prepare_text(filtered, "a1", .SMALLCAPS);
    // Flags filter other inputs of sequence lookups, never the current glyph
    // of a single/alternate substitution, even with matching GDEF classes.
    assert(filtered.temporary_glyphs[0].glyph_index_within_font == 8);
    assert(filtered.temporary_glyphs[1].glyph_index_within_font == 6);
    S.prepare_text(font, "2", .LINING_FIGURES);
    assert(font.temporary_glyphs[0].glyph_index_within_font == 6);
    S.prepare_text(filtered, "2", .LINING_FIGURES);
    assert(filtered.temporary_glyphs[0].glyph_index_within_font == 6);
    print("ok\n");
}
