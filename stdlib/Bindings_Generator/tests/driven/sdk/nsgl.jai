// Real AppKit/CGL constructors, format reporting, ownership and view attachment.
G :: #import "GL";
All :: #import "GL"(ENABLE_ALL_EXTENSIONS = true);
A :: #import "Objective_C/AppKit";
O :: #import "Objective_C";
S :: #import "SDL";

libc :: #library,system "c";
abort :: () #foreign libc;
libobjc :: #library,system "objc";
native_get :: (self: *void, op: O.Selector) -> *void #foreign libobjc "objc_msgSend";
native_release :: (self: *void, op: O.Selector) #foreign libobjc "objc_msgSend";
opengl :: #library,system "OpenGL";
get_current :: () -> *void #foreign opengl "CGLGetCurrentContext";
set_current :: (value: *void) -> s32 #foreign opengl "CGLSetCurrentContext";
get_format :: (value: *void) -> *void #foreign opengl "CGLGetPixelFormat";
describe :: (format: *void, screen: s32, attribute: u32, result: *s32) -> s32 #foreign opengl "CGLDescribePixelFormat";
probe :: #library "../../../../../.work/native/apple_abi_probe";
token :: (which: u32) -> u32 #foreign probe "binding_cgl_token";

check :: (value: bool) {
    if !value abort();
}

main :: () {
    pool := O.objc_new(O.NSAutoreleasePool);

    check(pool != null);
    defer O.NSAutoreleasePool.drain(pool);
    before := get_current();
    offscreen, samples := G.nsgl_create_context(major_version = 4, minor_version = 1, wanted_msaa = 4);

    check(offscreen != null);
    defer native_release(offscreen, O.selector("release"));
    check(get_current() == before);
    info := cast(*Type_Info_Struct) (cast(*Type_Info_Pointer) type_info(type_of(offscreen))).pointer_to;

    check(info.name == "NSOpenGLContext");
    native_context := native_get(offscreen, O.selector("CGLContextObj"));

    check(native_context != null);
    format := get_format(native_context);

    check(format != null);
    buffers, actual: s32;

    check(describe(format, 0, token(6), *buffers) == 0);
    if buffers > 0 check(describe(format, 0, token(5), *actual) == 0);

    check(samples == actual);
    check(set_current(native_context) == 0);
    major, minor := G.gl_get_version();

    check(major > 4 || (major == 4 && minor >= 1));
    check(set_current(before) == 0);

    other, other_samples := All.nsgl_create_context(major_version = 4, minor_version = 1, wanted_msaa = 0);

    #assert type_of(offscreen) == type_of(other);
    check(other != null && other_samples == 0);
    defer native_release(other, O.selector("release"));
    legacy, legacy_samples := G.nsgl_create_context(major_version = 2, minor_version = 1,
        compatibility = true, wanted_msaa = 0);

    check(legacy != null && legacy_samples == 0);
    defer native_release(legacy, O.selector("release"));
    check(set_current(native_get(legacy, O.selector("CGLContextObj"))) == 0);
    version := G.gl.glGetString(G.GL_VERSION);

    check(version != null && version[0] == #char "2" && version[1] == #char ".");
    check(G.gl.glGetError() == G.GL_NO_ERROR);
    check(set_current(before) == 0);

    unsupported, unsupported_samples := G.nsgl_create_context(major_version = 4, minor_version = 6);

    check(unsupported == null && unsupported_samples == 0);
    unsupported, unsupported_samples = G.nsgl_create_context(debug = true);
    check(unsupported == null && unsupported_samples == 0);

    check(S.SDL_Init(S.SDL_INIT_VIDEO) == 0);
    defer S.SDL_Quit();
    window := S.SDL_CreateWindow("NSOpenGL ABI test", 0, 0, 64, 64, S.SDL_WINDOW_HIDDEN);

    check(window != null);
    defer S.SDL_DestroyWindow(window);
    window_info: S.SDL_SysWMinfo;

    S.SDL_VERSION(*window_info.version);
    check(S.SDL_GetWindowWMInfo(window, *window_info) == S.SDL_TRUE);
    native_window := cast(*A.NSWindow) window_info.info.cocoa.window;
    title := "Binding title: café";

    A.NSWindow.setTitle(native_window, title);
    actual_title := cast(*O.NSString) native_get(native_window, O.selector("title"));

    check(actual_title != null && O.to_string(actual_title) == title);
    window_context := G.nsgl_create_context(native_window, major_version = 4, minor_version = 1);

    check(window_context != null);
    defer native_release(window_context, O.selector("release"));
    check(native_get(window_context, O.selector("view")) == cast(*void) A.NSWindow.contentView(native_window));
    check(get_current() == before);

    G.gl_create_context(native_window, major_version = 4, minor_version = 1, wanted_msaa = 0);
    generic_context := native_get(O.objc_getClass("NSOpenGLContext"), O.selector("currentContext"));

    check(generic_context != null);
    check(native_get(generic_context, O.selector("view")) == cast(*void) A.NSWindow.contentView(native_window));
    check(get_current() == native_get(generic_context, O.selector("CGLContextObj")));
    check(G.gl_get_version() >= 4);
    check(set_current(before) == 0);
    native_release(generic_context, O.selector("release"));
}
