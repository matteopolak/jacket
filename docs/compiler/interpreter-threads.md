# Threads under `jaic run`

## What it is

Programs that use `Thread` (`pthread_create`, mutexes, condition variables, `sleep`, or their
Win32 counterparts on Windows) run in the interpreter on a cooperative scheduler. There are two implementations, chosen by `Host::cooperative_threads()`:

- native `jaic run`: several OS threads exist, but only the one holding the "baton" executes
  interpreted code; threads inside C calls, and threads C started that call back into Jai, take
  part too;
- the sandbox host (browser, `jaic run -os wasm`): no OS threads; see [inline threads](#inline-threads-sandbox-host-browser).

## How it works

- `interp/threads.rs` intercepts the pthread/mutex/cond foreign calls in `call_foreign`
  (`thread_foreign`). `pthread_create` spawns an OS thread (256 MiB native stack, 8 MiB
  interpreter value stack) that waits for the baton.
- The scheduler state (`Sched`: threads, baton holder, emulated mutexes, condition variables and
  Win32 objects) sits behind one mutex in `threads::Shared`, made on first use (a thread, a lock,
  or a thunk C may call) and kept in `Interp::shared`. Making it sets `Interp.multi`. Every
  thread that wants the baton waits on its condition variable in `Shared::wait_turn`.
- The baton moves when its holder joins, blocks on a mutex, condition variable, semaphore or
  event, sleeps, yields, has executed `PREEMPT_TICKS` basic blocks (`preempt`, called from the
  `run` loop while `Interp.multi` is set; this is what lets busy waits on an atomic progress), or
  stays in one native call for `NATIVE_SLICE` (1 ms).
- Native calls (`Interp::call_unlocked`): the holder marks itself `in_native` (`enter_native`)
  and calls C *still holding the baton*, so a short call (`malloc`) costs two uncontended locks
  and no switch. A runnable waiter checks every `NATIVE_SLICE` whether the holder is still in the
  same call (`native_calls` counts them) and then takes the baton; the holder becomes
  `Block::Native`, which is neither runnable nor blocked on a scheduler object. When C returns,
  `leave_native` gives the thread the baton back at once if nobody took it, or makes it runnable
  and waits for its turn. So `read`, `accept` or a sleep in C no longer stop the other threads.
  `fork`/`vfork` keep the baton (the child has only the forking thread).
- Per-thread interpreter state (`ExecState`: value stack, `sp`, call depth, `calls`, current
  line) belongs to the thread: it takes it out of `Interp` before it gives the baton up or calls
  C, and puts it back once it has the baton again. A new thread starts from `GThread::start`.
- Mutexes and condition variables are emulated in the scheduler, never passed to libc, so a
  thread blocked on one cannot keep the others out. Waking a thread (`Sched::wake`, `wake_cond`,
  `wake_object_waiters`) sets `Sched::woke`, and `with_sched` then notifies the waiters.
- Deadlocks: when nothing is runnable, nothing waits for a deadline and no thread is in native
  code (`Sched::stuck`; a thread in C may come back and wake the others), the thread that blocked
  last gets ``deadlock: every thread is blocked``. Once C holds a thunk (`Sched::callbacks`), a
  thread the scheduler does not know yet may still call one and wake somebody, so the blocked
  threads wait `DEADLOCK_GRACE` (1 s) first and then report it to the first one. A deadlock that
  involves a thread in native code (the main thread in `pthread_join` on a C thread whose callback
  waits for a lock the main thread holds) hangs, as it does natively.
- Output order of racing threads is deterministic apart from sleep timing and native calls, but
  differs from a real run.

### Callbacks on threads C started

`Shared` is also the `native::Gate` every thunk enters (`Gate::run`; see
[callbacks from C](interpreter.md#callbacks-from-c)). `call_unlocked` records, in the thread-local
`native::calling_out`, which interpreter and scheduler thread is calling C on this OS thread.

- A callback on such a thread (C -> Jai -> C -> Jai, including a forwarded call on the macOS
  main thread) runs *as that thread*: if it still holds the baton it just clears `in_native`,
  otherwise it waits for its turn. Returning to C marks it `in_native` again.
- A callback on any other thread (Core Audio's render thread, a `pthread_create` inside a C
  library) is adopted: `Sched::adopt` gives it a scheduler slot (`GThread::adopted`, reused once
  it returns), it waits for the baton, runs on a value stack from `callback_stacks`, and finishes
  like a Jai thread when the procedure returns. It may block on mutexes, condition variables,
  semaphores, joins and sleeps and is woken like any other thread; `pthread_self` names its slot.
- Callbacks take the baton at once from a holder inside a native call (`urgent`; `enter_native`
  wakes them), and otherwise get it at the holder's next switch or preemption, so a real-time
  callback waits at most `PREEMPT_TICKS` blocks for a busy holder.
- The callback's Rust frames run on C's thread stack, which is often small, so deep recursion
  inside such a callback can overflow it.
- Once the interpreter is dropped (`Shared::close`), a callback stops with ``C called a procedure
  of an interpreter that has finished``, and waiting Jai threads never take the baton again.

### Win32 (Windows hosts)

`interp/threads/win32.rs` (`win32_foreign`, reached from `thread_foreign`) maps the Win32 API that
`stdlib/Thread` and `Basic` call when `OS == .WINDOWS` onto the same scheduler:

| Win32 | Scheduler |
|---|---|
| `CreateThread` | `pthread_create`; the handle is a scheduler object (`HANDLE_BASE` + 4n). `CREATE_SUSPENDED` traps. |
| `WaitForSingleObject(Ex)`, `WaitForMultipleObjects(Ex)`, `GetExitCodeThread`, `CloseHandle` | On scheduler handles: `Block::Object(deadline)`, rechecked whenever a thread finishes, a semaphore is released or an event set. Other handles (processes, files) go to the real call. |
| `InitializeCriticalSection*`, `Enter`/`TryEnter`/`LeaveCriticalSection`, `DeleteCriticalSection`, SRW locks | The recursive mutex emulation, keyed by address. Initialization also runs the real call so the memory is a valid lock for C code. Shared SRW acquisition is exclusive here. |
| `InitializeConditionVariable`, `Wake(All)ConditionVariable`, `SleepConditionVariableCS`/`SRW` | The condition variable emulation. A timeout returns 0 and sets the real last error to `ERROR_TIMEOUT` (1460), which `Thread`'s semaphore checks. |
| `CreateSemaphore*`, `ReleaseSemaphore`, `CreateEventA`/`W`, `SetEvent`, `ResetEvent` | Scheduler objects with a count, or a set flag (auto-reset events clear on a successful wait). |
| `Sleep`, `SleepEx`, `SwitchToThread` | Sleep or yield, once the scheduler exists (`Interp.multi`; POSIX `sleep`, `usleep`, `nanosleep` and `sched_yield` likewise). |

Thread ids and TLS are not intercepted: every scheduler thread, adopted callback threads included,
is a real OS thread, so `GetCurrentThreadId`, `TlsGetValue`, `errno` and `GetLastError` already
answer per thread. A real wait on a handle the scheduler does not own (a process, a file) is an
ordinary native call, so other threads run while it waits.
- The `compare_and_swap` intrinsic carries the operand width in bytes as a 4th argument
  (`emit_intrinsic` in `sema/calls.rs`, `asm.rs`). Comparing a `bool` field as 8 bytes would
  never match and `atomic_swap` would spin forever.

## Inline threads (sandbox host, browser)

`interp/threads_inline.rs` replaces the baton scheduler when the host returns `cooperative_threads() == true`
(`SandboxHost`, `SharedHost`). wasm32 has no `std::thread`, so nothing runs concurrently and a started
thread cannot be suspended: the interpreter is recursive Rust and a blocked thread keeps its Rust frames.

- `pthread_create` only records `(func, argument)`; the thread is `Pending`.
- A pending thread runs to completion *on top of the stack of the thread that blocks first*: at
  `pthread_join`, a contended mutex, `pthread_cond_wait`/`timedwait`, `sleep`/`usleep`/`nanosleep`, `sched_yield`,
  and every `PREEMPT_TICKS` basic blocks while pending threads exist (so a busy wait on an atomic progresses).
  The set of started threads is therefore a stack (`levels`), main at the bottom.
- `wait_until` is the single blocking primitive. If the wait is not satisfied and nothing is pending:
  a timed wait times out at once and advances the virtual clock (`Host::advance_clock`); otherwise it looks for
  the nearest thread lower on the stack whose own wait is over and *abandons* every thread above it (they are
  unwound with a special trap, their mutexes released, and they count as finished); with no such thread it
  reports `deadlock: every thread is blocked`.
- Abandoning is what lets a `Thread_Group` worker, parked on its semaphore with no work, hand control back to
  the main thread that polls for results. Limitation: an abandoned thread never resumes, so work added to a
  group *after* its workers were abandoned is never processed (the program ends in a deadlock error or keeps
  polling). `thread_is_done` (stdlib/Thread/primitives.jai) asks `pthread_tryjoin_np` on WASM so that
  `shutdown` still succeeds for abandoned workers.
- Output order is deterministic. Sleeping never takes real time.
- `Thread` on WASM uses the Linux x86-64 POSIX layouts (`POSIX_THREADS` includes `OS == .WASM`).

## How to change it

New blocking primitives need a `Block` variant and a case in `Sched::expire_waits` (timed waits) or
whatever makes the wait satisfiable, which must set `Sched::woke` (or notify `Shared::cv`) so the
woken OS thread notices. A timed one also needs `Block::deadline`, which `stuck` and `wait_turn`
read. Keep all `Interp` access on the baton holder, and keep a thread's `ExecState` out of `Interp`
whenever it does not hold the baton (`block`, `yield_now`, `call_native`). Code that can wait
outside the scheduler (any new way of calling C) must go through `call_unlocked`, or other threads
and callbacks stop while it waits. Never hold the `Shared` lock across a native call or `exec`.
Tuning: `NATIVE_SLICE` trades switch latency against wakeups of runnable threads (each ticks at
that period while another thread holds the baton). A new Win32 waitable
object is an `Object` variant with `signaled` and `consume`; call `wake_object_waiters` when it
becomes signaled. Unknown handles must keep falling through to the real procedure. For the inline scheduler add a `Wait` variant, its `satisfied` rule,
and a case in `inline_thread_foreign`.

## Configuration

None at run time. Constants in `threads.rs`: `PREEMPT_TICKS` (20,000 blocks), `NATIVE_SLICE`
(1 ms), `DEADLOCK_GRACE` (1 s), `THREAD_STACK` (8 MiB value stack), `NATIVE_STACK` (256 MiB).
`mod threads` (OS threads) is cfg-gated off on wasm32; `mod threads_inline` is always built.

## Dependencies

`stdlib/Thread/`, `stdlib/Atomics.jai`, `interp/native/callbacks.rs` (`Gate`, `calling_out`).
Tests: `tests/stdlib/threads-cooperative.jai`,
`tests/stdlib/threads-group-and-condition.jai` (both run natively and in the playground);
`c_thread_callbacks_block_on_jai_threads` in `crates/jaic-cli/tests/native.rs`
(`tests/native/c-callback-threads`: a C thread's callback waiting on a Jai mutex, condition
variable, join and sleep, nested callbacks on the main thread, a C thread and a Jai thread, a Jai
thread running while another waits in C, and both kinds of deadlock), checked against the native
build. On Windows
the Win32 side is exercised by `interpreted_threads` and `windows_runtime_program` in
`crates/jaic-cli/tests/native.rs`, which the Windows workflow runs on x64 and arm64.
