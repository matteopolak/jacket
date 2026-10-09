// Deadlocks `jaic run` reports instead of hanging (a native build hangs). With no argument, two
// Jai threads each wait for the other; with `callback`, a thread of C's own waits in a #c_call
// procedure for a mutex the main thread holds while the main thread waits for that procedure.
#import "Basic";
#import "Thread";

blocking :: #library "libblocking";

Callback :: #type (x: s32) -> s32 #c_call;
start_caller :: (cb: Callback, x: s32) #foreign blocking;

mutex: Mutex;
done: Semaphore;

lock_mutex :: (x: s32) -> s32 #c_call {
    push_context {
        lock(*mutex);
        signal(*done);
        unlock(*mutex);
    }
    return x;
}

wait_for_main :: (thread: *Thread) -> s64 {
    lock(*mutex);
    unlock(*mutex);
    return 0;
}

main :: () {
    // The test times the report from here, so how long the compile took does not count.
    write_string("started\n", true);
    init(*mutex);
    init(*done);
    lock(*mutex);
    args := get_command_line_arguments();
    if args.count > 1 && args[1] == "callback" {
        start_caller(lock_mutex, 1);
        wait_for(*done);
    } else {
        thread: Thread;
        thread_init(*thread, wait_for_main);
        thread_start(*thread);
        thread_deinit(*thread);
    }
    print("not reached\n");
}
