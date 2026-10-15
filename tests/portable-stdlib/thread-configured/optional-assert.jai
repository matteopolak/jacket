// Reviewed thread.md native synchronization/lifecycle. Verification stays enabled
// while the runner independently changes Thread's DEFAULT Basic specialization.
#import "Basic"(ENABLE_ASSERT = true);
T :: #import "Thread";

State :: struct {
    mutex: T.Mutex;
    cv: T.Condition_Variable;
    ready: T.Semaphore;
    go: bool;
    result: s64;
}

worker :: (thread: *T.Thread) -> s64 {
    state := cast(*State) thread.data;

    T.csection_lock(*state.mutex.csection);
    T.signal(*state.ready);
    while !state.go T.wait(*state.cv, *state.mutex);

    state.result = 17;
    T.csection_unlock(*state.mutex.csection);

    return 0;
}

group_worker :: (group: *T.Thread_Group, thread: *T.Thread, work: *void) -> T.Thread_Continue_Status {
    (cast(*s64) work).* = 91;
    return .CONTINUE;
}

main :: () {
    semaphore: T.Semaphore;

    T.init(*semaphore);
    T.signal(*semaphore);
    result := T.wait_for(*semaphore, 0);

    assert(result == .SUCCESS);
    result = T.wait_for(*semaphore, 0);
    assert(result == .TIMEOUT);
    T.destroy(*semaphore);

    state: State;

    T.init(*state.mutex);
    T.init(*state.cv);
    T.init(*state.ready);
    T.lock(*state.mutex);
    T.lock(*state.mutex);
    T.unlock(*state.mutex);
    T.unlock(*state.mutex);
    thread: T.Thread;
    thread.data = *state;
    initialized := T.thread_init(*thread, worker);

    assert(initialized);
    T.thread_start(*thread);
    result = T.wait_for(*state.ready, 5000);
    assert(result == .SUCCESS);
    T.lock(*state.mutex);
    state.go = true;
    T.wake_all(*state.cv);
    T.unlock(*state.mutex);
    done := T.thread_is_done(*thread, 5000);

    assert(done);
    T.thread_deinit(*thread);
    assert(thread.index == 0 && state.result == 17);
    T.destroy(*state.ready);
    T.destroy(*state.cv);
    T.destroy(*state.mutex);

    group: T.Thread_Group;
    group.logging = false;
    T.init(*group, 2, group_worker);
    T.start(*group);
    value: s64;

    T.add_work(*group, *value);

    for 0..5000 {
        completed := T.get_completed_work(*group);
        if completed.count > 0 break;

        // A real bounded semaphore timeout yields CPU time without optional checks.
        T.init(*semaphore);
        result = T.wait_for(*semaphore, 1);
        assert(result == .TIMEOUT);
        T.destroy(*semaphore);
    }

    stopped := T.shutdown(*group, 5000);

    assert(stopped && value == 91 && !group.initted);
    print("ok\n");
}
