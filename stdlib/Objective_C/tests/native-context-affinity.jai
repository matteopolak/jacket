#import "Basic";
O :: #import "Objective_C";

probe :: #library "window_gl_probe";
appkit :: #library "AppKit";
opengl :: #library "OpenGL";
set_marker :: (value: s32) #foreign probe "window_gl_probe_set_thread_marker";
get_marker :: () -> s32 #foreign probe "window_gl_probe_get_thread_marker";

// The probe object also holds AppKit and OpenGL code; naming one symbol of each framework links them.
application_load :: () -> bool #foreign appkit "NSApplicationLoad";
get_string :: (name: u32) -> *u8 #foreign opengl "glGetString";

main :: () {
    assert(cast(*void) application_load != null && cast(*void) get_string != null);
    set_marker(123);
    assert(get_marker() == 123);
    assert(O.objc_getClass("NSObject") != null);
    assert(get_marker() == 123, "late Objective-C initialization must preserve native thread-local state");
    print("ok\n");
}
