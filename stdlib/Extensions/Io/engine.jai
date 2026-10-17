// The completion engine: what the fiber calls underneath (`read`, `accept`, ...) turn into. Each
// scheduler has one. A fiber describes an operation (`Op`), hands it to `engine_submit`, and parks;
// the engine finishes it, now or later, and `finish_op` wakes the fiber. Nothing above this file
// knows which backend runs: io_uring (Linux), epoll (Linux), kqueue (macOS), or completion ports
// with an optional IoRing for files (Windows), picked at startup by `engine_start`.
//
// Rules the backends keep:
//   - A handle is an opaque descriptor plus a kind tag.
//   - The buffer and address of an operation belong to it until its completion arrives, even after
//     a cancel. A canceled operation is never abandoned: the fiber stays parked until the engine
//     reports it.
//   - A missing kernel feature is never an error: capabilities are probed and the operation
//     degrades (io_uring falls back to epoll as a whole when the ring cannot be had).
//   - Results are a byte count or the negative platform errno; `error_from_code` makes them portable.

Handle_Kind :: enum u8 {
    NONE;
    SOCKET;
    // A regular file: read and written at explicit offsets.
    FILE;
    // Anything else that has a descriptor and waits for readiness: pipes, terminals.
    STREAM;
}

// An open descriptor, as the engine sees it. `fd` is a file descriptor on POSIX and a SOCKET or
// HANDLE on Windows.
Handle :: struct {
    fd: s64 = -1;
    kind: Handle_Kind;
}

Op_Kind :: enum u8 {
    ACCEPT;
    CONNECT;
    RECV;
    SEND;
    // Files (at `offset`) and streams (at the current position).
    READ;
    WRITE;
    CLOSE;
    // Completes (with 0) when a `RECV` would not wait: bytes, the end of the stream or an error
    // are there. Takes no buffer and consumes nothing. Sockets only.
    WAIT_READABLE;
}

Op_State :: enum u8 {
    FREE;
    // Described but not yet submitted to the kernel (io_uring), or waiting for readiness (epoll,
    // kqueue).
    PENDING;
    DONE;
}

// The Win32 OVERLAPPED structure. It is the first member of `Op`, so the address the completion
// port hands back is the address of the operation.
Overlapped :: struct {
    internal: u64;
    internal_high: u64;
    offset: u32;
    offset_high: u32;
    event: s64;
}

// One operation, from submission to completion.
Op :: struct {
    // Windows completion ports: must stay the first member.
    overlapped: Overlapped;
    kind: Op_Kind;
    state: Op_State;
    handle: Handle;
    buffer: *void;
    length: s64;
    // File position; negative: the current one.
    offset: s64;
    // The peer, in and out (`connect`, `accept`).
    address: Sock_Storage;
    address_length: u32;
    // How far a multi-step operation got (connect: 0 not started, 1 waiting for the result).
    phase: u8;
    // Which readiness this operation waits for.
    wants_write: bool;
    fiber: *Fiber;
    // Not a fiber's: reported to `internal_completion`.
    internal: bool;
    canceling: bool;
    // Why `cancel_op` canceled it: `.CANCELED`, `.TIMED_OUT`; `.NONE` if the handle was closed.
    cancel_cause: Io_Error;
    // Windows: the socket `accept` made ahead of time (AcceptEx).
    aux: s64;
    // A file thread pool worker or an IoRing owns it; it runs to the end and cannot be interrupted.
    uncancelable: bool;
    // Which internal user an `internal` operation belongs to.
    tag: Internal_Tag;
    // The scheduler's end of the file thread pool that a worker reports to.
    link: *Pool_Link;
    // io_uring: the listener whose multishot accept this is.
    slot: *Accept_Slot;

    // The completion.
    result: s64;
    flags: u32;
    // More completions follow (multishot operations: the io_uring accept).
    more: bool;
    buffer_id: s32 = -1;

    // The free list, or the list of operations waiting on a descriptor.
    next: *Op;
    previous: *Op;
}

// What an internal operation (one that belongs to the engine, not to a fiber) is for.
Internal_Tag :: enum u8 {
    // The pipe that resolver threads write to.
    RESOLVER_WAKE;
    // The eventfd that file pool workers write to (epoll).
    POOL_WAKE;
}

Engine :: struct {
    allocator: Allocator;
    backend: Backend;
    caps: Capabilities;
    // Why this is not the backend that was asked for or would have been first choice. Empty if it is.
    fallback_reason: string;

    free_ops: *Op;

    // epoll and kqueue.
    poll_fd: s32 = -1;
    fds: [..] Fd_State;
    changes: [..] Kevent;

    uring: *Uring_State;
    iocp: *Iocp_State;

    // The file thread pool (epoll, kqueue): reads and writes longer than `file_inline` bytes go to
    // `file_threads` workers; 0 threads means everything runs inline.
    file_inline: s64;
    file_threads: s64;
    pool: *Pool_Link;
}

// What the poll backends remember per descriptor.
Fd_State :: struct {
    read_head: *Op;
    read_tail: *Op;
    write_head: *Op;
    write_tail: *Op;
    // kqueue: a filter is registered and has not fired.
    read_armed: bool;
    write_armed: bool;
    // epoll: registered at all (edge-triggered, for good), and the events it asks for.
    registered: bool;
    mask: u32;
    // epoll: the last try in that direction found nothing to do, and no event has said otherwise
    // since, so the next operation waits without trying.
    read_blocked: bool;
    write_blocked: bool;
    // The peer has shut down or the socket has an error: receives always try, since the end of the
    // stream is not an edge that comes again.
    hup: bool;
}

// ---------------------------------------------------------------------------------------------
// Starting and stopping

engine_start :: (scheduler: *Scheduler) {
    engine := New(Engine);
    engine.allocator = context.allocator;

    scheduler.engine = engine;

    request := requested_backend();

    #if OS == .LINUX {
        if request == .NONE || request == .IO_URING {
            engine.fallback_reason = uring_start(engine);
            if engine.backend == .IO_URING return;
        } else if request == .KQUEUE {
            engine.fallback_reason = "JAI_IO_BACKEND=kqueue is not available on Linux";
        }

        epoll_start(engine);
        pool_configure(engine);
    } else #if OS == .MACOS {
        if request == .IO_URING || request == .EPOLL engine.fallback_reason = "JAI_IO_BACKEND names a backend macOS does not have";

        kqueue_start(engine);
        pool_configure(engine);
    } else #if OS == .WINDOWS {
        iocp_start(engine, request);
    } else {
        engine.backend = .NONE;
        engine.fallback_reason = "IO is not implemented on this system yet";
    }
}

engine_stop :: (scheduler: *Scheduler) {
    engine := scheduler.engine;
    if !engine return;

    // Lookups still under way finish first: they need the engine to be woken.
    wake_stop(scheduler);

    pool_detach(scheduler, engine);

    if engine.backend == .IO_URING uring_stop(engine);
    else if engine.backend == .IOCP || engine.backend == .IORING iocp_stop(engine);
    else if engine.poll_fd >= 0 sys_close(engine.poll_fd);

    array_reset(*engine.fds);
    array_reset(*engine.changes);

    op := engine.free_ops;
    while op {
        next := op.next;
        release_owned(op, engine.allocator);
        op = next;
    }

    release_owned(engine, engine.allocator);
    scheduler.engine = null;
}

// The backend a user asked for with JAI_IO_BACKEND, or `.NONE` if it is unset or unknown.
requested_backend :: () -> Backend {
    scratch: [256] u8;
    name := env_text("JAI_IO_BACKEND", *scratch);

    if name == "io_uring" return .IO_URING;
    if name == "epoll" return .EPOLL;
    if name == "kqueue" return .KQUEUE;
    if name == "iocp" return .IOCP;
    if name == "ioring" return .IORING;

    return .NONE;
}

#if OS != .WINDOWS {
    c_getenv :: (name: *u8) -> *u8 #foreign libc "getenv";
}

// ---------------------------------------------------------------------------------------------
// Operations

new_op :: (engine: *Engine, kind: Op_Kind, handle: Handle) -> *Op {
    op := engine.free_ops;
    if op engine.free_ops = op.next;
    else op = allocate_owned(Op, engine.allocator);

    op.* = .{};
    op.kind = kind;
    op.handle = handle;
    op.state = .PENDING;
    op.offset = -1;

    return op;
}

free_op :: (engine: *Engine, op: *Op) {
    op.state = .FREE;
    op.next = engine.free_ops;
    engine.free_ops = op;
}

// Starts `op`. Returns true if it already finished (its result is set, nothing will be reported
// later), false if the engine will report it through `finish_op`.
engine_submit :: (scheduler: *Scheduler, op: *Op) -> bool {
    engine := scheduler.engine;

    if engine.backend == {
        case .IO_URING;
            uring_submit(scheduler, engine, op);
            scheduler.pending += 1;

            return false;

        case .EPOLL; #through;
        case .KQUEUE;
            if poll_submit(scheduler, engine, op) return true;

            scheduler.pending += 1;

            return false;

        case .IOCP; #through;
        case .IORING;
            if iocp_submit(scheduler, engine, op) return true;

            scheduler.pending += 1;

            return false;
    }

    op.result = -E_UNSUPPORTED;
    op.state = .DONE;

    return true;
}

// Asks the engine to end `op` early. The operation still completes, with the system's cancel
// error as its result, and only then does its fiber wake.
engine_cancel :: (scheduler: *Scheduler, op: *Op) {
    engine := scheduler.engine;

    if op.canceling || op.state != .PENDING return;

    op.canceling = true;

    if engine.backend == .IO_URING uring_cancel(scheduler, engine, op);
    else if engine.backend == .IOCP || engine.backend == .IORING iocp_cancel(scheduler, engine, op);
    else poll_cancel(scheduler, engine, op);
}

// Submits what was queued and waits up to `timeout` nanoseconds (forever if negative, not at all
// if 0) for completions, which wake their fibers.
engine_wait :: (scheduler: *Scheduler, timeout: s64) {
    engine := scheduler.engine;

    if engine.backend == {
        case .IO_URING; uring_wait(scheduler, engine, timeout);
        case .EPOLL; epoll_wait_events(scheduler, engine, timeout);
        case .KQUEUE; kqueue_wait_events(scheduler, engine, timeout);
        case .IOCP; #through;
        case .IORING; iocp_wait(scheduler, engine, timeout);
        case; if timeout > 0 sleep_milliseconds(cast(s32) min((timeout + 999_999) / 1_000_000, 0x7fff_ffff));
    }
}

// The engine reports `op` as complete.
finish_op :: (scheduler: *Scheduler, op: *Op, result: s64, flags: u32 = 0, more := false) {
    op.result = result;
    op.flags = flags;
    op.more = more;
    op.state = .DONE;

    if !more scheduler.pending -= 1;

    if op.internal {
        internal_completion(scheduler, op);

        return;
    }

    wake(op.fiber);
}

// Cancels the engine operation `fiber` is blocked in. Its fiber wakes when the operation completes.
cancel_op :: (fiber: *Fiber, cause: Io_Error) {
    op := fiber.op;
    if op.cancel_cause == .NONE op.cancel_cause = cause;

    engine_cancel(fiber.scheduler, op);
}

// ---------------------------------------------------------------------------------------------
// From a fiber

// Describes an operation for the running fiber.
begin_op :: (kind: Op_Kind, handle: Handle) -> *Op {
    scheduler := current_scheduler();

    return new_op(scheduler.engine, kind, handle);
}

end_op :: (op: *Op) {
    scheduler := current_scheduler();

    free_op(scheduler.engine, op);
}

// Runs `op` for the running fiber and blocks it until the operation completes, is canceled, or
// the fiber's deadline passes. The result stays in `op.result`.
await_op :: (op: *Op) -> Io_Error {
    fiber, scheduler := current_fiber();

    fiber.platform_code = 0;
    if fiber.canceled return .CANCELED;
    if fiber.deadline != 0 && now_nanoseconds() >= fiber.deadline return .TIMED_OUT;

    op.fiber = fiber;

    if !engine_submit(scheduler, op) {
        fiber.op = op;
        if fiber.deadline != 0 timer_add(scheduler, fiber, fiber.deadline);

        park(scheduler, fiber);

        fiber.op = null;
    }

    if op.result >= 0 return .NONE;

    code := -op.result;

    fiber.platform_code = cast(s32) code;

    if code == E_CANCELED {
        if op.cancel_cause != .NONE return op.cancel_cause;
        if fiber.canceled return .CANCELED;

        return .CLOSED;
    }

    return error_from_code(code);
}

// A portable error for a positive platform error number.
error_from_code :: (code: s64) -> Io_Error {
    if code == {
        case 0; return .NONE;
        case E_CONNRESET; return .CONNECTION_RESET;
        case E_CONNREFUSED; return .CONNECTION_REFUSED;
        case E_CONNABORTED; return .CONNECTION_ABORTED;
        case E_PIPE; return .BROKEN_PIPE;
        case E_NOTCONN; return .NOT_CONNECTED;
        case E_ADDRINUSE; return .ADDRESS_IN_USE;
        case E_ADDRNOTAVAIL; return .ADDRESS_UNAVAILABLE;
        case E_NETUNREACH; return .NETWORK_UNREACHABLE;
        case E_HOSTUNREACH; return .HOST_UNREACHABLE;
        case E_TIMEDOUT; return .TIMED_OUT;
        case E_NOENT; return .NOT_FOUND;
        case E_EXIST; return .ALREADY_EXISTS;
        case E_ACCES; return .PERMISSION_DENIED;
        case E_PERM; return .PERMISSION_DENIED;
        case E_INVAL; return .INVALID;
        case E_NOSPC; return .NO_SPACE;
        case E_MFILE; return .TOO_MANY_OPEN_FILES;
        case E_ISDIR; return .IS_A_DIRECTORY;
        case E_CANCELED; return .CANCELED;
        case E_UNSUPPORTED; return .UNSUPPORTED;
        case E_NOSYS; return .UNSUPPORTED;
    }

    #if OS == .WINDOWS return windows_error(code);
    else return .OTHER;
}

// The error of a plain system call that returned `result` (negative: -errno), for the running
// fiber, and remembers the code for `platform_code`.
error_from_result :: (result: s64) -> Io_Error {
    if result >= 0 return .NONE;

    fiber := current_fiber();
    fiber.platform_code = cast(s32) -result;

    return error_from_code(-result);
}

// ---------------------------------------------------------------------------------------------
// Completions that belong to the engine itself rather than a fiber: the wake-up pipe resolver
// threads write to (resolver.jai) and the eventfd of the file pool (files_pool.jai).

internal_completion :: (scheduler: *Scheduler, op: *Op) {
    if op.tag == .POOL_WAKE pool_wake_completed(scheduler, op);
    else wake_completed(scheduler, op);
}
