// Renders jobs on a Thread_Group. Each worker reuses one Out and resets temporary storage
// after each job. Shared results belong in data behind a mutex (Writer already does this).
// Zero threads chooses CPU count minus one, with a minimum of one.
// ```jai
// render :: (out: *Out, job: *int, count: *int) {
//     html(out, "<p>{job.*}</p>");
// }
// jobs := int.[1, 2, 3];
// count := 0;
// render_parallel(jobs, render, *count, threads = 2);
// ```
render_parallel :: (jobs: [] $T, render: (out: *Out, job: *T, data: *$D), data: *D, threads := 0) {
    if jobs.count == 0 return;

    workers := threads;
    if workers <= 0 workers = max(1, cast(int) System.get_number_of_processors(.ALL_LOGICAL) - 1);

    workers = min(workers, jobs.count);

    State :: struct {
        outputs: [] Out;
        render: (out: *Out, job: *T, data: *D);
        data: *D;
    }

    work :: (group: *Threads.Thread_Group, thread: *Threads.Thread, job: *void) -> Threads.Thread_Continue_Status {
        state := cast(*State) group.data;
        index := thread.worker_info.worker_index;
        out := *state.outputs[index];
        mark := get_temporary_storage_mark();

        state.render(out, cast(*T) job, state.data);
        reset(out);
        set_temporary_storage_mark(mark);

        return .CONTINUE;
    }

    outputs := NewArray(workers, Out);

    defer {
        for *outputs deinit(it);
        free(outputs.data);
    }

    state := State.{ outputs, render, data };
    group: Threads.Thread_Group;
    group.logging = false;
    Threads.init(*group, cast(s32) workers, work);
    group.data = *state;
    Threads.start(*group);
    for *jobs Threads.add_work(*group, it);

    assert(Threads.wait_for_all(*group));
    assert(Threads.shutdown(*group));
}

#scope_module
System :: #import "System";
