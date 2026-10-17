# Io: the completion engine, TCP and files (`Extensions/Io`)

## What it is

The IO half of `Extensions/Io`: blocking-style `tcp_listen`, `accept`, `connect`, `read`, `write`, `close`, file calls, `resolve`, deadlines and cancellation for [fibers](io-fibers.md) (the common `Reader`/`Writer` over files, TCP and pipes is in [io-streams.md](io-streams.md)), on top of an internal completion engine with five backends (io_uring and epoll on Linux, kqueue on macOS, completion ports with or without an IoRing for files on Windows) and a worker thread pool for files on epoll and kqueue.

```jai
Io :: #import "Extensions/Io";

handle :: (conn: Io.Tcp_Stream) {
    defer Io.close(conn);
    buffer: [4096] u8;

    while true {
        count, error := Io.read(conn, buffer);
        if error != .NONE || count == 0 return;      // 0 is the peer closing its end

        Io.write(conn, .{ count, buffer.data });      // sends all of it
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

## API

Everything that waits must run on a fiber inside `io_run`, blocks only that fiber, and returns its result plus an `Io_Error`.

| Area | Procedures |
|---|---|
| Addresses | `Address`, `address_v4`, `parse_address("1.2.3.4:80" / ":80" / "[::1]:80")`, `address_to_string`, `resolve(host, port)` |
| Listening | `tcp_listen(text or Address, backlog, reuse_port)`, `accept`, `local_address`, `close(listener, now := false)` |
| Streams | `connect(Address / "host:port")`, `read` (0 is end of stream), `read_full` (`.END_OF_FILE`), `wait_readable` (returns when a `read` would not wait, takes no buffer), `write` (all of it), `write_some`, `shutdown_write`, `peer_address`, `close(stream, now := false)`; `reader_of`/`writer_of` give an `Io.Reader`/`Io.Writer` ([io-streams.md](io-streams.md)) |
| Files | `open_file(path, File_Mode)` with `.READ .WRITE .APPEND .READ_WRITE`; on a `*Io_File` (it has a position, so by pointer): `read` / `read_full` / `write` / `write_some` (advance `position`), `read_at` / `write_at`, `file_size`, `close`; whole-file `read_file` (built on a reader) / `write_file` |
| Time | `sleep`, `with_deadline(ms, #code { ... })` (the one deadline primitive), `cancel(fiber)`, `is_canceled` |
| Threads | `io_run_threads(count, proc [, data])` (one scheduler per thread, no stealing), `processor_count` |
| Introspection | `backend()` (`.IO_URING .EPOLL .KQUEUE .IOCP .IORING .NONE`), `backend_name()`, `backend_reason()`, `capabilities()`, `last_platform_code()`, `io_statistics()` (diagnostic: `Io_Statistics` with `resolver_jobs` and `resolver_helpers`) |

- **Errors.** `Io_Error` is `NONE`, `CANCELED`, `CLOSED`, `TIMED_OUT`, `END_OF_FILE`, `CONNECTION_RESET`, `CONNECTION_REFUSED`, `CONNECTION_ABORTED`, `BROKEN_PIPE`, `NOT_CONNECTED`, `ADDRESS_IN_USE`, `ADDRESS_UNAVAILABLE`, `NETWORK_UNREACHABLE`, `HOST_UNREACHABLE`, `NOT_FOUND`, `ALREADY_EXISTS`, `PERMISSION_DENIED`, `INVALID`, `NO_SPACE`, `TOO_MANY_OPEN_FILES`, `IS_A_DIRECTORY`, `UNSUPPORTED`, `RESOLVE_FAILED` and `OTHER`. The enum only grows. `last_platform_code()` is the system's error number behind the running fiber's last failed call (the errno, or `getaddrinfo`'s code for `RESOLVE_FAILED`), so `OTHER` stays diagnosable and new kernel errors never break callers.
- **`close`** works for a canceled fiber (so `defer close(conn)` always cleans up) and wakes every fiber waiting on that handle with `.CLOSED`. `close(x, now = true)` closes at once without waiting, for code that cannot block (it replaces the old `close_now`).
- **Cancellation.** `cancel(fiber)` ends the fiber's current IO with `.CANCELED`, and all later IO of that fiber fails the same way (it is sticky, as for `sleep`). The fiber stays blocked until the engine reports the canceled operation, so a buffer is never written after the call returns.
- **Deadlines.** `with_deadline(ms, #code { ... })` limits every IO call and `sleep` in the block; the earlier deadline wins when they nest; an expired deadline fails the call before it starts, even when data is waiting (also on a pipe). A timed out call affects only itself. `NO_DEADLINE` (-1) adds no limit. `set_deadline`/`restore_deadline` are gone. jaic does not accept a trailing block after the call, so the block goes in as a `#code` argument. Channel and lock operations ignore deadlines.
- **`resolve`.** Address literals never leave the process. Names go to `getaddrinfo` on helper threads of the scheduler: one is started at the first lookup, another whenever a lookup is queued and every helper already has a job (up to `RESOLVER_MAX_THREADS`, 8), and all are joined when `io_run` ends (helpers are not retired earlier). Only the asking fiber waits. Lookups for the same name (exact text) that are queued or running share one job: later fibers join its waiter list and each gets the answer with its own port, so a fan-out to one origin makes one `getaddrinfo` call. There is no cache beyond that: once the answer is collected, the next lookup asks again. IPv4 is preferred when a name has both. A cancel or a deadline stops that fiber's wait at once and only takes it off the job; the lookup itself cannot be interrupted, so the job is freed when its answer is collected, whoever is still waiting. `io_statistics()` (an `Io_Statistics` with `resolver_jobs` and `resolver_helpers`: lookups made and helper threads started on this scheduler) is a diagnostic for tests.
- **Files.** On io_uring the kernel does reads and writes, on Windows completion ports or an IoRing do, and only the fiber waits. epoll and kqueue cannot report a regular file as not ready, so those backends send reads and writes longer than 32 KiB to a [thread pool](#file-thread-pool) and run shorter ones at once on the scheduler's thread. `.APPEND` always writes at the end, whatever the offset, because systems disagree on `pwrite` with `O_APPEND`. `open_file` is always a plain call. Paths are UTF-8 (converted to UTF-16 on Windows).
- **Several schedulers.** `io_run_threads(4, proc)` starts four threads, each with its own scheduler and engine, and runs `proc` on a fiber of each. Listen with `reuse_port = true` in each and the kernel spreads connections between the listeners (Linux balances by hash; macOS does not balance, but delivers every connection to some listener). Fibers never move between threads, and a `Channel` or `Wait_Group` must stay on one thread.

## How it works

Files: `platform.jai` (libc calls and per-OS numbers), `engine.jai` (types, backend choice, `await_op`), `engine_poll.jai` (epoll and kqueue), `engine_uring.jai`, `engine_iocp.jai` and `engine_ioring.jai` (Windows), `win32.jai` (Win32 declarations and the Windows `sys_*` calls), `files_pool.jai` (file thread pool), `net.jai`, `files.jai`, `stream.jai` (readers and writers), `json.jai` (`read_json`), `resolver.jai` (helper threads), `runtime.jai` (deadlines, threads, introspection).

**The engine.** A fiber describes an operation (`Op`: kind, `Handle`, buffer, offset, address), calls `engine_submit` and parks. The engine finishes it now (`engine_submit` returns true, nothing parks) or later (`finish_op` records result, flags, `more` and buffer id, and wakes the fiber). Kinds: accept, connect, recv, send, read and write (with an offset for files), close, and wait-readable, which completes when a `recv` would not wait and consumes nothing (it is `wait_readable`; a server uses it to keep an idle connection without a read buffer). A `Handle` is an opaque descriptor plus a kind tag (`SOCKET`, `FILE`, `STREAM`), so a SOCKET or HANDLE fits later. Results are a byte count or the negative platform errno; `error_from_code` makes them portable. The engine keeps these rules: buffers and addresses belong to an operation until its completion arrives, even after a cancel; a missing feature is a fallback, never an error.

**Scheduler hook.** `wait_step` (fibers have nothing ready) computes the time to the next timer and calls `engine_wait(timeout)`, which submits what is queued, waits for completions up to the timeout and wakes fibers; it replaced the old OS sleep. Every 64 switches a busy scheduler also polls the engine without waiting. The timer heap serves `sleep` and deadlines: a deadline is a timer on a fiber blocked in an operation, and when it fires the scheduler cancels the operation with cause `TIMED_OUT`. (There is no separate engine "timeout" operation: the wait timeout carries io_uring's `EXT_ARG`, `epoll_pwait2`'s nanoseconds (Linux 5.11; `epoll_wait`'s milliseconds, rounded up, where the call is missing) or kevent's timespec.) Nothing ready, no timers and nothing in flight is the old deadlock report.

**io_uring** (Linux with the 6.1 floor, on [Extensions/Io_Uring](io-uring.md)). One ring per scheduler thread, 512 entries, created on the scheduler's thread with `SINGLE_ISSUER | DEFER_TASKRUN | COOP_TASKRUN`, retrying with `COOP_TASKRUN` and then with no flags when the kernel answers `EINVAL`. A denial (`ENOSYS`, `EPERM`, `EACCES`, `ENOMEM`: old kernels, seccomp in containers, `kernel.io_uring_disabled`) ends the ladder, and the engine uses epoll and records why in `backend_reason()`. After setup it probes the opcodes it uses plus `SENDMSG_ZC` (the 6.1 marker) and needs `IORING_FEAT_EXT_ARG`; otherwise it falls back too. Submission only fills an entry (`user_data` is the `Op` address); entries reach the kernel in batches when the scheduler waits, polls, or the queue is full. The wait is one `io_uring_enter` with `GETEVENTS` and a timeout; completions are reaped straight from the ring. Sockets stay blocking (io_uring waits for them itself) and `accept` takes `SOCK_CLOEXEC` only. Cancel is `ASYNC_CANCEL` by `user_data`; close is a hard-linked `ASYNC_CANCEL` by descriptor followed by `CLOSE`, so fibers waiting on that descriptor wake (a closed fd alone would not end an in-flight receive). `accept` is one multishot request per listener (`IORING_ACCEPT_MULTISHOT`, in Linux 5.19, so always there on the 6.1 floor): the first `accept` on a listener arms it, each connection the kernel takes becomes a completion, and one that arrives while no fiber is in `accept` is queued (an `Accept_Slot` in the ring's state, keyed by the listener's descriptor) for the next call, so a burst of connections costs no request per connection. The slot holds one waiter; a second fiber accepting on the same listener at the same time uses an ordinary one-shot accept (sockets that end up in the queue then wait for the first fiber's next call). A fiber that calls `accept` while the request is being canceled (the queue was full) takes from the queue first, and waits on the request like any other; when the cancel ends the request without anyone having asked for it, the fiber arms a new one and waits again. (Before, it fell back to a one-shot accept at once, which never looks at the queue: a burst of more than 128 connections could leave sockets queued with their clients waiting for a server that had accepted them.) A cancel or deadline of the waiting fiber cancels the request (the next `accept` arms a new one); the queue is capped at 128 sockets, past which the request is canceled until the queue drains, and `close` closes whatever is queued and cancels the request with the descriptor (`close(listener, now = true)` submits that cancel itself, since the request would otherwise keep the socket open). The kernel ending the request (no `more` flag) clears the slot's request and the next call re-arms; an error completion goes to the waiter, as the one-shot accept's did. Only a parked fiber counts in `scheduler.pending`, so an armed request with nobody waiting does not hide a deadlock. `wait_readable` is `IORING_OP_POLL_ADD` with `POLLIN`, one shot; a hang-up or an error also completes it, and the `read` that follows reports which. Cancel results (`-ECANCELED`) are mapped to `.CANCELED`, `.TIMED_OUT` or `.CLOSED` by who asked.

**epoll and kqueue** share `engine_poll.jai`. A readiness API does not do IO, so an operation is tried at once without blocking (accept, `connect` returning `EINPROGRESS`, recv, send, pread); on `EAGAIN` it waits in a queue on its descriptor (one list of readers and one of writers, in `Fd_State`) and the descriptor is armed: kqueue `EV_ADD | EV_ONESHOT` per direction for one notification, epoll once for good (below). When it fires, the queue is served in order until an operation would block again. Two fibers on one descriptor therefore cannot lose each other's wake-up. kqueue changes are batched into the wait call. `connect` waits for writability and reads `SO_ERROR`. `wait_readable` is tried at once with a one-byte `recv(MSG_PEEK)`, which only looks, and waits in the reader queue like a `recv` when there is nothing; when the descriptor is reported readable the first waiter in line is completed without looking again. Closing a descriptor fails everything waiting on it first. Sockets are non-blocking, close-on-exec and `SO_NOSIGPIPE` (macOS) or `MSG_NOSIGNAL` (Linux).

**epoll is edge-triggered.** A descriptor is added when its first operation has to wait, with `EPOLLET` and the directions its waiters need (`EPOLLIN | EPOLLRDHUP` for readers, `EPOLLOUT` for writers; a socket that is connecting gets both), and the registration only grows (a `MOD` when a direction is needed later, which also reports a descriptor that is ready already, so nothing is lost between the failed try and the registration). A request/response connection therefore costs no `epoll_ctl` per request. The price is that the kernel no longer says "still readable", so `Fd_State` remembers per direction that the last try found nothing (`read_blocked`, `write_blocked`), and the next operation then queues without a system call until an event for that direction clears the flag. A try that found nothing is `EAGAIN`, or for a socket a `recv` that returned less than it was asked for (a stream socket with a short read is empty; a `send` that took less than offered has filled the buffer). The end of the stream is not an edge that comes again, so a hang-up, a shutdown by the peer (`EPOLLRDHUP`), an error or a `recv` of 0 set `hup` and from then on receives always try. A stale edge cannot report a drained socket readable: the kernel looks at a descriptor again when `epoll_wait` hands the event out and drops it if the data was taken meanwhile, which is why the first waiter can still be completed without looking. A keep-alive request now takes one `recv`, one `send` and a share of an `epoll_wait` (it took two `recv`, a `send` and an `epoll_ctl`). `tests/lib/suite.jai` (`test_readiness_after_early_reads`) covers data and an end of stream that arrive before anyone looks.

**Resolver and wake-ups.** Helper threads cannot call into a backend, so they hand finished jobs to the scheduler through a pipe: the scheduler keeps an ordinary internal `READ` on the pipe armed while lookups are out (so every backend wakes the same way), a helper appends its job to a mutex-protected list and writes a byte, and the completion wakes the fibers. `engine_stop` waits for outstanding lookups first (jobs, shared or abandoned, until every helper has answered), then joins every helper that was started, so a late answer never reaches freed memory.

**Idle stacks.** A parked fiber keeps the stack pages it has touched, about 12 KB of the 16 KB an idle server connection costs. Releasing the dead part before an idle wait was tried and dropped: the system call that releases the pages runs on the same stack, just below the boundary, so it can discard its own frame (Windows' `DiscardVirtualMemory` crashed every time), and a margin that makes it safe leaves about nothing to release.

**Windows** (`engine_iocp.jai`, `engine_ioring.jai`, `win32.jai`; x64 and arm64). One completion port per scheduler thread. Every operation is an overlapped call carrying its `Op`: `Op` starts with the `OVERLAPPED`, so the pointer a packet brings back is the operation. Calls never use "skip the packet on success", so a call that returns success or `ERROR_IO_PENDING` always finishes with a packet in `iocp_wait`, and one that fails at once finishes in `iocp_submit`.

| Operation | Call |
|---|---|
| accept | `AcceptEx` on a socket made ahead of time (so `getsockname`/`getpeername` work after `SO_UPDATE_ACCEPT_CONTEXT`) |
| connect | `ConnectEx` after a bind to the wildcard address, then `SO_UPDATE_CONNECT_CONTEXT` |
| recv, send | `WSARecv`, `WSASend` (at most 64 KiB per send, so a canceled `write` can report what went out) |
| wait-readable | `WSARecv` with a zero-length buffer, which completes when data, the end of the stream or an error is there; any error but the cancel counts as readable |
| read, write | `ReadFile`/`WriteFile` with the offset in the OVERLAPPED (end of file offset for appends), or the IoRing |
| cancel | `CancelIoEx`; the operation still completes, as `ERROR_OPERATION_ABORTED` |
| close | `CancelIoEx` on the whole handle, then `closesocket`/`CloseHandle`; fibers waiting on it wake with `.CLOSED` |

`AcceptEx` and `ConnectEx` are fetched with `WSAIoctl` at startup. The wait is `GetQueuedCompletionStatusEx` with the scheduler's timeout in milliseconds, rounded up; a timeout is reported as `WAIT_TIMEOUT` (258) or `ERROR_TIMEOUT` (1460) depending on the Windows version, and both mean no completions. A packet's result is the NTSTATUS in `OVERLAPPED.Internal`, taken from the dequeued entry (the handle may be closed by then) and turned into a Win32 error with `RtlNtStatusToDosError`; on Windows the `E_*` numbers are the Win32 and Winsock numbers themselves, so `last_platform_code()` is the real error, and `windows_error` maps the extra ones (`ERROR_NETNAME_DELETED` is a reset, `ERROR_CONNECTION_REFUSED` a refusal, ...). Gotchas: `bytes > 0` with an abort status counts as a short success; `ERROR_HANDLE_EOF` is a read of 0; Windows has no `SO_REUSEPORT`, so a plain listener uses `SO_EXCLUSIVEADDRUSE` (the opposite of `SO_REUSEADDR` there, which would let another program take the port) and `reuse_port = true` uses `SO_REUSEADDR`, which shares the port but hands connections to one listener rather than balancing them; timeouts are as fine as the system timer (about 15 ms unless something raised its resolution); a connect to a closed local port reports the refusal after a second or two.

Name lookups (`resolve`) use `getaddrinfo` on the same helper threads; a helper wakes the scheduler by posting a packet with no OVERLAPPED and `KEY_WAKE` to the port, instead of writing a pipe.

**IoRing** (`.IORING`, Windows 11 22H2+; `ioringapi.h`). It is for files only: sockets, `wait_readable` included, go through the completion port in this mode too. The API is loaded from KernelBase.dll with `GetProcAddress`, so older Windows still runs. A one-time probe checks that all the functions exist, that `QueryIoRingCapabilities` offers at least version 2, and then runs a self test for each version it knows, newest first (`IORING_VERSION_3`, which is 300 in `ioringapi.h`, then 2): it creates a throwaway ring of that version, reads two bytes of the program's own executable through it, waits for the completion event, checks for "MZ" and builds (without submitting) a write entry, which a ring version without file writes refuses (`0x80004021`). The first version that passes is the one every engine ring is created with. Anything that fails sends files to the completion port and records the step in `backend_reason()`. With a ring: reads and writes become submission entries (`BuildIoRingReadFile`/`BuildIoRingWriteFile`, the `Op` as user data) that `SubmitIoRing` hands over in a batch whenever the scheduler waits; completions are popped in user space. The ring's completion event is waited on by a small helper thread (started at the first file operation) that posts `KEY_RING` to the port; while operations are in the ring the scheduler never sleeps longer than 50 ms, so a lost event delays instead of hangs. The ring holds 128 operations and more wait in a list. Operations in a ring cannot be canceled (a file read ends in disk time), and `close` of a file waits for the ring to go idle. Files are not attached to the completion port in this mode. The by-value `IORING_HANDLE_REF`/`IORING_BUFFER_REF` structs rely on jaic's Win64 and Windows arm64 struct passing.

`JAI_IO_BACKEND=iocp` keeps files on the completion port; `ioring` asks for the ring but falls back (with a reason) when it is missing.

### File thread pool

`files_pool.jai`, for epoll and kqueue. One pool per process, shared by all schedulers: started when the first scheduler needs it, ended when the last one is done, so `io_run` can run any number of times and nothing outlives the program. The workers make plain `pread`/`pwrite` calls from one queue. Each scheduler has a `Pool_Link`: a mutex-protected list of finished operations, and the wake-up. A worker appends its operation and wakes the scheduler while holding the mutex; the scheduler collects under it, and takes it once more before freeing the link, so no worker touches freed memory. The wake-up is an `eventfd` on epoll, read by an internal READ that is armed while operations are out (the same device the resolver pipe uses, with `Internal_Tag.POOL_WAKE`), and an `EVFILT_USER` event (`EV_CLEAR` plus `NOTE_TRIGGER`) on kqueue. A pool operation cannot be interrupted: cancel and deadlines wait for it (`Op.uncancelable`), and `close` of a file waits until every operation out has come back.

*Inline or pool.* A hand-off to a worker and back costs about 3 to 5 microseconds when the data is in the page cache; a short read costs less than that, but a read that has to wait for the disk stalls every fiber of the thread. Page-cache reads on an Apple M-series Mac (kqueue, native `-O2`, `stdlib/Extensions/Io/bench/files.jai`, best of three, one scheduler thread):

| Read size | inline, 1 fiber | pool, 1 fiber | inline, 8 fibers | pool, 8 fibers |
|---|---|---|---|---|
| 512 B | 0.52 us | 4.9 us | 0.53 us | 3.1 us |
| 4 KiB | 0.78 us | 5.0 us | 0.78 us | 2.9 us |
| 16 KiB | 1.2 us | 5.6 us | 1.3 us | 3.3 us |
| 64 KiB | 3.6 us | 7.3 us | 4.0 us | 5.0 us |
| 256 KiB | 15.4 us | 18.6 us | 14.3 us | 9.4 us |
| 1 MiB | 57 us | 58 us | 56 us | 37 us |

Inline wins below about 128 KiB for a single fiber, and the pool wins from 256 KiB up once several reads overlap (27 GB/s against 18 GB/s at 1 MiB) and whenever the disk is cold, which these numbers do not show. The default threshold is 32 KiB: smaller operations (headers, small config and log files) stay inline at no cost, and anything a static file server reads in blocks goes to the pool. `JAI_IO_FILE_INLINE=0` sends everything to the pool, a huge value keeps everything inline.

## Who may wake a fiber

`wake` asserts that its target is `BLOCKED`. So whoever calls it must know the fiber is parked and that nothing else will wake that park. A fiber parks at once after it queues itself somewhere, and wakes once; calls the running fiber makes before it parks (`engine_submit`, `wake_arm`, `uring_cancel`) can finish other fibers' work, never its own.

- **`finish_op`** (all backends) wakes the fiber of a finished operation. The operation is taken off its queue (or its ring entry is consumed) first, so it finishes once. A fiber has an operation outstanding only after its submit returned false, so it is parked when the completion comes. A submit that fails or finishes inline returns true and reports through `op.result`; `poll_submit` does this too when the kernel refuses to watch the descriptor, and it does not go through `finish_op` for its own operation.
- **Multishot accept** (`accept_completed`, io_uring): the completions of a listener's request go to its one waiter (`Accept_Slot.waiter`, an `Accept_Wait` on the parked fiber's stack) or to the queue. The waiter is taken off the slot and `scheduler.pending` is lowered before the one `wake`, so a second completion in the same reap finds nobody and queues. The fiber is parked whenever `slot.waiter` is set (`uring_accept` sets it right before `park`, and nothing between them can reach the completion), and a canceled waiter (`cancel_op` finds the request through `fiber.op`) wakes only when the request's final completion arrives, like any canceled operation; when that completion ends the request, `fiber.op` is cleared first because the `Op` is freed in the same step.
- **Cancel and deadline of an operation** (`cancel_op`) only ask the engine to end it. The fiber wakes in `finish_op` when the canceled operation completes, so a completion racing a cancel is still one wake. A pool or IoRing operation ignores the cancel and wakes when it completes.
- **`poll_fail` and `poll_close`** finish the operations queued on a descriptor. Those belong to parked fibers; the fiber that closes has no operation queued.
- **File pool and IoRing completions** (`pool_collect`, `ring_drain`) finish operations that parked fibers hold. `pool_arm` runs before the new operation reaches a worker, so an inline `pool_collect` cannot find it.
- **Resolver** (`wake_fiber`): a job is collected once. The fiber that queued it is still running when `wake_arm` collects an answer that is already there, so `wake_fiber` clears `fiber.job` and leaves it; `resolve_with_helper` parks only while `fiber.job` is set. A job has a list of waiters (`Resolve_Waiter`, on each fiber's stack); `wake_collect` removes the job from the in-flight list, copies the answer into every waiter and wakes each once. A canceled or timed-out fiber leaves the list first (`abandon_job`), so a late answer wakes nobody and a shared job still answers the others.
- **`cancel_fiber`** does nothing unless the fiber is `BLOCKED`. The fiber then has either an operation (it wakes in `finish_op`), a lookup (`abandon_job` takes it off the job's waiter list, then one wake), or a wait list entry and maybe a timer (both removed, then one wake). A fiber that is `READY` or `RUNNING` only gets `canceled` set.
- **`fire_timers`** removes the timer before it acts, so one timer is one wake. For an operation it cancels (wake in `finish_op`), for a lookup it abandons the job and wakes, otherwise it takes the fiber off its wait list and wakes it. Leaving it on the list would let a producer wake it again before it runs.
- **`wake`** removes a timer the fiber still has, so a completion after a timer (or the reverse) never leaves a stale timer behind.
- **Pipes** (`pipe_wake`) and **channels, wait groups and mutexes** (`hand_to_receiver`, `take_ready`, `close`, `wake_all`, `unlock`): the waiter is popped from the list before its fiber is woken, so a waiter wakes once. A fiber is on a list only while parked, and the caller is running, so it never wakes itself. Cancel and deadline take a fiber off the list before they wake it.

A new `wake` caller has to answer three questions: can the target be the running fiber, can it be woken twice for one park (completion plus cancel, completion plus timer, two completions), can it be `READY`. `tests/pipe.jai` (`test_write_between_deadline_and_wakeup`) and `tests/backends.jai` (`test_epoll_refuses_a_descriptor`) are the regressions for the last two bugs; `test_cancel_resolve` covers the lookup that is answered early.

## How to change it

- **A new operation:** add an `Op_Kind` (`WAIT_READABLE` is the smallest example: `sys_peek` in `platform.jai`, a case in `poll_attempt`, `POLL_ADD` in `uring_submit`, the zero-byte receive in `iocp_submit`, and `wait_readable` in `net.jai`), a case in `poll_attempt` (poll backends) and in `uring_submit`, then a public procedure in `net.jai` or `files.jai` that fills an `Op` and calls `await_op`.
- **File pool:** the threshold, thread count and wake-up live in `files_pool.jai`; `poll_submit` decides inline or pool per operation.
- **A new backend:** a `Backend` member, `*_start`, `engine_submit`/`engine_cancel`/`engine_wait` cases, and `finish_op` for each completion. Keep the rules above: nothing is abandoned while the kernel may still touch its buffer.
- **A new io_uring feature** (provided buffers, zero-copy send; multishot accept is the example to follow, with the slot and the wake rules above): extend `uring_submit` and `uring_reap` (`more` and `buffer_id` already travel with the completion), probe it in `uring_start`, and set a `Capabilities` flag; callers never see it.
- **Errors:** add the `E_*` number for both OSes in `platform.jai` and a case in `error_from_code`; append the `Io_Error` member at the end of the enum.
- Gotchas: `open` and `fcntl` are variadic in C and must be declared `args: ..Any` (Apple arm64 passes variadic arguments on the stack, so a fixed declaration silently passes garbage); `epoll_event` is packed on x86-64 only; `accept`ed sockets on macOS need `fcntl` to become non-blocking; the Fd table is indexed by descriptor, so any raw `close` of a descriptor that went through a poll backend must go through `discard_handle` or the stale state will hang the next descriptor with that number.

## Configuration

| Setting | Effect |
|---|---|
| `JAI_IO_BACKEND=io_uring \| epoll \| kqueue \| iocp \| ioring` | Ask for a backend before `io_run`. A backend the system cannot offer is replaced by the default one (`backend_reason()` says so); an unknown value is ignored |
| `JAI_IO_FILE_THREADS` | epoll, kqueue: worker threads of the file pool, started at the first use (default 4, at most 16; `0` runs every file operation inline), read when an `io_run` starts |
| `JAI_IO_FILE_INLINE` | epoll, kqueue: file reads and writes up to this many bytes run inline on the scheduler's thread (default 32768; `0` sends all to the pool) |
| Module parameters | `STACK_BYTES`, `POOL_LIMIT`, `TEMPORARY_BYTES` ([fibers](io-fibers.md)) |
| `RING_ENTRIES` (512), `RESOLVER_MIN_THREADS` (1), `RESOLVER_MAX_THREADS` (8), `MAX_POLLED_FDS`, `RING_SUBMISSION` (128), `SEND_CHUNK` (64 KiB) | constants in `engine_uring.jai`, `resolver.jai`, `engine_poll.jai`, `engine_ioring.jai`, `win32.jai` |

## Dependencies

libc (`socket`, `kevent`/`epoll`, `eventfd`, `pread`, `getaddrinfo`, ...), [`Extensions/Io_Uring`](io-uring.md) on Linux, kernel32, ws2_32 and KernelBase (loaded at run time) on Windows, [`Atomics`](../specs/stdlib/modules/atomics.md), and `Thread` for the resolver, the file pool and `io_run_threads`. The Win32 declarations are private to `win32.jai`. Not available on wasm.

## Tests and numbers

`stdlib/Extensions/Io/tests/` (run by the [stdlib runtime tests](../tools/stdlib-runtime-tests.md) in `interp` and `native` on Linux, macOS and Windows; skipped on wasm): `tcp.jai` (addresses, 40 client fibers against an echo server, a 600 KB transfer, end of stream, reset, refused, address in use, close under a reader, accept (300 connections queued before the first `accept`, two acceptors on one listener, a listener closed with connections queued, `close(now = true)` releasing the port), name lookups, edge-triggered readiness (data and an end of stream that arrive before anyone looks), `wait_readable`: a deadline, data arriving, nothing consumed, the end of the stream, cancel, close, and a deadline racing the data), `streams.jai` (the [reader/writer](io-streams.md) operations over each source, `read_json`), `timeouts.jai` (deadlines, nesting, cancel during accept, read, write and lookup, a scheduler that ends with blocked fibers), `files.jai` (which includes a large read that is canceled or whose file is closed while it is out) and `backends.jai`, which sets `JAI_IO_BACKEND` for each backend the system offers (io_uring and epoll on Linux, kqueue on macOS, iocp and ioring on Windows) and runs every check on it, skipping a backend the system refuses and printing the engine's reason. On epoll and kqueue it also runs the file tests with every operation on the pool and with the pool off. The shared checks are in `tests/lib/suite.jai`. With a release `jaic -interp` each program takes about a second on macOS.

*What was verified where.* macOS (kqueue, the file pool): all tests, interp and native. Windows x64 and arm64, and Linux x64 and arm64: `jaic stdlib/Extensions/Io/tests/backends.jai -check -no_dce -os windows -cpu x64|arm64` (and `-os linux`) type-check; nothing of the Windows backends has run yet, so the first run is the `windows native` workflow.

`bench/bench.jai` (`jaic -O2 stdlib/Extensions/Io/bench/bench.jai -exe /tmp/io-bench`; `JAI_IO_BACKEND` forces a backend; a thread count argument runs the connection test on several schedulers) measures echo round trips on 64 connections and connections per second, server and clients on one scheduler. Apple silicon, kqueue, native `-O2`: about 140,000 echo round trips per second (64-byte messages), about 43,000 connections per second on one thread. Linux numbers are in [Numbers](#numbers).

### Numbers

Linux baseline, and what the performance pass after it changed. Machine: 4 vCPUs of an Intel Xeon @ 2.10GHz in a VM (Firecracker), Linux 6.18.44, jaic `-O2`, loopback. Load generators run on the same cores as the server, so these are floors, and the run-to-run spread is about 10% (more on the short connection benchmarks, up to 30% between single runs). Medians of 5 to 15 runs with the old and the new binary alternating, requests (or operations) per second; `JAI_IO_BACKEND` picks the backend.

```
jaic stdlib/Extensions/Http/bench/server_bench.jai -O2 -exe /tmp/http-bench
JAI_IO_BACKEND=io_uring /tmp/http-bench 4 3000      # threads, ms per route; also epoll, and 1 thread
jaic stdlib/Extensions/Http/bench/idle_bench.jai -O2 -exe /tmp/http-idle
JAI_IO_BACKEND=io_uring /tmp/http-idle 1000
jaic stdlib/Extensions/Io/bench/bench.jai -O2 -exe /tmp/io-bench
JAI_IO_BACKEND=io_uring /tmp/io-bench 4             # also 1 thread (no argument), and epoll
jaic stdlib/Extensions/Http/bench/client_bench.jai -O2 -exe /tmp/http-client-bench
JAI_IO_BACKEND=io_uring /tmp/http-client-bench 3000
```

Before (the first baseline, io_uring and epoll):

| Benchmark | io_uring | epoll | io_uring vs epoll |
|---|---|---|---|
| HTTP hello, 1 thread (5 runs of 3 s) | 99.4k | 84.0k | +18% |
| HTTP JSON, 1 thread | 83.1k | 69.8k | +19% |
| HTTP hello, 4 threads | 375k | 310k | +21% |
| HTTP JSON, 4 threads | 335k | 281k | +19% |
| Idle server RSS, 1,000 connections | 16.3 KB each | 16.2 KB each | same |
| Io echo round trips, 1 thread (64 connections, 64 bytes) | 148k | 126k | +18% |
| Io connections per second, 1 thread | 27.7k | 25.3k | +10% |
| Io connections per second, 4 threads | 47.2k | 37.7k | +25% |

After the pass (same machine, a later session, so the "before" column is measured again and differs from the table above by the run-to-run spread):

| Benchmark | io_uring before | io_uring after | epoll before | epoll after |
|---|---|---|---|---|
| HTTP hello, 1 thread | 97.6k | 95.0k (noise) | 73.8k | 81.2k (+10%) |
| HTTP JSON, 1 thread | 82.0k | 92.3k (noise) | 67.2k | 78.9k (+17%) |
| HTTP hello, 4 threads | 355k | 336k (noise) | 278k | 309k (+11%) |
| HTTP JSON, 4 threads | 303k | 353k (noise) | 252k | 288k (+14%) |
| HTTP client, 32 fibers against a server on the same scheduler | 61.7k | 64.9k (+5%) | 50.1k | 61.2k (+22%) |
| Io echo round trips, 1 thread | 130k | 119k (noise) | 109k | 117k (+7%) |
| Io connections per second, 1 thread (2,000 per client) | 25.8k | 27.1k (+5%) | 22.9k | 23.1k (same) |
| Io connections per second, 4 threads (2,000 per client) | 56.8k | 60.5k (+6%) | 59.3k | 56.4k (same) |

What did what. On io_uring the HTTP figures did not move beyond the spread (nothing on a keep-alive request's path changed there; the +12% and -5% scatter between the routes is that spread, and the series run alternately). Multishot accept is the only change that touches connection churn (+5% to +9% on one thread in the four longer series, +6% on four threads); the echo and request benchmarks do not accept. The idle wait gives 4 KB per connection back. epoll is where the request path got shorter: with edge-triggered registration a keep-alive request makes one `recv`, one `send` and a share of an `epoll_wait` (counted with `strace -c` on the HTTP benchmark, per request: 2.0 `recvfrom`, 1.0 `sendto` and 1.0 `epoll_ctl` before, 1.0, 1.0 and 0.03 after), and the gap to io_uring on the request benchmarks goes from about a quarter to about a sixth (the four rows average 25% before and 17% after). Connection churn on epoll makes fewer system calls (`epoll_ctl` 364k to 320k, `recvfrom` 371k to 339k for 64,000 connections) and is the same speed within the spread. The connection benchmark now runs 2,000 connections per client instead of 400 (the old run was 0.2 s long, and single runs differed by 2 to 3 times).

Tried and dropped, because they did not pay (more than about 5% beyond the spread): building the client's `Host` header by hand instead of with `tprint` (the profile put `tprint` at about 3% of the client benchmark; measured +5.7%, +4.5%, +1.5% and +2.4% in four alternating series); a cheaper `Date` and fewer clock reads on the server path (each is 1% to 2% of the samples; the whole of the server's own code is about a quarter of them, the rest is the kernel's TCP and the load generator, so even removing a tenth of it is below what the benchmark can see). Per-request allocation in the client (`allocator_proc` is about 7% of the client's samples, spread over a dozen call sites, none above 2%) was not attempted for the same reason. Latency percentiles and the io_uring tuning that was tried are below.

### Latency

`server_bench` records every request's round trip (nanoseconds, in a histogram of buckets about 6% wide) and prints p50, p90, p99, p99.9 and the largest per route. Without a rate the load is closed: each of the 32 connections per thread sends when its last answer arrived, so a stall delays the requests behind it and the sender does not notice (coordinated omission). With a rate (`/tmp/http-bench 4 3000 150000`: requests per second over all threads) the load is open: each connection has a schedule and a request is timed from when it was due, so every request that waited behind a stall counts. The open clients sleep in whole milliseconds, so the open load opens as many connections as it takes for each to wait about 3 ms (45,000 requests/s on one thread is about 135 connections). Hello route, medians of 5 runs of 3 s, milliseconds, rates as fractions of the closed-loop maximum measured in the same session (the JSON route is within 20%):

| Backend, threads | Load | Requests/s | p50 | p90 | p99 | p99.9 | max |
|---|---|---|---|---|---|---|---|
| io_uring, 1 | closed | 85.4k | 0.33 | 0.62 | 0.88 | 2.1 | 4.6 |
| | 25% | 21.1k | 0.19 | 0.49 | 2.1 | 5.0 | 6.1 |
| | 40% | 33.7k | 0.33 | 2.4 | 12.1 | 15.7 | 16.7 |
| | 50% | 41.4k | 0.56 | 23.1 | 52.4 | 56.6 | 57.7 |
| io_uring, 4 | closed | 307k | 0.34 | 0.66 | 0.98 | 2.6 | 20.7 |
| | 25% | 74.9k | 0.17 | 0.39 | 1.9 | 6.8 | 11.0 |
| | 40% | 119k | 0.46 | 5.2 | 22.0 | 33.6 | 35.3 |
| | 50% | 148k | 0.34 | 7.1 | 37.7 | 54.5 | 54.4 |
| epoll, 1 | closed | 78.3k | 0.36 | 0.66 | 0.88 | 2.5 | 4.6 |
| | 25% | 19.4k | 0.39 | 0.79 | 2.5 | 5.0 | 6.9 |
| | 40% | 30.9k | 0.75 | 5.0 | 12.1 | 14.2 | 15.5 |
| | 50% | 38.7k | 1.1 | 19.9 | 44.0 | 46.1 | 45.4 |
| epoll, 4 | closed | 264k | 0.39 | 0.72 | 0.98 | 2.9 | 11.3 |
| | 25% | 64.9k | 0.34 | 0.62 | 2.5 | 6.8 | 11.9 |
| | 40% | 103k | 0.51 | 1.2 | 5.5 | 9.4 | 12.9 |
| | 50% | 128k | 0.69 | 5.5 | 28.3 | 33.6 | 35.4 |

Reading it. A closed load at its maximum is tight (p99 is 2.5 to 3 times p50, p99.9 about 7 times): a request waits behind the others of its thread, 32 connections at 90,000 requests/s is 0.35 ms. An open load is not the closed load's half at 50%: **the capacity of the open load is about 45% of the closed maximum**. The clients sleep between requests, so the scheduler wakes for few requests at a time and a request costs 23 us of CPU at 40,000 requests/s against 14.6 us when the 32 connections keep the thread busy (user plus system time of the process; 36 us at 10,000 and 50 us at 5,000: the cost per request rises as the load falls, with wake-ups from idle and smaller batches, though even at 40,000 requests/s an `io_uring_enter` still takes 20 entries). So the rows at 40% and 50% are a thread at 90% to 100% CPU, and 80% is past what the open load can carry at all (p50 110 to 340 ms, achieved rate 78% to 90% of the offered one). The tail at 25% is 6 to 11 times p50 at p99 and 13 to 40 times at p99.9 (p99 1.9 to 2.5 ms, p99.9 5 to 7 ms for a p50 of 0.17 to 0.39 ms).

Where the tail comes from. Not from the scheduler: instrumenting `schedule` (time of every fiber run, every `engine_wait` against its timeout) shows that the long events are the whole thread stopping for 1 to 15 ms, in a fiber run (user code, no system call) or in a wait that ended 1 to 2.4 ms after its timeout, and that every cluster of slow requests lines up with one of them (a stall of s ms delays about s x rate requests, and the bigger ones are what p99.9 and max are). A busy loop in Python on an idle machine of this VM sees the same: 1% of the time stolen, 2 to 5 stalls of more than 1 ms a second, up to 5 ms (a sleep of 200 us overshoots by 90 us at p50 and 1.2 ms at p99.9). With the thread at 40% load a 3 ms stall leaves a backlog of 100 requests that takes as long again to drain, which is the p90 and p99 of the 40% rows. The candidates in `schedule` were looked at and are cheap or absent: polling the engine and the timers every 64 switches is a few hundred microseconds of fibers at most (16 and 256 instead of 64, tried in a copy of the engine: no difference in p50, p99 or p99.9 beyond the spread, 15 runs); the ready queue is FIFO and a burst of completions is served in order, which is the closed load's 0.35 ms; firing timers costs 1.5% of the loop (53 ms of 3.6 s at 137 connections, with a deadline timer per hot request); the `HOT_MS` rule keeps the buffer of a connection whose gap is below 10 ms, which an open load of 3 ms gaps is; `Date`, temporary storage and the buffer pool take microseconds; the submission queue is flushed when the scheduler waits, polls or the 512 entries fill, never because of a stall. What would cut the tail is a machine without hypervisor stalls; on this one a p99 below the stall length is not reachable.

One thing in the tail was ours, on epoll: the wait for the next timer was `epoll_wait`'s millisecond, rounded up, so a fiber that slept woke up to a millisecond late whenever another timer was due first (a timer 0.4 ms away waited 1 ms). `epoll_pwait2` (Linux 5.11, with a fallback to `epoll_wait` when the call is refused) takes nanoseconds. At 20,000 requests/s on one thread, 9 alternating runs, the clients' wake-ups stop being late: p50 1.02 to 0.38 ms and p90 1.6 to 0.69 ms (p99 and p99.9 are the stalls above and did not move: 2.8 and 5.8 ms to 2.8 and 5.5 ms); closed-loop requests per second and latency did not change (71.3k to 71.4k).

The open load also found a real bug. With 120 to 230 connections arriving in a burst, about one run in eight of the bench never ended on io_uring: `accept` fell back to a one-shot request while the multishot request was being canceled (the queue of accepted sockets had reached 128), and the one-shot request never looks at the queue, so 196 accepted connections waited for a server that had already taken them (`tests/lib/suite.jai`, `test_accept_bursts`, fails with 170 of 300 connections unserved before the fix and passes after).

### io_uring: where the lead over epoll comes from, and what was tried

Kernel 6.18.44: the engine's `capabilities()` is `single_issuer`, `defer_taskrun`, `coop_taskrun` and `ext_arg` all true with 63 opcodes, so the flags are in effect. System calls per keep-alive request on the server benchmark (closed load, one thread): epoll 2.0 `sendto`, 2.0 `recvfrom`, 0.09 `epoll_wait` and 0.03 `epoll_ctl` (`strace -c -f`: each side of the loopback makes its own); io_uring 0.115 `io_uring_enter` (a debug counter: 3.7 entries per request, 32 per call; 0.19 calls per request at 40,000 requests/s open). A null system call costs 0.41 us here, so epoll's 4.1 calls are about 1.7 us of the 12.5 to 14.5 us a request takes there (CPU time of the process) and io_uring's batching can give back at most that: 12% to 14%, and it gives 9% to 16% (85.4k against 78.3k on one thread, 307k against 264k on four in the table above). The rest of a request is the kernel's TCP (about 5 us, the same on both) and the fibers' own code (5.3 to 6.4 us). Putting a request's `SEND` and the next `RECV` in one submit would not save a call either: they already share the `io_uring_enter` of the batch they are in, and the only way to link them (`IOSQE_IO_LINK`) needs the fiber to know before the send completes that it will read next, which `Io.write` followed by `Io.read` cannot say.

Tried with an environment switch in a copy of the engine, interleaved, medians of 7 to 15 runs of 1.5 s of the server benchmark on one thread (spread of a single run: 65k to 95k), and of 9 runs of the Io benchmark (echo 100k to 150k, connections per second 14k to 27k). Nothing paid; none is in the engine.

| Variant | Server hello / JSON, requests/s | Io echo / connections per second | Verdict |
|---|---|---|---|
| as it is | 86.4k / 76.1k (15 runs), 80.4k / 77.3k (7 runs) | 127k / 24.0k | |
| no `DEFER_TASKRUN` (`COOP_TASKRUN` only) | 83.1k / 80.4k | 138k / 25.2k | same within the spread |
| no flags at all | 84.3k / 75.3k (7 runs) | | same |
| fixed files (sparse table, a registration per socket, `IOSQE_FIXED_FILE`) | 81.0k / 81.9k | 137k / 23.4k | same; saves two reference counts per operation, about 1% of a request, for two more calls per connection |
| submit after every queued entry | 63.1k / 54.0k | | 22% to 30% worse |
| submit every 8 entries | 80.0k / 70.9k | 122k / 22.2k | worse or same |
| submit before running a ready fiber (every switch / every 8th) | 71.0k / 66.1k, 88.3k / 71.3k | 129k / 22.7k (every 8th) | every switch 12% worse, every 8th same |
| `SQPOLL` (no `DEFER_TASKRUN` then) | 68.1k / 65.8k | 118k / 20.5k | 15% worse: the kernel thread takes a core from the four |
| engine poll every 16 or 256 switches instead of 64 | 85.2k / 74.8k, 87.5k / 82.8k (15 runs) | 137k / 24.0k (256) | same; p50, p99 and p99.9 the same too |

The latency of the variants that submit sooner was no better: none had a p99 better than the spread, and submitting before running every ready fiber moved p50 from 0.33 to 0.41 ms in the closed load (more calls, the same completions). Not tried, because the ceiling above leaves no room: multishot receive with provided buffers (saves an entry per read and makes `Reader` share buffer ownership with the kernel) and zero-copy send (a reply is a few hundred bytes). The milestone's rule holds: engine features that change nothing users see go in when the numbers show they pay, and these do not.


## Not done yet

UDP, provided buffers and multishot receive, zero-copy send, a stat call, a DNS timeout of our own, cancellation of IoRing file operations (`BuildIoRingCancelRequest`), registered files and buffers for IoRing, `FILE_SKIP_COMPLETION_PORT_ON_SUCCESS` to save a packet per immediate completion on Windows, and a timer resolution request on Windows.
