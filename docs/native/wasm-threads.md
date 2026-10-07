# Threads in WASI builds (green threads)

## What it is

WASI preview 1 gives a program one thread and no call to start another, and wasm code cannot switch stacks. A `jaic build -os wasm` program that uses the Thread module still runs its threads: Wasi_Runtime runs them as green threads, one at a time on the host's thread, and jaic's LLVM backend rewrites the procedures that may block so a thread can be set aside and resumed later. The program prints what it prints natively; it just never runs two threads at once.

This is only for native WASI builds. `jaic run -os wasm` and the browser engine interpret the program and schedule its threads themselves ([threads under `jaic run`](../compiler/interpreter-threads.md)).

## How it works

**Scheduling** (`stdlib/Extensions/Wasi_Runtime/threads.jai`). The pthread calls the Thread module makes are answered by Wasi_Runtime:

| Call | Green thread behavior |
| --- | --- |
| `pthread_create` | adds a thread to the run list, with a 1 MiB frame stack; it first runs when the creating thread blocks |
| `pthread_join`, `pthread_tryjoin_np` | waits until the thread is done, then frees it |
| `pthread_mutex_lock`/`trylock`/`unlock` | owner and depth in the first 16 bytes of the `pthread_mutex_t`; every mutex is recursive; a contended lock blocks |
| `pthread_cond_wait`/`timedwait`/`signal`/`broadcast` | the waiter releases the mutex, blocks on the condition's address, takes the mutex back; `signal` wakes the longest waiter |
| `nanosleep` (so `sleep_milliseconds`), `usleep`, `poll` with no descriptors | a timed wait for nothing, so other threads run meanwhile |

When `green.enabled`, `_start` calls `run_threads` (`jaic_wasi_run_threads`), which runs `main` as the first thread and then loops: pick the next runnable thread after the last one (round robin), call its entry, and on return either mark it done (waking its joiners) or, if it unwound, leave it blocked. With nothing runnable it sleeps until the earliest timed wait ends. When every thread waits with no time limit, they are deadlocked: the one that blocked last wakes with `EDEADLK` (`Semaphore` `wait_for` then returns `.ERROR`; `thread_deinit` asserts). `main` returning exits the process, as natively.

**Unwinding** (`crates/jaic-llvm/src/green.rs`, the technique of Binaryen's Asyncify applied to jaic's IR). A blocking call ends in `block_running`, which marks the thread blocked and calls `green_suspend` (`jaic_wasi_suspend`). That sets `green.state` to unwinding and returns, and every rewritten procedure on the way back to the scheduler sees the state after its call, stores which call it was in its frame, and returns. To resume the thread the scheduler sets the state to rewinding and calls the entry again: each rewritten procedure takes the same frame back and jumps straight to the call it left from, down to `green_suspend`, which sets the state back to running and returns into `block_running`.

`green::instrument` runs before LLVM lowering, on a copy of the program, when the program has Wasi_Runtime's `jaic_wasi_*` exports and reaches `pthread_create` (a call, an address taken, or a relocation):

1. **Which procedures may block.** `green_suspend`, then (to a fixed point) every procedure that calls one that may block. An indirect call may block when an address-taken procedure with the same lowered signature (parameter and result classes, convention) may; `#foreign` calls that Wasi_Runtime defines count as calls to that definition. `run_threads` and `_start` are where unwinding stops, so they are never rewritten.
2. **Stretches.** Each block starts a stretch, and so does each call that may block. A value used in a different stretch than the one defining it (and every parameter that is used at all) gets an 8-byte place in the frame: stored after its definition, loaded once at the start of each stretch that uses it. Constants and `SlotAddr`/`GlobalAddr`/`FuncAddr`/`ForeignAddr` results move to the new entry block instead.
3. **Frame.** Slots no longer become LLVM allocas: the entry block takes a frame from `green.top` (aligned to the largest slot alignment, at least 16), traps through `jaic_wasi_frames_exhausted` ("thread stack overflow") past `green.limit`, and lays out `[resume point: 8][slots][stored values]`. A normal return gives the frame back; an unwinding return keeps it. Frames sit at the same addresses when rewound, because the same procedures take them in the same order from the same start.
4. **Calls that may block** start a block of their own (the resume target). After the call, if `green.state` is unwinding, the procedure stores the call's 1-based index at the frame's start and returns zeros. On entry, if the state is rewinding, a `Switch` on that index jumps to the call, which reloads its arguments from the frame and calls again.

Debug info keeps lines and scopes for rewritten procedures but drops their variables (they live in the frame, not in slots).

**Example.** The tour's producer and consumer (`examples/tour/threads/threads.jai`): `main` waits on `filled_slots`, unwinds, and the producer runs, fills the four slots, and blocks on `free_slots`; `main` is rewound into `wait_for`, takes the four items, blocks again, and so on. The output, including "the producer had to wait for a free slot 2 times", matches a native build.

## How to change it

- **A new blocking call** in Wasi_Runtime: block with `block_running(object, deadline)` and wake with `wake_one`/`wake_all(object, ...)`. Anything that calls `block_running` may block, and the analysis finds it. Keep a path for `!green.enabled` (no thread can exist): there, a wait nothing can end returns `EDEADLK` and a timed one sleeps.
- **Never block below an unrewritten frame.** The scheduler and `_start` are not rewritten; neither is a procedure whose blocking is invisible to the analysis. `Program::check_failed` is called by the backend, not from IR, so it must not block.
- **`Green_Control`'s layout** is shared: the field offsets are constants in `green.rs` (`TOP`, `LIMIT`, `ENABLED`, state at 0) and the struct in `threads.jai`. `instrument` finds the global through `jaic_wasi_green_control`, sets `enabled` in its initial bytes, and calls `jaic_wasi_frames_exhausted` on overflow.
- **Frame stack sizes**: `MAIN_FRAME_BYTES` (8 MiB, from `heap_take`) and `THREAD_FRAME_BYTES` (just under 1 MiB, from `malloc`) in `threads.jai`. Only the slots and stored values of rewritten procedures use them; everything else uses the wasm stack, which is empty whenever threads switch.
- Gotchas:
  - Nothing preempts: a thread that busy-waits (a loop on `atomic_read` or a spin lock) without blocking keeps the turn forever (`threads-switch-while-locked` is skipped for `wasm-native`).
  - A `#c_call` procedure that may block and takes a struct by value gets a pointer to its caller's copy on the wasm stack, which is gone after a resume. Wasi_Runtime's own blocking procedures take only scalars and pointers.
  - Rewritten procedures keep their locals in memory LLVM cannot promote to registers, so they are slower; only procedures that may block are rewritten, and only in programs that start threads.

## Configuration

None: the rewrite is automatic for WASI builds that reach `pthread_create`. Programs that never start a thread are built as before, and their blocking calls behave as with one thread (`EDEADLK` for an endless wait).

## Dependencies

`crates/jaic-llvm/src/green.rs` (called from `emit_object`/`emit_objects` in `lib.rs`), `jaic::ir`, `stdlib/Extensions/Wasi_Runtime/threads.jai` and `module.jai` (`_start`, `nanosleep`), the Thread module's POSIX path (`stdlib/Thread/primitives.jai`), and WASI's `clock_time_get` and `poll_oneoff`. Tests: the `threads-*`, `atomics-contention` and `file-async` stdlib runtime tests in `wasm-native` mode (`tools/stdlib_runtime.py`), and the tour as a WASI command (`wasm_tour_runs_as_a_wasi_command` in `crates/jaic-cli/tests/wasm_target/`).
