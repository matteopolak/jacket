// test: skip_on(.WASM, reason = "needs a native window, and Window_Type has no wasm target")
#import "Basic";
S :: #import "Simp";
F :: #import "File";

main :: () {
    path := sprint("%fixtures/feature-font.ttf", #filepath);

    defer free(path);
    bytes, ok := F.read_entire_file(path);

    assert(ok, "original feature font must be generated before this test");
    defer free(bytes);
    font := S.get_font_at_size(.{ bytes.count, bytes.data }, 32);

    assert(font != null);

    // Several pieces make one run; ending it leaves the laid out quads to look at.
    S.begin_prepare_text(font);
    assert(font.current_quads.count == 0);
    S.append_prepare_text(font, "a");
    S.append_prepare_text(font, "1a");
    S.end_prepare_text(font);

    quads := font.current_quads;
    first_x := quads[0].ul.x;

    assert(quads.count == 3 && font.temporary_glyphs.count == 3);

    for quad, index: quads {
        glyph := font.temporary_glyphs[index];

        assert(quad.glyph == glyph);
        assert(quad.ul.x < quad.lr.x && quad.ul.y < quad.lr.y, "ul is the corner with the smaller x and y");
        assert(quad.lr.x - quad.ul.x == cast(float) glyph.width);
        assert(quad.lr.y - quad.ul.y == cast(float) glyph.height);
        assert(quad.ul == quad.p0 && quad.lr == quad.p2);
        assert(quad.uv0 == Vector2.{ quad.u0, quad.v1 } && quad.uv1 == Vector2.{ quad.u1, quad.v0 });
        assert(quad.uv0.x < quad.uv1.x, "the atlas box has a width");
        assert(quad.color == Vector4.{ 1, 1, 1, 1 });
    }

    // The pen moves by each glyph's advance, from x = 0.
    pen: float;

    for quad, index: quads {
        glyph := font.temporary_glyphs[index];

        assert(quad.ul.x == pen + cast(float) glyph.offset_x);
        pen += cast(float) glyph.advance;
    }

    assert(cast(s64) pen == font.temporary_glyphs_width_in_pixels);

    // The same run through prepare_text gives the same width, and drawing it places it elsewhere.
    width := S.prepare_text(font, "a1a");

    assert(width == font.temporary_glyphs_width_in_pixels);
    S.generate_quads_for_prepared_text(font, 10, 20, 0);
    assert(font.current_quads[0].ul.x == first_x + 10);

    // The colour recorded in the quads comes from the context.
    tinted := context;
    tinted.simp.text_color = .{ 1, 0.5, 0, 1 };

    push_context tinted {
        S.begin_prepare_text(font);
        S.append_prepare_text(font, "a");
        S.end_prepare_text(font);
    }

    assert(font.current_quads.count == 1 && font.current_quads[0].color == Vector4.{ 1, 0.5, 0, 1 });
    assert(context.simp.text_color == Vector4.{ 1, 1, 1, 1 });
    print("ok\n");
}

#import "Math";
