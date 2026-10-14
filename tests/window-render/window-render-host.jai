#import "Basic";
W :: #import "Window_Creation";
I :: #import "Input";
S :: #import "Simp";
GL :: #import "GL";
O :: #import "Objective_C";

main :: () {
    window := W.create_window(320, 200, "Production public render test", 40, 200);

    assert(window != W.WT.INVALID_WINDOW);
    I.update_window_events();
    actual := S.prepare_window(window, 0);

    assert(actual == 0);
    S.set_render_target(window);
    width, height := S.get_render_dimensions(window);

    assert(width == 320 && height == 200);
    S.clear_render_target(0, 0, 0, 1);
    S.set_shader_for_color();
    S.immediate_begin();
    S.immediate_quad(0, 0, 40, 40, .{ 0, 1, 0, 1 });
    S.immediate_flush();
    pixel: [4] u8;

    GL.gl.glReadPixels(4, 4, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
    assert(pixel[0] == 0 && pixel[1] == 255 && pixel[2] == 0 && pixel[3] == 255);
    S.swap_buffers(window, false);
    W.swap_view_buffers(window);
    second := W.create_window(160, 100, "Second production render window", 380, 200);

    assert(second != W.WT.INVALID_WINDOW);
    assert(S.prepare_window(second, 0) == 0);

    for 0..3 {
        S.set_render_target(second);
        S.clear_render_target(0, 0, 1, 1);
        S.set_shader_for_color();
        S.immediate_begin();
        S.immediate_quad(0, 0, 20, 20, .{ 1, 0, 0, 1 });
        S.immediate_flush();
        GL.gl.glReadPixels(4, 4, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
        assert(pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 0);
        S.set_render_target(window);
        S.set_shader_for_color();
        S.immediate_begin();
        S.immediate_quad(0, 0, 40, 40, .{ 0, 1, 0, 1 });
        S.immediate_flush();
        GL.gl.glReadPixels(4, 4, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
        assert(pixel[0] == 0 && pixel[1] == 255 && pixel[2] == 0);
        S.set_render_target(second);
        W.swap_view_buffers(window);
        assert(GL.gl.glGetString(GL.GL_VERSION) != null);
    }

    objc :: #library,system "objc";
    message :: () #foreign objc "objc_msgSend";
    Close :: #type (receiver: *void, selector: O.Selector) #c_call;
    close := cast(Close) message;

    retained := S.texture_create_render_target(8, 8, .RGBA8);

    S.set_render_target(*retained);
    S.clear_render_target(1, 0, 0, 1);
    S.set_render_target(window);
    close(window, O.selector("close"));
    O.release(window);
    assert(S.window_infos.count == 1);
    replacement := W.create_window(120, 80, "Reopened production window", 40, 200);

    assert(S.prepare_window(replacement, 0) == 0);
    S.set_render_target(replacement);
    assert(GL.gl.glIsTexture(retained.gl_handle));
    S.set_render_target(*retained);
    GL.gl.glReadPixels(2, 2, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
    assert(pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 0);
    S.set_render_target(second);
    S.set_render_target(*retained);
    pending := S.pixel_read_begin(*retained, .RGBA8);

    S.set_render_target(replacement);
    close(second, O.selector("close"));
    O.release(second);
    GL.gl.glFinish();
    assert(S.pixel_read_is_done(pending));
    readback := S.pixel_read_end(pending);

    assert(readback.width == 8 && readback.height == 8 && readback.data[0] == 255);
    S.deinit(*readback);
    S.texture_destroy(*retained);
    close(replacement, O.selector("close"));
    O.release(replacement);
    assert(S.window_infos.count == 0);
    print("ok\n");
}
