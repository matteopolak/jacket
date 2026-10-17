#import "Basic";
T :: #import "Thread";

Shared :: struct {
    mutex: T.Mutex;
    changed: T.Condition_Variable;
    ready: T.Semaphore;
    count: s64;
}

worker :: (thread: *T.Thread) -> s64 {
    state := cast(*Shared) thread.data;

    assert(context.thread_index == thread.index && thread.index > 0);
    T.lock(*state.mutex);
    state.count += 1;
    T.wake(*state.changed);
    T.unlock(*state.mutex);
    T.signal(*state.ready);

    return 0;
}

main :: () {
    #if OS == .WINDOWS {
        // A real SDK wait failure must not be reported as thread completion.
        invalid: T.Thread;
        errors: s64;
        saved_logger := context.logger;
        saved_data := context.logger_data;

        context.logger = count_wait_failure;
        context.logger_data = *errors;
        done := T.thread_is_done(*invalid, 0);

        context.logger = saved_logger;
        context.logger_data = saved_data;
        assert(!done && errors == 1);
    }

    shared: Shared;

    T.init(*shared.mutex);
    T.init(*shared.changed);
    T.init(*shared.ready);
    assert(T.wait_for(*shared.ready, 0) == .TIMEOUT);

    for 0..1 {
        thread: T.Thread;
        thread.data = *shared;
        assert(T.thread_init(*thread, worker, name = "I/O worker 雪"));
        assert(!T.thread_is_done(*thread, 0));
        T.thread_start(*thread);
        T.lock(*shared.mutex);
        while shared.count <= it T.wait(*shared.changed, *shared.mutex);

        T.unlock(*shared.mutex);
        assert(T.wait_for(*shared.ready, 5000) == .SUCCESS);
        assert(T.thread_is_done(*thread, 5000));
        T.thread_deinit(*thread);
        assert(thread.index == 0);
    }

    T.destroy(*shared.ready);
    T.destroy(*shared.changed);
    T.destroy(*shared.mutex);
    assert(shared.count == 2);
    print("ok\n");
}

#scope_file
#if OS == .WINDOWS count_wait_failure :: (message: string, data: *void, info: Log_Info) {
    assert(message == "thread completion wait failed\n" && info.common_flags == .ERROR);
    count := cast(*s64) data;
    count.* += 1;
}
