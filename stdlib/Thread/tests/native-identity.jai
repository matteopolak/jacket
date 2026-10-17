#import "Basic";
T :: #import "Thread";
P :: #import "POSIX";

Observation :: struct {
    identity: P.pthread_t;
    index: T.Thread_Index;
    calls: s64;
}

worker :: (thread: *T.Thread) -> s64 {
    observed := cast(*Observation) thread.data;
    observed.identity = P.pthread_self();
    observed.index = context.thread_index;
    observed.calls += 1;

    return 71;
}

main :: () {
    for 0..15 {
        observed: Observation;
        thread: T.Thread;
        thread.data = *observed;
        assert(T.thread_init(*thread, worker, temporary_storage_size = 4096, name = "thread-identity"));
        #if OS == .MACOS assert(thread.thread_handle != null);
        else assert(thread.thread_handle != 0);
        assert(thread.index > 0);
        assert(!T.thread_is_done(*thread, 0) && observed.calls == 0);
        T.thread_start(*thread);
        assert(T.thread_is_done(*thread, 5000));
        assert(observed.calls == 1 && observed.index == thread.index);
        // This is a native-only control: scheduler IDs must not reach libc.
        assert(P.pthread_equal(observed.identity, thread.thread_handle) != 0);
        T.thread_deinit(*thread);
        assert(thread.index == 0);
    }

    print("ok\n");
}
