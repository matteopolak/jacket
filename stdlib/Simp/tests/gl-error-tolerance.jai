#import "Basic";
S :: #import "Simp";
GL :: #import "GL";
W :: #import "Window_Creation";

probe :: #library "window_gl_probe";
begin_context :: () -> *void #foreign probe "window_gl_probe_begin";
end_context :: (context: *void) #foreign probe "window_gl_probe_end";

// An invalid GL call leaves GL_INVALID_VALUE pending, as a host program's own mistakes would.
leave_error_pending :: () {
    GL.gl.glViewport(0, 0, cast,force(GL.GLsizei) -1, cast,force(GL.GLsizei) -1);
}

main :: () {
    W.init_mac_app();
    native := begin_context();

    assert(native != null);
    defer end_context(native);
    target := S.texture_create_render_target(64, 64, .RGBA8);

    defer S.texture_destroy(*target);
    assert(target.gl_handle != 0);

    leave_error_pending();
    S.set_render_target(*target);
    leave_error_pending();
    S.clear_render_target(0, 0, 0, 1);

    leave_error_pending();
    S.set_scissor(4, 8, 12, 16);
    leave_error_pending();
    S.clear_scissor();

    leave_error_pending();
    S.set_shader_for_color();
    leave_error_pending();
    S.immediate_begin();
    S.immediate_quad(0, 0, 16, 16, .{ 1, 0, 0, 1 });
    leave_error_pending();
    S.immediate_flush();

    pixel: [4] u8;

    while GL.gl.glGetError() != GL.GL_NO_ERROR {}

    GL.gl.glReadPixels(4, 4, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
    assert(pixel[0] == 255 && pixel[3] == 255);

    leave_error_pending();
    other := S.texture_create_render_target(8, 8, .RGBA8);

    defer S.texture_destroy(*other);
    assert(other.gl_handle != 0);

    leave_error_pending();
    reader := S.pixel_read_begin(*target, .RGBA8);

    assert(reader != null);
    bitmap := S.pixel_read_end(reader);

    S.deinit(*bitmap);

    leave_error_pending();
    print("ok\n");
}
