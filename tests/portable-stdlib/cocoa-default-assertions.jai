#import "Basic";
W :: #import "Window_Creation";
I :: #import "Input";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

recover :: (loc: Source_Code_Location, message: string) -> bool {
    return false;
}

mark :: (calls: *s64) -> bool {
    calls.* += 1;
    return true;
}

main :: () {
    args := get_command_line_arguments();
    if args.count != 3 exit(2);

    calls: s64;

    assert(mark(*calls));

    if args[1] == "enabled" {
        if calls != 1 exit(3);
    } else if args[1] == "disabled" {
        if calls != 0 exit(4);
    } else exit(5);

    preparation := prepare();
    if preparation != 1 {
        print("SDK preparation failed: %\n", preparation);
        exit(6);
    }

    if args[2] == "application" {
        W.init_mac_app();
        if !W.application_initialized || policy() != 0 exit(7);
    } else if args[2] == "observer" {
        I.update_window_events();
        if observer_has_method() != 1 exit(8);
        I.update_window_events();
        if observer_has_method() != 1 exit(10);
    } else if args[2] == "collision" {
        W.init_mac_app();
        if !W.application_initialized || register_collision() != 1 exit(11);

        saved := context;
        scoped := context;
        scoped.logger = capture;
        scoped.assertion_failed = recover;
        context = scoped;
        I.update_window_events();
        I.update_window_events();
        context = saved;
        if errors != 2 || observer_has_method() != 0 exit(12);
    } else exit(9);

    write_string("ok\n");
}

#scope_file
prepare :: () -> s32 #foreign COCOA_PROBE "portable_cocoa_prepare";
policy :: () -> s32 #foreign COCOA_PROBE "portable_cocoa_policy";
observer_has_method :: () -> s32 #foreign COCOA_PROBE "portable_cocoa_observer_has_method";
register_collision :: () -> s32 #foreign COCOA_PROBE "portable_cocoa_register_collision";
