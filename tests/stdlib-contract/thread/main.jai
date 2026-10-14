// Mutex initialization and semaphore permit/timeout transitions.
#import "Basic";
Thread :: #import "Thread";

quick_worker :: (thread: *Thread.Thread) -> s64 {
    return 0;
}

main :: () {
    mutex: Thread.Mutex;
    Thread.init(*mutex);
    Thread.lock(*mutex);
    Thread.unlock(*mutex);
    Thread.destroy(*mutex);

    semaphore: Thread.Semaphore;
    Thread.init(*semaphore, initial_value=1);
    result := Thread.wait_for(*semaphore, milliseconds=0);
    assert(result == .SUCCESS);
    result = Thread.wait_for(*semaphore, milliseconds=0);
    assert(result == .TIMEOUT);
    Thread.signal(*semaphore);
    result = Thread.wait_for(*semaphore, milliseconds=0);
    assert(result == .SUCCESS);
    Thread.destroy(*semaphore);

    #if OS == .WINDOWS || OS == .LINUX || OS == .MACOS || OS == .IOS || OS == .ANDROID {
        worker: Thread.Thread;
        assert(Thread.thread_init(*worker, quick_worker));
        Thread.thread_start(*worker);
        assert(Thread.thread_is_done(*worker, milliseconds=5000));
        Thread.thread_deinit(*worker);
    }

    print("ok\n");
}
