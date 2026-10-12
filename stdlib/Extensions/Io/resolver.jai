// Name lookups for fibers, and the way helper threads wake a scheduler.
//
// `getaddrinfo` blocks, so it runs on a few helper threads that a scheduler starts the first time a
// fiber needs a lookup and joins when `io_run` ends. A helper hands its answer back through a pipe
// that the scheduler's own engine waits on (an ordinary internal READ operation, so every backend
// is woken the same way, with no backend-specific cross-thread call): it appends the finished job
// to a list under a mutex and writes a byte. The scheduler then wakes the fibers.
//
// A fiber that is canceled, or whose deadline passes, stops waiting at once; the lookup itself
// cannot be interrupted, so the job is abandoned and freed when its answer arrives.
//
// On Windows there is no pipe: a helper posts a `KEY_WAKE` packet to the scheduler's completion
// port, which `iocp_wait` turns into `wake_collect`, and `write_fd` holds that port.

Thread :: #import "Thread";

RESOLVER_THREADS :: 2;

Resolve_Job :: struct {
    host: *u8;
    port: u16;
    // 0, or the `getaddrinfo` error code.
    code: s32;
    address: Address;
    fiber: *Fiber;
    abandoned: bool;
    next: *Resolve_Job;
}

Wake_State :: struct {
    read_handle: Handle;
    write_fd: s64;
    // The READ on the pipe, while one is outstanding.
    op: *Op;
    buffer: [64] u8;

    // Lookups handed out whose answer has not been processed yet, abandoned ones included.
    outstanding: s64;

    // Shared with the helper threads.
    mutex: Thread.Mutex;
    work: Thread.Semaphore;
    queue_head: *Resolve_Job;
    queue_tail: *Resolve_Job;
    done_head: *Resolve_Job;
    done_tail: *Resolve_Job;
    quitting: bool;

    helpers: [RESOLVER_THREADS] Thread.Thread;
    helpers_started: s64;
}

resolve_with_helper :: (host: string, port: u16) -> Address, Io_Error {
    fiber, scheduler := current_fiber();

    fiber.platform_code = 0;
    if fiber.canceled return .{}, .CANCELED;
    if fiber.deadline != 0 && now_nanoseconds() >= fiber.deadline return .{}, .TIMED_OUT;

    state := wake_ensure(scheduler);
    if !state return .{}, .UNSUPPORTED;

    job := New(Resolve_Job);
    job.host = to_c_string(host);
    job.port = port;
    job.fiber = fiber;

    fiber.job = job;
    scheduler.pending += 1;
    state.outstanding += 1;

    Thread.lock(*state.mutex);
    if state.queue_tail state.queue_tail.next = job;
    else state.queue_head = job;

    state.queue_tail = job;
    Thread.unlock(*state.mutex);

    Thread.signal(*state.work);

    wake_arm(scheduler);

    if fiber.deadline != 0 timer_add(scheduler, fiber, fiber.deadline);

    reason := park(scheduler, fiber);

    // A cancel or a deadline already gave the job up.
    if reason != .NONE return .{}, reason;

    fiber.job = null;

    address := job.address;
    code := job.code;

    free(job.host);
    free(job);

    if code != 0 {
        fiber.platform_code = code;

        return .{}, .RESOLVE_FAILED;
    }

    return address, .NONE;
}

// Stops `fiber` waiting for its lookup. The helper still holds the job, so it is freed when the
// answer comes.
abandon_job :: (fiber: *Fiber) {
    job := fiber.job;

    job.abandoned = true;
    job.fiber = null;
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

    for 0..RESOLVER_THREADS - 1 {
        helper := *state.helpers[it];

        Thread.thread_init(helper, helper_main);
        helper.data = state;

        if Thread.thread_start(helper) state.helpers_started += 1;
    }

    return state;
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

        if job.abandoned {
            free(job.host);
            free(job);
        } else {
            scheduler.pending -= 1;
            wake_fiber(job);
        }

        job = next;
    }
}

wake_fiber :: (job: *Resolve_Job) {
    fiber := job.fiber;

    fiber.job = null;
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

    for 1..state.helpers_started Thread.signal(*state.work);
    for 0..state.helpers_started - 1 Thread.thread_deinit(*state.helpers[it]);

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
        job.address.port = job.port;
    } else {
        job.code = -1;
    }

    free_address_info(found);
}
