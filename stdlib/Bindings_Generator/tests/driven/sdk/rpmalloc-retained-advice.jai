// Reviewed exact POSIX retained-span advice; actual kernel syscall observation.
RPM :: #import "rpmalloc"(ENABLE_GLOBAL_CACHE = false);
Free :: #import "rpmalloc"(ENABLE_GLOBAL_CACHE = false, LINUX_FREE_ADVICE = .FREE);

libc :: #library,system "c";
abort :: () #foreign libc;
probe :: #library,system "rpmalloc_advice_probe";
begin :: (fail: s32) #foreign probe "binding_advice_begin";
end :: () #foreign probe "binding_advice_end";
count :: (index: u32) -> u64 #foreign probe "binding_advice_count";
report :: () #foreign probe "binding_advice_report";

#add_context advice_marker: s64;
errors: s64;

check :: (condition: bool) {
    if !condition abort();
}

on_error :: (message: string, location: Source_Code_Location) {
    check(context.advice_marker == 713);
    check(message.count > 0 && location.line_number > 0);
    errors += 1;
}

exercise :: (proc: Allocator_Proc, wanted: u32, fail: bool) {
    held := proc(.ALLOCATE, 1048576, 0, null, null);
    blocks: [3] *void;

    check(held != null);
    (cast(*u8) held)[0] = 37;

    for *blocks {
        it.* = proc(.ALLOCATE, 1048576, 0, null, null);
        check(it.* != null);
    }

    begin(ifx fail then 1 else 0);
    for blocks proc(.FREE, 0, 0, it, null);

    proc(.THREAD_STOP, 0, 0, null, null);
    end();
    report();
    check(count(wanted) > 0);
    check(count(1 - wanted) == 0 && count(2) == 0);

    if fail {
        check(count(3) > 0 && count(4) != 0 && errors > 0);
    } else {
        check(count(3) == 0 && errors == 0);
    }

    proc(.THREAD_START, 0, 0, null, null);
    check((cast(*u8) held)[0] == 37);
    proc(.FREE, 0, 0, held, null);
    proc(.SHUTDOWN, 0, 0, null, null);
}

main :: () {
    check(RPM.initialize() == 0);
    check(RPM.config().span_map_count > 0);
    exercise(RPM.rpmalloc_allocator_proc, 0, false);
    check(Free.initialize() == 0);
    exercise(Free.rpmalloc_allocator_proc, 1, false);
    context.advice_marker = 713;
    config: RPM.rpmalloc_config_t;
    config.error_callback = on_error;
    check(RPM.initialize_config(*config) == 0);
    exercise(RPM.rpmalloc_allocator_proc, 0, true);
    check(context.advice_marker == 713);
}
