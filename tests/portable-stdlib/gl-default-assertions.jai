// GL_PROBE is the exact own source-built offscreen SDK helper from the runner.
#import "Basic";
S :: #import "Simp";
G :: #import "GL";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

recover :: (loc: Source_Code_Location, message: string) -> bool {
    return false;
}

mark :: (calls: *s64) -> bool {
    calls.* += 1;
    return true;
}

main :: () {
    args := get_command_line_arguments();
    if args.count != 3 exit(2);

    calls: s64;

    assert(mark(*calls));

    if args[1] == "enabled" {
        if calls != 1 exit(3);
    } else if args[1] == "disabled" {
        if calls != 0 exit(4);
    } else exit(5);

    native := begin_context();
    if !native exit(6);

    defer end_context(native);
    G.gl_load(*G.gl);
    if !S.is_supported(.RGBA8) exit(7);

    if args[2] == "projection" {
        if *G.glCreateShader != *G.gl.glCreateShader exit(15);

        original := G.glCreateShader;
        if !original exit(16);

        G.glCreateShader = null;
        if G.gl.glCreateShader != null exit(17);

        G.gl.glCreateShader = original;
        if G.glCreateShader != original exit(18);

        G.gl.glCreateShader = null;
        G.gl_load(*G.gl);
        if G.glCreateShader == null || G.glCreateShader != G.gl.glCreateShader exit(19);

        write_string("ok\n");

        return;
    }

    bitmap: S.Bitmap;

    S.bitmap_alloc(*bitmap, 2, 2, .RGBA8);
    defer S.deinit(*bitmap);
    for *byte, index: bitmap.data byte.* = cast(u8) (index * 11);

    texture: S.Texture;

    defer S.texture_destroy(*texture);

    if args[2] == "realloc" {
        S.backend_realloc_texture(*texture, *bitmap);
        if texture.gl_handle == 0 || texture.width != 2 || texture.height != 2 exit(8);

        pixels: [16] u8;

        G.gl.glBindTexture(G.GL_TEXTURE_2D, texture.gl_handle);
        G.gl.glGetTexImage(G.GL_TEXTURE_2D, 0, G.GL_RGBA, G.GL_UNSIGNED_BYTE, pixels.data);
        if G.gl.glGetError() != G.GL_NO_ERROR || memcmp(pixels.data, bitmap.data.data, 16) != 0 exit(9);
    } else if args[2] == "unmap" {
        if !S.texture_load_from_bitmap(*texture, *bitmap) exit(10);

        reader := S.pixel_read_begin(*texture, .RGBA8);
        if !reader exit(11);

        G.gl.glUnmapBuffer = real_unmap;
        result := S.pixel_read_end(reader);

        defer S.deinit(*result);
        if unmap_count() != 1 exit(12);
        if result.width != 2 || result.height != 2 || memcmp(result.data.data, bitmap.data.data, 16) != 0 exit(13);
    } else if args[2] == "failure" {
        if !S.texture_load_from_bitmap(*texture, *bitmap) exit(20);

        old := texture.gl_handle;
        invalid := bitmap;
        invalid.mipmap_count = 2;
        invalid.format = cast(S.Texture_Format) 65535;
        saved := context;
        scoped := context;
        scoped.logger = capture;
        scoped.assertion_failed = recover;
        context = scoped;
        S.backend_realloc_texture(*texture, *invalid);
        context = saved;
        if errors == 0 || texture.gl_handle != old || texture.width != 2 || texture.height != 2 exit(21);

        pixels: [16] u8;

        G.gl.glBindTexture(G.GL_TEXTURE_2D, texture.gl_handle);
        G.gl.glGetTexImage(G.GL_TEXTURE_2D, 0, G.GL_RGBA, G.GL_UNSIGNED_BYTE, pixels.data);
        if G.gl.glGetError() != G.GL_NO_ERROR || memcmp(pixels.data, bitmap.data.data, 16) != 0 exit(22);
    } else exit(14);

    write_string("ok\n");
}

#scope_file
begin_context :: () -> *void #foreign GL_PROBE "window_gl_probe_begin";
end_context :: (context: *void) #foreign GL_PROBE "window_gl_probe_end";
real_unmap :: (target: G.GLenum) -> G.GLboolean #foreign GL_PROBE "portable_test_unmap_buffer";
unmap_count :: () -> u32 #foreign GL_PROBE "portable_test_unmap_count";
