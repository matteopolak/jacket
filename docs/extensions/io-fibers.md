# Io: fibers (`Extensions/Io`)

## What it is

`#import "Extensions/Io";` gives cooperative fibers on a per-thread scheduler: `io_run`, `spawn`, `yield`, `sleep`, `cancel`, a bounded `Channel(T)`, `Wait_Group` and `Fiber_Mutex`. A jaic extension. Sockets, files, deadlines and name lookups run on the same scheduler through a completion engine: see [the Io engine](io-engine.md); files, TCP and pipes share one reader/writer interface and one deadline primitive: see [io-streams.md](io-streams.md). It builds on the [fiber intrinsics](../compiler/fibers.md) and is a compile error on wasm.

```jai
Io :: #import "Extensions/Io";

main :: () {
    Io.io_run(() {
        results := Io.make_channel(int, 4);
        for 1..4 Io.spawn((n: int) { Io.sleep(10 * n); Io.send(results, n); }, it);
        for 1..4 { value, err := Io.recv(results); print("%\n", value); }
    });
}
```

## How it works

- Each fiber has an own stack from `mmap`/`VirtualAlloc` with a guard page (`STACK_BYTES`, 256 KiB). Stacks of finished fibers are pooled up to `POOL_LIMIT`.
- The scheduler lives in `context.io_scheduler` (the language has no thread-locals) and runs on the OS thread's own stack. `io_run(proc)` starts a root fiber and returns when it returns, canceling and draining the rest. A fiber belongs to the thread that spawned it.
- Ready queue, plus a timer min-heap. When nothing is ready the thread sleeps until the next timer (`wait_step` calls `engine_wait`, which waits on the [IO engine](io-engine.md) with that timeout). Nothing ready, no timers and nothing pending prints a deadlock message and exits 1.
- Blocking calls (`sleep`, `recv`, `send`, `wait`, `lock`, `yield`) return an `Io_Error`: `.NONE`, `.CANCELED` after `cancel(handle)` (sticky for that fiber, see `is_canceled()`), or `.CLOSED` for a closed channel. Waiters sit in intrusive lists on the blocked fiber's stack.
- Each fiber gets a copy of the spawner's `context` with its own temporary storage (`TEMPORARY_BYTES`, reset when the fiber ends).
- `in_scheduler()` says whether the caller runs on a fiber (inside `io_run`), for code that starts a scheduler of its own when there is none (the [HTTP client](http-client.md) does).
- `make_pipe()` returns two connected `*Pipe_Stream` ends, an in-memory duplex stream with the shape of a TCP connection: `read` (waits for bytes, 0 once the other end closed and everything was read, honours `with_deadline` and `cancel`), `read_full`, `write` and `write_some` (never block, `.BROKEN_PIPE` after the peer closed), `shutdown_write` and `close` (each end once; the pair frees itself after the second). `reader_of(pipe)`/`writer_of(pipe)` make an `Io.Reader`/`Io.Writer` ([io-streams.md](io-streams.md)). Both ends belong to one scheduler. It is what lets the [HTTP test client](http-testing.md) run a server connection without a socket. `pipe.jai`; test: `Extensions/Io/tests/pipe.jai`.
- `Fiber_Handle` carries a generation, so a stale handle (to a finished, reused fiber) is ignored by `cancel`.
- Stack overflow (POSIX): a SIGSEGV/SIGBUS handler on an alternate stack checks the fault address against the running fiber's guard page and prints `fatal: fiber stack overflow: ...`, exit status 139. On Windows the runtime's usual access violation report applies. Under `jaic -interp` the interpreter's own stack-depth trap fires instead.

## How to change it

`scheduler.jai` (fibers, timers, pooling), `sync.jai` (channel, wait group, mutex, waiter lists), `stacks.jai` (stack memory), `overflow.jai` (guard report). `lock`/`unlock` names match `Thread`'s; qualify with `Io.` if both are imported. New blocking primitives follow `wait_on`: put a stack-allocated `Waiter` on a list and `park`; waking passes a reason.

## Configuration

Module parameters: `STACK_BYTES := 256*1024`, `POOL_LIMIT := 1024`, `TEMPORARY_BYTES := 2048`:

```jai
Io :: #import "Extensions/Io"(STACK_BYTES = 64 * 1024);
```

## Dependencies

`Basic`, [`Extensions/Json`](json.md) (only for `read_json`), the fiber intrinsics, `POSIX`/`Windows` for stack memory and signals. Tests: `tests/stdlib/io-fibers-*.jai` (interp and native; skipped on wasm), `tests/native/fibers/`.
