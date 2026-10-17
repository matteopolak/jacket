prepare :: (window: WT.Window_Type, wanted_msaa: s32) -> s32 {
    existing := find(window);
    if existing return existing.samples;

    value: Context;
    value.window = window;

    #if OS == .MACOS {
        created, actual := GL.nsgl_create_context(3, 3, false, false, wanted_msaa);

        Basic.assert(created != null, "native NSOpenGL context creation failed");
        value.native = created;
        value.samples = actual;

        if shared_root != null {
            format := Cocoa.result_message(created, "pixelFormat", *void);
            allocated := O.objc_alloc(O.objc_getClass("NSOpenGLContext"));
            Init :: #type (receiver: *void, selector: O.Selector, format: *void, shared: *void) -> *void #c_call;
            init := cast(Init) message;

            value.native = init(allocated, O.selector("initWithFormat:shareContext:"), format, shared_root);
            Cocoa.release(created);
            Basic.assert(value.native != null, "native shared NSOpenGL context creation failed");
        }

        if shared_root == null shared_root = value.native;

        Cocoa.attach_render_context(window, value.native);
        Lifecycle.listen(close_window);
    } else #if OS == .LINUX {
        created, config, actual := GL.glx_create_context(window, 3, 3, false, false);

        Basic.assert(created != null, "native GLX context creation failed");
        value.native = created;
        value.samples = actual;

        if contexts.count > 0 {
            Create :: #type (display: *X.Display, config: X.GLXFBConfig, shared: X.GLXContext, direct: s32, attributes: *s32) -> X.GLXContext #c_call;
            create := cast(Create) X.glXGetProcAddress("glXCreateContextAttribsARB");

            Basic.assert(create != null, "native GLX shared-context entry point is unavailable");
            attributes := s32.[0x2091, 3, 0x2092, 3, 0x9126, 1, 0];

            value.native = create(X.x_global_display, config, contexts[0].native, 1, attributes.data);
            X.glXDestroyContext(X.x_global_display, created);
            Basic.assert(value.native != null, "native shared GLX context creation failed");
        }
    } else #if OS == .WINDOWS {
        previous := W.wglGetCurrentContext();
        previous_dc := W.wglGetCurrentDC();

        native_clear_error(0);
        created, dc, actual := GL.wgl_create_context(window, shared_root, 3, 3, false, false, wanted_msaa);

        error_after_cleanup := cast(u32) W.GetLastError();

        if !created || !dc log_wgl_creation_failure(window, wanted_msaa, error_after_cleanup, previous, previous_dc);

        Basic.assert(created != null && dc != null, "native WGL context creation failed");
        Basic.assert(W.wglGetCurrentContext() == previous && W.wglGetCurrentDC() == previous_dc, "native WGL constructor did not restore the previous current context/DC");
        value.native = created;
        value.aux = dc;
        value.samples = actual;
        prepare_retirement_surface(*value);

        if shared_root == null {
            shared_root = created;
            shared_dc = value.retirement_dc;
        }

        Lifecycle.listen(close_window);
    } else {
        #assert false, "this rendering backend requires a native OpenGL context path";
    }

    if !contexts.allocator.proc contexts.allocator = context.default_allocator;

    Basic.array_add(*contexts, value);
    make_current(window);

    return value.samples;
}

Closing_Proc :: #type (window: WT.Window_Type, native_identity: *void);
on_closing: Closing_Proc;

make_current :: (window: WT.Window_Type) {
    found := find(window);

    Basic.assert(found != null, "window has no renderer context");
    previous := current_native();
    desired := found.native;

    #if OS == .MACOS desired = Cocoa.result_message(found.native, "CGLContextObj", *void);
    fence: GL.GLsync;

    if previous != null && previous != desired && GL.gl.glFlush {
        if owned_native(previous) {
            Basic.assert(GL.gl.glFenceSync != null && GL.gl.glWaitSync != null && GL.gl.glDeleteSync != null, "shared contexts require native GPU synchronization");
            fence = GL.gl.glFenceSync(GL.GL_SYNC_GPU_COMMANDS_COMPLETE, 0);
            Basic.assert(fence != 0, "shared-context fence allocation failed");
        }

        GL.gl.glFlush();
    }

    #if OS == .MACOS Cocoa.void_message(found.native, "makeCurrentContext");
    else #if OS == .LINUX Basic.assert(X.glXMakeCurrent(X.x_global_display, window, found.native) != 0, "GLX context selection failed");
    else #if OS == .WINDOWS {
        dc := found.aux;
        if found.retiring dc = found.retirement_dc;
        select_wgl(dc, found.native, window, "select");
    }

    if fence != 0 {
        GL.gl.glWaitSync(fence, 0, 0xffffffffffffffff);
        GL.gl.glDeleteSync(fence);
        Basic.assert(GL.gl.glGetError() == GL.GL_NO_ERROR, "shared-context GPU synchronization failed");
    }
}

update :: (window: WT.Window_Type) {
    found := find(window);
    Basic.assert(found != null);
    #if OS == .MACOS Cocoa.void_message(found.native, "update");
}

swap :: (window: WT.Window_Type, vsync: bool) {
    make_current(window);
    found := find(window);

    #if OS == .MACOS {
        interval: s32 = cast(s32) vsync;
        Set :: #type (receiver: *void, selector: O.Selector, values: *s32, parameter: u64) #c_call;
        set := cast(Set) message;

        set(found.native, O.selector("setValues:forParameter:"), *interval, 222);
        Cocoa.void_message(found.native, "flushBuffer");
    } else #if OS == .LINUX {
        Swap_Interval :: #type (display: *X.Display, window: X.GLXDrawable, interval: s32) #c_call;
        interval := cast(Swap_Interval) X.glXGetProcAddress("glXSwapIntervalEXT");
        if interval interval(X.x_global_display, window, cast(s32) vsync);
        X.glXSwapBuffers(X.x_global_display, window);
    } else #if OS == .WINDOWS {
        // The canonical loader rejects WGL's non-null failure sentinels.
        GL.gl_load(*GL.gl);
        if GL.gl.wglSwapIntervalEXT Basic.assert(GL.gl.wglSwapIntervalEXT(cast(s32) vsync) != 0, "WGL swap interval failed");
        Basic.assert(W.SwapBuffers(found.aux) != 0, "native window buffer swap failed");
    }
}

current_native :: () -> *void {
    #if OS == .MACOS return native_current_context();
    else #if OS == .LINUX return X.glXGetCurrentContext();
    else #if OS == .WINDOWS return W.wglGetCurrentContext();
    else return null;
}

Selection :: struct {
    native: *void;
    display_or_dc: *void;
    drawable: u64;
}

capture :: () -> Selection {
    value: Selection;
    value.native = current_native();

    #if OS == .LINUX {
        value.display_or_dc = current_display();
        value.drawable = current_drawable();
    }

    #if OS == .WINDOWS value.display_or_dc = W.wglGetCurrentDC();

    return value;
}

restore :: (value: Selection) {
    #if OS == .MACOS {
        status := native_set_context(value.native);
        if status != 0 {
            context.logger("CGL context restoration failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            Basic.assert(false, "CGL context restoration failed");
        }
    }

    #if OS == .LINUX {
        restored := X.glXMakeCurrent(value.display_or_dc, value.drawable, value.native);
        if restored == 0 {
            context.logger("GLX context restoration failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            Basic.assert(false, "GLX context restoration failed");
        }
    }

    #if OS == .WINDOWS select_wgl(value.display_or_dc, value.native, null, "restore");
}

is_shared_native :: (native: *void) -> bool {
    return native != null && owned_native(native);
}

share_owner_selection :: () -> Selection {
    value: Selection;

    #if OS == .MACOS {
        Basic.assert(shared_root != null);
        value.native = Cocoa.result_message(shared_root, "CGLContextObj", *void);
    } else #if OS == .WINDOWS {
        Basic.assert(shared_root != null);
        value.native = shared_root;
        value.display_or_dc = shared_dc;
    } else {
        Basic.assert(false, "this backend has no detached share-owner context");
    }

    return value;
}

#scope_module
WT :: #import "Window_Type";
GL :: #import "GL";
Basic :: #import "Basic";
Lifecycle :: #import "internal/window/lifecycle";

#if OS == .MACOS {
    Cocoa :: #import "internal/window/cocoa";
    O :: #import "Objective_C";

    objc :: #library,system "objc";
    message :: () #foreign objc "objc_msgSend";
    native_gl :: #library,system "OpenGL";
    native_current_context :: () -> *void #foreign native_gl "CGLGetCurrentContext";
    native_set_context :: (native: *void) -> s32 #foreign native_gl "CGLSetCurrentContext";
}

#if OS == .LINUX {
    X :: #import "X11";

    native_glx :: #library,system "GLX";
    current_display :: () -> *X.Display #foreign native_glx "glXGetCurrentDisplay";
    current_drawable :: () -> X.GLXDrawable #foreign native_glx "glXGetCurrentDrawable";
}

#if OS == .WINDOWS {
    W :: #import "Windows";

    context_user32 :: #library,system "user32";
    context_kernel32 :: #library,system "kernel32";
    native_class_style :: (window: W.HWND, index: s32) -> u32 #foreign context_user32 "GetClassLongW";
    native_clear_error :: (error: u32) #foreign context_kernel32 "SetLastError";

    select_wgl :: (dc: W.HDC, native: W.HGLRC, window: W.HWND, stage: string) {
        previous := W.wglGetCurrentContext();
        previous_dc := W.wglGetCurrentDC();

        native_clear_error(0);
        if W.wglMakeCurrent(dc, native) != 0 return;

        error := cast(u32) W.GetLastError();
        report := Basic.sprint("WGL native failure: stage=%, error=%, window=%, context=%, dc=%, prior_context=%, prior_dc=%\n", stage, error, window, native, dc, previous, previous_dc);

        defer Basic.free(report);
        context.logger(report, context.logger_data, Log_Info.{ common_flags = .ERROR });
        Basic.assert(false, "WGL context selection failed");
    }

    prepare_retirement_surface :: (value: *Context) {
        // An actual native window/DC keeps the context drawable valid after
        // driver WM_DESTROY hooks retire the application's visible surface.
        // Copy the immutable pixel format exactly, including multisampling.
        bounds: W.RECT;

        Basic.assert(W.GetWindowRect(value.window, *bounds) != 0, "retirement surface placement failed");
        name := u16.[83, 84, 65, 84, 73, 67, 0];

        value.retirement_window = W.CreateWindowExW(0, name.data, null, 0, bounds.left, bounds.top, 1, 1, null, null, null, null);
        Basic.assert(value.retirement_window != null, "retirement native surface creation failed");
        value.retirement_dc = W.GetDC(value.retirement_window);
        Basic.assert(value.retirement_dc != null, "retirement native DC acquisition failed");
        format := W.GetPixelFormat(value.aux);
        descriptor: W.PIXELFORMATDESCRIPTOR;

        Basic.assert(format != 0 && W.DescribePixelFormat(value.aux, format, cast(u32) size_of(W.PIXELFORMATDESCRIPTOR), *descriptor) != 0, "retirement pixel format query failed");
        Basic.assert(W.SetPixelFormat(value.retirement_dc, format, *descriptor) != 0, "retirement pixel format installation failed");
    }

    log_wgl_creation_failure :: (window: W.HWND, samples: s32, error_after_cleanup: u32, previous: W.HGLRC, previous_dc: W.HDC) {
        report := Basic.sprint("WGL creation failed: requested=3.3 core, debug=false, MSAA=%, window=%, share=%, post_cleanup_error=%, prior_context=%, prior_dc=%, current_context=%, current_dc=%\n", samples, window, shared_root, error_after_cleanup, previous, previous_dc, W.wglGetCurrentContext(), W.wglGetCurrentDC());

        defer Basic.free(report);
        context.logger(report, context.logger_data, Log_Info.{ common_flags = .ERROR });
        // The binding's cleanup can replace GetLastError. Report that fact rather
        // than mislabelling this as the error at the failed native creation call.
        dc := W.GetDC(window);
        if dc {
            format := W.GetPixelFormat(dc);
            descriptor: W.PIXELFORMATDESCRIPTOR;
            described := false;

            if format != 0 described = W.DescribePixelFormat(dc, format, cast(u32) size_of(W.PIXELFORMATDESCRIPTOR), *descriptor) != 0;

            detail: string;
            if described detail = Basic.sprint("WGL target surface: pixel_format=%, PFD_size=%, flags=%, RGBA_type=%, color_bits=%, alpha_bits=%, depth_bits=%, stencil_bits=%\n", format, size_of(W.PIXELFORMATDESCRIPTOR), descriptor.dwFlags, descriptor.iPixelType, descriptor.cColorBits, descriptor.cAlphaBits, descriptor.cDepthBits, descriptor.cStencilBits);
            else detail = Basic.sprint("WGL target surface: pixel_format=%, descriptor unavailable, diagnostic_error=%\n", format, cast(u32) W.GetLastError());

            defer Basic.free(detail);
            context.logger(detail, context.logger_data, Log_Info.{ common_flags = .ERROR });
            native_clear_error(0);
            style := native_class_style(window, -26);

            Basic.assert(style != 0 || cast(u32) W.GetLastError() == 0, "WGL diagnostic window class query failed");
            if (style & (W.CS_OWNDC | 0x40)) == 0 Basic.assert(W.ReleaseDC(window, dc) != 0, "WGL diagnostic common DC release failed");
        } else {
            context.logger("WGL target surface: GetDC failed during diagnostic capture\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        }
    }
}

Context :: struct {
    window: WT.Window_Type;
    native: *void;
    aux: *void;
    samples: s32;
    #if OS == .WINDOWS {
        retirement_window: W.HWND;
        retirement_dc: W.HDC;
        retiring: bool;
    }
}

contexts: [..] Context;
shared_root: *void;
#if OS == .WINDOWS shared_dc: W.HDC;

owned_native :: (native: *void) -> bool {
    #if OS == .MACOS {
        if shared_root && Cocoa.result_message(shared_root, "CGLContextObj", *void) == native return true;
        for value: contexts if Cocoa.result_message(value.native, "CGLContextObj", *void) == native return true;
    } else {
        if shared_root == native return true;
        for value: contexts if value.native == native return true;
    }

    return false;
}

close_window :: (window: WT.Window_Type) {
    #if OS == .MACOS {
        for value, index: contexts if value.window == window {
            previous := capture();
            identity := Cocoa.result_message(value.native, "CGLContextObj", *void);

            if on_closing on_closing(window, identity);
            if GL.gl.glFinish GL.gl.glFinish();

            Cocoa.detach_render_context(window, value.native);

            if previous.native == identity {
                Cocoa.void_message(shared_root, "makeCurrentContext");
            } else restore(previous);

            if value.native != shared_root Cocoa.release(value.native);

            Basic.array_unordered_remove_by_index(*contexts, index);

            return;
        }
    } else #if OS == .WINDOWS {
        for value, index: contexts if value.window == window {
            previous := capture();

            // Native driver hooks can retire the visible drawable before WM_DESTROY
            // reaches our procedure. Never select that drawable during cleanup.
            contexts[index].retiring = true;

            if previous.native == value.native {
                previous.display_or_dc = value.retirement_dc;
                // Renderer retirement flushes queued work before deleting its
                // objects. A current dying drawable must already be replaced.
                select_wgl(value.retirement_dc, value.native, window, "retiring current window");
            }

            if on_closing on_closing(window, value.native);
            if GL.gl.glFinish GL.gl.glFinish();

            if previous.native != value.native {
                restore(previous);
            } else {
                next: *Context;

                for *other: contexts if other.window != window {
                    next = other;
                    break;
                }

                if next select_wgl(next.aux, next.native, next.window, "surviving window");
                else select_wgl(shared_dc, shared_root, null, "share owner");
            }

            if value.native != shared_root {
                Basic.assert(W.wglDeleteContext(value.native) != 0, "WGL context deletion failed");
                Basic.assert(W.ReleaseDC(value.retirement_window, value.retirement_dc) != 0, "retirement common DC release failed");
                Basic.assert(W.DestroyWindow(value.retirement_window) != 0, "retirement surface destruction failed");
            }

            native_clear_error(0);
            style := native_class_style(window, -26); // GCL_STYLE, a DWORD rather than a pointer.

            Basic.assert(style != 0 || cast(u32) W.GetLastError() == 0, "native window class query failed");
            // The OS owns class/private DCs. Only common GetDC results need ReleaseDC.
            if (style & (W.CS_OWNDC | 0x40)) == 0 Basic.assert(W.ReleaseDC(window, value.aux) != 0, "native window DC release failed");

            Basic.array_unordered_remove_by_index(*contexts, index);

            return;
        }
    }
}

find :: (window: WT.Window_Type) -> *Context {
    for *item: contexts if item.window == window return item;
    return null;
}
