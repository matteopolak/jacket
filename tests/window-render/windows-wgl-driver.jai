// Original native SDK probe: prints actual driver identity and each failure stage.
// No version override, GL substitute, skipped rendering assertion or hidden retry.
#import "Basic";
W :: #import "Windows";
WC :: #import "Window_Creation";
GL :: #import "GL";

kernel32 :: #library,system "kernel32";
clear_error :: (value: u32) #foreign kernel32 "SetLastError";

required :: (ok: bool, stage: string) {
    error := cast(u32) W.GetLastError();
    if !ok print("WGL native failure: stage=%, error=%\n", stage, error);
    assert(ok, "native WGL probe stage failed; see preceding diagnostic");
}

native_text :: (bytes: *u8) -> string {
    assert(bytes != null, "native driver returned no identity string");
    count: int;
    while bytes[count] != 0 count += 1;
    return .{ count, bytes };
}

extensions :: (dc: W.HDC) -> string {
    Get :: #type (dc: W.HDC) -> *u8 #c_call;
    pointer := W.wglGetProcAddress("wglGetExtensionsStringARB");
    address := cast(u64) pointer;

    if address <= 3 || address == 0xffffffffffffffff return "";

    get := cast(Get) pointer;
    result := get(dc);

    if !result return "";

    return native_text(result);
}

main :: () {
    // PFD is the native 40-byte wingdi.h record on both supported Windows CPUs.
    assert(size_of(W.PIXELFORMATDESCRIPTOR) == 40);
    window := WC.create_window(200, 120, "Native WGL driver diagnostics");

    required(window != null, "public create_window");
    defer required(W.DestroyWindow(window) != 0, "DestroyWindow");
    prior := W.wglGetCurrentContext();
    prior_dc := W.wglGetCurrentDC();
    dc := W.GetDC(window);

    required(dc != null, "GetDC");
    // This public window has CS_OWNDC: Windows owns the DC until destruction.
    descriptor: W.PIXELFORMATDESCRIPTOR;
    descriptor.nSize = cast(s16) size_of(W.PIXELFORMATDESCRIPTOR);
    descriptor.nVersion = 1;
    descriptor.dwFlags = W.PFD_DRAW_TO_WINDOW | W.PFD_SUPPORT_OPENGL | W.PFD_DOUBLEBUFFER;
    descriptor.iPixelType = W.PFD_TYPE_RGBA;
    descriptor.cColorBits = 24;
    descriptor.cAlphaBits = 8;
    descriptor.cDepthBits = 24;
    clear_error(0);
    chosen := W.ChoosePixelFormat(dc, *descriptor);

    required(chosen != 0, "ChoosePixelFormat");
    required(W.DescribePixelFormat(dc, chosen, cast(u32) size_of(W.PIXELFORMATDESCRIPTOR), *descriptor) != 0, "DescribePixelFormat");
    print("PFD: size=%, format=%, flags=%, color=%, alpha=%, depth=%, stencil=%\n", descriptor.nSize, chosen, descriptor.dwFlags, descriptor.cColorBits, descriptor.cAlphaBits, descriptor.cDepthBits, descriptor.cStencilBits);
    needed := W.PFD_DRAW_TO_WINDOW | W.PFD_SUPPORT_OPENGL | W.PFD_DOUBLEBUFFER;

    required((descriptor.dwFlags & needed) == needed && descriptor.iPixelType == W.PFD_TYPE_RGBA, "required PFD capabilities");
    required(W.SetPixelFormat(dc, chosen, *descriptor) != 0, "SetPixelFormat");
    clear_error(0);
    legacy := W.wglCreateContext(dc);

    required(legacy != null, "legacy wglCreateContext");
    defer required(W.wglDeleteContext(legacy) != 0, "legacy wglDeleteContext");
    required(W.wglMakeCurrent(dc, legacy) != 0, "legacy wglMakeCurrent");
    defer required(W.wglMakeCurrent(prior_dc, prior) != 0, "restore previous current context");
    print("legacy GL_VENDOR=%\nGL_RENDERER=%\nGL_VERSION=%\nWGL_EXTENSIONS=%\n", native_text(GL.gl.glGetString(GL.GL_VENDOR)), native_text(GL.gl.glGetString(GL.GL_RENDERER)), native_text(GL.gl.glGetString(GL.GL_VERSION)), extensions(dc));
    module_name := u16.[111, 112, 101, 110, 103, 108, 51, 50, 46, 100, 108, 108, 0];
    library := W.GetModuleHandleW(module_name.data);
    path: [32768] u16;
    count := W.GetModuleFileNameW(library, path.data, cast(s32) path.count);

    required(library != null && count > 0 && count < path.count, "loaded opengl32 module identity");
    utf8: [131072] u8;
    bytes := W.WideCharToMultiByte(65001, 0, path.data, count, utf8.data, cast(s32) utf8.count, null, null);

    required(bytes > 0, "module path UTF-8 conversion");
    print("opengl32=%\n", string.{ bytes, utf8.data });

    // Probe the canonical constructor on an already installed pixel format.
    clear_error(0);
    modern, modern_dc, actual := GL.wgl_create_context(window, null, 3, 3, false, false, 0);
    error_after_cleanup := cast(u32) W.GetLastError();

    print("canonical WGL3.3: context=%, dc=%, samples=%, post_cleanup_error=%\n", modern, modern_dc, actual, error_after_cleanup);
    assert(W.wglGetCurrentContext() == legacy && W.wglGetCurrentDC() == dc, "constructor must restore both previous current handles");
    assert(modern != null && modern_dc != null, "actual OpenGL3.3 core driver support is required");
    defer required(W.wglDeleteContext(modern) != 0, "modern wglDeleteContext");
    required(W.wglMakeCurrent(modern_dc, modern) != 0, "modern wglMakeCurrent");
    // Run before context deletion and before the earlier legacy restoration defer.
    defer required(W.wglMakeCurrent(dc, legacy) != 0, "restore legacy for modern deletion");
    GL.gl_load(*GL.gl);
    major, minor := GL.gl_get_version();

    assert(major > 3 || (major == 3 && minor >= 3));
    profile: s32;

    GL.gl.glGetIntegerv(GL.GL_CONTEXT_PROFILE_MASK, *profile);
    assert((profile & 1) != 0 && GL.gl.glGetError() == GL.GL_NO_ERROR);
    print("modern GL_VERSION=%, profile=%\n", native_text(GL.gl.glGetString(GL.GL_VERSION)), profile);
    GL.gl.glDrawBuffer(GL.GL_BACK);
    GL.gl.glClearColor(1, 0, 1, 1);
    GL.gl.glClear(GL.GL_COLOR_BUFFER_BIT);
    pixel: [4] u8;

    GL.gl.glReadPixels(0, 0, 1, 1, GL.GL_RGBA, GL.GL_UNSIGNED_BYTE, pixel.data);
    assert(GL.gl.glGetError() == GL.GL_NO_ERROR);
    assert(pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 255 && pixel[3] == 255);
    // A genuine unsupported version must fail while preserving the current
    // context/DC and the irreversible, already installed target pixel format.
    rejected, rejected_dc, rejected_samples := GL.wgl_create_context(window, null, 99, 0, false, false, 0);

    print("unsupported version: context=%, dc=%, samples=%, post_cleanup_error=%\n", rejected, rejected_dc, rejected_samples, cast(u32) W.GetLastError());
    assert(rejected == null && rejected_dc == null && rejected_samples == 0);
    assert(W.wglGetCurrentContext() == modern && W.wglGetCurrentDC() == modern_dc);
    assert(W.GetPixelFormat(modern_dc) == chosen);
    print("ok\n");
}
