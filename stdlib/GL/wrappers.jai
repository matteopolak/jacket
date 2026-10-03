// Version and extension discovery operate on the caller's current GL context.
gl_parse_version :: (version: string) -> major: s32, minor: s32 {
    text := Native.terminated_copy(version);

    defer Native.release(text);
    major, minor: s32;
    if binding_sscanf(text, "GL_VERSION_%d_%d", *major, *minor) != 2 return 0, 0;

    return major, minor;
}

gl_get_version :: () -> major: s32, minor: s32 {
    return binding_query_version(*gl);
}

gl_supports_extension :: (ext: string) -> bool {
    major := gl_get_version();
    return binding_supports_extension(*gl, ext, major);
}

gl_load :: (procs: *GL_Procedures, GetProcAddress: GetProcAddressFunction = null) {
    major, minor := binding_query_version(procs);
    if major == 0 return;

    resolve := GetProcAddress;
    if !resolve resolve = binding_resolve;

    for member: type_info(GL_Procedures).members {
        if (member.flags & .CONSTANT) != 0 continue;

        address := (cast(*u8) procs) + member.offset_in_bytes;

        if member.type.type == .PROCEDURE || (member.flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) != 0 {
            name := Native.terminated_copy(member.name);
            symbol := resolve(name);

            Native.release(name);

            if symbol {
                <<cast(**void) address = symbol;
            }
        }
    }

    // Indexed extension queries need the just-resolved glGetStringi pointer.
    for member: type_info(GL_Procedures).members {
        if (member.flags & .CONSTANT) != 0 continue;

        address := (cast(*u8) procs) + member.offset_in_bytes;

        if member.type.type == .BOOL {
            available := false;

            if binding_starts_with(member.name, "GL_VERSION_") {
                wanted_major, wanted_minor := gl_parse_version(member.name);
                available = wanted_major < major || (wanted_major == major && wanted_minor <= minor);
            } else {
                available = binding_supports_extension(procs, member.name, major);
            }

            if available {
                <<cast(*bool) address = true;
            }
        }
    }
}

// Explicit source lengths preserve embedded NULs and concatenation boundaries.
_glShaderSource :: (shader: GLuint, source: string) {
    if source.count > 0x7fff_ffff binding_abort();
    data := source.data;
    length := cast(GLint) source.count;
    gl.glShaderSource(shader, 1, *data, *length);
}

_glShaderSource :: (shader: GLuint, sources: [] string) {
    if sources.count > 0xffff_ffff binding_abort();

    data := cast(**GLchar) Native.allocate(sources.count * size_of(*GLchar));
    lengths := cast(*GLint) Native.allocate(sources.count * size_of(GLint));

    defer Native.release(data);
    defer Native.release(lengths);

    for i: 0..sources.count - 1 {
        if sources[i].count > 0x7fff_ffff binding_abort();
        data[i] = sources[i].data;
        lengths[i] = cast(GLint) sources[i].count;
    }

    gl.glShaderSource(shader, cast(GLsizei) sources.count, data, lengths);
}

_glGetUniformLocation :: inline (program: u32, $$name: string) -> s32 {
    text := Native.terminated_copy(name);
    defer Native.release(text);
    return gl.glGetUniformLocation(program, text);
}

_glBindAttribLocation :: inline (program: u32, index: u32, $$name: string) {
    text := Native.terminated_copy(name);
    defer Native.release(text);
    gl.glBindAttribLocation(program, index, text);
}

_glGetAttribLocation :: (program: GLuint, $$name: string) -> GLint {
    text := Native.terminated_copy(name);
    defer Native.release(text);
    return gl.glGetAttribLocation(program, text);
}

gl_label_object :: (identifier: GLenum, name: GLuint, label: string) {
    if label.count > 0x7fff_ffff binding_abort();
    gl.glObjectLabel(identifier, name, cast(GLsizei) label.count, label.data);
}

// These strings describe caller-owned writable storage, not text to terminate
// or copy. Keep the requested capacity and every native output parameter.
_glGetActiveAttrib :: (program: GLuint, index: GLuint, bufSize: GLsizei,
                      length: *GLsizei, size: *GLint, type: *GLenum, $$name: string) {
    if bufSize > name.count binding_abort();
    gl.glGetActiveAttrib(program, index, bufSize, length, size, type, name.data);
}

_glGetActiveUniform :: (program: GLuint, index: GLuint, bufSize: GLsizei,
                       length: *GLsizei, size: *GLint, type: *GLenum, $$name: string) {
    if bufSize > name.count binding_abort();
    gl.glGetActiveUniform(program, index, bufSize, length, size, type, name.data);
}

DumpShaderInfoLog :: (shader: GLuint, name: string) {
    capacity: GLint;

    gl.glGetShaderiv(shader, GL_INFO_LOG_LENGTH, *capacity);
    if capacity <= 1 return;

    text := cast(*u8) Native.allocate(capacity);

    defer Native.release(text);
    written: GLsizei;

    gl.glGetShaderInfoLog(shader, cast(GLsizei) capacity, *written, text);
    if binding_printf("Shader %.*s: %.*s\n", cast(s32) name.count, name.data,
                      cast(s32) written, text) < 0 binding_abort();
}

DumpProgramInfoLog :: (program: GLuint, name: string) {
    capacity: GLint;

    gl.glGetProgramiv(program, GL_INFO_LOG_LENGTH, *capacity);
    if capacity <= 1 return;

    text := cast(*u8) Native.allocate(capacity);

    defer Native.release(text);
    written: GLsizei;

    gl.glGetProgramInfoLog(program, cast(GLsizei) capacity, *written, text);
    if binding_printf("Program %.*s: %.*s\n", cast(s32) name.count, name.data,
                      cast(s32) written, text) < 0 binding_abort();
}

#if DUMP_GL_ERRORS {
    DumpGLErrors :: (tag: string, loc := #caller_location) -> bool {
        had_error := false;

        while true {
            error := gl.glGetError();
            if error == GL_NO_ERROR break;

            had_error = true;
            if binding_printf("OpenGL error 0x%x [%.*s] at %.*s:%lld\n", error,
                cast(s32) tag.count, tag.data, cast(s32) loc.fully_pathed_filename.count,
                loc.fully_pathed_filename.data, loc.line_number) < 0 binding_abort();
        }

        return had_error;
    }
} else {
    DumpGLErrors :: inline (#discard tag: string, #discard loc := #caller_location) -> bool {
        return false;
    }
}

#if OS == .MACOS {
    nsgl_create_context :: (major_version: s32 = 0, minor_version: s32 = 0,
                           compatibility := false, debug := false,
                           wanted_msaa: s32 = DEFAULT_MSAA) -> *NSOpenGLContext, chosen_msaa: s32 {
        return NativeNSGL.create(major_version, minor_version, compatibility, debug, wanted_msaa);
    }

    nsgl_create_context :: (win: *NSWindow, major_version: s32 = 0, minor_version: s32 = 0,
                           compatibility := false, debug := false) -> *NSOpenGLContext {
        if !win {
            NativeNSGL.diagnostic("NSOpenGL requires a native window");
            return null;
        }

        view := AppKit.NSWindow.contentView(win);
        if !view {
            NativeNSGL.diagnostic("NSOpenGL window has no content view");
            return null;
        }

        created := nsgl_create_context(major_version, minor_version, compatibility, debug, DEFAULT_MSAA);
        if created NativeNSGL.attach(created, view);

        return created;
    }

    nsgl_get_proc_address :: (name: string) -> *void {
        text := Native.terminated_copy(name);
        defer Native.release(text);
        return binding_dlsym(cast(*void) -2, text);
    }
}

gl_enable_debug_output :: (break_on_error: bool = false) {
    Set :: #type (callback: GLDEBUGPROC, user_data: *void) #c_call;
    install: Set;

    if gl.GL_VERSION_4_3 {
        install = cast(Set) gl.glDebugMessageCallback;
    } else if gl.GL_ARB_debug_output {
        install = cast(Set) gl.glDebugMessageCallbackARB;
    } else {
        return;
    }

    binding_debug_state.logger = context.logger;
    binding_debug_state.logger_data = context.logger_data;
    binding_debug_state.break_on_error = break_on_error;
    install(binding_debug_message, *binding_debug_state);
    if break_on_error gl.glEnable(GL_DEBUG_OUTPUT_SYNCHRONOUS);
}

// The reviewed convenience API intentionally leaves native lifetime ownership
// with the caller and exposes no result. Use constructors for explicit handles.
gl_create_context :: (win: $Window, major_version: s32 = 0, minor_version: s32 = 0,
                      compatibility := false, debug := false, wanted_msaa: s32 = DEFAULT_MSAA) {
    #if OS == .WINDOWS {
        created, dc := wgl_create_context(win, null, major_version, minor_version,
                                         compatibility, debug, wanted_msaa);
        if !created {
            binding_context_failure();
            return;
        }

        if Win.wglMakeCurrent(dc, created) == 0 {
            if Win.wglDeleteContext(created) == 0 binding_abort();
            if Win.ReleaseDC(win, dc) == 0 binding_abort();
            binding_context_failure();
        }
    } else #if OS == .LINUX {
        created := glx_create_context(win, major_version, minor_version, compatibility, debug);
        if !created {
            binding_context_failure();
            return;
        }

        // Explicit reviewed behavior: this void wrapper does not check or
        // expose glXMakeCurrent's result. Native lifetime remains caller-owned.
        X.glXMakeCurrent(X.x_global_display, win, created);
    } else #if OS == .MACOS {
        if !win {
            binding_context_failure();
            return;
        }

        view := AppKit.NSWindow.contentView(win);
        if !view {
            binding_context_failure();
            return;
        }

        created := nsgl_create_context(major_version, minor_version, compatibility, debug, wanted_msaa);
        if !created return; // Native constructor logged the failed request.

        NativeNSGL.attach(created, view);
        NativeNSGL.make_current(created);
    } else {
        #assert false, "OpenGL has no reviewed context path for this target";
    }
}

#if OS == .WINDOWS {
    wgl_create_context :: (hwnd: HWND, share: HGLRC = null, major_version: s32 = 0, minor_version: s32 = 0,
                          compatibility := false, debug := false, wanted_msaa: s32 = DEFAULT_MSAA) -> HGLRC, HDC, chosen_msaa: s32 {
        return NativeWGL.create(hwnd, share, major_version, minor_version, compatibility, debug, wanted_msaa);
    }
}

#if OS == .LINUX {
    glx_create_context :: (window: Window_Type, major_version: s32 = 0, minor_version: s32 = 0,
                          compatibility := false, debug := false) -> GLXContext, GLXFBConfig, chosen_msaa: s32 {
        display := X.x_global_display;
        if !display || window == 0 || major_version < 0 || minor_version < 0 return null, null, 0;

        server_major, server_minor: s32;
        if binding_glx_version(display, *server_major, *server_minor) == 0 return null, null, 0;
        if server_major < 1 || (server_major == 1 && server_minor < 3) return null, null, 0;

        attributes: X.XWindowAttributes;
        if X.XGetWindowAttributes(display, window, *attributes) == 0 return null, null, 0;

        screen := X.XScreenNumberOfScreen(attributes.screen);
        visual_id := X.XVisualIDFromVisual(attributes.visual);
        count: s32;
        configs := X.glXChooseFBConfig(display, screen, null, *count);

        if !configs return null, null, 0;

        defer X.XFree(configs);

        multisample := binding_glx_extension(display, screen, "GLX_ARB_multisample") ||
                       binding_glx_extension(display, screen, "GLX_SGIS_multisample");

        for i: 0..count - 1 {
            config := configs[i];
            drawable, render: s32;

            if X.glXGetFBConfigAttrib(display, config, X.GLX_DRAWABLE_TYPE, *drawable) != 0 continue;
            if X.glXGetFBConfigAttrib(display, config, X.GLX_RENDER_TYPE, *render) != 0 continue;
            if (drawable & X.GLX_WINDOW_BIT) == 0 || (render & X.GLX_RGBA_BIT) == 0 continue;

            visual := binding_glx_visual(display, config);
            if !visual continue;

            defer X.XFree(visual);
            if visual.visualid != visual_id continue;

            samples: s32;

            if multisample {
                buffers: s32;
                if X.glXGetFBConfigAttrib(display, config, X.GLX_SAMPLE_BUFFERS, *buffers) != 0 continue;
                if buffers != 0 && X.glXGetFBConfigAttrib(display, config, X.GLX_SAMPLES, *samples) != 0 continue;
            }

            created: GLXContext;

            if major_version == 0 && minor_version == 0 && !debug && !compatibility {
                created = X.glXCreateContext(display, visual, null, 1);
            } else {
                if !binding_glx_extension(display, screen, "GLX_ARB_create_context") return null, null, 0;

                profile := major_version > 3 || (major_version == 3 && minor_version >= 2);
                if profile && !binding_glx_extension(display, screen, "GLX_ARB_create_context_profile") return null, null, 0;

                create := cast(Binding_GLX_Create) X.glXGetProcAddress("glXCreateContextAttribsARB");
                if !create return null, null, 0;

                // GLX's default request is 1.0 when no version is supplied.
                requested_major := ifx major_version == 0 then 1 else major_version;
                flags: s32 = ifx debug then X.GLX_CONTEXT_DEBUG_BIT_ARB else 0;
                options: [9] s32;
                options[0] = X.GLX_CONTEXT_MAJOR_VERSION_ARB;
                options[1] = requested_major;
                options[2] = X.GLX_CONTEXT_MINOR_VERSION_ARB;
                options[3] = minor_version;
                options[4] = X.GLX_CONTEXT_FLAGS_ARB;
                options[5] = flags;

                if profile {
                    options[6] = X.GLX_CONTEXT_PROFILE_MASK_ARB;
                    options[7] = ifx compatibility then X.GLX_CONTEXT_COMPATIBILITY_PROFILE_BIT_ARB else X.GLX_CONTEXT_CORE_PROFILE_BIT_ARB;
                }

                created = create(display, config, null, 1, options.data);
            }

            if created return created, config, samples;
        }

        return null, null, 0;
    }
}

#scope_file
Native :: #import "internal/bindings/native";

binding_context_failure :: () {
    if binding_printf("OpenGL context creation/current selection failed\n") < 0 binding_abort();
    if binding_fflush(null) != 0 binding_abort();
}

#if OS == .MACOS {
    AppKit :: #import "Objective_C/AppKit";
    NativeNSGL :: #import "internal/bindings/nsgl";

    NSWindow :: AppKit.NSWindow;
    NSOpenGLContext :: NativeNSGL.NSOpenGLContext;
}

#if OS == .WINDOWS {
    Win :: #import "Windows";
    NativeWGL :: #import "internal/bindings/wgl";

    HWND :: Win.HWND;
    HGLRC :: Win.HGLRC;
    HDC :: Win.HDC;
}

#if OS == .LINUX {
    X :: #import "X11";
    WT :: #import "Window_Type";

    Window_Type :: WT.Window_Type;
    GLXContext :: X.GLXContext;
    GLXFBConfig :: X.GLXFBConfig;
    binding_glx_lib :: #library,system "GLX";
    binding_glx_visual :: (display: *X.Display, config: X.GLXFBConfig) -> *X.XVisualInfo #foreign binding_glx_lib "glXGetVisualFromFBConfig";
    binding_glx_extensions :: (display: *X.Display, screen: s32) -> *u8 #foreign binding_glx_lib "glXQueryExtensionsString";
    binding_glx_version :: (display: *X.Display, major: *s32, minor: *s32) -> s32 #foreign binding_glx_lib "glXQueryVersion";

    Binding_GLX_Create :: #type (display: *X.Display, config: X.GLXFBConfig,
                                share: X.GLXContext, direct: s32, options: *s32) -> X.GLXContext #c_call;

    binding_glx_extension :: (display: *X.Display, screen: s32, wanted: string) -> bool {
        pointer := binding_glx_extensions(display, screen);
        if !pointer return false;

        text: string = .{ data = pointer, count = cast(s64) binding_strlen(pointer) };
        begin: s64;

        for i: 0..text.count {
            if i == text.count || text[i] == #char " " {
                token: string = .{ data = text.data + begin, count = i - begin };
                if token == wanted return true;
                begin = i + 1;
            }
        }

        return false;
    }
}

libc :: #library,system "c";
binding_sscanf :: (text: *u8, format: *u8, args: ..Any) -> s32 #foreign libc "sscanf";
binding_printf :: (format: *u8, args: ..Any) -> s32 #foreign libc "printf";
binding_fflush :: (stream: *void) -> s32 #foreign libc "fflush";
binding_strlen :: (text: *u8) -> u64 #foreign libc "strlen";
binding_abort :: () #foreign libc "abort";

// Every registration shares this module-owned state. A later call deliberately
// changes the logger and break policy observed by previously installed callbacks.
Binding_Debug_State :: struct {
    logger: Logger;
    logger_data: *void;
    break_on_error: bool;
}

binding_debug_state: Binding_Debug_State;

binding_debug_message :: (source: GLenum, type: GLenum, id: GLuint, severity: GLenum,
                          length: GLsizei, message: *GLchar, user_data: *void) #c_call {
    if source == GL_DEBUG_SOURCE_APPLICATION || type == GL_DEBUG_TYPE_OTHER ||
        type == GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR return;

    state := cast(*Binding_Debug_State) user_data;

    push_context, defer_pop;
    text: string = .{ data = message, count = length };

    state.logger(text, state.logger_data, .{});
    if state.break_on_error && type == GL_DEBUG_TYPE_ERROR debug_break();
}

#if OS == .MACOS {
    binding_dlsym :: (handle: *void, name: *u8) -> *void #foreign libc "dlsym";
}

binding_starts_with :: (text: string, prefix: string) -> bool {
    if text.count < prefix.count return false;
    for i: 0..prefix.count - 1 if text[i] != prefix[i] return false;
    return true;
}

binding_query_version :: (procs: *GL_Procedures) -> s32, s32 {
    major, minor: s32;

    if procs.glGetIntegerv {
        procs.glGetIntegerv(GL_MAJOR_VERSION, *major);
        procs.glGetIntegerv(GL_MINOR_VERSION, *minor);
    }

    if major == 0 {
        if !procs.glGetString binding_abort();
        text := procs.glGetString(GL_VERSION);
        if text && binding_sscanf(text, "%d.%d", *major, *minor) != 2 return 0, 0;
    }

    return major, minor;
}

binding_supports_extension :: (procs: *GL_Procedures, ext: string, major: s32) -> bool {
    if ext.count == 0 return false;

    if major >= 3 {
        if !procs.glGetIntegerv || !procs.glGetStringi {
            if binding_printf("OpenGL indexed extension queries require loaded procedure pointers\n") < 0 binding_abort();
            if binding_fflush(null) != 0 binding_abort();
            binding_abort();
        }

        count: GLint;

        procs.glGetIntegerv(GL_NUM_EXTENSIONS, *count);

        for i: 0..count - 1 {
            pointer := procs.glGetStringi(GL_EXTENSIONS, cast(GLuint) i);
            if !pointer continue;
            name: string = .{ data = pointer, count = cast(s64) binding_strlen(pointer) };
            if name == ext return true;
        }
    } else {
        pointer := procs.glGetString(GL_EXTENSIONS);
        if !pointer return false;

        list: string = .{ data = pointer, count = cast(s64) binding_strlen(pointer) };
        begin: s64;

        for i: 0..list.count {
            if i == list.count || list[i] == #char " " {
                candidate: string = .{ data = list.data + begin, count = i - begin };
                if candidate == ext return true;
                begin = i + 1;
            }
        }
    }

    return false;
}

binding_resolve :: (name: *u8) -> *void #c_call {
    #if OS == .WINDOWS {
        symbol := Win.wglGetProcAddress(name);
        address := cast(u64) symbol;
        if address > 3 && address != 0xffff_ffff_ffff_ffff return symbol;
        return Win.GetProcAddress(Win.GetModuleHandleA("opengl32.dll"), name);
    } else #if OS == .LINUX {
        return X.glXGetProcAddress(name);
    } else {
        return binding_dlsym(cast(*void) -2, name);
    }
}
