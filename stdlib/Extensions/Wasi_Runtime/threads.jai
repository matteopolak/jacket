// Threads on WASI. Preview 1 gives a program one thread and no way to start another, so the
// pthread calls the Thread module makes are answered here with green threads: every thread runs
// on the host's thread, one at a time, until it blocks (a mutex another thread holds, a
// condition variable, a join, a sleep). Then the scheduler picks another. Nothing preempts a
// thread that never blocks.
//
// Wasm code cannot switch stacks, so a thread that blocks unwinds instead: jaic rewrites every
// procedure that may reach `green_suspend` (crates/jaic-llvm/src/green.rs) to keep its locals in
// a frame on the thread's own frame stack, to return when `green.state` says the thread is
// unwinding, and to jump back to where it was when the scheduler resumes the thread by calling
// its entry again. jaic does that only to programs that call `pthread_create`, and then sets
// `green.enabled`; without it, a wait that nothing could end reports EDEADLK at once, as there
// is no other thread. See docs/native/wasm-threads.md.

// The compiler reads these by offset: state at 0, top at 8, limit at 16, enabled at 24.
Green_Control :: struct {
    state: s64;
    top: *u8; // Where the running thread's next frame goes.
    limit: *u8; // The end of its frame stack.
    enabled: s64;
}

GREEN_RUNNING :: 0;
GREEN_UNWINDING :: 1;
GREEN_REWINDING :: 2;

green: Green_Control;

#program_export "jaic_wasi_green_control"
green_control :: () -> *Green_Control #c_call {
    return *green;
}

// Called where the running thread blocks. Unwinding starts here; when the thread is resumed,
// the rewound calls arrive here again and the thread goes on from this point.
#program_export "jaic_wasi_suspend"
green_suspend :: () #c_call {
    if green.state == GREEN_REWINDING {
        green.state = GREEN_RUNNING;
        return;
    }
    green.state = GREEN_UNWINDING;
}

#program_export "jaic_wasi_frames_exhausted"
green_frames_exhausted :: () #c_call {
    message :: "thread stack overflow: a thread's frames outgrew its frame stack\n";
    write_all(2, message.data, message.count);
    trap();
}

Thread_Entry :: #type (argument: *void) -> *void #c_call;

Green_Thread :: struct {
    entry: Thread_Entry;
    argument: *void;
    result: *void;
    frames: *u8;
    frames_end: *u8;
    next: *Green_Thread;
    status: Thread_Status;
    started: bool;
    // While BLOCKED: what it waits for (a mutex, a condition, a thread; null for a sleep), until
    // when (monotonic nanoseconds, 0 for no limit), and when it started waiting.
    waiting_for: *void;
    deadline: u64;
    ticket: u64;
    wake_result: s32;
}

Thread_Status :: enum s32 {
    RUNNABLE;
    BLOCKED;
    DONE;
}

MAIN_FRAME_BYTES :: 8 * 1024 * 1024;
// Just under 1 MiB, so the heap's power-of-two classes fit it with its tag.
THREAD_FRAME_BYTES :: 1024 * 1024 - 64;

first_thread: *Green_Thread;
running: *Green_Thread;
next_ticket: u64;

main_argument_count: s32;
main_arguments: **u8;

main_thread_entry :: (argument: *void) -> *void #c_call {
    wasi_proc_exit(program_main(main_argument_count, main_arguments));
    return null;
}

// Runs `main` as the first green thread, then whichever thread can go on, until the program
// exits. Each turn calls the thread's entry: from the start the first time, rewinding later.
// It returns when the thread is done or when it has unwound because it blocked.
#program_export "jaic_wasi_run_threads"
run_threads :: () #c_call {
    main_thread := add_thread(main_thread_entry, null, heap_take(MAIN_FRAME_BYTES), MAIN_FRAME_BYTES);
    if !main_thread wasi_proc_exit(71);
    while true {
        thread := next_to_run();
        running = thread;
        green.top = thread.frames;
        green.limit = thread.frames_end;
        green.state = ifx thread.started then GREEN_REWINDING else GREEN_RUNNING;
        thread.started = true;
        result := thread.entry(thread.argument);
        if green.state == GREEN_UNWINDING {
            green.state = GREEN_RUNNING;
            continue;
        }
        thread.status = .DONE;
        thread.result = result;
        if thread != main_thread wasi_free(thread.frames);
        thread.frames = null;
        wake_all(thread, 0);
    }
}

add_thread :: (entry: Thread_Entry, argument: *void, frames: *u8, frame_bytes: u64) -> *Green_Thread #c_call {
    if !frames return null;
    thread := cast(*Green_Thread) wasi_calloc(1, size_of(Green_Thread));
    if !thread return null;
    thread.entry = entry;
    thread.argument = argument;
    thread.frames = frames;
    thread.frames_end = frames + frame_bytes;
    // Last in the list, so threads take turns in the order they were started.
    link := *first_thread;
    while link.* link = *link.*.next;
    link.* = thread;
    return thread;
}

remove_thread :: (thread: *Green_Thread) #c_call {
    link := *first_thread;
    while link.* && link.* != thread link = *link.*.next;
    if link.* link.* = thread.next;
    wasi_free(thread);
}

// The next thread to run, after the one that just ran. With none ready, it sleeps until the
// first timed wait runs out. When every thread waits for something only another waiting thread
// could do, they are deadlocked: the one that blocked last gets EDEADLK.
next_to_run :: () -> *Green_Thread #c_call {
    while true {
        now := monotonic_nanoseconds();
        soonest: u64 = 0;
        thread := first_thread;
        while thread {
            if thread.status == .BLOCKED && thread.deadline != 0 {
                if thread.deadline <= now wake(thread, ETIMEDOUT);
                else if soonest == 0 || thread.deadline < soonest soonest = thread.deadline;
            }
            thread = thread.next;
        }
        start := ifx running && running.next then running.next else first_thread;
        thread = start;
        while true {
            if thread.status == .RUNNABLE return thread;
            thread = ifx thread.next then thread.next else first_thread;
            if thread == start break;
        }
        if soonest != 0 {
            sleep_nanoseconds(soonest - now);
            continue;
        }
        latest: *Green_Thread;
        thread = first_thread;
        while thread {
            if thread.status == .BLOCKED && (!latest || thread.ticket > latest.ticket) latest = thread;
            thread = thread.next;
        }
        if !latest trap();
        wake(latest, EDEADLK);
        return latest;
    }
}

// Blocks the running thread until another wakes it or `deadline` passes, and says why it woke:
// 0, ETIMEDOUT or EDEADLK. Only for programs jaic made able to unwind (`green.enabled`).
block_running :: (waiting_for: *void, deadline: u64) -> s32 #c_call {
    thread := running;
    thread.status = .BLOCKED;
    thread.waiting_for = waiting_for;
    thread.deadline = deadline;
    thread.wake_result = 0;
    next_ticket += 1;
    thread.ticket = next_ticket;
    green_suspend();
    return running.wake_result;
}

wake :: (thread: *Green_Thread, result: s32) #c_call {
    thread.status = .RUNNABLE;
    thread.waiting_for = null;
    thread.deadline = 0;
    thread.wake_result = result;
}

// Wakes the thread that has waited longest for `object`.
wake_one :: (object: *void) #c_call {
    first: *Green_Thread;
    thread := first_thread;
    while thread {
        if thread.status == .BLOCKED && thread.waiting_for == object && (!first || thread.ticket < first.ticket) first = thread;
        thread = thread.next;
    }
    if first wake(first, 0);
}

wake_all :: (object: *void, result: s32) #c_call {
    thread := first_thread;
    while thread {
        if thread.status == .BLOCKED && thread.waiting_for == object wake(thread, result);
        thread = thread.next;
    }
}

monotonic_nanoseconds :: () -> u64 #c_call {
    now: u64;
    if wasi_clock_time_get(1, 1, offset32(*now)) != 0 return 0;
    return now;
}

// Sleeps the whole interval: a host may wake a poll slightly early (Node's can).
sleep_nanoseconds :: (total: u64) -> bool #c_call {
    if total == 0 return true;
    now := monotonic_nanoseconds();
    deadline := now + total;
    while true {
        subscription: Clock_Subscription;
        subscription.kind = 0; // clock
        subscription.clock_id = 1; // monotonic
        subscription.timeout = total;
        event: Poll_Event;
        happened: u32;
        if wasi_poll_oneoff(offset32(*subscription), offset32(*event), 1, offset32(*happened)) != 0 return false;
        now = monotonic_nanoseconds();
        if now == 0 || now >= deadline break;
        total = deadline - now;
    }
    return true;
}

// A sleep is a timed wait for nothing, so other threads run meanwhile.
sleep_running :: (nanoseconds: u64) -> bool #c_call {
    if !green.enabled || !first_thread.next return sleep_nanoseconds(nanoseconds);
    if nanoseconds == 0 return true;
    block_running(null, monotonic_nanoseconds() + nanoseconds);
    return true;
}

// ---------------------------------------------------------------------------------------------
// The pthread calls. Handles are `Green_Thread` pointers; a mutex keeps its owner and lock
// count in the first bytes of a `pthread_mutex_t`. Every mutex is recursive.

Green_Mutex :: struct {
    owner: *Green_Thread;
    depth: s64;
}

EBUSY :: 16;

#program_export "pthread_create"
wasi_pthread_create :: (handle: *u64, attributes: *void, entry: Thread_Entry, argument: *void) -> s32 #c_call {
    if !green.enabled return EAGAIN;
    thread := add_thread(entry, argument, wasi_malloc(THREAD_FRAME_BYTES), THREAD_FRAME_BYTES);
    if !thread return EAGAIN;
    handle.* = cast(u64) thread;
    return 0;
}

#program_export "pthread_join"
wasi_pthread_join :: (handle: u64, result: **void) -> s32 #c_call {
    thread := cast(*Green_Thread) handle;
    if !thread return ESRCH;
    if thread == running return EDEADLK;
    while thread.status != .DONE {
        if block_running(thread, 0) == EDEADLK return EDEADLK;
    }
    if result result.* = thread.result;
    remove_thread(thread);
    return 0;
}

#program_export "pthread_tryjoin_np"
wasi_pthread_tryjoin_np :: (handle: u64, result: **void) -> s32 #c_call {
    thread := cast(*Green_Thread) handle;
    if !thread return ESRCH;
    if thread.status != .DONE return EBUSY;
    if result result.* = thread.result;
    remove_thread(thread);
    return 0;
}

#program_export "pthread_mutexattr_init"
wasi_pthread_mutexattr_init :: (attributes: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutexattr_settype"
wasi_pthread_mutexattr_settype :: (attributes: *void, kind: s32) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutexattr_destroy"
wasi_pthread_mutexattr_destroy :: (attributes: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutex_init"
wasi_pthread_mutex_init :: (mutex: *Green_Mutex, attributes: *void) -> s32 #c_call {
    mutex.owner = null;
    mutex.depth = 0;
    return 0;
}

#program_export "pthread_mutex_destroy"
wasi_pthread_mutex_destroy :: (mutex: *Green_Mutex) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutex_lock"
wasi_pthread_mutex_lock :: (mutex: *Green_Mutex) -> s32 #c_call {
    while mutex.owner && mutex.owner != running {
        if !green.enabled return EDEADLK;
        if block_running(mutex, 0) == EDEADLK return EDEADLK;
    }
    mutex.owner = running;
    mutex.depth += 1;
    return 0;
}

#program_export "pthread_mutex_trylock"
wasi_pthread_mutex_trylock :: (mutex: *Green_Mutex) -> s32 #c_call {
    if mutex.owner && mutex.owner != running return EBUSY;
    mutex.owner = running;
    mutex.depth += 1;
    return 0;
}

#program_export "pthread_mutex_unlock"
wasi_pthread_mutex_unlock :: (mutex: *Green_Mutex) -> s32 #c_call {
    if mutex.owner != running return EPERM;
    mutex.depth -= 1;
    if mutex.depth == 0 {
        mutex.owner = null;
        wake_one(mutex);
    }
    return 0;
}

#program_export "pthread_cond_init"
wasi_pthread_cond_init :: (condition: *void, attributes: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_cond_destroy"
wasi_pthread_cond_destroy :: (condition: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_cond_signal"
wasi_pthread_cond_signal :: (condition: *void) -> s32 #c_call {
    wake_one(condition);
    return 0;
}

#program_export "pthread_cond_broadcast"
wasi_pthread_cond_broadcast :: (condition: *void) -> s32 #c_call {
    wake_all(condition, 0);
    return 0;
}

#program_export "pthread_cond_wait"
wasi_pthread_cond_wait :: (condition: *void, mutex: *Green_Mutex) -> s32 #c_call {
    // With no other thread, nothing could ever signal.
    if !green.enabled return EDEADLK;
    return wait_condition(condition, mutex, 0);
}

// `deadline` is CLOCK_REALTIME, as pthread_cond_timedwait takes it by default.
#program_export "pthread_cond_timedwait"
wasi_pthread_cond_timedwait :: (condition: *void, mutex: *Green_Mutex, deadline: *Seconds_And_Nanoseconds) -> s32 #c_call {
    if !deadline return EINVAL;
    now: Seconds_And_Nanoseconds;
    if wasi_clock_gettime(0, *now) != 0 return EINVAL;
    remaining := (deadline.seconds - now.seconds) * 1_000_000_000 + deadline.nanoseconds - now.nanoseconds;
    if remaining <= 0 return ETIMEDOUT;
    if !green.enabled {
        sleep_nanoseconds(cast(u64) remaining);
        return ETIMEDOUT;
    }
    return wait_condition(condition, mutex, monotonic_nanoseconds() + cast(u64) remaining);
}

// Releases the mutex for the length of the wait, however deeply the thread holds it.
wait_condition :: (condition: *void, mutex: *Green_Mutex, deadline: u64) -> s32 #c_call {
    if mutex.owner != running return EPERM;
    depth := mutex.depth;
    mutex.owner = null;
    mutex.depth = 0;
    wake_one(mutex);
    result := block_running(condition, deadline);
    while mutex.owner && mutex.owner != running {
        if block_running(mutex, 0) == EDEADLK return EDEADLK;
    }
    mutex.owner = running;
    mutex.depth = depth;
    return result;
}
