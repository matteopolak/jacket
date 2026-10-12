// The readiness backends: epoll (Linux) and kqueue (macOS). They share everything but the calls
// that register interest and wait.
//
// A readiness API does not do the IO, so these backends try each operation at once, without
// blocking. If the system says "not now" (EAGAIN), the operation waits in a queue on its
// descriptor, one queue for readers and one for writers, and the descriptor is armed in the kernel
// for one notification (kqueue EV_ONESHOT, epoll EPOLLONESHOT). When it fires, the queue is served
// in order until an operation would block again, and the descriptor is armed again if anyone is
// left. Two fibers waiting on one descriptor therefore never lose each other's wake-up.
//
// Regular files never report "not ready", so they cannot wait in the poller. A short read or write
// on a file runs at once (blocking the thread for the page cache's few microseconds); a longer one
// goes to the file thread pool (files_pool.jai) and only its fiber waits. Closing a descriptor fails
// every operation still waiting on it with the cancel error (`.CLOSED` for its fiber).

// Descriptor numbers above this are not tracked.
MAX_POLLED_FDS :: 1 << 22;

poll_submit :: (scheduler: *Scheduler, engine: *Engine, op: *Op) -> bool {
    fd := op.handle.fd;

    if op.kind == .CLOSE {
        // A pool worker may be using the file.
        if op.handle.kind == .FILE pool_drain(scheduler, engine);

        poll_close(scheduler, engine, fd);
        op.result = sys_close_handle(op.handle);
        op.state = .DONE;

        return true;
    }

    if (op.kind == .READ || op.kind == .WRITE) && op.handle.kind == .FILE && engine.file_threads > 0 && op.length > engine.file_inline {
        if pool_submit(scheduler, engine, op) return false;
    }

    if poll_attempt(op) {
        op.state = .DONE;

        return true;
    }

    if fd < 0 || fd >= MAX_POLLED_FDS {
        op.result = -E_MFILE;
        op.state = .DONE;

        return true;
    }

    if fd >= engine.fds.count {
        old := engine.fds.count;
        array_resize(*engine.fds, max(fd + 1, old * 2));
    }

    wait_append(*engine.fds[fd], op);
    poll_arm(scheduler, engine, fd);

    return false;
}

// One non-blocking try. True: finished, `op.result` is set. False: it must wait for readiness
// (`op.wants_write` says which).
poll_attempt :: (op: *Op) -> bool {
    fd := op.handle.fd;
    result: s64;

    if op.kind == {
        case .ACCEPT;
            op.wants_write = false;

            while true {
                result = sys_accept(fd, *op.address, *op.address_length);
                if result != -E_CONNABORTED break;
            }

        case .CONNECT;
            op.wants_write = true;

            if op.phase == 0 {
                result = sys_connect(fd, *op.address, op.address_length);
                if result == -E_INPROGRESS {
                    op.phase = 1;

                    return false;
                }
            } else {
                result = -sys_socket_error(fd);
            }

        case .RECV;
            op.wants_write = false;
            result = sys_recv(fd, op.buffer, op.length);

        case .SEND;
            op.wants_write = true;
            result = sys_send(fd, op.buffer, op.length);

        case .READ;
            op.wants_write = false;
            result = sys_read(fd, op.buffer, op.length, ifx op.handle.kind == .FILE then op.offset else -1);

        case .WRITE;
            op.wants_write = true;
            result = sys_write(fd, op.buffer, op.length, ifx op.handle.kind == .FILE then op.offset else -1);

        case .CLOSE;
            result = sys_close(fd);
    }

    if result == -E_AGAIN return false;

    op.result = result;

    return true;
}

// Serves the operations waiting on `fd` now that it may be ready, then arms it again if any remain.
poll_ready :: (scheduler: *Scheduler, engine: *Engine, fd: s64, readable: bool, writable: bool) {
    if fd >= engine.fds.count return;

    state := *engine.fds[fd];

    while readable && state.read_head {
        op := state.read_head;
        if !poll_attempt(op) break;

        wait_unlink(state, op);
        finish_op(scheduler, op, op.result);
    }

    while writable && state.write_head {
        op := state.write_head;
        if !poll_attempt(op) break;

        wait_unlink(state, op);
        finish_op(scheduler, op, op.result);
    }

    poll_arm(scheduler, engine, fd);
}

poll_cancel :: (scheduler: *Scheduler, engine: *Engine, op: *Op) {
    // A pool worker has it and will report it.
    if op.uncancelable return;

    wait_unlink(*engine.fds[op.handle.fd], op);
    finish_op(scheduler, op, -E_CANCELED);
}

// Ends every operation waiting on `fd` with `result`.
poll_fail :: (scheduler: *Scheduler, engine: *Engine, fd: s64, result: s64, readers: bool, writers: bool) {
    if fd >= engine.fds.count return;

    state := *engine.fds[fd];

    while readers && state.read_head {
        op := state.read_head;
        wait_unlink(state, op);
        finish_op(scheduler, op, result);
    }

    while writers && state.write_head {
        op := state.write_head;
        wait_unlink(state, op);
        finish_op(scheduler, op, result);
    }
}

// `fd` is about to be closed: fail what waits on it and forget what the kernel was told.
poll_close :: (scheduler: *Scheduler, engine: *Engine, fd: s64) {
    if fd < 0 || fd >= engine.fds.count return;

    poll_fail(scheduler, engine, fd, -E_CANCELED, true, true);

    state := *engine.fds[fd];

    if engine.backend == .KQUEUE kqueue_flush(engine);
    if engine.backend == .EPOLL && state.registered epoll_remove(engine, fd);

    state.* = .{};
}

wait_append :: (state: *Fd_State, op: *Op) {
    op.next = null;

    if op.wants_write {
        op.previous = state.write_tail;
        if state.write_tail state.write_tail.next = op;
        else state.write_head = op;

        state.write_tail = op;
    } else {
        op.previous = state.read_tail;
        if state.read_tail state.read_tail.next = op;
        else state.read_head = op;

        state.read_tail = op;
    }
}

wait_unlink :: (state: *Fd_State, op: *Op) {
    if op.previous op.previous.next = op.next;
    else if op.wants_write state.write_head = op.next;
    else state.read_head = op.next;

    if op.next op.next.previous = op.previous;
    else if op.wants_write state.write_tail = op.previous;
    else state.read_tail = op.previous;

    op.next = null;
    op.previous = null;
}

// Makes sure the kernel will report `fd` when someone is waiting on it.
poll_arm :: (scheduler: *Scheduler, engine: *Engine, fd: s64) {
    state := *engine.fds[fd];

    if engine.backend == .KQUEUE {
        if state.read_head && !state.read_armed {
            kqueue_change(engine, fd, KQ_READ);
            state.read_armed = true;
        }

        if state.write_head && !state.write_armed {
            kqueue_change(engine, fd, KQ_WRITE);
            state.write_armed = true;
        }
    } else {
        epoll_arm(scheduler, engine, fd);
    }
}

// ---------------------------------------------------------------------------------------------
// kqueue

Kevent :: struct {
    ident: u64;
    filter: s16;
    flags: u16;
    fflags: u32;
    data: s64;
    udata: *void;
}

Timespec :: struct {
    seconds: s64;
    nanoseconds: s64;
}

KQ_READ :: -1;
KQ_WRITE :: -2;

#if OS == .MACOS {
    EV_ADD :: 0x1;
    EV_ONESHOT :: 0x10;
    EV_ERROR :: 0x4000;

    c_kqueue :: () -> s32 #foreign libc "kqueue";
    c_kevent :: (kq: s32, changes: *Kevent, change_count: s32, events: *Kevent, event_count: s32, timeout: *Timespec) -> s32 #foreign libc "kevent";

    kqueue_start :: (engine: *Engine) {
        engine.backend = .KQUEUE;
        engine.poll_fd = c_kqueue();
        assert(engine.poll_fd >= 0, "Io: could not create a kqueue");
        c_fcntl(engine.poll_fd, F_SETFD, cast(s32) FD_CLOEXEC);
    }

    kqueue_change :: (engine: *Engine, fd: s64, filter: s16) {
        change: Kevent;
        change.ident = cast(u64) fd;
        change.filter = filter;
        change.flags = EV_ADD | EV_ONESHOT;

        array_add(*engine.changes, change);
    }

    // Hands the changes queued so far to the kernel.
    kqueue_flush :: (engine: *Engine) {
        if engine.changes.count == 0 return;

        c_kevent(engine.poll_fd, engine.changes.data, cast(s32) engine.changes.count, null, 0, null);
        engine.changes.count = 0;
    }

    kqueue_wait_events :: (scheduler: *Scheduler, engine: *Engine, timeout: s64) {
        events: [64] Kevent;
        limit: Timespec;
        limit_pointer: *Timespec;

        if timeout >= 0 {
            limit.seconds = timeout / 1_000_000_000;
            limit.nanoseconds = timeout % 1_000_000_000;
            limit_pointer = *limit;
        }

        // The queued changes and the wait go in one call. A change that fails comes back as an
        // event with EV_ERROR.
        count := c_kevent(engine.poll_fd, engine.changes.data, cast(s32) engine.changes.count, events.data, 64, limit_pointer);

        engine.changes.count = 0;

        if count < 0 {
            assert(errno_now() == E_INTR, "Io: kevent failed");

            return;
        }

        for 0..count - 1 {
            event := *events[it];
            fd := cast(s64) event.ident;

            if event.filter == EVFILT_USER {
                pool_collect(scheduler, engine);

                continue;
            }

            if fd >= engine.fds.count continue;

            if event.flags & EV_ERROR {
                if event.data != 0 poll_fail(scheduler, engine, fd, -event.data, event.filter == KQ_READ, event.filter == KQ_WRITE);

                continue;
            }

            if event.filter == KQ_READ {
                engine.fds[fd].read_armed = false;
                poll_ready(scheduler, engine, fd, true, false);
            } else if event.filter == KQ_WRITE {
                engine.fds[fd].write_armed = false;
                poll_ready(scheduler, engine, fd, false, true);
            }
        }
    }
} else {
    kqueue_start :: (engine: *Engine) {}
    kqueue_change :: (engine: *Engine, fd: s64, filter: s16) {}
    kqueue_flush :: (engine: *Engine) {}
    kqueue_wait_events :: (scheduler: *Scheduler, engine: *Engine, timeout: s64) {}
}

// ---------------------------------------------------------------------------------------------
// epoll

// The kernel's `struct epoll_event` is packed on x86-64 (12 bytes) and naturally aligned
// elsewhere (16 bytes); the 64-bit user data is used as the descriptor.
#if CPU == .X64 {
    Epoll_Event :: struct {
        events: u32;
        fd: u32;
        extra: u32;
    }
} else {
    Epoll_Event :: struct {
        events: u32;
        padding: u32;
        fd: u32;
        extra: u32;
    }
}

#if OS == .LINUX {
    EPOLLIN :: 0x1;
    EPOLLOUT :: 0x4;
    EPOLLERR :: 0x8;
    EPOLLHUP :: 0x10;
    EPOLLONESHOT :: 0x4000_0000;
    EPOLL_CTL_ADD :: 1;
    EPOLL_CTL_DEL :: 2;
    EPOLL_CTL_MOD :: 3;
    EPOLL_CLOEXEC :: 0x80000;

    c_epoll_create1 :: (flags: s32) -> s32 #foreign libc "epoll_create1";
    c_epoll_ctl :: (epfd: s32, operation: s32, fd: s32, event: *Epoll_Event) -> s32 #foreign libc "epoll_ctl";
    c_epoll_wait :: (epfd: s32, events: *Epoll_Event, max_events: s32, milliseconds: s32) -> s32 #foreign libc "epoll_wait";

    // Whether epoll can be had at all; it always can on a Linux this module supports.
    epoll_start :: (engine: *Engine) {
        engine.backend = .EPOLL;
        engine.poll_fd = c_epoll_create1(EPOLL_CLOEXEC);
        assert(engine.poll_fd >= 0, "Io: could not create an epoll descriptor");
    }

    epoll_remove :: (engine: *Engine, fd: s64) {
        event: Epoll_Event;
        c_epoll_ctl(engine.poll_fd, EPOLL_CTL_DEL, cast(s32) fd, *event);
    }

    // The registration is one-shot for the whole descriptor, so it is rewritten whenever the set of
    // waiters differs from what is registered, or it has fired.
    epoll_arm :: (scheduler: *Scheduler, engine: *Engine, fd: s64) {
        state := *engine.fds[fd];

        wanted: u32;

        if state.read_head wanted |= EPOLLIN;
        if state.write_head wanted |= EPOLLOUT;

        if wanted == 0 return;
        if state.live && state.mask == wanted return;

        event: Epoll_Event;
        event.events = wanted | EPOLLONESHOT;
        event.fd = cast(u32) fd;

        operation := ifx state.registered then EPOLL_CTL_MOD else EPOLL_CTL_ADD;
        result := c_epoll_ctl(engine.poll_fd, cast(s32) operation, cast(s32) fd, *event);

        // Our bookkeeping and the kernel's disagree (the descriptor number was reused): do the other.
        if result < 0 && operation == EPOLL_CTL_MOD && errno_now() == E_NOENT {
            result = c_epoll_ctl(engine.poll_fd, EPOLL_CTL_ADD, cast(s32) fd, *event);
        } else if result < 0 && operation == EPOLL_CTL_ADD && errno_now() == E_EXIST {
            result = c_epoll_ctl(engine.poll_fd, EPOLL_CTL_MOD, cast(s32) fd, *event);
        }

        if result < 0 {
            poll_fail(scheduler, engine, fd, -cast(s64) errno_now(), true, true);

            return;
        }

        state.registered = true;
        state.live = true;
        state.mask = wanted;
    }

    epoll_wait_events :: (scheduler: *Scheduler, engine: *Engine, timeout: s64) {
        events: [64] Epoll_Event;

        milliseconds: s32 = -1;

        if timeout >= 0 milliseconds = cast(s32) min((timeout + 999_999) / 1_000_000, 0x7fff_ffff);

        count := c_epoll_wait(engine.poll_fd, events.data, 64, milliseconds);
        if count < 0 {
            assert(errno_now() == E_INTR, "Io: epoll_wait failed");

            return;
        }

        for 0..count - 1 {
            event := *events[it];
            fd := cast(s64) event.fd;

            if fd >= engine.fds.count continue;

            engine.fds[fd].live = false;

            problem := (event.events & (EPOLLERR | EPOLLHUP)) != 0;

            poll_ready(scheduler, engine, fd, problem || (event.events & EPOLLIN) != 0, problem || (event.events & EPOLLOUT) != 0);
        }
    }
} else {
    epoll_start :: (engine: *Engine) {}
    epoll_remove :: (engine: *Engine, fd: s64) {}
    epoll_arm :: (scheduler: *Scheduler, engine: *Engine, fd: s64) {}
    epoll_wait_events :: (scheduler: *Scheduler, engine: *Engine, timeout: s64) {}
}
