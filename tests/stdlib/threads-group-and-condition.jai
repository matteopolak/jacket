// Condition variables, a semaphore with a timed wait, and a Thread_Group that squares numbers. Native
// builds run these on OS threads; the browser build runs them one after another on the interpreter stack.
#import "Basic";
#import "Atomics";
#import "Thread";

Box :: struct {
    mutex: Mutex;
    ready: Condition_Variable;
    value: s64;
    produced: bool;
}

producer :: (thread: *Thread) -> s64 {
    box := cast(*Box) thread.data;
    lock(*box.mutex);
    box.value = 41;
    box.produced = true;
    unlock(*box.mutex);
    wake(*box.ready);
    return 7;
}

Job :: struct {
    input: s64;
    output: s64;
}

square :: (group: *Thread_Group, thread: *Thread, work: *void) -> Thread_Continue_Status {
    job := cast(*Job) work;
    job.output = job.input * job.input;
    return .CONTINUE;
}

main :: () {
    // The main thread waits on a condition that the new thread signals.
    box: Box;
    init(*box.mutex);
    init(*box.ready);
    producer_thread: Thread;
    producer_thread.data = *box;
    thread_init(*producer_thread, producer);
    thread_start(*producer_thread);
    lock(*box.mutex);
    while !box.produced wait(*box.ready, *box.mutex);
    assert(box.value == 41, "value %", box.value);
    unlock(*box.mutex);
    thread_deinit(*producer_thread);

    // A semaphore handoff and a timed wait that nobody signals.
    sem: Semaphore;
    init(*sem);
    assert(wait_for(*sem, 5) == .TIMEOUT);
    signal(*sem);
    assert(wait_for(*sem, 5) == .SUCCESS);

    // A thread group: add work, poll for the results, shut down.
    group: Thread_Group;
    assert(init(*group, 3, square, false));
    jobs: [10] Job;
    for *jobs it.input = it_index + 1;
    assert(start(*group));
    for *jobs add_work(*group, it);
    done := 0;
    for 0..1000 {
        done += get_completed_work(*group).count;
        if done == jobs.count break;
        sleep_milliseconds(1);
    }
    assert(done == jobs.count, "completed %", done);
    for jobs assert(it.output == it.input * it.input, "job % gave %", it_index, it.output);
    shutdown(*group);
    print("ok\n");
}
