#import "Basic";
S :: #import "Simp";
GL :: #import "GL";
W :: #import "Window_Creation";

probe :: #library "window_gl_probe";
begin_context :: () -> *void #foreign probe "window_gl_probe_begin";
end_context :: (context: *void) #foreign probe "window_gl_probe_end";

main :: () {
    args := get_command_line_arguments();

    defer free(args.data);
    assert(args.count == 2);
    W.init_mac_app();
    native := begin_context();

    assert(native != null);
    defer end_context(native);
    target := S.texture_create_render_target(64, 64, .RGBA8);

    defer S.texture_destroy(*target);
    S.set_render_target(*target);
    S.clear_render_target(0, 0, 0, 0);
    S.set_shader_for_color();
    S.immediate_begin();
    S.immediate_quad(0, 0, 16, 16, .{ 1, 0, 0, 1 });
    S.immediate_flush();
    pixel: [4] u8;

    GL.gl.glReadPixels(4, 4, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
    assert(pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 0 && pixel[3] == 255);
    GL.gl.glReadPixels(48, 48, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
    assert(pixel[0] == 0 && pixel[3] == 0);
    reader := S.pixel_read_begin(*target, .RGBA8);

    assert(reader != null && reader.pbo != 0 && reader.dest_size == 64 * 64 * 4);
    GL.gl.glFinish();
    assert(S.pixel_read_is_done(reader));
    bitmap := S.pixel_read_end(reader);

    assert(bitmap.width == 64 && bitmap.height == 64 && bitmap.format == .RGBA8);
    assert(bitmap.data[4 * bitmap.stride + 4 * 4] == 255);
    assert(bitmap.data[48 * bitmap.stride + 48 * 4] == 0);
    S.deinit(*bitmap);
    reader = S.pixel_read_begin(*target, .RGBA8, 32, 32, false, true);
    bitmap = S.pixel_read_end(reader);
    assert(bitmap.width == 32 && bitmap.height == 32);
    assert(bitmap.data[29 * bitmap.stride + 2 * 4] == 255);
    assert(bitmap.data[2 * bitmap.stride + 2 * 4] == 0);
    S.deinit(*bitmap);
    font := S.load_font("", args[1], 24);

    assert(font != null);
    S.clear_render_target(0, 0, 0, 0);
    width := S.draw_text(font, 4, 30, "A");

    assert(width > 0);
    S.immediate_flush();
    pixels: [64 * 64 * 4] u8;

    GL.gl.glReadPixels(0, 0, 64, 64, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixels.data);
    ink: u64;
    for index: 0..64 * 64 - 1 ink += pixels[index * 4 + 3];

    assert(ink > 0);
    S.set_render_target(*target, .LEFT_HANDED);
    S.clear_render_target(0, 0, 0, 0);
    assert(S.draw_text(font, 4, 34, "A") == width);
    flipped: [64 * 64 * 4] u8;

    GL.gl.glReadPixels(0, 0, 64, 64, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, flipped.data);
    assert(memcmp(pixels.data, flipped.data, pixels.count) == 0, "text coordinate systems must agree after baseline conversion");

    for coords: S.Simp_Coordinate_System.[.LEFT_HANDED, .RIGHT_HANDED] {
        S.set_render_target(*target, coords);
        S.clear_render_target(0, 0, 0, 0);
        S.set_scissor(4, 8, 12, 16);
        S.clear_render_target(0, 0, 1, 1);
        S.clear_scissor();
        native_y: s32 = 10;
        if coords == .LEFT_HANDED native_y = 53;

        GL.gl.glReadPixels(6, native_y, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
        assert(pixel[2] == 255 && pixel[3] == 255);
        GL.gl.glReadPixels(20, native_y, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
        assert(pixel[2] == 0 && pixel[3] == 0);
    }

    feature_path := sprint("%fixtures/feature-font.ttf", #filepath);

    defer free(feature_path);
    feature_font := S.load_font("", feature_path, 32);

    assert(feature_font != null && feature_font.feature_smallcaps != null && feature_font.feature_lining_numbers != null);
    ink_by_effect: [4] u64;

    for effects, index: S.Font_Effects.[0, .SMALLCAPS, .LINING_FIGURES, .SMALLCAPS | .LINING_FIGURES] {
        S.set_render_target(*target);
        S.clear_render_target(0, 0, 0, 0);
        assert(S.draw_text(feature_font, 4, 10, "a1", effects = effects) > 0);
        GL.gl.glReadPixels(0, 0, 64, 64, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixels.data);
        for pixel_index: 0..64 * 64 - 1 ink_by_effect[index] += pixels[pixel_index * 4 + 3];

        assert(ink_by_effect[index] > 0);
        for previous: 0..index - 1 assert(ink_by_effect[index] != ink_by_effect[previous]);
    }

    assert(GL.gl.glGetError() == GL.GL_NO_ERROR);
    print("ok\n");
}
