#import "Basic";
T :: #import "Thread";

read_data :: (thread: *T.Thread) -> s64 {
    p := cast(*int) thread.data;
    return p.*;
}

free_running :: () {
    thread: T.Thread;
    p := New(int);

    thread.data = p;
    assert(T.thread_init(*thread, read_data));
    T.thread_start(*thread);
    free(p);
    assert(T.thread_is_done(*thread, -1));
    T.thread_deinit(*thread);
}

finish_first :: () {
    thread: T.Thread;
    p := New(int);

    thread.data = p;
    assert(T.thread_init(*thread, read_data));
    T.thread_start(*thread);
    assert(T.thread_is_done(*thread, -1));
    free(p);
    T.thread_deinit(*thread);
}

Worker :: struct {
    thread: T.Thread;
}

receiver_is_still_owned_by_caller :: (workers: [] Worker) {
    for *worker: workers {
        T.thread_deinit(*worker.thread);
        index := worker.thread.index;
    }
}
