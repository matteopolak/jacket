// Simp: left-handed (y down) software rendering, lazy texture state, and a String.join result that can be freed.
#import "Basic";
Math :: #import "Math";
String :: #import "String";

// Simp needs native C libraries (stb_image, FreeType) and a window system, which the WASM targets
// lack: they are listed in tests/stdlib-runtime-skips.txt.
S :: #import "Simp"(render_api = .SOFTWARE);

main :: () {
    target := S.texture_create_render_target(4, 4, .RGBA8);
    assert(target != null);
    defer {
        S.deinit(target);
        free(target);
    }
    S.set_render_target(target, .LEFT_HANDED);
    S.set_shader_for_color(true);
    S.clear_render_target(0, 0, 0, 0);
    // y grows downward, so (0,0)-(2,2) is the top-left corner of the memory image.
    S.immediate_quad(0, 0, 2, 2, Math.Vector4.{ 1, 0, 0, 1 });
    S.immediate_flush();
    bitmap := S.texture_bitmap(target);
    for y: 0..3 for x : 0..3 {
        inside := x < 2 && y < 2;
        red := bitmap.data[y * bitmap.stride + x * 4];
        assert((red == 255) == inside, "left-handed pixel (%, %) red %", x, y, red);
    }

    // Texture state is per struct: a fresh texture deinits safely without any GPU or image library.
    tex: S.Texture;
    S.deinit(*tex);

    // String.join results come from the context allocator and must be freeable.
    joined := String.join("a", "b", "c", separator = "/");
    assert(joined == "a/b/c");
    free(joined.data);
    print("ok\n");
}
