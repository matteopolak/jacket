// Executable Windows SDK regression; cross-target checks alone are source evidence.
G :: #import "GL";
All :: #import "GL"(ENABLE_ALL_EXTENSIONS = true);
W :: #import "Windows";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    invalid, invalid_dc, invalid_samples := G.wgl_create_context(null);

    check(invalid == null && invalid_dc == null && invalid_samples == 0);
    #assert type_of(invalid) == W.HGLRC;
    #assert type_of(invalid_dc) == W.HDC;
    #assert type_of(invalid_samples) == s32;

    class_name: [7] u16 = .[83, 84, 65, 84, 73, 67, 0];
    window := W.CreateWindowExW(0, class_name.data, null, 0, 0, 0, 64, 64,
                                null, null, null, null);

    check(window != null);
    defer check(W.DestroyWindow(window) != 0);
    previous := W.wglGetCurrentContext();
    previous_dc := W.wglGetCurrentDC();
    created, dc, samples := G.wgl_create_context(window, wanted_msaa = 0);

    check(created != null && dc != null && samples == 0);
    defer check(W.ReleaseDC(window, dc) != 0);
    defer check(W.wglDeleteContext(created) != 0);
    check(W.wglGetCurrentContext() == previous && W.wglGetCurrentDC() == previous_dc);
    check(W.GetPixelFormat(dc) != 0);
    check(W.wglMakeCurrent(dc, created) != 0);
    check(G.gl.glGetString(G.GL_VERSION) != null);
    check(G.gl.glGetError() == G.GL_NO_ERROR);

    // An installed format must be reused, and creation must restore current state.
    shared, shared_dc, shared_samples := All.wgl_create_context(window, share = created, wanted_msaa = 0);

    #assert type_of(shared) == type_of(created);
    check(shared != null && shared_dc != null && shared_samples == samples);
    check(W.wglGetCurrentContext() == created && W.wglGetCurrentDC() == dc);
    check(W.wglDeleteContext(shared) != 0);
    check(W.ReleaseDC(window, shared_dc) != 0);
    check(W.wglMakeCurrent(previous_dc, previous) != 0);
}
