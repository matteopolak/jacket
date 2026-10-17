#import "Basic";
C :: #import "Runtime_Support_Crash_Handler";
P :: #import "POSIX";
T :: #import "Thread";

State :: struct {
    ready: *T.Semaphore;
    release: *T.Semaphore;
    stack: *void;
}

worker :: (thread: *T.Thread) -> s64 {
    state := cast(*State) thread.data;

    C.init();
    C.init();
    current: P.stack_t;

    assert(P.sigaltstack(null, *current) == 0 && current.ss_sp != null && current.ss_size >= 65536);
    state.stack = current.ss_sp;
    T.signal(state.ready);
    assert(T.wait_for(state.release, 5000) == .SUCCESS);

    return 0;
}

main :: () {
    #assert !#exists(C.c_malloc);
    C.init();
    C.init();
    current: P.stack_t;

    assert(P.sigaltstack(null, *current) == 0 && current.ss_sp != null && current.ss_size >= 65536);
    ready: T.Semaphore;
    release: T.Semaphore;

    T.init(*ready);
    T.init(*release);
    defer T.destroy(*ready);
    defer T.destroy(*release);
    states: [4] State;
    threads: [4] T.Thread;

    for i: 0..3 {
        states[i].ready = *ready;
        states[i].release = *release;
        threads[i].data = *states[i];
        assert(T.thread_init(*threads[i], worker));
        T.thread_start(*threads[i]);
    }

    for 0..3 assert(T.wait_for(*ready, 5000) == .SUCCESS);

    for i: 0..3 {
        assert(states[i].stack != current.ss_sp);
        for j: 0..i - 1 assert(states[i].stack != states[j].stack);
    }

    for 0..3 T.signal(*release);

    for *thread: threads {
        assert(T.thread_is_done(thread, 5000));
        T.thread_deinit(thread);
    }

    print("ok\n");
}
