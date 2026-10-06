# Threads under `jaic run`

## What it is

Programs that use `Thread` (`pthread_create`, mutexes, condition variables, `sleep`, or their
Win32 counterparts on Windows) run in the interpreter on a cooperative scheduler. There are two implementations, chosen by `Host::cooperative_threads()`:

- native `jaic run`: several OS threads exist, but only the one holding the "baton" executes
  interpreted code;
- the sandbox host (browser, `jaic run -os wasm`): no OS threads; see [inline threads](#inline-threads-sandbox-host-browser).

## How it works

- `interp/threads.rs` intercepts the pthread/mutex/cond foreign calls in `call_foreign`
  (`thread_foreign`). `pthread_create` spawns an OS thread (256 MiB native stack, 8 MiB
  interpreter value stack) that waits for the baton.
- The baton moves only when a thread joins, blocks on a mutex or condition variable, sleeps,
  yields, or has executed `PREEMPT_TICKS` basic blocks (`preempt`, called from the `run` loop
  while `Interp.multi` is set). The preemption is what lets busy waits on an atomic progress.
- Per-thread interpreter state (value stack, `sp`, call depth, current line) is swapped in and out
  of `Interp` on a switch. Mutexes and condition variables are emulated in the scheduler, never
  passed to libc, so a blocked thread cannot hold the baton. If every thread is blocked the
  scheduler wakes one with a deadlock error.
- Output order of racing threads is deterministic apart from sleep timing, but differs from a real
  run.
- Threads C starts itself (an audio render thread calling a `#c_call` procedure) are not scheduler
  threads: they run the callback when the interpreter's gate is free, between the baton holder's
  native calls, and cannot block on the scheduler (`block` traps, `yield_now` returns at once). See
  [callbacks from C](interpreter.md#callbacks-from-c).

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
| `Sleep`, `SleepEx`, `SwitchToThread` | Sleep or yield, once a second thread exists. |

Thread ids and TLS are not intercepted: every scheduler thread is a real OS thread, so
`GetCurrentThreadId`, `TlsGetValue` and `GetLastError` already answer per thread.
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
whatever makes the wait satisfiable. Keep all `Interp` access on the baton holder. A new Win32 waitable
object is an `Object` variant with `signaled` and `consume`; call `wake_object_waiters` when it
becomes signaled. Unknown handles must keep falling through to the real procedure. For the inline scheduler add a `Wait` variant, its `satisfied` rule,
and a case in `inline_thread_foreign`.

## Configuration

None. `mod threads` (OS threads) is cfg-gated off on wasm32; `mod threads_inline` is always built.

## Dependencies

`stdlib/Thread/`, `stdlib/Atomics.jai`. Tests: `tests/stdlib/threads-cooperative.jai`,
`tests/stdlib/threads-group-and-condition.jai` (both run natively and in the playground). On Windows
the Win32 side is exercised by `interpreted_threads` and `windows_runtime_program` in
`crates/jaic-cli/tests/native.rs`, which the Windows workflow runs on x64 and arm64.
