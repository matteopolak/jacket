// Deadlines, which backend is running, and one scheduler per thread.

// The one way to bound a wait. Limits every IO call (and `sleep`) in `body` to `milliseconds`
// from now: one that is still waiting then returns `.TIMED_OUT`, and so does a call made after the
// limit, even when data is ready. `0` is a limit that has already passed. Deadlines nest; the
// earlier one wins, and the previous deadline is back when `body` ends. A negative `milliseconds`
// adds no limit (`NO_DEADLINE`), so a timeout setting that may be off needs no second code path.
// Channel and lock operations are not IO and ignore it.
//
//     with_deadline(2000, #code {
//         count, error = read(conn, buffer);       // .TIMED_OUT after two seconds without data
//     });
with_deadline :: (milliseconds: s64, body: Code) #expand {
    previous_deadline := push_deadline(milliseconds);
    defer pop_deadline(previous_deadline);

    #insert body;
}

// A `with_deadline` argument that adds no limit.
NO_DEADLINE :: -1;

Backend :: enum u8 {
    // No IO on this system: every operation fails with `.UNSUPPORTED`.
    NONE;
    IO_URING;
    EPOLL;
    KQUEUE;
    // Windows: completion ports for sockets and files.
    IOCP;
    // Windows 11 22H2 and newer: completion ports for sockets, an IoRing for files.
    IORING;
}

name_of_backend :: (backend: Backend) -> string {
    if backend == {
        case .IO_URING; return "io_uring";
        case .EPOLL; return "epoll";
        case .KQUEUE; return "kqueue";
        case .IOCP; return "iocp";
        case .IORING; return "ioring";
    }

    return "none";
}

// What the engine found it could use. Backends fill in what applies.
Capabilities :: struct {
    // io_uring: the setup flags the kernel accepted.
    single_issuer: bool;
    defer_taskrun: bool;
    coop_taskrun: bool;
    // io_uring: waits with a timeout in one call (Linux 5.11).
    ext_arg: bool;
    // io_uring: how many operations the kernel's probe lists.
    kernel_ops: s64;
}

// The backend the running scheduler uses: `.IO_URING`, `.EPOLL`, `.KQUEUE`, `.IOCP`, `.IORING`, or
// `.NONE` where IO is not implemented. Set `JAI_IO_BACKEND` to `io_uring`, `epoll`, `kqueue`,
// `iocp` or `ioring` before `io_run` to ask for one; a backend the system cannot offer is
// replaced (see `backend_reason`).
backend :: () -> Backend {
    scheduler := current_scheduler();

    return scheduler.engine.backend;
}

// `backend()` as text: "io_uring", "epoll", "kqueue", "iocp", "ioring" or "none".
backend_name :: () -> string {
    return name_of_backend(backend());
}

// Why the running scheduler does not use the backend that was asked for or is the first choice on
// this system (for example "io_uring is disabled here"). Empty when it does.
backend_reason :: () -> string {
    scheduler := current_scheduler();

    return scheduler.engine.fallback_reason;
}

// What the engine found it could use.
capabilities :: () -> Capabilities {
    scheduler := current_scheduler();

    return scheduler.engine.caps;
}

// The number of processors, for sizing `io_run_threads`.
processor_count :: () -> s64 {
    return sys_processors();
}

// Starts `count` schedulers, each on a thread of its own, runs `proc` as the first fiber of every
// one, and returns when all have finished. Fibers never move between them. With `reuse_port`
// listeners (`tcp_listen(..., reuse_port = true)`) each thread accepts its own connections:
//
//     io_run_threads(4, () {
//         listener, error := tcp_listen(":8080", reuse_port = true);
//         while true { conn := accept(listener); spawn(handle, conn); }
//     });
//
// `count` 0 means one per processor. Call it from outside any `io_run`.
io_run_threads :: (count: s64, proc: $P) {
    Block :: struct {
        proc: P;
    }

    block: Block;
    block.proc = proc;

    entry :: (thread: *Thread.Thread) -> s64 {
        block := cast(*Block) thread.data;
        io_run(block.proc);

        return 0;
    }

    run_threads(count, entry, *block);
}

io_run_threads :: (count: s64, proc: $P, data: $T) {
    Block :: struct {
        proc: P;
        data: T;
    }

    block: Block;
    block.proc = proc;
    block.data = data;

    entry :: (thread: *Thread.Thread) -> s64 {
        block := cast(*Block) thread.data;
        io_run(block.proc, block.data);

        return 0;
    }

    run_threads(count, entry, *block);
}

#scope_module

// Starts a deadline for the running fiber and returns what `pop_deadline` is given when the
// guarded work is over.
push_deadline :: (milliseconds: s64) -> s64 {
    fiber := current_fiber();
    previous := fiber.deadline;

    if milliseconds < 0 return previous;

    limit := now_nanoseconds() + milliseconds * 1_000_000;

    if previous == 0 || limit < previous fiber.deadline = limit;

    return previous;
}

pop_deadline :: (previous: s64) {
    fiber := current_fiber();

    fiber.deadline = previous;
}

run_threads :: (count: s64, entry: Thread.Native_Thread_Proc, data: *void) {
    count = ifx count <= 0 then processor_count() else count;

    threads := NewArray(count, Thread.Thread);

    defer free(threads.data);

    started := 0;

    for 0..count - 1 {
        thread := *threads[it];

        assert(Thread.thread_init(thread, entry), "Io: could not prepare a thread");
        thread.data = data;

        assert(Thread.thread_start(thread), "Io: could not start a thread");
        started += 1;
    }

    for 0..started - 1 Thread.thread_deinit(*threads[it]);
}

// ---------------------------------------------------------------------------------------------
// Per-scheduler storage for libraries

#scope_export

// What `set_scheduler_local` is given to release a value when its scheduler ends. It runs on the
// scheduler's thread after every fiber has ended, so it must not block: close sockets with
// `close(stream, now = true)`.
Local_Drop :: #type (value: *void);

Local_Entry :: struct {
    key: *void;
    value: *void;
    drop: Local_Drop;
}

// The value a library stored for `key` on the running scheduler, or null. Keys are addresses of
// something the library owns (a global works). This is the per-thread variable the language does
// not have: the Http client keeps its connection pool here.
scheduler_local :: (key: *void) -> *void {
    scheduler := current_scheduler();

    for scheduler.locals if it.key == key return it.value;

    return null;
}

// Stores `value` for `key` on the running scheduler; `drop` (if not null) gets it when the
// scheduler ends. Replaces an earlier value for the key without dropping it.
set_scheduler_local :: (key: *void, value: *void, drop: Local_Drop) {
    scheduler := current_scheduler();

    for *scheduler.locals {
        if it.key == key {
            it.value = value;
            it.drop = drop;

            return;
        }
    }

    array_add(*scheduler.locals, Local_Entry.{ key, value, drop });
}

#scope_module

drop_locals :: (scheduler: *Scheduler) {
    previous := context.io_scheduler;

    context.io_scheduler = scheduler;

    defer context.io_scheduler = previous;

    while scheduler.locals.count > 0 {
        entry := scheduler.locals[scheduler.locals.count - 1];
        scheduler.locals.count -= 1;

        if entry.drop entry.drop(entry.value);
    }
}
