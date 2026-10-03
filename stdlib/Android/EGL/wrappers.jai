// Choose a real ES2 window configuration; report the actual native sample count.
egl_create_context :: (window: *ANativeWindow, wanted_msaa: s32 = 0) -> EGL_Info, chosen_msaa: s32 {
    info: EGL_Info;
    if !window || wanted_msaa < 0 return info, 0;

    info.display = eglGetDisplay(EGL_DEFAULT_DISPLAY);
    if info.display == EGL_NO_DISPLAY return info, 0;

    major, minor: EGLint;
    if eglInitialize(info.display, *major, *minor) == EGL_FALSE return info, 0;
    if eglBindAPI(EGL_OPENGL_ES_API) == EGL_FALSE return info, 0;

    samples := wanted_msaa;

    while true {
        attributes: [19] EGLint = .[EGL_SURFACE_TYPE, EGL_WINDOW_BIT,
            EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT, EGL_RED_SIZE, 8,
            EGL_GREEN_SIZE, 8, EGL_BLUE_SIZE, 8, EGL_ALPHA_SIZE, 8,
            EGL_DEPTH_SIZE, 24, EGL_SAMPLE_BUFFERS, 0, EGL_SAMPLES, samples, EGL_NONE];
        if samples > 0 attributes[15] = 1;

        count: EGLint;
        if eglChooseConfig(info.display, attributes.data, *info.config, 1, *count) == EGL_FALSE return info, 0;
        if count != 0 && info.config != null break;
        if samples == 0 return info, 0;

        samples /= 2;
    }

    buffers, actual: EGLint;
    if eglGetConfigAttrib(info.display, info.config, EGL_SAMPLE_BUFFERS, *buffers) == EGL_FALSE return info, 0;
    if buffers != 0 && eglGetConfigAttrib(info.display, info.config, EGL_SAMPLES, *actual) == EGL_FALSE return info, 0;

    options: [3] EGLint = .[EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];

    info.context_ = eglCreateContext(info.display, info.config, EGL_NO_CONTEXT, options.data);
    if info.context_ == EGL_NO_CONTEXT return info, 0;

    if !egl_create_surface(*info, window) {
        // Preserve any failed-cleanup handles in the returned record. Display
        // initialization is shared by EGL clients; do not terminate another
        // caller's display behind its back on a constructor failure.
        if eglDestroyContext(info.display, info.context_) != EGL_FALSE info.context_ = EGL_NO_CONTEXT;

        return info, 0;
    }

    return info, actual;
}

// Surface lifetime forwarding. A live display/config/context must already be
// present; destroy the previous surface before creating a replacement.
egl_create_surface :: (info: *EGL_Info, window: *ANativeWindow) -> bool {
    if info.surface != EGL_NO_SURFACE return false;

    visual: EGLint;
    if eglGetConfigAttrib(info.display, info.config, EGL_NATIVE_VISUAL_ID, *visual) == EGL_FALSE return false;
    if binding_window_geometry(window, 0, 0, visual) != 0 return false;

    surface := eglCreateWindowSurface(info.display, info.config, window, null);
    if surface == EGL_NO_SURFACE return false;

    if eglMakeCurrent(info.display, surface, surface, info.context_) == EGL_FALSE {
        if eglDestroySurface(info.display, surface) == EGL_FALSE {
            // Retain the failed cleanup handle so the caller can retry.
            info.surface = surface;
        }

        return false;
    }

    info.surface = surface;

    return true;
}

egl_destroy_surface :: (info: *EGL_Info) -> bool {
    if eglMakeCurrent(info.display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT) == EGL_FALSE return false;
    if eglDestroySurface(info.display, info.surface) == EGL_FALSE return false;
    info.surface = EGL_NO_SURFACE;
    return true;
}

egl_error_to_string :: (error: EGLint) -> error: string, description: string {
    if error == EGL_SUCCESS return "EGL_SUCCESS", "The operation succeeded.";
    if error == EGL_NOT_INITIALIZED return "EGL_NOT_INITIALIZED", "The display connection could not be initialized.";
    if error == EGL_BAD_ACCESS return "EGL_BAD_ACCESS", "The requested resource cannot be accessed.";
    if error == EGL_BAD_ALLOC return "EGL_BAD_ALLOC", "Resources could not be allocated.";
    if error == EGL_BAD_ATTRIBUTE return "EGL_BAD_ATTRIBUTE", "An attribute or attribute value is invalid.";
    if error == EGL_BAD_CONFIG return "EGL_BAD_CONFIG", "The configuration is invalid.";
    if error == EGL_BAD_CONTEXT return "EGL_BAD_CONTEXT", "The context is invalid.";
    if error == EGL_BAD_CURRENT_SURFACE return "EGL_BAD_CURRENT_SURFACE", "The current surface is invalid.";
    if error == EGL_BAD_DISPLAY return "EGL_BAD_DISPLAY", "The display connection is invalid.";
    if error == EGL_BAD_MATCH return "EGL_BAD_MATCH", "The supplied resources or attributes are incompatible.";
    if error == EGL_BAD_NATIVE_PIXMAP return "EGL_BAD_NATIVE_PIXMAP", "The native pixmap is invalid.";
    if error == EGL_BAD_NATIVE_WINDOW return "EGL_BAD_NATIVE_WINDOW", "The native window is invalid.";
    if error == EGL_BAD_PARAMETER return "EGL_BAD_PARAMETER", "A parameter is invalid.";
    if error == EGL_BAD_SURFACE return "EGL_BAD_SURFACE", "The surface is invalid.";
    if error == EGL_CONTEXT_LOST return "EGL_CONTEXT_LOST", "The graphics context was lost.";

    return "EGL_UNKNOWN_ERROR", "The native EGL implementation returned an unknown error.";
}

egl_get_error_string :: () -> string, string #expand {
    return egl_error_to_string(eglGetError());
}

#scope_file
#import "Android";

binding_android :: #library,system "android";
// Official NDK C signature; the public Android facade's camera-context alias
// and enum spelling are not required by this private native operation.
binding_window_geometry :: (window: *ANativeWindow, width: s32, height: s32, format: s32) -> s32 #foreign binding_android "ANativeWindow_setBuffersGeometry";
