// Native SDK unit: a uniquely named pasteboard, never the user's general board.
#load "../../internal/clipboard/cocoa.jai";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

recover :: (loc: Source_Code_Location, message: string) -> bool {
    return false;
}

mark :: (count: *s64) -> bool {
    count.* += 1;
    return true;
}

main :: () {
    args := get_command_line_arguments();
    if args.count != 2 exit(2);

    calls: s64;

    assert(mark(*calls));

    if args[1] == "enabled" {
        if calls != 1 exit(10);
    } else if args[1] == "disabled" {
        if calls != 0 exit(11);
    } else exit(12);

    pool := new_object("NSAutoreleasePool");
    if !pool exit(69);

    defer release_object(pool);
    pb := send_object(objc_class("NSPasteboard"), "pasteboardWithUniqueName");
    if !pb exit(70);

    Release :: #type (receiver: *void, selector: *void) #c_call;
    release := cast(Release) objc_send;

    defer release(pb, selector("releaseGlobally"));
    Clear :: #type (receiver: *void, selector: *void) -> s64 #c_call;
    clear := cast(Clear) objc_send;

    if clear(pb, selector("clearContents")) < 0 exit(71);

    Write :: #type (receiver: *void, selector: *void, text: *void, type: *void) -> s8 #c_call;
    write := cast(Write) objc_send;
    kind := native_string("public.utf8-plain-text");
    value := native_string("private SDK regression");

    if write(pb, selector("setString:forType:"), value, kind) == 0 exit(72);

    before := send_object_arg(pb, "stringForType:", kind);
    if !before exit(73);
    if !clear_contents(pb) exit(75);

    after := send_object_arg(pb, "stringForType:", kind);
    if after != null exit(74);

    saved := context;
    scoped := context;
    scoped.logger = capture;
    scoped.assertion_failed = recover;
    context = scoped;
    errors = 0;
    unavailable := new_object("Cleanroom_NoSuchClipboardClass_73C9");
    if unavailable != null || errors == 0 exit(76);

    errors = 0;
    os_clipboard_set_bitmap(-1, 1, null, 4);
    if errors == 0 exit(77);

    context = saved;
    write_string("ok\n");
}

#scope_module
#import "Basic";
