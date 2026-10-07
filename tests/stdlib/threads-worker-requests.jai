// A worker that waits on a semaphore for requests serves every one of them, however long it
// waited. The browser sandbox and `jaic run -os wasm` run all threads on one host thread; they
// used to give up on a worker waiting for its next request once the main thread could go on,
// so the second request was never served and the program hung (File_Async did this).
#import "Basic";
#import "Thread";

ROUNDS :: 6;

Channel :: struct {
    request: Semaphore;
    reply: Semaphore;
    mutex: Mutex;
    value: int;
    served: int;
    quit: bool;
}

channel: Channel;

serve :: (thread: *Thread) -> s64 {
    while true {
        wait_for(*channel.request);
        lock(*channel.mutex);
        quit := channel.quit;
        if !quit {
            channel.value *= 2;
            channel.served += 1;
        }
        unlock(*channel.mutex);
        signal(*channel.reply);
        if quit break;
    }
    return 0;
}

main :: () {
    assert(init(*channel.request));
    assert(init(*channel.reply));
    init(*channel.mutex);

    worker: Thread;
    assert(thread_init(*worker, serve));
    assert(thread_start(*worker));

    for round: 1..ROUNDS {
        // Main has work of its own between requests, so the worker sits waiting meanwhile.
        sum := 0;
        for 1..2000 sum += it % 3;
        assert(sum > 0);

        lock(*channel.mutex);
        channel.value = round;
        unlock(*channel.mutex);
        signal(*channel.request);
        assert(wait_for(*channel.reply) == .SUCCESS);

        lock(*channel.mutex);
        assert(channel.value == round * 2, "round %: got %", round, channel.value);
        assert(channel.served == round, "round %: served %", round, channel.served);
        unlock(*channel.mutex);
    }

    lock(*channel.mutex);
    channel.quit = true;
    unlock(*channel.mutex);
    signal(*channel.request);
    assert(wait_for(*channel.reply) == .SUCCESS);
    thread_deinit(*worker);

    destroy(*channel.request);
    destroy(*channel.reply);
    destroy(*channel.mutex);
    print("ok\n");
}
