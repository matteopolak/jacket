init :: (using group: *Thread_Group, num_threads: s32, group_proc: Thread_Group_Proc, enable_work_stealing := false, temporary_storage_size: s32 = 16384, group_name := "") {
    assert(!initted && num_threads > 0 && group_proc != null);
    if !allocator.proc allocator = context.allocator;

    push_allocator(allocator);
    proc = group_proc;
    name = group_name;
    if !time_proc time_proc = clock_seconds;

    records, base := NewArray(num_threads, Thread_Group.Worker_Info, alignment = cast(s32) max(cast(s64) CACHE_LINE_SIZE, align_of(Thread_Group.Worker_Info)));

    worker_info = records;
    worker_info_data_to_free = base;
    next_worker_index = 0;
    should_exit = false;

    for *worker, i: worker_info {
        worker.group = group;
        worker.worker_index = cast(s32) i;
        init_work_list(*worker.available);
        init_work_list(*worker.completed);

        if enable_work_stealing && num_threads > 1 {
            indices, allocation := NewArray(num_threads - 1, s32);

            // Natural s32 alignment leaves this allocation's base unchanged.
            assert(indices.data == allocation);
            worker.work_steal_indices = indices;
            index: s64;

            for j: 0..num_threads - 1 if j != i {
                indices[index] = cast(s32) j;
                index += 1;
            }
        }

        worker.thread.worker_info = worker;
        created := prepare_thread(*worker.thread, group_worker, temporary_storage_size, null, "", *thread_init_handler);

        #if OS == .WINDOWS {
            if !windows_thread_check(created, "worker thread creation failed") exit(1);
        } else if !created {
            sdk_ok(-1, "worker thread creation failed");
            // Keep running workers' storage alive until process teardown.
            thread_abort();
        }

    }

    initted = true;
}

start :: (group: *Thread_Group) {
    assert(group.initted && !group.started);
    group.started = true;
    for *worker: group.worker_info thread_start(*worker.thread);
}

add_work :: (group: *Thread_Group, work: *void, logging_name := "") {
    assert(group.initted && !group.should_exit);
    _, index := compare_and_swap(*group.next_worker_index, cast(s32) 0, cast(s32) 0);

    while true {
        next := cast(s32) ((index + 1) % group.worker_info.count);
        changed, old := compare_and_swap(*group.next_worker_index, index, next);
        if changed break;
        index = old;
    }

    selected := index;
    attempts: s64;

    while read_done(*group.worker_info[selected].thread) {
        selected = cast(s32) ((selected + 1) % group.worker_info.count);
        attempts += 1;
        assert(attempts < group.worker_info.count, "no live worker remains");
    }

    entry := cast(*Work_Entry) group.allocator.proc(.ALLOCATE, size_of(Work_Entry), 0, null, group.allocator.data);

    assert(entry != null);
    entry.* = .{};
    entry.work = work;
    entry.logging_name = logging_name;
    entry.issue_time = group.time_proc();
    entry.work_list_index = selected;
    append_work(*group.worker_info[selected].available, entry);

    if group.logging {
        message := sprint("% queued %\n", group.name, logging_name);
        defer free(message);
        context.logger(message, context.logger_data, Log_Info.{});
    }

    if group.worker_info[0].work_steal_indices.count > 0
        for *worker, i: group.worker_info if i != selected signal(*worker.available.semaphore);
}

get_completed_work :: (group: *Thread_Group) -> [] *void {
    collected: [..] *void;

    defer array_free(collected);

    for *worker: group.worker_info {
        while true {
            entry := take_work(*worker.completed);
            if !entry break;
            array_add(*collected, entry.work);
            group.allocator.proc(.FREE, 0, size_of(Work_Entry), entry, group.allocator.data);
        }
    }

    if collected.count == 0 return .{};

    result: [] *void = .{ collected.count, cast(**void) talloc(collected.count * size_of(*void)) };

    assert(result.data != null);
    memcpy(result.data, collected.data, collected.count * size_of(*void));

    return result;
}

shutdown :: (using group: *Thread_Group, timeout_milliseconds: s32 = -1) -> bool {
    if !initted return true;
    if !started start(group);

    // Setting the stop flag is idempotent; an already true value is success.
    compare_and_swap(*should_exit, false, true);

    for *worker: worker_info signal(*worker.available.semaphore);

    deadline: s64;

    if timeout_milliseconds >= 0 {
        now, ok := monotonic_milliseconds();
        if !ok return false;
        deadline = now + timeout_milliseconds;
    }

    for *worker: worker_info {
        wait_time := timeout_milliseconds;
        if wait_time >= 0 {
            now, ok := monotonic_milliseconds();
            if !ok return false;
            remaining := max(cast(s64) 0, deadline - now);
            wait_time = cast(s32) min(remaining, cast(s64) 0x7fffffff);
        }

        if !thread_is_done(*worker.thread, wait_time) return false;
    }

    push_allocator(allocator);

    for *worker: worker_info {
        if worker.thread.index == 0 continue;
        thread_deinit(*worker.thread);
        // A failed join retains the native thread and all storage it can access.
        if worker.thread.index != 0 return false;
    }

    for *worker: worker_info {
        free_work_list(*worker.available, allocator);
        free_work_list(*worker.completed, allocator);
        free(worker.work_steal_indices.data);
    }

    free(worker_info_data_to_free);
    worker_info = .{};
    worker_info_data_to_free = null;
    initted = false;
    started = false;
    if logging context.logger("thread group stopped\n", context.logger_data, Log_Info.{});

    return true;
}

#scope_file
#if OS != .WINDOWS {
    thread_libc :: #system_library "libc";
    thread_abort :: () #foreign thread_libc "abort";
}

clock_seconds :: () -> float64 {
    #if OS == .WINDOWS {
        counter: W.LARGE_INTEGER;
        frequency: W.LARGE_INTEGER;
        counted := W.QueryPerformanceCounter(*counter);
        timed := W.QueryPerformanceFrequency(*frequency);

        if !windows_thread_check(counted != 0 && timed != 0 && frequency.QuadPart > 0, "performance clock query failed") exit(1);

        return cast(float64) counter.QuadPart / cast(float64) frequency.QuadPart;
    } else {
        value: P.timespec;
        result := P.clock_gettime(P.clockid_t.MONOTONIC, *value);
        if !sdk_ok(result, "thread group clock query failed") thread_abort();
        return cast(float64) value.tv_sec + cast(float64) value.tv_nsec / 1000000000.0;
    }

}

init_work_list :: (list: *Work_List) {
    init(*list.mutex);
    init(*list.semaphore, 0);
}

append_work :: (list: *Work_List, entry: *Work_Entry) {
    lock(*list.mutex);
    entry.next = null;
    if list.last list.last.next = entry;
    else list.first = entry;

    list.last = entry;
    list.count += 1;
    unlock(*list.mutex);
    signal(*list.semaphore);
}

take_work :: (list: *Work_List) -> *Work_Entry {
    lock(*list.mutex);
    defer unlock(*list.mutex);
    entry := list.first;
    if !entry return null;

    list.first = entry.next;
    if !list.first list.last = null;

    list.count -= 1;

    return entry;
}

free_work_list :: (list: *Work_List, allocator: Allocator) {
    entry := list.first;
    while entry {
        next := entry.next;
        allocator.proc(.FREE, 0, size_of(Work_Entry), entry, allocator.data);
        entry = next;
    }

    list.first = null;
    list.last = null;
    list.count = 0;
    destroy(*list.semaphore);
    destroy(*list.mutex);
} @Releases

group_worker :: (thread: *Thread) -> s64 {
    worker := thread.worker_info;
    group := worker.group;

    while true {
        entry := take_work(*worker.available);
        if !entry for index: worker.work_steal_indices {
            entry = take_work(*group.worker_info[index].available);
            if entry break;
        }

        if entry {
            entry.thread_index = thread.index;
            status := group.proc(group, thread, entry.work);

            append_work(*worker.completed, entry);
            if status == .STOP break;

            continue;
        }

        _, exiting := compare_and_swap(*group.should_exit, false, false);
        if exiting break;

        status := wait_for(*worker.available.semaphore, -1);
        if status == .ERROR {
            context.logger("worker semaphore wait failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            break;
        }
    }

    return 0;
}
