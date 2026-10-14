#import "Basic";
W :: #import "Windows";
WC :: #import "Window_Creation";
S :: #import "Simp";
GL :: #import "GL";

main :: () {
    first := WC.create_window(200, 100, "Native WGL share owner");
    second := WC.create_window(200, 100, "Native WGL second window");

    assert(first != null && second != null);
    assert(S.prepare_window(first, 0) == 0);
    S.set_render_target(first);
    texture := S.texture_create_render_target(8, 8, .RGBA8);

    S.set_render_target(*texture);
    S.clear_render_target(1, 0, 0, 1);
    reader := S.pixel_read_begin(*texture, .RGBA8);

    assert(S.prepare_window(second, 0) == 0);
    S.set_render_target(second);
    S.swap_buffers(second, false);
    assert(W.DestroyWindow(first) != 0);
    assert(S.window_infos.count == 1);
    readback := S.pixel_read_end(reader);

    assert(readback.width == 8 && readback.height == 8 && readback.data[0] == 255);
    S.deinit(*readback);
    S.set_render_target(*texture);
    reader = S.pixel_read_begin(*texture, .RGBA8);
    assert(W.DestroyWindow(second) != 0);
    assert(S.window_infos.count == 0);
    reopened := WC.create_window(200, 100, "Native WGL reopened window");

    assert(reopened != null && S.prepare_window(reopened, 0) == 0);
    S.set_render_target(reopened);
    assert(GL.gl.glIsTexture(texture.gl_handle));
    readback = S.pixel_read_end(reader);
    assert(readback.data[0] == 255 && readback.data[3] == 255);
    S.deinit(*readback);
    S.texture_destroy(*texture);
    assert(W.DestroyWindow(reopened) != 0);
    assert(S.window_infos.count == 0);
    print("ok\n");
}
