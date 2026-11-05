// test: skip_on(.WASM, reason = "requires native threads")
#import "Basic";
T :: #import "Thread";

work :: (group: *T.Thread_Group, thread: *T.Thread, item: *void) -> T.Thread_Continue_Status {
    value := cast(*s64) item;
    value.* += 1;
    return .CONTINUE;
}

main :: () {
    group: T.Thread_Group;
    group.logging = false;
    T.init(*group, 2, work);
    T.start(*group);
    items: [32] s64;
    for *item: items T.add_work(*group, item);

    assert(T.wait_for_all(*group, 5000));
    for items assert(it == 1);

    // Completed queues are polled by get_completed_work; no worker waits on their semaphore.
    for *worker: group.worker_info {
        assert(T.wait_for(*worker.completed.semaphore, 0) == .TIMEOUT,
               "completed queue posted a notification with no consumer");
    }

    completed := T.get_completed_work(*group);

    assert(completed.count == items.count);
    for *worker: group.worker_info assert(worker.completed.count == 0);

    assert(T.shutdown(*group, 5000));
    print("ok\n");
}
