// Thread lifecycle and the synchronization primitives under real contention: a mutex guarding a
// non-atomic read-modify-write, semaphores with timeouts, a condition variable woken one at a time
// and all at once, thread_is_done with and without a timeout, and the context a thread starts with.
//
// The browser and `jaic run -os wasm` run a started thread on top of the stack of whichever thread
// blocks first, and a thread that blocks after that point is abandoned rather than resumed
// (docs/compiler/interpreter-threads.md). The checks that need a thread to sleep and be woken
// again later only run where threads really are concurrent.
#import "Basic";
#import "Thread";
#import "Atomics";

THREADS :: 4;
INCREMENTS :: 5000;

Shared :: struct {
    mutex: Mutex;
    value: s64;
    racy_window: s64;
}
shared: Shared;

add_under_mutex :: (thread: *Thread) -> s64 {
    for 1..INCREMENTS {
        lock(*shared.mutex);
        // A deliberately non-atomic update with work in between: without the lock, another thread
        // scheduled here would lose increments.
        seen := shared.value;
        for 1..3 shared.racy_window += 1;
        shared.value = seen + 1;
        unlock(*shared.mutex);
    }
    return 0;
}

Handoff :: struct {
    items: Semaphore;
    taken: s64;
}

take_all :: (thread: *Thread) -> s64 {
    handoff := cast(*Handoff) thread.data;
    for 1..10 {
        result := wait_for(*handoff.items);
        assert(result == .SUCCESS, "wait_for without a timeout gave %", result);
        handoff.taken += 1;
    }
    return 0;
}

Gate :: struct {
    mutex: Mutex;
    opened: Condition_Variable;
    open: bool;
    waiting: s64;
    passed: s64;
}

wait_at_gate :: (thread: *Thread) -> s64 {
    gate := cast(*Gate) thread.data;
    lock(*gate.mutex);
    gate.waiting += 1;
    while !gate.open wait(*gate.opened, *gate.mutex);
    gate.passed += 1;
    unlock(*gate.mutex);
    return 0;
}

Observed :: struct {
    thread_index: u32;
    logged: [..] string;
    user_value: s64;
    temporary_is_own: bool;
    allocator_is_inherited: bool;
}

capture_logger :: (message: string, data: *void, info: Log_Info) {
    observed := cast(*Observed) data;
    array_add(*observed.logged, copy_string(message));
}

counting_allocator_calls: s64;
counting_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    atomic_add(*counting_allocator_calls, 1);
    return Context_Base.default_allocator.proc(mode, size, old_size, old_memory, allocator_data);
}

look_at_context :: (thread: *Thread) -> s64 {
    observed := cast(*Observed) thread.data;
    observed.thread_index = context.thread_index;
    observed.user_value = ifx context.logger_data == observed then 1 else 0;
    log("hello from thread %", thread.index);
    // Temporary storage is the thread's own, not the starting thread's.
    memory := talloc(64);
    begin := cast(*u8) thread.starting_temporary_storage.data;
    observed.temporary_is_own = cast(*u8) memory >= begin && cast(*u8) memory < begin + thread.starting_temporary_storage.size;
    before := counting_allocator_calls;
    block := alloc(32);
    free(block);
    observed.allocator_is_inherited = counting_allocator_calls == before + 2;
    return 0;
}

Release :: struct {
    go: Semaphore;
}

wait_for_release :: (thread: *Thread) -> s64 {
    release := cast(*Release) thread.data;
    wait_for(*release.go);
    return 99;
}

nothing :: (thread: *Thread) -> s64 {
    return 0;
}

main :: () {
    assert(context.thread_index == 0, "main runs as thread %", context.thread_index);

    // Many threads, one mutex, an exact final count.
    init(*shared.mutex, "counter");
    threads: [THREADS] Thread;
    for *threads assert(thread_init(it, add_under_mutex), "thread_init %", it_index);
    indices: [THREADS] s64;
    for threads indices[it_index] = it.index;
    for i: 0..THREADS - 1 for j : i + 1..THREADS - 1 assert(indices[i] != indices[j], "thread indices repeat: %", indices);
    for *threads assert(thread_start(it), "thread_start %", it_index);
    for *threads thread_deinit(it);
    assert(shared.value == THREADS * INCREMENTS, "mutex counter %", shared.value);
    assert(shared.racy_window == 3 * THREADS * INCREMENTS, "racy window %", shared.racy_window);
    destroy(*shared.mutex);

    // thread_init and thread_start refuse bad requests.
    {
        thread: Thread;
        assert(!thread_init(null, nothing), "thread_init with a null thread");
        null_proc: Native_Thread_Proc;
        assert(!thread_init(*thread, null_proc), "thread_init with a null proc");
        assert(!thread_init(*thread, nothing, -1), "thread_init with negative temporary storage");
        assert(!thread_start(*thread), "thread_start before thread_init");
        assert(thread_is_done(*thread), "an uninitialized thread counts as done");
        assert(thread_init(*thread, nothing));
        assert(!thread_init(*thread, nothing), "a second thread_init");
        assert(thread_is_done(*thread), "an initialized thread that never started counts as done");
        assert(thread_start(*thread));
        assert(!thread_start(*thread), "a second thread_start");
        thread_destroy(*thread);
        assert(!thread.initialized && thread_is_done(*thread));
        // A destroyed thread can be initialized and run again.
        assert(thread_init(*thread, nothing));
        assert(thread_start(*thread));
        thread_deinit(*thread);
        // Deinit of a thread that never started.
        assert(thread_init(*thread, nothing));
        thread_deinit(*thread);
        thread_deinit(*thread);
    }

    // Semaphores: counts, timeouts, bad arguments.
    {
        sem: Semaphore;
        assert(wait_for(*sem, 0) == .ERROR, "waiting on an uninitialized semaphore");
        assert(wait_for(null) == .ERROR);
        assert(!init(*sem, -1), "negative initial count");
        assert(init(*sem, 2));
        assert(!init(*sem), "a second init");
        assert(wait_for(*sem, 0) == .SUCCESS);
        assert(wait_for(*sem, 0) == .SUCCESS);
        assert(wait_for(*sem, 0) == .TIMEOUT, "count should be used up");
        start := current_time_monotonic();
        assert(wait_for(*sem, 30) == .TIMEOUT);
        waited := to_milliseconds(current_time_monotonic() - start);
        #if OS != .WASM assert(waited >= 25, "a 30 ms timed wait returned after % ms", waited);
        for 1..3 signal(*sem);
        assert(wait_for(*sem, 1000) == .SUCCESS);
        assert(wait_for(*sem) == .SUCCESS);
        assert(wait_for(*sem, 0) == .SUCCESS);
        assert(wait_for(*sem, 0) == .TIMEOUT);
        destroy(*sem);
        destroy(*sem);
        assert(wait_for(*sem, 0) == .ERROR, "waiting on a destroyed semaphore");
        assert(init(*sem), "a destroyed semaphore can be initialized again");
        destroy(*sem);

        // Ten signals handed to a thread that takes them one by one.
        handoff: Handoff;
        init(*handoff.items);
        taker: Thread;
        taker.data = *handoff;
        thread_init(*taker, take_all);
        thread_start(*taker);
        for 1..10 signal(*handoff.items);
        thread_deinit(*taker);
        assert(handoff.taken == 10, "taken %", handoff.taken);
        assert(wait_for(*handoff.items, 0) == .TIMEOUT, "every signal was consumed exactly once");
        destroy(*handoff.items);
    }

    // A condition variable: waiters block until the gate opens; wake releases one, wake_all the rest.
    #if OS != .WASM {
        gate: Gate;
        init(*gate.mutex);
        init(*gate.opened, "gate");
        waiters: [THREADS] Thread;
        for *waiters {
            it.data = *gate;
            thread_init(it, wait_at_gate);
            thread_start(it);
        }
        // Wait until every waiter is asleep on the condition variable.
        while true {
            lock(*gate.mutex);
            waiting := gate.waiting;
            unlock(*gate.mutex);
            if waiting == THREADS break;
            sleep_milliseconds(1);
        }
        lock(*gate.mutex);
        assert(gate.passed == 0, "nobody passes a closed gate");
        unlock(*gate.mutex);
        // wake with the gate still closed: the woken waiter goes straight back to sleep.
        wake(*gate.opened);
        sleep_milliseconds(5);
        lock(*gate.mutex);
        assert(gate.passed == 0, "a spurious wake let % through", gate.passed);
        gate.open = true;
        wake_all(*gate.opened);
        unlock(*gate.mutex);
        for *waiters thread_deinit(it);
        assert(gate.passed == THREADS, "passed %", gate.passed);
        destroy(*gate.opened);
        destroy(*gate.mutex);
    }

    // thread_is_done before, while (with a timeout) and after a thread runs.
    {
        release: Release;
        init(*release.go);
        thread: Thread;
        thread.data = *release;
        assert(thread_init(*thread, wait_for_release));
        assert(thread_start(*thread));
        assert(!thread_is_done(*thread), "a thread blocked on a semaphore is not done");
        #if OS != .WASM {
            start := current_time_monotonic();
            assert(!thread_is_done(*thread, 20), "a timed thread_is_done on a blocked thread");
            waited := to_milliseconds(current_time_monotonic() - start);
            assert(waited >= 15, "thread_is_done(20) returned after % ms", waited);
        }
        signal(*release.go);
        assert(thread_is_done(*thread, 10000), "thread_is_done after release");
        // The liveness signal is handed back, so asking again (and deinit) still sees it.
        assert(thread_is_done(*thread, 10000));
        assert(thread_is_done(*thread));
        thread_deinit(*thread);
        destroy(*release.go);
    }

    // A thread starts with the context of thread_init, but its own thread index and temporary storage.
    {
        observed: Observed;
        thread: Thread;
        thread.data = *observed;
        scoped := context;
        push_context, defer_pop scoped;
        context.logger = capture_logger;
        context.logger_data = *observed;
        context.allocator = .{ counting_allocator, null };
        assert(thread_init(*thread, look_at_context));
        assert(thread_start(*thread));
        thread_deinit(*thread);
        assert(observed.thread_index == cast(u32) thread.index && observed.thread_index != 0, "thread index % (thread %)", observed.thread_index, thread.index);
        assert(observed.user_value == 1, "logger_data reached the thread");
        assert(observed.logged.count == 1 && begins_with(observed.logged[0], "hello from thread "), "logged %", observed.logged);
        assert(observed.temporary_is_own, "talloc in a thread uses the thread's temporary storage");
        assert(observed.allocator_is_inherited, "the thread allocates through the inherited allocator");
        assert(context.thread_index == 0);
    }

    // A thread can use storage the caller hands it for temporary storage.
    {
        storage: Temporary_Storage;
        assert(init_temporary_storage(*storage, 4096));
        observed: Observed;
        thread: Thread;
        thread.data = *observed;
        scoped := context;
        push_context, defer_pop scoped;
        context.logger = capture_logger;
        context.logger_data = *observed;
        assert(thread_init(*thread, look_at_context, 0, *storage));
        assert(!thread.owns_temporary_storage);
        thread_start(*thread);
        thread_deinit(*thread);
        assert(observed.temporary_is_own, "talloc used the caller's storage");
        deinit_temporary_storage(*storage, free_original = true);
    }
    print("ok\n");
}

begins_with :: (s: string, prefix: string) -> bool {
    if s.count < prefix.count return false;
    return string.{ prefix.count, s.data } == prefix;
}
