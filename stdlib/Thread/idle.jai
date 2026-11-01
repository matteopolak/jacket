// Waits until every piece of work added to `group` has been run by a worker, or until `timeout_milliseconds` pass (-1 waits as
// long as it takes). Returns `false` on a timeout. The group has to be started, and has to have a live worker to run the work:
// if every worker stopped with work still queued, only a timeout ends the wait.
//
// What was run is then ready for `get_completed_work`. More work can be added afterwards.
//
// ```jai
// Counter :: struct { mutex: Mutex; total: s64; }
//
// count :: (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status {
//     counter := cast(*Counter) group.data;
//     lock(*counter.mutex);
//     counter.total += 1;
//     unlock(*counter.mutex);
//     return .CONTINUE;
// }
//
// counter: Counter;
// init(*counter.mutex);
// group: Thread_Group;
// group.logging = false;
// init(*group, 4, count);
// group.data = *counter;
// start(*group);
// for 1..100 add_work(*group, null);
//
// assert(wait_for_all(*group));
// assert(counter.total == 100);
// assert(shutdown(*group));
// destroy(*counter.mutex);
// ```
//
// - timeout_milliseconds: how long to wait at most, or -1 for no limit.
wait_for_all :: (group: *Thread_Group, timeout_milliseconds: s32 = -1) -> bool {
    assert(group.initted, "wait_for_all needs a group that has been set up and not shut down");

    deadline: s64;

    if timeout_milliseconds >= 0 {
        now, ok := monotonic_milliseconds();
        if !ok return false;
        deadline = now + timeout_milliseconds;
    }

    lock(*group.idle_mutex);
    defer unlock(*group.idle_mutex);

    while true {
        _, pending := compare_and_swap(*group.outstanding, cast(s64) 0, cast(s64) 0);
        if pending == 0 return true;

        if timeout_milliseconds < 0 {
            wait(*group.idle_changed, *group.idle_mutex);
            continue;
        }

        now, ok := monotonic_milliseconds();
        if !ok || now >= deadline return false;

        cvariable_wait(*group.idle_changed.condition_variable, *group.idle_mutex.csection, cast(s32) min(deadline - now, cast(s64) 0x7fffffff));
    }

    return false;
}
