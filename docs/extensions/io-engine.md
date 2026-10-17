# Io: fibers, streams and the completion engine (`Extensions/Io`)

Shared rules (pointers vs values, `close(x, now)`, errors, who frees, milliseconds): [README](README.md#conventions-of-io-tls-http-and-json).

## What it is

Cooperative fibers on a per-thread scheduler (`io_run`, `spawn`, `sleep`, `cancel`, `Channel(T)`, `Wait_Group`, `Fiber_Mutex`), plus blocking-style TCP, files, name lookups, deadlines and cancellation on top of an internal completion engine: io_uring and epoll (Linux), kqueue (macOS), completion ports with an optional IoRing for files (Windows). A common `Reader`/`Writer` covers files, TCP and pipes. Built on the [fiber intrinsics](../compiler/interpreter.md); a compile error on wasm.

```jai
Io :: #import "Extensions/Io";

handle :: (conn: Io.Tcp_Stream) {
    defer Io.close(conn);
    buffer: [4096] u8;
    while true {
        count, error := Io.read(conn, buffer);
        if error != .NONE || count == 0 return;       // 0 is the peer closing
        Io.write(conn, .{ count, buffer.data });       // sends all of it
    }
}

main :: () {
    Io.io_run(() {
        listener, error := Io.tcp_listen(":8080");
        while true {
            conn, error := Io.accept(listener);
            if error != .NONE break;
            Io.spawn(handle, conn);
        }
    });
}
```

Everything that waits must run on a fiber inside `io_run`; it blocks only that fiber and returns its result plus an `Io_Error`.

## Fibers

- Each fiber has an `mmap`/`VirtualAlloc` stack with a guard page. The scheduler lives in `context.io_scheduler` (no thread-locals) and runs on the OS thread's own stack. `io_run(proc)` returns when the root fiber returns, canceling and draining the rest. Fibers never move between threads; a `Channel` or `Wait_Group` stays on one thread.
- Ready queue plus a timer heap. With nothing ready the thread waits in the engine until the next timer; nothing ready, no timers and nothing pending prints a deadlock message and exits 1.
- Blocking calls return `Io_Error`: `.CANCELED` after `cancel(handle)` (sticky for that fiber), `.CLOSED` for a closed channel. `Fiber_Handle` has a generation, so a stale handle is ignored.
- Each fiber gets a copy of the spawner's `context` with its own temporary storage. `in_scheduler()` tells code whether it runs on a fiber (the HTTP client starts its own scheduler when not).
- `make_pipe()` returns two connected `*Pipe_Stream` ends (in-memory duplex, shaped like TCP). It lets the [HTTP test client](http-server.md#testing) run a server connection without a socket. Close each end once; the pair frees itself after the second.
- Stack overflow (POSIX) is caught by a SIGSEGV handler on an alternate stack and printed as `fatal: fiber stack overflow` (exit 139).
- `io_run_threads(n, proc)` runs one scheduler per thread. Listen with `reuse_port = true` in each; Linux balances by hash, macOS delivers to some listener.

## Readers, writers, deadlines

`Reader` is `{ data, read_proc, close_proc, wait_proc }` and `Writer` is `{ data, write_proc, close_proc }`, a vtable so users can add sources (a TLS stream is one). Built-ins come from `reader_of(x)`/`writer_of(x)` for `*Io_File`, `Tcp_Stream`, `*Pipe_Stream`. Operations: `read` (0 is the end), `read_full` (`.END_OF_FILE` with the bytes read), `write` (all of it), `write_some`, `close`, and `wait_readable` (returns when a `read` would not wait, consumes nothing, `.UNSUPPORTED` without a `wait_proc`; the HTTP server uses it to keep idle connections without a buffer). `read_all`, `read_file`, `write_file`, and `Io.read_json(reader, T)` (buffers the whole document, then `from_json`; Json takes a `Json_Source` because it must build for wasm) sit on top. A reader and writer over one TCP stream or pipe end share the connection: close only one. A file's reader and writer share its position.

```jai
file, error := Io.open_file("log.txt", .READ);
defer Io.close(*file);
source := Io.reader_of(*file);
rest, error = Io.read_all(source);
Io.with_deadline(2000, #code { count, error = Io.read(source, buffer); });   // .TIMED_OUT
```

`with_deadline` limits every IO call and `sleep` in the block; the earlier deadline wins when nested; an expired deadline fails a call before it starts, even with data waiting (pipes too). jaic has no trailing-block call syntax, so the body is a `#code` argument: assign to variables declared before it and do not `return` inside. Channel and lock operations ignore deadlines. `cancel(fiber)` ends the fiber's current IO with `.CANCELED` and all later IO fails the same way; the buffer is never written after the call returns.

`connect(string)` takes `"host:port"`, an address literal or `"[::1]:80"` and resolves names. `resolve` runs `getaddrinfo` on helper threads (1 to `RESOLVER_MAX_THREADS` = 8, joined when `io_run` ends); lookups of the same name share one job; a cancel or deadline only takes the fiber off the job.

Files: io_uring and Windows do the IO in the kernel; epoll and kqueue cannot report a regular file as not ready, so reads/writes over 32 KiB go to a file thread pool and shorter ones run inline. `.APPEND` always writes at the end.

## The engine

A fiber fills an `Op` (kind, `Handle`, buffer, offset, address), calls `engine_submit` and parks; the engine completes it inline (submit returns true) or later via `finish_op`, which wakes the fiber. Rule: buffers and addresses belong to an operation until its completion arrives, even after a cancel; a missing feature is a fallback, never an error. Deadlines are timers that cancel the operation with cause `TIMED_OUT`.

- **io_uring** (Linux 6.1 floor, on [Io_Uring](io-uring.md)): one 512-entry ring per scheduler thread with `SINGLE_ISSUER | DEFER_TASKRUN | COOP_TASKRUN`, retried with fewer flags on `EINVAL`. A denial (`ENOSYS`, `EPERM`, `EACCES`, `ENOMEM`: old kernels, seccomp, `kernel.io_uring_disabled`) falls back to epoll and records why in `backend_reason()`. `accept` is one multishot request per listener with a queue of at most 128 sockets; a cancel or close tears it down.
- **epoll / kqueue** (`engine_poll.jai`): try the operation at once; on `EAGAIN` queue it on the descriptor (`Fd_State`). epoll is edge-triggered and registrations only grow; `Fd_State` remembers per direction that the last try found nothing (`read_blocked`, `write_blocked`), so a keep-alive request costs no `epoll_ctl`. After hang-up or `recv` of 0, reads always try.
- **Windows** (x64, arm64): one completion port per thread, overlapped `AcceptEx`/`ConnectEx`/`WSARecv`/`WSASend`/`ReadFile`. IoRing (Windows 11 22H2+) is for files only, probed with a self test, and falls back to the port with a reason. Windows has no `SO_REUSEPORT`: plain listeners use `SO_EXCLUSIVEADDRUSE`, `reuse_port` uses `SO_REUSEADDR` (shares the port, does not balance).
- **Helper threads** (resolver, file pool) wake the scheduler through a pipe/`eventfd`/`EVFILT_USER`/completion packet; `engine_stop` waits for outstanding jobs first.

### Who may wake a fiber

`wake` asserts the target is `BLOCKED`, so a caller must know the fiber is parked and nothing else will wake that park. A new caller must answer: can the target be the running fiber, can it be woken twice for one park (completion plus cancel or timer), can it be `READY`. Waiters are popped from their list (or the op taken off its queue) before the wake; `fire_timers` removes the timer before it acts; `wake` removes a leftover timer. Regressions: `tests/pipe.jai` (`test_write_between_deadline_and_wakeup`), `tests/backends.jai`, `test_cancel_resolve`.

## How to change it

- **New operation:** add an `Op_Kind` (`WAIT_READABLE` is the smallest example: `sys_peek` in `platform.jai`, `poll_attempt`, `uring_submit`, `iocp_submit`, `wait_readable` in `net.jai`), then a public procedure in `net.jai`/`files.jai` that fills an `Op` and calls `await_op`.
- **New backend:** a `Backend` member, `*_start`, `engine_submit`/`engine_cancel`/`engine_wait` cases, `finish_op` per completion.
- **New io_uring feature:** extend `uring_submit`/`uring_reap` (`more`, `buffer_id` already travel), probe in `uring_start`, set a `Capabilities` flag; callers never see it.
- **New error:** add the `E_*` number for both OSes in `platform.jai` and a case in `error_from_code`; append the `Io_Error` member at the end (the enum only grows). `last_platform_code()` keeps `OTHER` diagnosable.
- **New reader/writer source:** write the procedures with the `Read_Proc`/`Write_Proc`/`Close_Proc` signatures and add `reader_of`/`writer_of` overloads in `stream.jai`; also give the source direct `read`, `read_full`, `write`, `write_some`, `close`. A pipe-like source checks the deadline itself (`deadline_passed` in `pipe.jai`) since it has no engine.
- **New blocking primitive:** follow `wait_on` in `sync.jai`: stack-allocated `Waiter` on a list, then `park`.
- Files: `scheduler.jai`, `sync.jai`, `stacks.jai`, `overflow.jai`, `pipe.jai`, `stream.jai`, `net.jai`, `files.jai`, `files_pool.jai`, `resolver.jai`, `runtime.jai`, `engine*.jai`, `platform.jai`, `win32.jai`.
- Gotchas: `open`/`fcntl` are variadic in C and must be declared `args: ..Any` (Apple arm64 passes variadics on the stack). `epoll_event` is packed on x86-64 only. The Fd table is indexed by descriptor, so any raw `close` of a descriptor that went through a poll backend must use `discard_handle`. A parked fiber keeps the stack pages it touched (about 12 KB of the 16 KB an idle server connection costs); releasing them was tried and dropped because the release call runs on the same stack.

## Configuration

| Setting | Effect |
| --- | --- |
| `JAI_IO_BACKEND=io_uring\|epoll\|kqueue\|iocp\|ioring` | Ask for a backend before `io_run`; an unavailable one is replaced and `backend_reason()` says why |
| `JAI_IO_FILE_THREADS` | epoll/kqueue file pool size (default 4, max 16, 0 = all inline) |
| `JAI_IO_FILE_INLINE` | file IO up to this many bytes runs inline (default 32768, 0 = all to the pool) |
| module parameters | `STACK_BYTES` (256 KiB), `POOL_LIMIT` (1024), `TEMPORARY_BYTES` (2048): `#import "Extensions/Io"(STACK_BYTES = 64 * 1024)` |
| constants | `RING_ENTRIES`, `RESOLVER_MAX_THREADS`, `MAX_POLLED_FDS`, `RING_SUBMISSION`, `SEND_CHUNK` |

Introspection: `backend()`, `backend_name()`, `backend_reason()`, `capabilities()`, `io_statistics()`.

## Dependencies

libc, [Io_Uring](io-uring.md) on Linux, kernel32/ws2_32/KernelBase on Windows, `Atomics`, `Thread` (resolver, file pool, `io_run_threads`), [Json](json.md) (only `read_json`). Tests: `stdlib/Extensions/Io/tests/` (`tcp`, `streams`, `timeouts`, `files`, `pipe`, and `backends.jai`, which runs every check on each backend the system offers); benchmarks in `bench/` (`jaic -O2 stdlib/Extensions/Io/bench/bench.jai`).

Not done: UDP, provided buffers, multishot receive, zero-copy send, DNS timeout, IoRing cancel, registered files.
