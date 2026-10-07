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
  last gets ``deadlock: every thread is blocked``. Once C may hold a thunk (`Sched::callbacks`), a
  thread the scheduler does not know yet may still call one and wake somebody, so the blocked
  threads wait `DEADLOCK_GRACE` (1 s) first and then report it to the first one. C may hold a
  thunk once a foreign call runs after it was made (`Interp::hand_thunks_to_c`: as an argument or
  through memory, C can only have seen it from a call). A thunk passed to an intercepted
  `pthread_create` or `CreateThread` is dropped from `Interp::unseen_thunks` first: the scheduler
  runs it, so `Thread` programs get their deadlock reports at once. A thunk written to memory
  that a C thread started earlier reads without another foreign call would escape this; nothing
  in the corpus does that. A deadlock that
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
(`SandboxHost`, `SharedHost`). wasm32 has no `std::thread`, so all threads take turns on the one host thread,
each on a value stack of its own (`THREAD_STACK`, 8 MiB; the first thread keeps `Interp::call`'s).

- `pthread_create` only records `(func, argument)`; the thread is `Pending` until the scheduler picks it.
- The interpreter is recursive Rust, so a thread cannot be paused in place. A thread that must wait *suspends*:
  it returns a `TrapKind::Suspended` trap, and each interpreted frame it unwinds through is saved by `exec`
  (`SuspFrame`: the block and op it stopped at, its value registers and what `exec` restores on return; the
  frame's memory stays on the thread's value stack). `run_code` records the stopping point in
  `Interp::suspend_at`. The trap ends at the scheduler loop, `inline_schedule`, which runs in the outermost
  `Interp::call`.
- `inline_schedule` saves the thread's frames and `ExecState`, picks the next thread and either starts it or
  resumes it: `resume_frames` runs the innermost saved frame again from its op (the blocking foreign call, which
  now finds its wait over) and hands each frame's results to its caller, which goes on after the call.
  A blocking call whose progress the scheduler cannot recompute keeps a `Resume` record on its thread: a
  condition wait that already released its mutex (`Cond`, then `Relock` while it takes the mutex back), a
  sleep's deadline, a yield. Mutex locks and joins simply try again.
- Threads switch when one blocks (`pthread_join`, a contended mutex, `pthread_cond_wait`/`timedwait`,
  `sleep`/`usleep`/`nanosleep`), yields (`sched_yield`), or has polled `POLLS_BEFORE_SWITCH` times (so a busy wait
  on an atomic progresses). A poll (`inline_poll`) is a `compare_and_swap` that fails or writes the value already
  there (`atomic_read`, a spin on a taken lock), a `pause`, or a `trylock`/`tryjoin` that finds the mutex or
  thread busy. A busy wait that polls nothing (a plain, non-atomic flag) never lets other threads run.
- Picking (`inline_pick`), round-robin from the thread that stopped: first a thread that is pending or whose wait
  is over; else the clock jumps to the earliest deadline (timed waits and sleeps, `Host::advance_clock`) and that
  thread's wait times out; else a thread that only yielded. So time passes only when no thread can run, and a
  thread spinning on a flag that another sets after a sleep still sees it. Nothing to pick is
  `deadlock: every thread is blocked`; `wait_until` reports it at once when the blocking thread can see that
  nothing else could ever run.
- `wait_until` is the single blocking primitive: it returns when the wait holds or its deadline passed, and
  otherwise suspends the thread (or, when no other thread can run before this deadline, advances the clock).
- Threads switch only inside the outermost `Interp::call` (`call_nesting == 1`), where every Rust frame of a
  thread is an interpreted frame that can be saved. In a nested call (a C callback, a compiler hook running Jai)
  a wait that is not over times out at once if it is timed and is otherwise reported as a deadlock.
- A switch can happen anywhere a thread polls, also inside a spin lock built on `compare_and_swap` (the default
  allocator's ledger, `Runtime_Support`'s output lock) or while it holds a mutex: the holder simply runs again
  later. When the first thread returns, the program ends, whatever the others are doing.
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
and a case in `inline_thread_foreign`. A blocking call there must be safe to run again after it suspends:
it is re-executed from the start when its thread resumes, so anything it did before suspending that the
retry cannot see (releasing a mutex, computing a deadline) goes in a `Resume` record. A Rust frame that is
not an `exec`/`run_code` frame and can see a suspension must raise `call_nesting` around it.

## Configuration

None at run time. Constants in `threads.rs`: `PREEMPT_TICKS` (20,000 blocks), `NATIVE_SLICE`
(1 ms), `DEADLOCK_GRACE` (1 s), `THREAD_STACK` (8 MiB value stack), `NATIVE_STACK` (256 MiB).
`mod threads` (OS threads) is cfg-gated off on wasm32; `mod threads_inline` is always built.

## Dependencies

`stdlib/Thread/`, `stdlib/Atomics.jai`, `interp/native/callbacks.rs` (`Gate`, `calling_out`).
Tests: `tests/stdlib/threads-cooperative.jai`,
`tests/stdlib/threads-group-and-condition.jai`, `tests/stdlib/threads-worker-requests.jai` (a worker serving
repeated requests from a semaphore) and `tests/stdlib/file-async.jai` (all run natively and in the playground);
`sandbox_threads_wake_waiters_and_report_deadlocks` in `crates/jaic-cli/tests/cli.rs` (deadlocks and timed waits
under `-os wasm`);
`c_thread_callbacks_block_on_jai_threads` in `crates/jaic-cli/tests/native.rs`
(`tests/native/c-callback-threads`: a C thread's callback waiting on a Jai mutex, condition
variable, join and sleep, nested callbacks on the main thread, a C thread and a Jai thread, a Jai
thread running while another waits in C, and both kinds of deadlock), checked against the native
build. On Windows
the Win32 side is exercised by `interpreted_threads` and `windows_runtime_program` in
`crates/jaic-cli/tests/native.rs`, which the Windows workflow runs on x64 and arm64.
