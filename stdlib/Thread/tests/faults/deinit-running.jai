// test: should_fail("thread_is_done(*thread, -1)")
// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)")
#import "Basic";
T :: #import "Thread";

// Tearing down a thread that is still running is a mistake, and the message says how to wait for it.
spin :: (thread: *T.Thread) -> s64 {
    while true sleep_milliseconds(10);

    return 0;
}

main :: () {
    thread: T.Thread;
    assert(T.thread_init(*thread, spin));
    T.thread_start(*thread);
    T.thread_deinit(*thread);
}
