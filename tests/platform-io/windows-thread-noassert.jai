// Reviewed Thread lifecycle/synchronization contract with optional asserts off.
#import "Basic"(ENABLE_ASSERT = false);
T :: #import "Thread";

Shared :: struct {
    mutex: T.Mutex;
    changed: T.Condition_Variable;
    ready: T.Semaphore;
    count: s64;
}

discarded_calls: s64;
reported_failures: s64;
unexpected_failure: bool;

record_failure :: (loc: Source_Code_Location, message: string) -> bool {
    if message != "semaphore signal failed" && message != "thread handle cleanup failed"
        unexpected_failure = true;
    reported_failures += 1;
    return false;
}

discarded :: () -> bool {
    discarded_calls += 1;
    return false;
}

worker :: (thread: *T.Thread) -> s64 {
    shared := cast(*Shared) thread.data;

    if context.thread_index != thread.index || thread.index <= 0 exit(11);

    T.lock(*shared.mutex);
    shared.count += 1;
    T.wake_all(*shared.changed);
    T.unlock(*shared.mutex);
    T.signal(*shared.ready);

    return 0;
}

group_worker :: (group: *T.Thread_Group, thread: *T.Thread, work: *void) -> T.Thread_Continue_Status {
    value := cast(*s64) work;
    if thread.worker_info.group != group || context.thread_index != thread.index exit(12);
    value.* = 47;
    return .CONTINUE;
}

main :: () {
    assert(discarded());
    if discarded_calls != 0 exit(1);

    // An invalid semaphore signal must diagnose even with a returning handler.
    invalid: T.Semaphore;
    saved_handler := context.assertion_failed;

    context.assertion_failed = record_failure;
    T.signal(*invalid);
    context.assertion_failed = saved_handler;
    if reported_failures != 1 || unexpected_failure exit(13);

    shared: Shared;

    T.init(*shared.mutex);
    T.init(*shared.changed);
    T.init(*shared.ready);
    if T.wait_for(*shared.ready, 0) != .TIMEOUT exit(2);

    thread: T.Thread;
    thread.data = *shared;
    if !T.thread_init(*thread, worker) exit(3);
    if T.thread_is_done(*thread, 0) exit(4);

    T.thread_start(*thread);
    T.lock(*shared.mutex);
    while shared.count == 0 T.wait(*shared.changed, *shared.mutex);

    T.unlock(*shared.mutex);
    if T.wait_for(*shared.ready, 5000) != .SUCCESS exit(5);
    if !T.thread_is_done(*thread, 5000) exit(6);

    // A failed close must retain the real SDK handle and storage for retry.
    retained := thread.windows_thread;
    retained_index := thread.index;
    retained_storage := thread.starting_temporary_storage.data;

    if set_handle_information(retained, 2, 2) == 0 exit(14);

    context.assertion_failed = record_failure;
    T.thread_deinit(*thread);
    context.assertion_failed = saved_handler;
    if reported_failures != 2 || unexpected_failure || thread.windows_thread != retained
        || thread.index != retained_index || thread.starting_temporary_storage.data != retained_storage exit(17);
    if set_handle_information(retained, 2, 0) == 0 exit(15);

    T.thread_deinit(*thread);
    if thread.index != 0 || thread.windows_thread != null exit(7);

    T.destroy(*shared.ready);
    if shared.ready.event != null exit(8);

    T.destroy(*shared.changed);
    T.destroy(*shared.mutex);

    group: T.Thread_Group;
    group.logging = false;
    T.init(*group, 1, group_worker);
    item: s64;

    T.add_work(*group, *item);
    T.start(*group);
    // Graceful shutdown must drain queued work and reap the real worker.
    if !T.shutdown(*group, 5000) exit(9);
    if item != 47 || group.initted || group.worker_info.count != 0 exit(10);

    print("ok\n");
}

#scope_file
kernel32 :: #system_library "kernel32";
set_handle_information :: (handle: *void, mask: u32, flags: u32) -> s32 #foreign kernel32 "SetHandleInformation";
