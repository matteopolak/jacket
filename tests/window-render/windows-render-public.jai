#import "Basic";
W :: #import "Windows";
WC :: #import "Window_Creation";
S :: #import "Simp";
GL :: #import "GL";

main :: () {
    window := WC.create_window(200, 120, "Public native WGL draw/readback regression");

    assert(window != null);
    defer assert(W.DestroyWindow(window) != 0);
    assert(S.prepare_window(window, 0) == 0);
    S.set_render_target(window);
    S.clear_render_target(0, 0, 0, 1);
    S.set_shader_for_color();
    S.immediate_begin();
    S.immediate_quad(0, 0, 32, 32, .{ 0, 1, 0, 1 });
    S.immediate_flush();
    pixel: [4] u8;

    GL.gl.glReadPixels(4, 4, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR);
    assert(pixel[0] == 0 && pixel[1] == 255 && pixel[2] == 0 && pixel[3] == 255);
    image := S.texture_create_render_target(8, 8, .RGBA8);

    defer S.texture_destroy(*image);
    S.set_render_target(*image);
    S.clear_render_target(1, 0, 1, 1);
    S.set_render_target(window);
    S.set_shader_for_images(*image);
    S.immediate_begin();
    S.immediate_quad(0, 0, 32, 32, .{ 1, 1, 1, 1 });
    S.immediate_flush();
    GL.gl.glReadPixels(4, 4, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR);
    assert(pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 255 && pixel[3] == 255);
    S.swap_buffers(window, false);
    print("ok\n");
}
