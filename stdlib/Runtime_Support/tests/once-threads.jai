// One actual winner publishes initialized data to four real native workers.
#import "Basic";
R :: #import "Runtime_Support";
T :: #import "Thread";
A :: #import "Atomics";

Shared :: struct {
    sync: s32;
    calls: s64;
    published: s64;
    failed: s32;
}

worker :: (thread: *T.Thread) -> s64 {
    shared := cast(*Shared) thread.data;

    R.one_time_init(*shared.sync, #code {
        shared.calls += 1;
        shared.published = 37;
    });

    if shared.published != 37 {
        A.atomic_write(*shared.failed, 1);
        return 1;
    }

    return 0;
}

main :: () {
    shared: Shared;
    threads: [4] T.Thread;

    for *thread: threads {
        thread.data = *shared;
        initialized := T.thread_init(thread, worker);
        if !initialized exit(2);
        T.thread_start(thread);
    }

    for *thread: threads {
        done := T.thread_is_done(thread, 10000);
        if !done exit(3);
        T.thread_deinit(thread);
    }

    if shared.calls != 1 || shared.published != 37 || shared.failed != 0 exit(5);

    write_string("ok\n");
}
