#import "Basic";
S :: #import "Simp";
File :: #import "File";
FT :: #import "freetype";

main :: () {
    args := get_command_line_arguments();

    defer free(args.data);
    assert(args.count == 2);
    bytes, ok := File.read_entire_file(args[1]);

    assert(ok);
    defer free(bytes);
    S.init_fonts(128, 128);
    font := S.get_font_at_size(.{ bytes.count, bytes.data }, 24);

    assert(font != null && font.private1 != null);
    assert(S.get_font_at_size(.{ bytes.count, bytes.data }, 24) == font);
    other := S.get_font_at_size(.{ bytes.count, bytes.data }, 32);

    assert(other != null && other != font && other.private1 == font.private1);
    width := S.prepare_text(font, "Aé A");

    assert(width > 0 && font.temporary_glyphs.count == 4);
    assert(font.temporary_glyphs_byte_offsets[0] == 0 && font.temporary_glyphs_byte_offsets[2] == 3);
    measured: s64;
    for glyph: font.temporary_glyphs measured += S.get_glyph_width_in_pixels(glyph);

    assert(font.temporary_glyphs[0].glyph_index_within_font == FT.FT_Get_Char_Index(font.private1, #char "A"));

    assert(width == measured);
    assert(font.private1.size.metrics.y_ppem == 24);
    assert(S.get_baseline_height(font) > 0);
    assert(S.prepare_text(font, "Aé A", .NO_KERNING | .LEFT_JUSTIFIED) == width);

    if font.feature_smallcaps || font.feature_lining_numbers {
        assert(S.prepare_text(font, "a1", .SMALLCAPS | .LINING_FIGURES) > 0);
        assert(font.temporary_glyphs.count == 2);
        if font.feature_smallcaps assert(font.feature_smallcaps.expanded);
        if font.feature_lining_numbers assert(font.feature_lining_numbers.expanded);

        assert(S.prepare_text(font, "Aé A") == width);
    }

    S.generate_quads_for_prepared_text(font, 10, 20, 0);
    assert(font.current_quads.count == 4);
    assert(font.current_quads[0].glyph == font.temporary_glyphs[0]);
    assert(font.current_quads[0].p1.x - font.current_quads[0].p0.x == cast(float) font.temporary_glyphs[0].width);
    assert(S.get_character_width_in_pixels(font, "A".data, 0) == cast(u32) font.temporary_glyphs[0].advance);
    assert(S.is_latin(0x24f) && S.is_latin(0x2000) && !S.is_latin(0x250) && !S.is_latin(0x2190));
    assert(S.isdigit(#char "9") && !S.isdigit(#char ":"));
    assert(S.isLegalUTF8("é".data, 2));
    illegal := u8.[0xc0, 0xaf];

    assert(!S.isLegalUTF8(illegal.data, 2));
    missing := S.prepare_text(font, "\U0010ffff");

    assert(font.glyph_conversion_failed && missing >= 0);
    S.release_font(font);
    assert(S.get_font_at_size(.{ bytes.count, bytes.data }, 24) == font);
    assert(S.prepare_text(font, "A") > 0);
    print("ok\n");
}
