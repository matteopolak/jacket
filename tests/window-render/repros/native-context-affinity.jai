#import "Basic";
O :: #import "Objective_C";

probe :: #library "window_gl_probe";
set_marker :: (value: s32) #foreign probe "window_gl_probe_set_thread_marker";
get_marker :: () -> s32 #foreign probe "window_gl_probe_get_thread_marker";

main :: () {
    set_marker(123);
    assert(get_marker() == 123);
    assert(O.objc_getClass("NSObject") != null);
    assert(get_marker() == 123, "late Objective-C initialization must preserve native thread-local state");
    print("ok\n");
}
