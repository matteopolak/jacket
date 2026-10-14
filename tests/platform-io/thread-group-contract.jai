#import "Basic";
T :: #import "Thread";

Work :: struct {
    index: s64;
    result: s64;
    thread_index: T.Thread_Index;
}

Shared :: struct {
    ready: T.Semaphore;
}

worker :: (group: *T.Thread_Group, thread: *T.Thread, work: *void) -> T.Thread_Continue_Status {
    item := cast(*Work) work;
    item.result = item.index * 7 + 3;
    item.thread_index = thread.index;
    assert(context.thread_index == thread.index && thread.worker_info.group == group);
    shared := cast(*Shared) group.data;

    T.signal(*shared.ready);

    return .CONTINUE;
}

main :: () {
    for stealing: bool.[false, true] {
        shared: Shared;

        T.init(*shared.ready);
        group: T.Thread_Group;
        group.logging = false;
        group.data = *shared;
        T.init(*group, 3, worker, enable_work_stealing = stealing, group_name = "work 雪");
        assert(group.worker_info.count == 3);
        assert(cast(u64) group.worker_info.data % 64 == 0);

        for *info: group.worker_info {
            assert(info.work_steal_indices.count == ifx stealing then 2 else 0);
            assert(!T.thread_is_done(*info.thread, 0));
        }

        work: [96] Work;
        seen: [96] bool;

        for *item, i: work {
            item.index = i;
            T.add_work(*group, item);
        }

        T.start(*group);
        completed: s64;

        for 0..999 {
            mark := get_temporary_storage_mark();

            for pointer: T.get_completed_work(*group) {
                item := cast(*Work) pointer;

                assert(item >= work.data && item < work.data + work.count);
                assert(item.index >= 0 && item.index < work.count && !seen[item.index]);
                assert(item.result == item.index * 7 + 3 && item.thread_index > 0);
                seen[item.index] = true;
                completed += 1;
            }

            set_temporary_storage_mark(mark);
            if completed == work.count break;

            assert(T.wait_for(*shared.ready, 10) != .ERROR);
        }

        assert(completed == work.count);
        for flag: seen assert(flag);

        assert(T.shutdown(*group, 5000));
        assert(!group.initted && group.worker_info.count == 0 && group.worker_info_data_to_free == null);
        assert(T.shutdown(*group, 0));
        T.destroy(*shared.ready);
    }

    print("ok\n");
}
