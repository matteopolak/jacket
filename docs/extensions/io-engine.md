# Io

Cooperative fibers on a per-thread scheduler (`io_run`, `spawn`, `sleep`, `cancel`, `Channel(T)`, `Wait_Group`, `Fiber_Mutex`, `Condition`, `with_grace`) plus blocking-style TCP, Unix sockets, files, name lookups, deadlines and cancellation on a completion engine: io_uring and epoll (Linux), kqueue (macOS), completion ports with optional IoRing for files (Windows). A common `Reader`/`Writer` covers files, TCP and pipes. Built on the [fiber intrinsics](../compiler/interpreter.md); a compile error on wasm. Shared rules: [README](README.md#conventions-of-io-tls-http-and-json).

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

- Each fiber has its own stack with a guard page. The scheduler lives in `context.io_scheduler` and runs on the thread's own stack. `io_run(proc)` returns when the root fiber returns, canceling and draining the rest. Fibers never move between threads.
- With nothing ready the thread waits in the engine until the next timer. Nothing ready, no timers and nothing pending prints a deadlock message and exits 1.
- `.CANCELED` after `cancel(handle)` is sticky for that fiber; `.CLOSED` is a closed channel. A stale `Fiber_Handle` is ignored.
- Each fiber gets a copy of the spawner's `context` with its own temporary storage. `in_scheduler()` tells code whether it runs on a fiber.
- `make_pipe()` returns two connected in-memory `*Pipe_Stream` ends, which lets the [HTTP test client](http-server.md#testing) run without a socket.
- Stack overflow on POSIX prints `fatal: fiber stack overflow` (exit 139). `release_unused_stack()` returns the pages below the running frames to the system, for a fiber about to wait a long time.
- `io_run_threads(n, proc)` runs one scheduler per thread. Listen with `reuse_port = true` in each.

## Readers, writers, deadlines

`Reader` and `Writer` are small vtables, so you can add sources (a TLS stream is one). `reader_of(x)`/`writer_of(x)` wrap `*Io_File`, `Tcp_Stream` and `*Pipe_Stream`. Operations: `read` (0 is the end), `read_full`, `write` (all of it), `write_some`, `close`, `wait_readable` (returns when a read wouldn't wait; the HTTP server uses it to keep idle connections bufferless), plus `read_all`, `read_file`, `write_file`, `read_json`. A reader and writer over one TCP stream share the connection: close only one.

```jai
file, error := Io.open_file("log.txt", .READ);
defer Io.close(*file);
source := Io.reader_of(*file);
rest, error = Io.read_all(source);
Io.with_deadline(2000, #code { count, error = Io.read(source, buffer); });   // .TIMED_OUT
```

`open_stream_file(path)` opens a terminal, FIFO or `/dev/stdin` for non-blocking reads from the current position (no `pread`), so a fiber can wait on a line without stalling the others. POSIX only; Windows returns `.UNSUPPORTED`. The descriptor stays non-blocking while open, so nothing else should read it meanwhile; closing a duplicated stdin restores its flags.

- `with_deadline` limits every IO call and `sleep` in the block; the earlier deadline wins when nested, and an expired deadline fails a call before it starts. The body is a `#code` argument: assign to variables declared before it and don't `return` inside. Channel and lock operations ignore deadlines.
- `cancel(fiber)` ends the fiber's current IO with `.CANCELED` and fails later IO the same way.
- `with_grace(ms, #code {...})` runs cleanup after a deadline or cancel fired, with its own limit; a cancel arriving meanwhile applies when the outermost grace block ends.
- `Condition` (`wait`, `notify_one`, `notify_all`) is the one coordination primitive that honours deadlines. A notify with no waiter is lost, so loop on your own state.
- Unix sockets: `connect_unix(path)` and `unix_listen(path, backlog, replace)` return the same `Tcp_Stream`/`Tcp_Listener`. `unix_listen` fails with `.ADDRESS_IN_USE` if the file exists unless `replace = true`; the file is never removed on close. Not supported on Windows.
- `connect("host:port")` resolves names with `getaddrinfo` on helper threads (up to 8); lookups of one name share a job.
- Files: io_uring and Windows do file IO in the kernel. epoll and kqueue can't report a regular file as not ready, so reads/writes over 32 KiB go to a file thread pool and shorter ones run inline.

## The engine

A fiber fills an `Op`, calls `engine_submit` and parks; the engine completes it inline or later via `finish_op`, which wakes the fiber. Buffers and addresses belong to an operation until its completion arrives, even after a cancel. A missing feature is a fallback, never an error. Deadlines are timers that cancel the operation with cause `TIMED_OUT`.

- **io_uring** (Linux 6.1 floor, on [Io_Uring](io-uring.md)): one ring per scheduler thread. A denial (old kernel, seccomp, `kernel.io_uring_disabled`) falls back to epoll and records why in `backend_reason()`. `accept` is one multishot request per listener.
- **epoll / kqueue** (`engine_poll.jai`): try the operation at once; on `EAGAIN` queue it on the descriptor. epoll is edge-triggered and remembers per direction that the last try found nothing, so a keep-alive request costs no `epoll_ctl`.
- **Windows**: one completion port per thread with overlapped socket and file calls. IoRing (Windows 11 22H2+) is files only, probed at start. No `SO_REUSEPORT`: `reuse_port` shares the port without balancing.
- Helper threads (resolver, file pool) wake the scheduler through a pipe, `eventfd`, `EVFILT_USER` or completion packet.

### Who may wake a fiber

`wake` asserts the target is `BLOCKED`. A new caller must answer: can the target be the running fiber, can it be woken twice for one park (completion plus cancel or timer), can it be `READY`. Waiters are removed from their list before the wake; `fire_timers` removes the timer before acting. Regressions: `tests/pipe.jai`, `tests/backends.jai`, `test_cancel_resolve`.

## How to change it

- New operation: add an `Op_Kind` (`WAIT_READABLE` is the smallest example: `sys_peek`, `poll_attempt`, `uring_submit`, `iocp_submit`, `wait_readable`), then a public procedure in `net.jai`/`files.jai` that fills an `Op` and calls `await_op`.
- New backend: a `Backend` member, `*_start`, `engine_submit`/`engine_cancel`/`engine_wait` cases, `finish_op` per completion.
- New error: add the `E_*` number for both OSes in `platform.jai` and a case in `error_from_code`; append the `Io_Error` member at the end (the enum only grows).
- New reader/writer source: the `Read_Proc`/`Write_Proc`/`Close_Proc` signatures plus `reader_of`/`writer_of` overloads in `stream.jai`. A pipe-like source checks the deadline itself.
- New blocking primitive: follow `wait_on` in `sync.jai` (stack-allocated `Waiter`, then `park`).
- Gotchas: `open`/`fcntl` are variadic in C and must be declared `args: ..Any`. `epoll_event` is packed on x86-64 only. A raw `close` of a descriptor that went through a poll backend must use `discard_handle`. `discard_pages` in `stacks.jai` uses `MADV_DONTNEED` on Linux, fresh zero pages on macOS and `DiscardVirtualMemory` on Windows, which needs a 16 KiB stack margin (`DISCARD_MARGIN`). Call `release_unused_stack()` when a fiber goes idle, not per request.

## Configuration

- `JAI_IO_BACKEND=io_uring|epoll|kqueue|iocp|ioring` asks for a backend before `io_run`; an unavailable one is replaced and `backend_reason()` says why.
- `JAI_IO_FILE_THREADS` (default 4, max 16, 0 = all inline) and `JAI_IO_FILE_INLINE` (default 32768 bytes) tune the epoll/kqueue file pool.
- Module parameters: `STACK_BYTES` (256 KiB), `POOL_LIMIT` (1024), `TEMPORARY_BYTES` (2048), e.g. `#import "Extensions/Io"(STACK_BYTES = 64 * 1024)`.
- Introspection: `backend()`, `backend_name()`, `backend_reason()`, `capabilities()`, `io_statistics()`.

The shared-port test keeps the first worker’s port-zero listener open while publishing its chosen port through the ready barrier. The second worker then binds that same port with `reuse_port`. Keep this continuous ownership: a closed probe leaves a gap in which another test can select the port, and reuse-port binds can silently join the other test’s listeners. Both workers still serve the same twenty one-byte echo clients.

Unexpected setup, accept, read or write errors emit at most `PORT_DIAGNOSTIC_LIMIT` (32) `IO_PORT_FAILURE` rows, with final port, acceptance and failure counts if an operation or final check fails. Successful runs stay quiet. The helper and limit live beside the shared-port test in `tests/support/suite.jai`; change them there when adding a failure stage. Run the check with `JAIC_STDLIB=$PWD/stdlib jacket test --path stdlib/Extensions/Io --no-doc --exact backends --interp`.

Tests: `stdlib/Extensions/Io/tests/` (`backends.jai` runs every check on each backend the system offers). Benchmarks: `jaic -O2 stdlib/Extensions/Io/bench/bench.jai`. Not done: UDP, provided buffers, zero-copy send, DNS timeout.
