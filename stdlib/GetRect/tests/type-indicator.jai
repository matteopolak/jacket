#import "Basic";
#import "Math";
W :: #import "Window_Creation";
Default :: #import "GetRect";
G :: #import "GetRect"(Type_Indicator = Renderer);
L :: #import "GetRect_LeftHanded"(Type_Indicator = Renderer);

Renderer :: struct {
    Font :: struct {
        em_width: int;
        character_height: int;
        max_ascender: int;
        max_descender: int;
        y_offset_for_centering: int;
    }
    Font_Effects :: enum_flags u16 {
        OUTLINE;
    }
    Texture :: struct {
        handle: u64;
    }
    Window_Type :: W.Window_Type;
}

font: Renderer.Font;
loaded: int;
drawn: int;

main :: () {
    assert(type_of(G.Label_Theme.font) == type_of(*font));
    assert(type_of(L.Label_Theme.font) == type_of(*font));
    assert(type_info(Renderer.Font).name == "Font");
    font = .{ 18, 20, 15, -5, 0 };
    assert(G.em(2, *font) == 36 && L.default_text_baseline(*font, 40) == 25);
    p: G.Draw_Procs;
    p.prepare_text = prepare;
    p.draw_prepared_text = draw;
    p.texture_load_from_memory = load;
    G.ui_init(procs = *p);
    assert(Default.draw_procs.prepare_text == null);
    theme: G.Label_Theme;
    theme.font = *font;
    width := G.label(.{ 10, 20, 100, 40 }, "Wi", *theme, false);

    assert(width == 19 && drawn == 1);
    t := G.require_texture("owned");

    assert(t != null && t.handle == 42 && loaded == 1);
    assert(G.require_texture("owned") == t && loaded == 1);
    print("alternate type indicator ok\n");
}

prepare :: (value: *Renderer.Font, text: string, effects: Renderer.Font_Effects = 0) -> s64 {
    assert(value == *font && text == "Wi" && effects == 0);
    return 19;
}

draw :: (value: *Renderer.Font, x: s64, y: s64, color: Vector4, effects: Renderer.Font_Effects = 0) {
    assert(value == *font && x == 51 && y == 35 && color.w == 1 && effects == 0);
    drawn += 1;
}

load :: (texture: *Renderer.Texture, memory: [] u8, srgb: bool, build_mipmaps: bool) -> bool {
    assert(texture.handle == 0 && memory.count == 5 && srgb && build_mipmaps);
    texture.handle = 42;
    loaded += 1;
    return true;
}
