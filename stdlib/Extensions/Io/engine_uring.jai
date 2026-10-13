// The io_uring backend (Linux 6.1 and newer), on top of Extensions/Io_Uring.
//
// One ring per scheduler thread. It is set up with SINGLE_ISSUER | DEFER_TASKRUN | COOP_TASKRUN
// when the kernel allows it (the scheduler thread is the only one that submits or reaps, and
// completion work runs only when the scheduler enters the kernel, which it does when it has
// nothing else to do), and with fewer flags on a kernel that refuses some. Where the ring cannot
// be had at all (an old kernel, seccomp in a container, `kernel.io_uring_disabled`), `uring_start`
// returns why and the engine uses epoll instead.
//
// `uring_submit` only fills in a submission entry; entries reach the kernel in batches, whenever
// the scheduler waits (`uring_wait`) or the queue is full. `user_data` is the address of the `Op`,
// so a completion finds its operation without a lookup. Values below 16 are used for entries the
// engine does not track (the cancel requests).

// One listening socket's multishot accept. A fiber in `accept` waits on it; connections that
// arrive while nobody waits are queued for the next call.
Accept_Slot :: struct {
    fd: s64;
    // The armed request, null when the kernel has ended it (or it was never armed).
    op: *Op;
    // `close` ran: queued sockets are closed, and the slot is freed when the request ends.
    closing: bool;
    waiter: *Accept_Wait;
    queue: [..] s32;
    head: s64;
}

// The answer a parked `accept` is waiting for; lives on that fiber's stack.
Accept_Wait :: struct {
    fiber: *Fiber;
    result: s64;
    cause: Io_Error;
}

#if OS == .LINUX {
    Uring :: #import "Extensions/Io_Uring";

    // Submission queue size. The completion queue is twice that, and overflowing completions are
    // kept by the kernel until reaped.
    RING_ENTRIES :: 512;

    // `user_data` of entries nobody waits for.
    UNTRACKED :: 1;

    MSG_NOSIGNAL :: 0x4000;
    POLLIN :: 0x1;
    // Accepted sockets block: io_uring waits for them itself.
    SOCK_CLOEXEC :: 0x80000;

    Uring_State :: struct {
        ring: Uring.Ring;
        // Listeners that have (or had) a multishot accept armed.
        accepts: [..] *Accept_Slot;
    }

    // Accepted connections wait here when no fiber is in `accept`; more than this and the
    // multishot request is canceled until the queue drains.
    ACCEPT_QUEUE_LIMIT :: 128;

    // Tries to make the ring; returns why it could not, or "" if `engine` now uses io_uring.
    uring_start :: (engine: *Engine) -> string {
        state := New(Uring_State);

        flags := u32.[
            Uring.IORING_SETUP_SINGLE_ISSUER | Uring.IORING_SETUP_DEFER_TASKRUN | Uring.IORING_SETUP_COOP_TASKRUN,
            Uring.IORING_SETUP_COOP_TASKRUN,
            0
        ];

        result: s32 = -E_NOSYS;
        chosen: u32;

        for flags {
            result = Uring.ring_init(*state.ring, RING_ENTRIES, it);
            chosen = it;

            // Only an unknown flag is worth another try; a denial is final.
            if result != -E_INVAL break;
        }

        if result < 0 {
            free(state);

            if result == -E_NOSYS return "io_uring is not supported by this kernel";
            if result == -E_PERM || result == -E_ACCES return "io_uring is disabled here (seccomp, or kernel.io_uring_disabled)";
            if result == -E_NOMEM return "io_uring: not enough locked memory";

            return "io_uring setup failed";
        }

        ring := *state.ring;

        // The Linux 6.1 floor: the operations v1 uses all exist by then, and so does SENDMSG_ZC.
        probe: Uring.Probe;
        probed := Uring.probe_ops(ring, *probe);

        needed := Uring.io_uring_op.[.ACCEPT, .CONNECT, .RECV, .SEND, .READ, .WRITE, .CLOSE, .ASYNC_CANCEL, .POLL_ADD, .SENDMSG_ZC];
        usable := probed == 0 && (ring.features & Uring.IORING_FEAT_EXT_ARG) != 0;

        for needed {
            if !Uring.supports(*probe, it) usable = false;
        }

        if !usable {
            Uring.ring_deinit(ring);
            free(state);

            return "the kernel is older than Linux 6.1";
        }

        for 0..255 {
            if probe.supported[it] engine.caps.kernel_ops += 1;
        }

        engine.caps.single_issuer = (chosen & Uring.IORING_SETUP_SINGLE_ISSUER) != 0;
        engine.caps.defer_taskrun = (chosen & Uring.IORING_SETUP_DEFER_TASKRUN) != 0;
        engine.caps.coop_taskrun = (chosen & Uring.IORING_SETUP_COOP_TASKRUN) != 0;
        engine.caps.ext_arg = true;

        engine.uring = state;
        engine.backend = .IO_URING;

        return "";
    }

    uring_stop :: (engine: *Engine) {
        for engine.uring.accepts {
            for i: it.head..it.queue.count - 1 sys_close(it.queue[i]);

            array_reset(*it.queue);
            free(it);
        }

        array_reset(*engine.uring.accepts);

        Uring.ring_deinit(*engine.uring.ring);
        free(engine.uring);
        engine.uring = null;
    }

    // A free submission entry. With none free, the queued ones go to the kernel first, and
    // completions are reaped so that a full completion queue cannot hold submission back.
    uring_entry :: (scheduler: *Scheduler, engine: *Engine) -> *Uring.io_uring_sqe {
        ring := *engine.uring.ring;
        sqe := Uring.get_sqe(ring);

        while !sqe {
            Uring.submit(ring);
            uring_reap(scheduler, engine);

            sqe = Uring.get_sqe(ring);
        }

        return sqe;
    }

    uring_submit :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {
        if op.kind == .CLOSE {
            uring_close(scheduler, engine, op);

            return;
        }

        sqe := uring_entry(scheduler, engine);

        fd := cast(s32) op.handle.fd;
        length := cast(u32) min(op.length, MAX_TRANSFER);
        position: u64 = cast(u64) op.offset;

        if op.kind == {
            case .ACCEPT;
                op.address_length = size_of(Sock_Storage);
                Uring.prep_rw(sqe, .ACCEPT, fd, *op.address, 0, cast(u64) cast(*void) *op.address_length);
                sqe.accept_flags = SOCK_CLOEXEC;

            case .CONNECT;
                Uring.prep_rw(sqe, .CONNECT, fd, *op.address, 0, op.address_length);

            case .RECV;
                Uring.prep_rw(sqe, .RECV, fd, op.buffer, length, 0);

            case .SEND;
                Uring.prep_rw(sqe, .SEND, fd, op.buffer, length, 0);
                sqe.msg_flags = MSG_NOSIGNAL;

            case .READ;
                Uring.prep_rw(sqe, .READ, fd, op.buffer, length, position);

            case .WRITE;
                Uring.prep_rw(sqe, .WRITE, fd, op.buffer, length, position);

            case .CLOSE;
                Uring.prep_rw(sqe, .CLOSE, fd, null, 0, 0);

            case .WAIT_READABLE;
                // One shot; the result is the events that fired, which `wait_readable` ignores.
                Uring.prep_rw(sqe, .POLL_ADD, fd, null, 0, 0);
                sqe.poll32_events = POLLIN;
        }

        sqe.user_data = cast(u64) cast(*void) op;
    }

    // Operations still waiting on a descriptor would keep the file alive past the close, so they
    // are canceled first. The cancel is hard-linked so that finding nothing to cancel does not
    // stop the close.
    uring_close :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {
        ring := *engine.uring.ring;

        if Uring.sq_pending(ring) + 2 > ring.sq_entries Uring.submit(ring);

        fd := cast(s32) op.handle.fd;

        sqe := uring_entry(scheduler, engine);
        sqe.opcode = cast(u8) Uring.io_uring_op.ASYNC_CANCEL;
        sqe.fd = fd;
        sqe.cancel_flags = Uring.IORING_ASYNC_CANCEL_FD | Uring.IORING_ASYNC_CANCEL_ALL;
        sqe.flags = Uring.IOSQE_IO_HARDLINK;
        sqe.user_data = UNTRACKED;

        sqe = uring_entry(scheduler, engine);
        Uring.prep_rw(sqe, .CLOSE, fd, null, 0, 0);
        sqe.user_data = cast(u64) cast(*void) op;
    }

    uring_cancel :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {
        sqe := uring_entry(scheduler, engine);

        Uring.prep_rw(sqe, .ASYNC_CANCEL, -1, cast(*void) op, 0, 0);
        sqe.user_data = UNTRACKED;
    }

    uring_wait :: (scheduler: *Scheduler, engine: *Engine, timeout: s64) {
        ring := *engine.uring.ring;

        unconsumed := Uring.publish_sqes(ring);

        // Completions that are already in the ring are taken without waiting.
        if timeout != 0 && Uring.cq_ready(ring) > 0 timeout = 0;

        flags: u32 = Uring.IORING_ENTER_GETEVENTS;
        wait_for: u32 = 1;

        argument: Uring.io_uring_getevents_arg;
        limit: Uring.__kernel_timespec;
        argument_pointer: *void;
        argument_bytes: u64;

        if timeout == 0 {
            wait_for = 0;
        } else if timeout > 0 {
            limit.tv_sec = timeout / 1_000_000_000;
            limit.tv_nsec = timeout % 1_000_000_000;
            argument.ts = cast(u64) cast(*void) *limit;
            argument_pointer = *argument;
            argument_bytes = size_of(Uring.io_uring_getevents_arg);
            flags |= Uring.IORING_ENTER_EXT_ARG;
        }

        // -ETIME (the wait ran out), -EINTR and -EBUSY need nothing: reap whatever is there.
        Uring.io_uring_enter(ring.fd, unconsumed, wait_for, flags, argument_pointer, argument_bytes);

        uring_reap(scheduler, engine);
    }

    uring_reap :: (scheduler: *Scheduler, engine: *Engine) {
        ring := *engine.uring.ring;

        // `finish_op` can re-enter this function (through `uring_entry` when the submission queue is
        // full), and the inner call consumes entries too: `head` is read again for every entry and
        // each one is consumed before its handler runs.
        tail := atomic_load(ring.cq_tail, .ACQUIRE);

        while ring.cq_head.* != tail {
            head := ring.cq_head.*;
            cqe := cast(*Uring.io_uring_cqe) byte_offset(ring.cqes, (head & ring.cq_mask) * ring.cqe_bytes);

            user_data := cqe.user_data;
            result := cqe.res;
            flags := cqe.flags;

            atomic_store(ring.cq_head, head + 1, .RELEASE);

            if user_data > 15 {
                op := cast(*Op) cast(*void) user_data;

                if op.slot {
                    accept_completed(scheduler, op, result, flags);
                    tail = atomic_load(ring.cq_tail, .ACQUIRE);

                    continue;
                }

                if flags & Uring.IORING_CQE_F_BUFFER op.buffer_id = cast(s32) (flags >> Uring.IORING_CQE_BUFFER_SHIFT);

                finish_op(scheduler, op, result, flags, (flags & Uring.IORING_CQE_F_MORE) != 0);
            }

            tail = atomic_load(ring.cq_tail, .ACQUIRE);
        }
    }

    accept_slot :: (state: *Uring_State, fd: s64) -> *Accept_Slot {
        for state.accepts {
            if it.fd == fd && !it.closing return it;
        }

        return null;
    }

    // `accept` through one multishot request per listener instead of one request per connection.
    // Returns false (and does nothing) when this call has to use a one-shot `accept` instead:
    // another fiber is already waiting on the listener, or the request is being canceled.
    uring_accept :: (handle: Handle) -> bool, s64, Io_Error {
        fiber, scheduler := current_fiber();
        engine := scheduler.engine;
        state := engine.uring;

        slot := accept_slot(state, handle.fd);

        if !slot {
            slot = New(Accept_Slot);
            slot.fd = handle.fd;
            array_add(*state.accepts, slot);
        }

        if slot.waiter || (slot.op && slot.op.canceling) return false, 0, .NONE;

        fiber.platform_code = 0;
        if fiber.canceled return true, 0, .CANCELED;
        if fiber.deadline != 0 && now_nanoseconds() >= fiber.deadline return true, 0, .TIMED_OUT;

        if slot.head < slot.queue.count {
            fd := slot.queue[slot.head];

            slot.head += 1;

            if slot.head == slot.queue.count {
                slot.head = 0;
                slot.queue.count = 0;
            }

            return true, fd, .NONE;
        }

        if !slot.op {
            op := new_op(engine, .ACCEPT, handle);
            op.slot = slot;

            sqe := uring_entry(scheduler, engine);

            Uring.prep_rw(sqe, .ACCEPT, cast(s32) handle.fd, null, 0, 0);
            sqe.ioprio = Uring.IORING_ACCEPT_MULTISHOT;
            sqe.accept_flags = SOCK_CLOEXEC;
            sqe.user_data = cast(u64) cast(*void) op;

            slot.op = op;
        }

        wait: Accept_Wait;
        wait.fiber = fiber;

        slot.waiter = *wait;
        fiber.op = slot.op;
        scheduler.pending += 1;

        if fiber.deadline != 0 timer_add(scheduler, fiber, fiber.deadline);

        park(scheduler, fiber);

        fiber.op = null;

        if wait.result >= 0 return true, wait.result, .NONE;

        code := -wait.result;

        fiber.platform_code = cast(s32) code;

        if code == E_CANCELED {
            if wait.cause != .NONE return true, 0, wait.cause;
            if fiber.canceled return true, 0, .CANCELED;

            return true, 0, .CLOSED;
        }

        return true, 0, error_from_code(code);
    }

    accept_completed :: (scheduler: *Scheduler, op: *Op, result: s64, flags: u32) {
        slot := op.slot;
        more := (flags & Uring.IORING_CQE_F_MORE) != 0;
        waiter := slot.waiter;

        if !more {
            slot.op = null;
            op.state = .DONE;
        }

        if waiter {
            // Connections go to the waiting fiber, and so does the end of the request if it ends
            // with an error. Taking `waiter` off the slot first makes this the one wake.
            if result >= 0 && slot.closing {
                sys_close(result);
            } else {
                waiter.result = result;
                waiter.cause = op.cancel_cause;
                slot.waiter = null;
                scheduler.pending -= 1;

                if !more waiter.fiber.op = null;

                wake(waiter.fiber);
            }
        } else if result >= 0 {
            if slot.closing {
                sys_close(result);
            } else {
                array_add(*slot.queue, cast(s32) result);

                if more && slot.queue.count - slot.head >= ACCEPT_QUEUE_LIMIT && !op.canceling {
                    engine_cancel(scheduler, op);
                }
            }
        }

        if !more {
            free_op(scheduler.engine, op);
            if slot.closing accept_free(scheduler.engine.uring, slot);
        }
    }

    accept_free :: (state: *Uring_State, slot: *Accept_Slot) {
        for state.accepts {
            if it == slot {
                state.accepts[it_index] = state.accepts[state.accepts.count - 1];
                state.accepts.count -= 1;

                break;
            }
        }

        array_reset(*slot.queue);
        free(slot);
    }

    // The listener `fd` is being closed: sockets accepted ahead of time are closed, and the slot
    // goes when the kernel has ended its request (the close cancels it).
    uring_listener_closing :: (scheduler: *Scheduler, engine: *Engine, fd: s64) {
        state := engine.uring;
        slot := accept_slot(state, fd);

        if !slot return;

        for i: slot.head..slot.queue.count - 1 sys_close(slot.queue[i]);

        slot.queue.count = 0;
        slot.head = 0;
        slot.closing = true;

        if !slot.op accept_free(state, slot);
    }

    // `close(now = true)`: the descriptor is closed without waiting, so the multishot request has
    // to be canceled by hand or it would keep the socket open.
    uring_listener_abandoned :: (scheduler: *Scheduler, engine: *Engine, fd: s64) {
        state := engine.uring;
        slot := accept_slot(state, fd);

        if !slot return;

        had_request := slot.op != null;

        uring_listener_closing(scheduler, engine, fd);

        if !had_request return;

        sqe := uring_entry(scheduler, engine);
        sqe.opcode = cast(u8) Uring.io_uring_op.ASYNC_CANCEL;
        sqe.fd = cast(s32) fd;
        sqe.cancel_flags = Uring.IORING_ASYNC_CANCEL_FD | Uring.IORING_ASYNC_CANCEL_ALL;
        sqe.user_data = UNTRACKED;

        Uring.submit(*engine.uring.ring);
    }

    byte_offset :: (base: *void, bytes: $T) -> *void {
        return cast(*void) (cast(u64) base + cast(u64) bytes);
    }
} else {
    Uring_State :: struct {}

    uring_start :: (engine: *Engine) -> string {
        return "io_uring exists only on Linux";
    }

    uring_stop :: (engine: *Engine) {}
    uring_listener_closing :: (scheduler: *Scheduler, engine: *Engine, fd: s64) {}
    uring_listener_abandoned :: (scheduler: *Scheduler, engine: *Engine, fd: s64) {}
    uring_submit :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {}
    uring_cancel :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {}
    uring_wait :: (scheduler: *Scheduler, engine: *Engine, timeout: s64) {}
}
