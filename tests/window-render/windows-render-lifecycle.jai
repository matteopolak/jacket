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
    second_context := W.wglGetCurrentContext();
    second_dc := W.wglGetCurrentDC();

    assert(W.DestroyWindow(first) != 0);
    assert(S.window_infos.count == 1);
    assert(W.wglGetCurrentContext() == second_context && W.wglGetCurrentDC() == second_dc);
    readback := S.pixel_read_end(reader);

    assert(readback.width == 8 && readback.height == 8 && readback.data[0] == 255);
    S.deinit(*readback);
    S.set_render_target(*texture);
    reader = S.pixel_read_begin(*texture, .RGBA8);
    S.set_shader_for_color();
    S.immediate_begin();
    S.immediate_quad(0, 0, 8, 8, .{ 0, 1, 0, 1 });
    // Destruction must flush queued writes to surviving texture storage through
    // a live drawable, after the driver's visible-drawable destruction hook.
    assert(W.DestroyWindow(second) != 0);
    assert(S.window_infos.count == 0);
    // The share owner must retain an actual drawable, not the destroyed public
    // window's DC or a context handle that cannot be selected again.
    owner := W.wglGetCurrentContext();
    owner_dc := W.wglGetCurrentDC();

    assert(owner != null && owner_dc != null && W.WindowFromDC(owner_dc) != null);
    assert(W.WindowFromDC(owner_dc) != first && W.WindowFromDC(owner_dc) != second);
    assert(W.wglMakeCurrent(null, null) != 0);
    assert(W.wglMakeCurrent(owner_dc, owner) != 0);
    assert(GL.gl.glIsTexture(texture.gl_handle));
    reopened := WC.create_window(200, 100, "Native WGL reopened window");

    assert(reopened != null && S.prepare_window(reopened, 0) == 0);
    S.set_render_target(reopened);
    assert(GL.gl.glIsTexture(texture.gl_handle));
    readback = S.pixel_read_end(reader);
    assert(readback.data[0] == 255 && readback.data[3] == 255);
    S.deinit(*readback);
    reader = S.pixel_read_begin(*texture, .RGBA8);
    readback = S.pixel_read_end(reader);
    assert(readback.data[0] == 0 && readback.data[1] == 255 && readback.data[3] == 255);
    S.deinit(*readback);
    S.texture_destroy(*texture);
    assert(W.DestroyWindow(reopened) != 0);
    assert(S.window_infos.count == 0);

    // Repeated retirement includes the requested multisample pixel format; a
    // single DC copied from the first zero-sample window is not sufficient.
    for iteration: 0..2 {
        window := WC.create_window(80 + iteration, 60, "Native WGL repeated retirement");

        assert(window != null);
        assert(S.prepare_window(window, 4) >= 0);
        S.set_render_target(window);
        S.clear_render_target(0, 1, 0, 1);
        S.swap_buffers(window, false);
        assert(W.DestroyWindow(window) != 0);
        assert(S.window_infos.count == 0);
        assert(W.wglGetCurrentContext() == owner && W.wglGetCurrentDC() == owner_dc);
    }

    print("ok\n");
}
