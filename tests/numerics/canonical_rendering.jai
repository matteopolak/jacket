#import "Basic";
#import "Math";
S :: #import "Simp";
G :: #import "GetRect";
L :: #import "GetRect_LeftHanded";

prepared: int;
drawn: int;
loaded: int;
test_font: S.Dynamic_Font;

main :: () {
    assert(type_of(G.Label_Theme.font) == type_of(*test_font));
    assert(type_of(L.Label_Theme.font) == type_of(*test_font));
    texture: S.Texture;

    assert(type_of(G.require_texture("")) == type_of(*texture));
    test_font.character_height = 20;
    test_font.em_width = 18;
    test_font.max_ascender = 15;
    test_font.max_descender = -5;
    assert(G.em(2, *test_font) == 36);
    assert(G.default_text_baseline(*test_font, 40) == 15);
    assert(L.default_text_baseline(*test_font, 40) == 25);
    lo, hi := G.get_y_range(*test_font, 15);

    assert(lo == 10 && hi == 30);
    lo, hi = L.get_y_range(*test_font, 25);
    assert(lo == 10 && hi == 30);
    p: G.Draw_Procs;
    p.prepare_text = prepare;
    p.draw_prepared_text = draw;
    p.texture_load_from_memory = load;
    G.ui_init(procs = *p);
    width, height := G.text_size(*test_font, "Wi");

    assert(width == 19 && height == 20);
    theme: G.Label_Theme;
    theme.font = *test_font;
    width = G.label(.{ 10, 20, 100, 40 }, "Wi", *theme, false);
    assert(width == 19 && drawn == 1 && prepared == 2);
    t := G.require_texture("pixels");

    assert(t != null && t.width == 7 && t.height == 9 && t.gl_handle == 123 && loaded == 1);
    assert(G.require_texture("pixels") == t && loaded == 1);
    assert(G.require_texture("!bad") == null && loaded == 2);
    print("canonical rendering types and metrics ok\n");
}

prepare :: (font: *S.Dynamic_Font, text: string, effects: S.Font_Effects = 0) -> s64 {
    assert(font == *test_font && text == "Wi" && effects == 0);
    prepared += 1;
    return 19;
}

draw :: (font: *S.Dynamic_Font, x: s64, y: s64, color: Vector4, effects: S.Font_Effects = 0) {
    assert(font == *test_font && x == 51 && y == 35 && color.w == 1 && effects == 0);
    drawn += 1;
}

load :: (texture: *S.Texture, memory: [] u8, srgb: bool, build_mipmaps: bool) -> bool {
    assert(texture != null && srgb && build_mipmaps && texture.gl_handle == 0);
    assert(texture.width == 1 && texture.mipmap_count == 1);
    loaded += 1;
    if memory[0] == 33 return false;

    texture.width = 7;
    texture.height = 9;
    texture.gl_handle = 123; // Test renderer's resource ID; never passed to GL.

    return true;
}
