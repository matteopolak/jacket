// test: skip_on(.WASM, reason = "needs a native window, and Window_Type has no wasm target")
// Text whose theme names no font uses the default font: the bundled one when `ui_init` gets
// none, loaded once per pixel height, which follows the rectangle's height.
#import "Basic";
#import "Math";
G :: #import "GetRect";
Simp :: #import "Simp";

Request :: struct {
    bytes: int;
    pixel_height: int;
}

requests: [..] Request;
fonts: [4] Simp.Dynamic_Font;
drawn_with: type_of(G.Label_Theme.font);

main :: () {
    procs: G.Draw_Procs;
    procs.prepare_text = (font: type_of(G.Label_Theme.font), text: string, effects: type_of(G.Label_Theme.effects) = 0) -> s64 {
        return text.count * 8;
    };
    procs.draw_prepared_text = (font: type_of(G.Label_Theme.font), x: s64, y: s64, color: Vector4, effects: type_of(G.Label_Theme.effects) = 0) {
        drawn_with = font;
    };

    procs.set_scissor = (x0: s32, y0: s32, x1: s32, y1: s32) {};
    procs.clear_scissor = () {};
    procs.get_font_at_size = (memory: [] u8, pixel_height: int) -> type_of(G.Label_Theme.font) {
        array_add(*requests, .{ memory.count, pixel_height });
        return *fonts[requests.count - 1];
    };

    G.ui_init(procs = *procs);

    theme: G.Label_Theme;
    theme.alignment = .Center;
    theme.text_baseline_vertical_position = 0.25;
    assert(theme.font == null);

    G.label(.{ 0, 0, 200, 60 }, "tall", *theme);
    G.label(.{ 0, 0, 200, 60 }, "again", *theme);
    G.label(.{ 0, 0, 200, 20 }, "short", *theme);

    assert(requests.count == 2);
    // The bundled font is a whole TrueType file.
    assert(requests[0].bytes > 100_000 && requests[1].bytes == requests[0].bytes);
    assert(requests[0].pixel_height == 36 && requests[1].pixel_height == 12);
    assert(drawn_with == *fonts[1]);

    // A font given to `ui_init` replaces the bundled one.
    G.ui_init("not a real font", *procs);
    G.label(.{ 0, 0, 200, 60 }, "given", *theme);
    assert(requests.count == 3 && requests[2].bytes == "not a real font".count);

    // A theme's own font wins.
    theme.font = *fonts[3];
    G.label(.{ 0, 0, 200, 60 }, "own", *theme);
    assert(requests.count == 3 && drawn_with == *fonts[3]);
    print("default font ok\n");
}
