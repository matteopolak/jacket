egl_create_context :: (window: *ANativeWindow, wanted_msaa: s32 = 0) -> EGLContext, EGLDisplay, EGLSurface, chosen_msaa: s32 {
    if window == null || wanted_msaa < 0 return null, null, null, 0;
    display := eglGetDisplay(EGL_DEFAULT_DISPLAY);
    if display == null return null, null, null, 0;
    major: EGLint; minor: EGLint;
    if eglInitialize(display, *major, *minor) == EGL_FALSE return null, null, null, 0;
    success := false;
    surface: EGLSurface;
    native_context: EGLContext;
    defer if !success {
        if native_context != null eglDestroyContext(display, native_context);
        if surface != null eglDestroySurface(display, surface);
        eglTerminate(display);
    }
    if eglBindAPI(EGL_OPENGL_ES_API) == EGL_FALSE return null, null, null, 0;
    // Each lower sample request permits the driver to pick its nearest supported
    // configuration. Query the selected configuration for the actual count.
    requested := wanted_msaa;
    config: EGLConfig;
    while true {
        attributes: [19] EGLint = .{EGL_SURFACE_TYPE, EGL_WINDOW_BIT,
            EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT, EGL_RED_SIZE, 8,
            EGL_GREEN_SIZE, 8, EGL_BLUE_SIZE, 8, EGL_ALPHA_SIZE, 8,
            EGL_DEPTH_SIZE, 24, EGL_SAMPLE_BUFFERS, 0, EGL_SAMPLES, requested, EGL_NONE};
        if requested > 0 attributes[15] = 1;
        matches: EGLint = 0;
        if eglChooseConfig(display, *attributes[0], *config, 1, *matches) != EGL_FALSE && matches > 0 break;
        if requested == 0 return null, null, null, 0;
        requested /= 2;
    }
    visual: EGLint;
    if eglGetConfigAttrib(display, config, EGL_NATIVE_VISUAL_ID, *visual) == EGL_FALSE return null, null, null, 0;
    if ANativeWindow_setBuffersGeometry(window, 0, 0, visual) != 0 return null, null, null, 0;
    surface = eglCreateWindowSurface(display, config, cast(EGLNativeWindowType) window, null);
    if surface == null return null, null, null, 0;
    // ES 2 is the guaranteed rendering type requested above.
    context_attributes: [3] EGLint = .{EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE};
    native_context = eglCreateContext(display, config, EGL_NO_CONTEXT, *context_attributes[0]);
    if native_context == null return null, null, null, 0;
    if eglMakeCurrent(display, surface, surface, native_context) == EGL_FALSE return null, null, null, 0;
    samples: EGLint = 0;
    if eglGetConfigAttrib(display, config, EGL_SAMPLES, *samples) == EGL_FALSE {
        eglMakeCurrent(display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
        return null, null, null, 0;
    }
    eglSwapInterval(display, 1);
    success = true;
    return native_context, display, surface, samples;
}
// PRIVATE HELPERS
// All names above are genuine SDK declarations already in EGL/bindings.jai.
