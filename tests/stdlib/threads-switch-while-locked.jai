// A started thread's work is never lost to a lock its starter holds. The browser sandbox and
// `jaic run -os wasm` run every thread on one host thread; a thread that gets its turn while main
// holds a mutex or spin lock must wait for main to run again and release it, not be given up on
// ("ran 0") or spin forever. A busy wait on an atomic still lets the other thread run.
#import "Basic";
#import "Thread";
#import "Atomics";

mutex: Mutex;
spin: s32;
ran_with_mutex: s64;
ran_with_spin: s64;
flag: s32;

lock_spin :: () {
    while true {
        if compare_and_swap(*spin, 0, 1) return;
    }
}

unlock_spin :: () {
    compare_and_swap(*spin, 1, 0);
}

take_mutex :: (thread: *Thread) -> s64 {
    lock(*mutex);
    ran_with_mutex += 1;
    unlock(*mutex);
    return 0;
}

take_spin :: (thread: *Thread) -> s64 {
    lock_spin();
    ran_with_spin += 1;
    unlock_spin();
    return 0;
}

raise_flag :: (thread: *Thread) -> s64 {
    atomic_write(*flag, 1);
    return 0;
}

busy :: () -> s64 {
    sum := 0;
    for 1..200000 sum += it % 7;
    return sum;
}

main :: () {
    init(*mutex);
    {
        thread: Thread;
        assert(thread_init(*thread, take_mutex));
        assert(thread_start(*thread));
        lock(*mutex);
        // Long enough to be preempted while holding the lock.
        sum := busy();
        // Polling while holding the lock does not start the thread either.
        for 1..5000 atomic_read(*flag);
        unlock(*mutex);
        thread_deinit(*thread);
        assert(sum > 0);
        assert(ran_with_mutex == 1, "ran %", ran_with_mutex);
    }
    {
        thread: Thread;
        assert(thread_init(*thread, take_spin));
        assert(thread_start(*thread));
        lock_spin();
        sum := busy();
        unlock_spin();
        thread_deinit(*thread);
        assert(sum > 0);
        assert(ran_with_spin == 1, "ran %", ran_with_spin);
    }
    {
        thread: Thread;
        assert(thread_init(*thread, raise_flag));
        assert(thread_start(*thread));
        while atomic_read(*flag) == 0 {}
        thread_deinit(*thread);
    }
    destroy(*mutex);
    print("ok\n");
}
