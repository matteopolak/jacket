#import "Basic";
T :: #import "Thread";
P :: #import "POSIX";

Work :: struct {
    input: s64;
    result: s64;
}

worker :: (group: *T.Thread_Group, thread: *T.Thread, work: *void) -> T.Thread_Continue_Status {
    item := cast(*Work) work;
    item.result = item.input * 3;
    assert(context.thread_index == thread.index && context.temporary_storage != null);
    memory := talloc(32768);

    assert(memory != null);

    return .CONTINUE;
}

main :: () {
    group: T.Thread_Group;
    group.logging = false;
    T.init(*group, 3, worker, enable_work_stealing = true);
    items: [60] Work;
    for *item, i: items {
        item.input = i;
        T.add_work(*group, item);
    }

    T.start(*group);
    completed: s64;

    for attempt: 0..9999 {
        ready := T.get_completed_work(*group);
        for pointer: ready {
            item := cast(*Work) pointer;
            assert(item.result == item.input * 3);
            completed += 1;
        }

        if completed == items.count break;

        assert(P.usleep(1000) == 0);
    }

    assert(completed == items.count);
    assert(T.shutdown(*group, 5000));
    assert(!group.initted && group.worker_info.count == 0);
    print("ok\n");
}
