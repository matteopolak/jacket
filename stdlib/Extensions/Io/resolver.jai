// Name lookups for fibers, and the way helper threads wake a scheduler.
//
// `getaddrinfo` blocks, so it runs on a few helper threads that a scheduler starts the first time a
// fiber needs a lookup and joins when `io_run` ends. A helper hands its answer back through a pipe
// that the scheduler's own engine waits on (an ordinary internal READ operation, so every backend
// is woken the same way, with no backend-specific cross-thread call): it appends the finished job
// to a list under a mutex and writes a byte. The scheduler then wakes the fibers.
//
// Lookups for the same name that are under way share one job: later fibers join the job's waiter
// list, and each gets the answer with its own port. A fiber that is canceled, or whose deadline
// passes, stops waiting at once and only leaves the list; the lookup itself cannot be
// interrupted, so the job (with or without waiters) is freed when its answer is collected. Helper
// threads start one at a time as lookups queue up, up to `RESOLVER_MAX_THREADS`.
//
// On Windows there is no pipe: a helper posts a `KEY_WAKE` packet to the scheduler's completion
// port, which `iocp_wait` turns into `wake_collect`, and `write_fd` holds that port.

Thread :: #import "Thread";

// Helper threads at least one lookup needs, and the most a scheduler starts. A helper stays busy
// until the OS answers, so a lookup that finds every helper busy starts another.
RESOLVER_MIN_THREADS :: 1;
RESOLVER_MAX_THREADS :: 8;

// One `getaddrinfo` call, shared by every fiber that asked for the same name while it was under way.
Resolve_Job :: struct {
    host: *u8;
    host_length: s64;
    // 0, or the `getaddrinfo` error code. Written by the helper, read after it is collected.
    code: s32;
    // The answer with port 0; each waiter applies its own port.
    address: Address;
    // Fibers waiting for the answer (scheduler thread only).
    waiters: *Resolve_Waiter;
    // The helper queues (mutex-protected).
    next: *Resolve_Job;
    // Jobs not yet collected, to find one to join (scheduler thread only).
    inflight_next: *Resolve_Job;
}

// One fiber's wait for a job. It lives on that fiber's stack, which is alive while the fiber waits
// (a canceled fiber leaves the list before it runs again).
Resolve_Waiter :: struct {
    job: *Resolve_Job;
    fiber: *Fiber;
    port: u16;
    code: s32;
    address: Address;
    next: *Resolve_Waiter;
    prev: *Resolve_Waiter;
}

Wake_State :: struct {
    read_handle: Handle;
    write_fd: s64;
    // The READ on the pipe, while one is outstanding.
    op: *Op;
    buffer: [64] u8;

    // Jobs handed out whose answer has not been collected yet, ones nobody waits for included. A
    // job holds a helper from the time it is queued, so this is also the number of helpers needed.
    outstanding: s64;
    inflight: *Resolve_Job;
    // Jobs made over this scheduler's life (for tests and diagnostics).
    jobs_made: s64;

    // Shared with the helper threads.
    mutex: Thread.Mutex;
    work: Thread.Semaphore;
    queue_head: *Resolve_Job;
    queue_tail: *Resolve_Job;
    done_head: *Resolve_Job;
    done_tail: *Resolve_Job;
    quitting: bool;

    helpers: [RESOLVER_MAX_THREADS] Thread.Thread;
    helpers_started: s64;
}

resolve_with_helper :: (host: string, port: u16) -> Address, Io_Error {
    fiber, scheduler := current_fiber();

    fiber.platform_code = 0;
    if fiber.canceled return .{}, .CANCELED;
    if fiber.deadline != 0 && now_nanoseconds() >= fiber.deadline return .{}, .TIMED_OUT;

    state := wake_ensure(scheduler);
    if !state return .{}, .UNSUPPORTED;

    job := find_job(state, host);

    if !job {
        // Every helper may be stuck in a slow lookup: one more for this job.
        wake_grow(state, state.outstanding + 1);
        if state.helpers_started == 0 return .{}, .UNSUPPORTED;

        job = New(Resolve_Job);
        job.host = to_c_string(host);
        job.host_length = host.count;
        job.inflight_next = state.inflight;

        state.inflight = job;
        state.outstanding += 1;
        state.jobs_made += 1;

        Thread.lock(*state.mutex);
        if state.queue_tail state.queue_tail.next = job;
        else state.queue_head = job;

        state.queue_tail = job;
        Thread.unlock(*state.mutex);

        Thread.signal(*state.work);
    }

    waiter: Resolve_Waiter;
    waiter.job = job;
    waiter.fiber = fiber;
    waiter.port = port;
    waiter.next = job.waiters;
    if job.waiters job.waiters.prev = *waiter;

    job.waiters = *waiter;

    fiber.job = *waiter;
    scheduler.pending += 1;

    // On a readiness backend the pipe READ can complete here and collect this very job, if the
    // helper was quick; `wake_fiber` then leaves the answer without waking this running fiber.
    wake_arm(scheduler);

    if fiber.job {
        if fiber.deadline != 0 timer_add(scheduler, fiber, fiber.deadline);

        reason := park(scheduler, fiber);

        // A cancel or a deadline already took this fiber off the job.
        if reason != .NONE return .{}, reason;
    }

    fiber.job = null;

    if waiter.code != 0 {
        fiber.platform_code = waiter.code;

        return .{}, .RESOLVE_FAILED;
    }

    return waiter.address, .NONE;
}

// The job for `host` that is queued or running, if any.
find_job :: (state: *Wake_State, host: string) -> *Resolve_Job {
    job := state.inflight;

    while job {
        if job.host_length == host.count {
            same := true;

            for 0..host.count - 1 {
                if job.host[it] != host[it] {
                    same = false;

                    break;
                }
            }

            if same return job;
        }

        job = job.inflight_next;
    }

    return null;
}

// Stops `fiber` waiting for its lookup. The job stays: a helper may be running it, and others may
// wait for it. It is freed when its answer is collected.
abandon_job :: (fiber: *Fiber) {
    waiter := fiber.job;
    job := waiter.job;

    if waiter.prev waiter.prev.next = waiter.next;
    else job.waiters = waiter.next;

    if waiter.next waiter.next.prev = waiter.prev;

    waiter.next = null;
    waiter.prev = null;

    fiber.job = null;
    fiber.scheduler.pending -= 1;
}

// ---------------------------------------------------------------------------------------------
// The scheduler's side

wake_ensure :: (scheduler: *Scheduler) -> *Wake_State {
    if scheduler.waker return scheduler.waker;

    state := New(Wake_State);

    #if OS == .WINDOWS {
        state.write_fd = scheduler.engine.iocp.port;
    } else {
        pipe: [2] s32;

        if sys_pipe(*pipe, scheduler.engine.backend != .IO_URING) < 0 {
            free(state);

            return null;
        }

        state.read_handle = .{ pipe[0], .STREAM };
        state.write_fd = pipe[1];
    }

    Thread.init(*state.mutex);
    Thread.init(*state.work);

    scheduler.waker = state;

    wake_grow(state, RESOLVER_MIN_THREADS);

    return state;
}

// Starts helpers until there are `wanted` (at most `RESOLVER_MAX_THREADS`). A helper that cannot be
// started is left out; the caller checks `helpers_started`.
wake_grow :: (state: *Wake_State, wanted: s64) {
    while state.helpers_started < min(wanted, RESOLVER_MAX_THREADS) {
        helper := *state.helpers[state.helpers_started];

        Thread.thread_init(helper, helper_main);
        helper.data = state;

        if !Thread.thread_start(helper) break;

        state.helpers_started += 1;
    }
}

// Makes sure a READ on the pipe is outstanding while lookups are, and takes answers that are
// already there.
wake_arm :: (scheduler: *Scheduler) {
    #if OS == .WINDOWS return;

    state := scheduler.waker;

    while !state.op && state.outstanding > 0 {
        op := new_op(scheduler.engine, .READ, state.read_handle);
        op.internal = true;
        op.tag = .RESOLVER_WAKE;
        op.buffer = state.buffer.data;
        op.length = state.buffer.count;

        if engine_submit(scheduler, op) {
            free_op(scheduler.engine, op);
            wake_collect(scheduler);
        } else {
            state.op = op;
        }
    }
}

// The READ on the pipe completed.
wake_completed :: (scheduler: *Scheduler, op: *Op) {
    state := scheduler.waker;

    state.op = null;

    canceled := op.result == -E_CANCELED;

    free_op(scheduler.engine, op);

    if canceled return;

    wake_collect(scheduler);
    wake_arm(scheduler);
}

// Takes the finished jobs and wakes their fibers.
wake_collect :: (scheduler: *Scheduler) {
    state := scheduler.waker;

    Thread.lock(*state.mutex);
    job := state.done_head;

    state.done_head = null;
    state.done_tail = null;
    Thread.unlock(*state.mutex);

    while job {
        next := job.next;

        state.outstanding -= 1;

        link := *state.inflight;

        while link.* != job link = *link.*.inflight_next;

        link.* = job.inflight_next;

        // Each waiter keeps its own copy: the job goes now, the waiters run later.
        waiter := job.waiters;

        job.waiters = null;

        while waiter {
            following := waiter.next;

            waiter.next = null;
            waiter.prev = null;
            waiter.code = job.code;
            waiter.address = job.address;
            waiter.address.port = waiter.port;

            scheduler.pending -= 1;
            wake_fiber(waiter);

            waiter = following;
        }

        free(job.host);
        free(job);

        job = next;
    }
}

wake_fiber :: (waiter: *Resolve_Waiter) {
    fiber := waiter.fiber;

    fiber.job = null;

    // The fiber that queued the job and has not parked yet finds its answer when `wake_arm` returns.
    if fiber.state == .RUNNING return;

    wake(fiber);
}

// Waits for the lookups still under way, ends the helpers and closes the pipe.
wake_stop :: (scheduler: *Scheduler) {
    state := scheduler.waker;
    if !state return;

    while state.outstanding > 0 {
        wake_arm(scheduler);
        if state.outstanding > 0 engine_wait(scheduler, -1);
    }

    if state.op {
        engine_cancel(scheduler, state.op);

        while state.op engine_wait(scheduler, -1);
    }

    Thread.lock(*state.mutex);
    state.quitting = true;
    Thread.unlock(*state.mutex);

    // Every helper started so far, later ones included, is joined.
    for 1..state.helpers_started Thread.signal(*state.work);

    index := 0;

    while index < state.helpers_started {
        Thread.thread_deinit(*state.helpers[index]);

        index += 1;
    }

    #if OS != .WINDOWS {
        sys_close(state.read_handle.fd);
        sys_close(state.write_fd);
    }

    Thread.destroy(*state.mutex);
    Thread.destroy(*state.work);

    free(state);
    scheduler.waker = null;
}

// ---------------------------------------------------------------------------------------------
// The helper threads

helper_main :: (thread: *Thread.Thread) -> s64 {
    state := cast(*Wake_State) thread.data;

    while true {
        Thread.wait_for(*state.work);

        Thread.lock(*state.mutex);

        job := state.queue_head;
        if job {
            state.queue_head = job.next;
            if !state.queue_head state.queue_tail = null;

            job.next = null;
        }

        quitting := state.quitting;

        Thread.unlock(*state.mutex);

        if !job {
            if quitting break;

            continue;
        }

        lookup(job);

        Thread.lock(*state.mutex);
        if state.done_tail state.done_tail.next = job;
        else state.done_head = job;

        state.done_tail = job;
        Thread.unlock(*state.mutex);

        #if OS == .WINDOWS {
            iocp_post(state.write_fd, KEY_WAKE);
        } else {
            byte: u8 = 1;

            sys_write(state.write_fd, *byte, 1, -1);
        }
    }

    return 0;
}

#if OS == .WINDOWS {
    Addrinfo :: struct {
        flags: s32;
        family: s32;
        socket_type: s32;
        protocol: s32;
        address_length: u64;
        canonical_name: *u8;
        address: *Sock_Storage;
        next: *Addrinfo;
    }
} else #if OS == .MACOS {
    Addrinfo :: struct {
        flags: s32;
        family: s32;
        socket_type: s32;
        protocol: s32;
        address_length: u32;
        canonical_name: *u8;
        address: *Sock_Storage;
        next: *Addrinfo;
    }
} else {
    Addrinfo :: struct {
        flags: s32;
        family: s32;
        socket_type: s32;
        protocol: s32;
        address_length: u32;
        address: *Sock_Storage;
        canonical_name: *u8;
        next: *Addrinfo;
    }
}

#if OS != .WINDOWS {
    c_getaddrinfo :: (node: *u8, service: *u8, hints: *Addrinfo, result: **Addrinfo) -> s32 #foreign libc "getaddrinfo";
    c_freeaddrinfo :: (result: *Addrinfo) #foreign libc "freeaddrinfo";

    address_info :: (host: *u8, hints: *Addrinfo, found: **Addrinfo) -> s32 {
        return c_getaddrinfo(host, null, hints, found);
    }

    free_address_info :: (found: *Addrinfo) {
        c_freeaddrinfo(found);
    }
} else {
    address_info :: (host: *u8, hints: *Addrinfo, found: **Addrinfo) -> s32 {
        return win_get_address_info(host, null, hints, found);
    }

    free_address_info :: (found: *Addrinfo) {
        win_free_address_info(found);
    }
}

// Runs on a helper thread: no context is needed for plain C calls.
lookup :: (job: *Resolve_Job) {
    hints: Addrinfo;
    hints.socket_type = SOCK_STREAM;

    found: *Addrinfo;

    job.code = address_info(job.host, *hints, *found);
    if job.code != 0 return;

    best: *Addrinfo;

    entry := found;

    while entry {
        if entry.address && (entry.family == AF_INET || entry.family == AF_INET6) {
            if entry.family == AF_INET {
                best = entry;

                break;
            }

            if !best best = entry;
        }

        entry = entry.next;
    }

    if best {
        job.address = from_sockaddr(best.address);
    } else {
        job.code = -1;
    }

    free_address_info(found);
}

#scope_export

// Lookups made and helper threads started on the running scheduler so far, for tests and
// diagnostics. Both are 0 before the first name lookup.
resolver_statistics :: () -> s64, s64 {
    scheduler := current_scheduler();
    state := scheduler.waker;

    if !state return 0, 0;

    return state.jobs_made, state.helpers_started;
}

#scope_module
