#import "Basic";
S :: #import "Simp";
GL :: #import "GL";
W :: #import "Window_Creation";

probe :: #library "window_gl_probe";
begin_context :: () -> *void #foreign probe "window_gl_probe_begin";
end_context :: (context: *void) #foreign probe "window_gl_probe_end";

main :: () {
    W.init_mac_app();
    native := begin_context();

    assert(native != null);
    defer end_context(native);

    for format: S.Texture_Format.[.D16, .D24S8, .D32F, .D32FS8] {
        texture := S.texture_create_render_target(7, 5, format);

        S.set_render_target(*texture);
        GL.gl.glClearDepth(0.25);
        GL.gl.glClearStencil(73);
        S.clear_render_target(0, 0, 0, 0);
        reader := S.pixel_read_begin(*texture, format, 3, 2, false, true);
        bitmap := S.pixel_read_end(reader);

        assert(bitmap.width == 3 && bitmap.height == 2 && bitmap.format == format);

        for pixel: 0..5 {
            if format == .D16 {
                value := (cast(*u16) bitmap.data.data)[pixel];
                assert(value >= 16383 && value <= 16385);
            } else if format == .D24S8 {
                value := (cast(*u32) bitmap.data.data)[pixel];
                assert((value & 0xff) == 73);
                assert((value >> 8) >= 4194303 && (value >> 8) <= 4194305);
            } else if format == .D32F {
                assert((cast(*float) bitmap.data.data)[pixel] == 0.25);
            } else {
                address := bitmap.data.data + pixel * 8;
                assert((cast(*float) address).* == 0.25);
                assert(((cast(*u32) (address + 4)).* & 0xff) == 73);
            }
        }

        S.deinit(*bitmap);
        S.texture_destroy(*texture);
        assert(GL.gl.glGetError() == GL.GL_NO_ERROR);
    }

    print("ok\n");
}
