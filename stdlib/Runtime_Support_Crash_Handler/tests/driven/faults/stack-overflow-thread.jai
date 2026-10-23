#import "Basic";
T :: #import "Thread";
C :: #import "Runtime_Support_Crash_Handler";

depth :: (n: s64, pad: *s64) -> s64 {
    buffer: [64] s64;
    buffer[n % 64] = n;
    pad.* += buffer[(n + 1) % 64];
    return depth(n + 1, pad) + buffer[n % 64];
}

worker :: (thread: *T.Thread) -> s64 {
    pad: s64;
    return depth(0, *pad);
}

main :: () {
    C.init();
    thread: T.Thread;

    assert(T.thread_init(*thread, worker));
    T.thread_start(*thread);
    T.thread_is_done(*thread, -1);
}
