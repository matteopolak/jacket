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
    bitmap: S.Bitmap;

    S.bitmap_alloc(*bitmap, 2, 2, .RGBA8);
    defer S.deinit(*bitmap);
    for *byte: bitmap.data byte.* = 128;

    texture: S.Texture;
    texture.flags = .Render_Target | .No_Skip_Mipmaps;
    defer S.texture_destroy(*texture);

    // Reload the same public resource in both directions. Stored GL format and
    // public flags must follow the incoming bitmap, preserving unrelated flags.
    for operation: 0..2 for srgb : bool.[true, false, true, false] {
        bitmap.flags &= ~ .IS_SRGB;
        if srgb bitmap.flags |= .IS_SRGB;

        previous_name := texture.gl_handle;

        if operation == 0 {
            assert(S.texture_load_from_bitmap(*texture, *bitmap));
        } else if operation == 1 {
            S.backend_realloc_texture(*texture, *bitmap);
            assert(texture.gl_handle == previous_name);
        } else {
            S.backend_update_texture(*texture, *bitmap);
            assert(texture.gl_handle == previous_name);
        }

        assert(((texture.flags & .sRGB) != 0) == srgb);
        assert((texture.flags & (.Render_Target | .No_Skip_Mipmaps)) == (.Render_Target | .No_Skip_Mipmaps));
        GL.gl.glBindTexture(GL.GL_TEXTURE_2D, texture.gl_handle);
        actual: s32;

        GL.gl.glGetTexLevelParameteriv(GL.GL_TEXTURE_2D, 0, GL.GL_TEXTURE_INTERNAL_FORMAT, *actual);
        expected := GL.GL_RGBA8;
        if srgb expected = GL.GL_SRGB8_ALPHA8;

        assert(actual == cast(s32) expected && GL.gl.glGetError() == GL.GL_NO_ERROR);
        pixels: [16] u8;

        GL.gl.glGetTexImage(GL.GL_TEXTURE_2D, 0, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixels.data);
        assert(GL.gl.glGetError() == GL.GL_NO_ERROR);
        for byte: pixels assert(byte == 128);
    }

    print("ok\n");
}
