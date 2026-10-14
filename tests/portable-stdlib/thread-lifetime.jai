#import "Basic";
T :: #import "Thread";

called: s64;

handler :: (thread: *T.Thread) {
    called += 1;
}

run :: (thread: *T.Thread) -> s64 {
    assert(context.thread_index == thread.index);
    assert(context.temporary_storage == *thread.starting_temporary_storage);
    pointer := cast(*s64) thread.data;
    pointer.* = 71;

    return 9;
}

main :: () {
    value: s64;
    thread: T.Thread;
    thread.data = *value;
    assert(T.thread_init(*thread, run, name = "portable-worker"));
    assert(!T.thread_is_done(*thread, 0) && value == 0);
    T.thread_init_handler = handler;
    T.thread_start(*thread);
    assert(T.thread_is_done(*thread, 5000));
    assert(value == 71 && called == 1);
    T.thread_deinit(*thread);
    assert(thread.index == 0);
    T.thread_init_handler = null;
    print("ok\n");
}
