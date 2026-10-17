// Android device embedding must call exercise with a real live native window.
// Desktop execution cannot establish this test's native behavior.
A :: #import "Android";
E :: #import "Android/EGL";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

exercise :: (window: *A.ANativeWindow) {
    check(window != null);
    info, samples := E.egl_create_context(window, wanted_msaa = 4);

    check(info.display != E.EGL_NO_DISPLAY && info.context_ != E.EGL_NO_CONTEXT);
    check(info.surface != E.EGL_NO_SURFACE && info.config != null);
    check(E.eglGetCurrentContext() == info.context_);
    buffers, actual: E.EGLint;

    check(E.eglGetConfigAttrib(info.display, info.config, E.EGL_SAMPLE_BUFFERS, *buffers) != E.EGL_FALSE);
    if buffers != 0 check(E.eglGetConfigAttrib(info.display, info.config, E.EGL_SAMPLES, *actual) != E.EGL_FALSE);

    check(samples == actual);
    check(E.eglSwapBuffers(info.display, info.surface) != E.EGL_FALSE);
    check(E.egl_destroy_surface(*info));
    check(E.eglDestroyContext(info.display, info.context_) != E.EGL_FALSE);
    check(E.eglTerminate(info.display) != E.EGL_FALSE);
}

main :: () {
    invalid, samples := E.egl_create_context(null);
    check(invalid.context_ == null && invalid.surface == null && samples == 0);
}
