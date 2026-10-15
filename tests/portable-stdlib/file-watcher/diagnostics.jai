// Reviewed watcher/Basic signatures; independent continuing-handler lifetime safety.
#import "Basic";
W :: #import "File_Watcher";

reported: s64;
registration_errors: s64;
original: (loc: Source_Code_Location, message: string) -> bool;

Tracking :: struct {
    base: Allocator;
    live: s64;
}

tracked :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracking) allocator_data;
    result := state.base.proc(mode, requested_size, old_size, old_memory, state.base.data);

    if mode == .ALLOCATE && result state.live += 1;
    if mode == .FREE && old_memory state.live -= 1;

    return result;
}

log_registration_error :: (message: string, data: *void, info: Log_Info) {
    check(message.count > 0 && (info.common_flags & .ERROR) != 0);
    registration_errors += 1;
}

check :: (condition: bool) {
    previous := context.assertion_failed;
    context.assertion_failed = original;
    defer context.assertion_failed = previous;
    assert(condition, "watcher diagnostic regression failed");
}

capture :: (loc: Source_Code_Location, message: string) -> bool {
    reported += 1;
    check(message.count > 0);
    return false;
}

callback :: (watcher: *W.File_Watcher(void), change: *W.File_Change, data: *void) {
    check(false);
}

main :: () {
    previous := context.assertion_failed;

    original = previous;

    context.assertion_failed = capture;
    defer context.assertion_failed = previous;
    missing: *W.File_Watcher(void);
    initialized := W.init(missing, callback);

    check(!initialized && reported == 1);
    watcher: W.File_Watcher(void);
    changed, waiting, delay := W.process_changes(*watcher);

    check(!changed && !waiting && delay == 0 && reported == 2);
    added := W.add_directories(*watcher, ".");

    check(!added && reported == 3);
    W.deinit(watcher);
    check(reported == 3);

    initialized = W.init(*watcher, callback);
    check(initialized);
    W.deinit(watcher);
    W.deinit(watcher);
    check(reported == 4);
    changed, waiting, delay = W.process_changes(*watcher);
    check(!changed && !waiting && delay == 0 && reported == 5);
    initialized = W.init(*watcher, callback);
    check(!initialized && reported == 6);
    watcher = .{};
    initialized = W.init(*watcher, callback);
    check(initialized);
    W.deinit(watcher);
    check(!context.handling_assertion_failure && reported == 6);

    owner := context.allocator;
    tracking := Tracking.{ base = owner };
    logger := context.logger;

    context.allocator = .{ tracked, *tracking };
    context.logger = log_registration_error;
    watcher = .{};
    initialized = W.init(*watcher, callback);
    check(initialized);
    added = W.add_directories(*watcher, #file);
    check(!added && registration_errors == 1);
    retained_allocations := tracking.live;

    for 0..4095 {
        added = W.add_directories(*watcher, #file);
        check(!added && tracking.live == retained_allocations);
    }

    check(registration_errors == 4097);
    W.deinit(watcher);
    check(tracking.live == 0 && reported == 6);
    context.allocator = owner;
    context.logger = logger;
    print("ok\n");
}
