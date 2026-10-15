#import "Basic";
T :: #import "Thread";

Shared :: struct {
    ready: T.Semaphore;
    go: T.Semaphore;
    first: [8] float64;
}

Worker :: struct {
    shared: *Shared;
    index: s64;
}

worker :: (thread: *T.Thread) -> s64 {
    state := cast(*Worker) thread.data;

    T.signal(*state.shared.ready);
    assert(T.wait_for(*state.shared.go, 5000) == .SUCCESS);
    first := seconds_since_init();

    state.shared.first[state.index] = first;
    previous := first;

    for i: 0..511 {
        current := seconds_since_init();
        assert(current >= previous);
        previous = current;
    }

    return 0;
}

main :: () {
    shared: Shared;

    T.init(*shared.ready);
    T.init(*shared.go);
    states: [8] Worker;
    threads: [8] T.Thread;

    for i: 0..7 {
        states[i] = .{ *shared, i };
        threads[i].data = *states[i];
        assert(T.thread_init(*threads[i], worker));
        T.thread_start(*threads[i]);
    }

    for i: 0..7 assert(T.wait_for(*shared.ready, 5000) == .SUCCESS);
    for i: 0..7 T.signal(*shared.go);

    lowest: float64 = 1e300;

    for i: 0..7 {
        assert(T.thread_is_done(*threads[i], 10000));
        T.thread_deinit(*threads[i]);
        assert(shared.first[i] >= 0);
        lowest = min(lowest, shared.first[i]);
    }

    assert(lowest == 0);
    assert(seconds_since_init() >= 0);
    T.destroy(*shared.go);
    T.destroy(*shared.ready);
    print("ok\n");
}
