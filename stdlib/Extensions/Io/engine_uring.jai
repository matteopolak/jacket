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

#if OS == .LINUX {
    Uring :: #import "Extensions/Io_Uring";

    Uring_State :: struct {
        ring: Uring.Ring;
    }

    // Submission queue size. The completion queue is twice that, and overflowing completions are
    // kept by the kernel until reaped.
    RING_ENTRIES :: 512;

    // `user_data` of entries nobody waits for.
    UNTRACKED :: 1;

    MSG_NOSIGNAL :: 0x4000;
    // Accepted sockets block: io_uring waits for them itself.
    SOCK_CLOEXEC :: 0x80000;

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

        needed := Uring.io_uring_op.[.ACCEPT, .CONNECT, .RECV, .SEND, .READ, .WRITE, .CLOSE, .ASYNC_CANCEL, .SENDMSG_ZC];
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

        head := ring.cq_head.*;
        tail := atomic_load(ring.cq_tail, .ACQUIRE);

        while head != tail {
            cqe := cast(*Uring.io_uring_cqe) byte_offset(ring.cqes, (head & ring.cq_mask) * ring.cqe_bytes);

            user_data := cqe.user_data;
            result := cqe.res;
            flags := cqe.flags;

            head += 1;
            atomic_store(ring.cq_head, head, .RELEASE);

            if user_data > 15 {
                op := cast(*Op) cast(*void) user_data;

                if flags & Uring.IORING_CQE_F_BUFFER op.buffer_id = cast(s32) (flags >> Uring.IORING_CQE_BUFFER_SHIFT);

                finish_op(scheduler, op, result, flags, (flags & Uring.IORING_CQE_F_MORE) != 0);
            }

            tail = atomic_load(ring.cq_tail, .ACQUIRE);
        }
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
    uring_submit :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {}
    uring_cancel :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {}
    uring_wait :: (scheduler: *Scheduler, engine: *Engine, timeout: s64) {}
}
