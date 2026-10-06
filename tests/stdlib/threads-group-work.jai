// Thread_Group with many work items: every item runs exactly once on one of the group's threads,
// with and without work stealing; a worker that returns .STOP leaves its later work unprocessed;
// shutdown with a timeout gives up on a busy worker; a group can be set up again after shutdown.
//
// In the browser and `jaic run -os wasm` a worker parked with no work is abandoned rather than
// resumed (docs/compiler/interpreter-threads.md), so all work is added before the first wait for
// results, and the timed shutdown of a blocked worker only runs where threads are concurrent.
#import "Basic";
#import "Thread";
#import "Atomics";

Job :: struct {
    input: s64;
    output: s64;
    runs: s64;
    thread_index: Thread_Index;
    context_thread_index: u32;
}

group_threads: [] Thread;
total_runs: s64;

square :: (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status {
    job := cast(*Job) work;
    job.output = job.input * job.input;
    atomic_add(*job.runs, 1);
    atomic_add(*total_runs, 1);
    job.thread_index = thread.index;
    job.context_thread_index = context.thread_index;
    return .CONTINUE;
}

// Stops its worker when it meets a negative input.
square_until_negative :: (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status {
    job := cast(*Job) work;
    job.runs += 1;
    if job.input < 0 return .STOP;
    job.output = job.input * job.input;
    return .CONTINUE;
}

Blocker :: struct {
    release: Semaphore;
    entered: s64;
}

wait_for_release :: (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status {
    blocker := cast(*Blocker) group.data;
    atomic_add(*blocker.entered, 1);
    wait_for(*blocker.release);
    return .CONTINUE;
}

fixed_clock :: () -> float64 {
    return 1234.5;
}

collect :: (group: *Thread_Group, expected: s64) -> [..] *void {
    results: [..] *void;
    for 1..20000 {
        for get_completed_work(group) array_add(*results, it);
        if results.count >= expected break;
        sleep_milliseconds(1);
    }
    return results;
}

run_jobs :: (thread_count: s32, job_count: s64, stealing: bool) {
    total_runs = 0;
    group: Thread_Group;
    group.name = "squares";
    assert(init(*group, thread_count, square, stealing), "init % threads", thread_count);
    assert(group.threads.count == thread_count);
    jobs := NewArray(job_count, Job);
    defer free(jobs.data);
    for *jobs it.input = it_index - job_count / 2;
    // Work added before start waits in the queues.
    half := job_count / 2;
    for i: 0..half - 1 assert(add_work(*group, *jobs[i], "early"));
    assert(start(*group));
    assert(!start(*group), "a second start");
    for i: half..job_count - 1 assert(add_work(*group, *jobs[i]));

    results := collect(*group, job_count);
    defer array_free(results);
    assert(results.count == job_count, "% threads, stealing %: % of % results", thread_count, stealing, results.count, job_count);
    assert(get_completed_work(*group).count == 0, "results are handed out once");

    // Every job came back exactly once and was run once, on one of the group's threads.
    seen := NewArray(job_count, bool);
    defer free(seen.data);
    for results {
        job := cast(*Job) it;
        index := job - jobs.data;
        assert(index >= 0 && index < job_count, "result % is not one of the jobs", it);
        assert(!seen[index], "job % came back twice", index);
        seen[index] = true;
    }
    for jobs {
        assert(it.runs == 1, "job % ran % times", it_index, it.runs);
        assert(it.output == it.input * it.input, "job % gave %", it_index, it.output);
        assert(it.context_thread_index == cast(u32) it.thread_index, "context.thread_index % on thread %", it.context_thread_index, it.thread_index);
        found := false;
        for thread: group.threads if thread.index == it.thread_index found = true;
        assert(found, "job % ran on thread %, which is not in the group", it_index, it.thread_index);
    }
    assert(total_runs == job_count);
    assert(shutdown(*group), "shutdown after all work finished");
    assert(group.threads.count == 0 && !group.initted);
    assert(!add_work(*group, *jobs[0]), "add_work after shutdown");
    assert(shutdown(*group), "a second shutdown is harmless");
}

main :: () {
    // Bad arguments.
    {
        group: Thread_Group;
        assert(!init(*group, 0, square), "zero threads");
        assert(!init(*group, -2, square), "negative threads");
        null_proc: Native_Thread_Group_Proc;
        assert(!init(*group, 2, null_proc), "no proc");
        assert(!start(*group), "start before init");
        job: Job;
        assert(!add_work(*group, *job), "add_work before init");
        assert(get_completed_work(*group).count == 0);
        assert(get_completed_work(null).count == 0);
        assert(shutdown(*group), "shutdown before init");
        assert(init(*group, 2, square));
        assert(!init(*group, 2, square), "a second init");
        // Shutting down a group that never started.
        assert(shutdown(*group));
    }

    run_jobs(1, 40, false);
    run_jobs(3, 200, false);
    run_jobs(4, 200, true);
    // The untyped-proc overload (as `jai` declares it) and a single worker with stealing on.
    {
        group: Thread_Group;
        assert(init(*group, 1, cast(Thread_Group_Proc) square, true));
        group.time_proc = fixed_clock;
        jobs: [5] Job;
        for *jobs {
            it.input = it_index;
            add_work(*group, it);
        }
        start(*group);
        results := collect(*group, jobs.count);
        assert(results.count == jobs.count, "% results", results.count);
        for jobs assert(it.output == it_index * it_index);
        // One worker processes its queue in the order the work was added.
        for results assert(it == *jobs[it_index], "result % out of order", it_index);
        array_free(results);
        assert(shutdown(*group));
    }

    // A worker that returns .STOP processes nothing after that item; shutdown reports it.
    {
        group: Thread_Group;
        assert(init(*group, 1, square_until_negative));
        jobs: [6] Job;
        for *jobs it.input = ifx it_index == 2 then -1 else it_index + 1;
        for *jobs add_work(*group, it);
        start(*group);
        results := collect(*group, 3);
        assert(results.count == 3, "% results before the stop", results.count);
        array_free(results);
        assert(thread_is_done(*group.threads[0], 10000), "the stopped worker exits");
        for jobs assert(it.runs == xx (it_index <= 2), "job % ran % times", it_index, it.runs);
        assert(!shutdown(*group), "shutdown reports the unprocessed work");
        assert(!group.initted);
    }

    // shutdown with a timeout gives up while a worker is still busy; a later shutdown finishes.
    #if OS != .WASM {
        blocker: Blocker;
        init(*blocker.release);
        group: Thread_Group;
        group.data = *blocker;
        assert(init(*group, 2, wait_for_release));
        start(*group);
        dummy: Job;
        add_work(*group, *dummy);
        while atomic_read(*blocker.entered) == 0 sleep_milliseconds(1);
        start_time := current_time_monotonic();
        assert(!shutdown(*group, 20), "shutdown with a busy worker and a 20 ms timeout");
        waited := to_milliseconds(current_time_monotonic() - start_time);
        assert(waited >= 15 && waited < 5000, "timed shutdown took % ms", waited);
        assert(group.initted, "a timed-out shutdown leaves the group set up");
        assert(!add_work(*group, *dummy), "no new work once shutdown was requested");
        signal(*blocker.release);
        assert(shutdown(*group), "shutdown once the worker is released");
        destroy(*blocker.release);

        // The same Thread_Group value can be set up and used again.
        jobs: [8] Job;
        for *jobs it.input = it_index;
        assert(init(*group, 2, square));
        for *jobs add_work(*group, it);
        start(*group);
        results := collect(*group, jobs.count);
        assert(results.count == jobs.count);
        array_free(results);
        assert(shutdown(*group));
    }
    print("ok\n");
}
