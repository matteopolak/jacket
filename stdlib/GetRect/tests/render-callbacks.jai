#import "Basic";
#import "Math";
G :: #import "GetRect";
L :: #import "GetRect_LeftHanded";

prepared: int;
drawn: int;
last_x: s64;
last_y: s64;

main :: () {
    p: G.Draw_Procs;
    p.prepare_text = prepare;
    p.draw_prepared_text = draw;
    G.ui_init(procs = *p);
    t: G.Label_Theme;
    t.text_baseline_vertical_position = 0.3;
    width := G.label(.{ 10, 20, 100, 40 }, "Wi", *t, false);

    assert(width == 19 && last_x == 51 && last_y == 32);
    assert(prepared == 1 && drawn == 1);
    G.my_draw_text(null, -7, 11, "Wi", .{ 1, 1, 1, 1 });
    assert(last_x == -7 && last_y == 11 && prepared == 2 && drawn == 2);
    // A supplied finite baseline is honored in both coordinate conventions.
    lp: L.Draw_Procs;
    lp.prepare_text = prepare;
    lp.draw_prepared_text = draw;
    L.ui_init(procs = *lp);
    lt: L.Label_Theme;
    lt.text_baseline_vertical_position = 0.75;
    width = L.label(.{ 10, 20, 100, 40 }, "Wi", *lt, false);
    assert(width == 19 && last_x == 51 && last_y == 50);
    assert(prepared == 3 && drawn == 3);
    assert(type_of(G.Label_Theme.font) == type_of(L.Label_Theme.font));
    assert(type_of(G.require_texture("")) == type_of(L.require_texture("")));
    print("render callbacks ok\n");
}

prepare :: (font: type_of(G.Label_Theme.font), text: string, effects: type_of(G.Label_Theme.effects) = 0) -> s64 {
    assert(font == null && effects == 0 && text == "Wi");
    prepared += 1;
    return 19; // The caller's actual renderer width, intentionally not text.count*8.
}

draw :: (font: type_of(G.Label_Theme.font), x: s64, y: s64, color: Vector4, effects: type_of(G.Label_Theme.effects) = 0) {
    assert(font == null && effects == 0 && color.w == 1);
    record_draw(x, y);
}

record_draw :: (x: s64, y: s64) {
    assert(prepared == drawn + 1);
    drawn += 1;
    last_x = x;
    last_y = y;
}
