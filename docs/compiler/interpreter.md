# Interpreter

`crates/jaic/src/interp` runs IR: `#run` and other compile-time code, `jaic -interp` programs, and the browser playground. Memory is real host memory. Foreign procedures are called natively (`native.rs`) or through a `Host` shim where there is no dynamic linker (wasm).

```sh
jaic file.jai -interp -- a b     # main gets argc/argv: [file.jai, a, b]
```

Arguments between `-` and `--` go to a metaprogram. The program runs in the main file's directory.

## Data layout

`Interp::new(host, layout)` takes the program's `DataLayout`. Compiler reads and writes of program memory (`read_u64`, `read_ptr`, `read_view`, `put_ptr`) go through it. Targets with the host's pointer width and byte order use host addresses. A narrower target (wasm32) runs in an **address window** (`interp/window.rs`): one 3 GiB host region shared by the whole process, where a program address is the offset from the region's base, so a 4-byte pointer holds every address the program can name. The low 64 KiB is never mapped, so null and small integers trap. The next 256 MiB is a pool of 16 read-only slots (16 MiB each) for compile-time constants (`ReadOnlyData`); the rest is the heap, committed 1 MiB at a time with a free list.

- `Interp::set_host` attaches the window to any host, so `SandboxHost` and a native host see the same addresses. `host_addr`/`program_addr` convert; loads and stores add the base.
- Procedure values are `0xE000_0000 | id` (interpreted) and `0xF000_0000 | id` (foreign), outside the heap.
- Data the compiler hands to compile-time code (syntax strings, `Compiler` records) must live in the window: use `keep_narrow`, `zeroed_block` and `Interp::keep`, not host allocations.
- A big-endian program's loads, stores and atomics are swapped. Which helper moves which value is in [interpreter memory model](interpreter-memory-model.md).

## How it works

`Interp::call` is the entry from the compiler; `run_code` (`interp/code.rs`) runs a body.

### Code form

- The IR is shaped for LLVM (every local in a stack slot), so on a procedure's first call `code::build` turns each block into 16-byte `Op`s: slot loads and stores become register ops, constants fold, and compare-and-branch pairs fuse.
- `promote.rs` turns scalar slots whose address is never taken into registers; `code/sink.rs` drops dead local stores and moves constant setup into the branch that uses it.
- Registers live on the interpreter stack, so a call allocates nothing.
- Locations are positional: a `Loc` op sets the location of every op after it, and blocks without one inherit from the first block that branches to them.
- The program's last-error value (`GetLastError`, `errno`) is shared with the interpreter and preserved across the interpreter's own system calls.

### Traps and limits

- `debug_break()` stops the program with a runtime error, so a failed `assert` ends it.
- Bounds checks are the `BoundsCheck` intrinsic ([pointers and arrays](../language/pointers-and-arrays.md#bounds-checks)); pointer indexing is never checked.
- `step_budget` counts steps: loop back-edges, calls and `switch` jumps and traps with `execution budget exhausted`. The language server, jailint and the playground set it; `jaic` does too unless `-run_budget 0` ([limits](compile-time-limits.md)).
- String literals and other `read_only` globals live in one page-protected region, so a write traps instead of changing every equal literal.

### Hosts

`Host::foreign` can implement a foreign symbol itself. `SandboxHost` (`interp/sandbox.rs`) is the browser's operating system: a virtual clock, an in-memory file system over a read-only base, one CPU and the heap. `jaic -interp -os wasm file.jai` runs a program on it natively. Its paths start at `/`; on a Windows host it names a drive's files `/C:/...`, so `C:/a` and `/C:/a` are one file, and `host_path` turns a sandbox path back into the host's.

The editor compiles one program for every target, so the sandbox also stands in for Windows and Objective-C calls a metaprogram makes. Process creation and shared memory fail with `EPERM` instead of trapping. An unknown call traps with "unavailable".

To add a call: add a member to the API's enum in `sandbox/symbols.rs` (one place names every symbol; a symbol must not appear in two APIs) and an arm in the area's `*_call`. Arguments arrive as raw `u64`s; report errors through `set_errno` and return -1.

### Compile-time code for another target

`#run` runs on the compiling machine whatever `-os` says. In a cross build, plain `#run` is evaluated by a second compiler with the host `OS` and `CPU` (the host twin) and the results are converted for the target ([compile-time host execution](compile-time-host-execution.md)).

Compile-time code that stays on the target compiler was built against the target's system API, and native calls have to match it:

- **Windows target on another host**: Win32 file, time and memory calls are answered over real host files (`windows_native.rs`, sharing `windows.rs` with the sandbox).
- **A target whose OS is not the host's**: the C libraries (`c`, `libc`, `m`, `pthread`, `dl`) differ in symbols (`open64`, `stat$INODE64`), records (`struct stat`, `struct dirent`), flags (`O_CREAT` is `0x40` on Linux and `0x200` on macOS), `errno` numbers and system call numbers. `reach` in `foreign.rs` decides each procedure once:
  - `SAME_IN_EVERY_C_LIBRARY` (memory, strings, numbers every library sizes alike, `getenv`, exit) runs natively everywhere, the Windows C runtime included.
  - Between Linux and macOS, `SAME_IN_POSIX_C_LIBRARIES` (`long` numbers, descriptor reads and writes, `FILE*` streams, path calls such as `mkdir`, `mkdtemp` and `rename`) runs natively too, and `POSIX_RENAMED` maps a name only one library has (`fopen64`, `realpath$DARWIN_EXTSN`) to the other's name for the same procedure.
  - Between Linux and macOS, `posix/native.rs` answers in the target's ABI over host files: `errno` (a cell of the program's own, in its numbers), `open`/`openat` flags, the `stat` family (Darwin's `$INODE64` names, glibc's `__stat64_time64`, the stat system calls), `opendir`/`readdir`/`closedir` and `strerror`. Around every native call of the target's C library the program's `errno` is handed to the host's and taken back (`last_error::translate_next`).
  - Everything else (`uname`, `mmap`, `fcntl`, a Windows C runtime `fopen`) is refused with an `unavailable` error naming the call.

  Other libraries (libclang, a framework, a program's own `#library`) keep their native lookup.

The per-OS numbers and records live in `interp/posix.rs` and `interp/posix/records.rs`, shared with the sandbox's libc, which answers macOS programs in Darwin's flags and error numbers the same way. Values pass between the two ABIs as neutral ones: an `Open` for open flags, Linux numbers for errors. `posix/tests.rs` compiles the stdlib's generated POSIX declarations for every target and asserts each table value, so a table that drifts from the declarations fails there.

To answer another call: add it to `PosixFn` in `posix/native.rs`, and the constants or record it needs to `PosixAbi` or `records.rs` with an assertion in `posix/tests.rs`. Only a procedure whose arguments, results and effects mean the same in both libraries belongs in a `SAME_IN_*` list or `POSIX_RENAMED`.

### Native foreign calls

Every call in `native.rs` goes through one C prototype with 8 integer and 8 `double` register arguments plus 16 stack slots. Arguments are classified with `jaic::abi` (shared with `jaic-llvm`).

- A foreign procedure's call plan (`native::Plan`, kept in `ForeignPlan`) is made on its first call: the C ABI classification of its result and aggregate arguments, its scheduling kind and, on macOS, whether the callee needs the main thread. Later calls only marshal arguments. `foreign_flags` also records once per symbol whether `call_foreign` must look at its name (thread, process-ending and executable-path calls: `is_special`), so an ordinary C call does no name matching, and a `Host` whose `answers_foreign` is false (`NativeHost`) is not asked. A call whose arguments and result are all scalars that fit the registers (`Plan::scalar`) skips the stack slots and takes `register_call`. Change how a signature is classified in `Plan::new`, not in the call path; `native::call` makes a throwaway plan for calls through a procedure pointer.
- A `#c_call` procedure's value is a real C thunk (`native/callbacks.rs`): 64 per return shape, freed with the interpreter.
- On macOS `jaic` parks the main thread and runs the compiler on a 1 GiB worker. After the first AppKit call, foreign calls go to the main thread; libc calls stay on the worker.
- A fault in native code is caught and reported ([diagnostics](diagnostics.md)).

### Threads

Programs using `Thread` run on a cooperative scheduler, chosen by `Host::cooperative_threads()`. Thread slots and handles, mutexes and condition queues share one model (`interp/thread_sync.rs`); the schedulers differ only in how a blocked thread gives up its turn.

A thread's slot is free again once the thread has finished and its handle is gone (`pthread_join`, `pthread_detach`, `CloseHandle`). A handle is the slot plus one with the slot's generation in the high 32 bits, so one kept past that is rejected (`EINVAL`) instead of naming the thread that took the slot. A mutex held by a thread that ended stays held, by nobody; `Mutexes` tracks what each thread owns, so ending a thread costs what it held. Compile-time code cannot suspend, so the inline scheduler's tests run a program (`threads_inline_tests.rs`) where the native ones use `#assert`.

- **Native** (`threads.rs`): several OS threads exist but only the one holding the baton runs interpreted code. Callbacks from foreign threads are adopted into a scheduler slot and may block like a Jai thread.
- **Win32** (`threads/win32.rs`): `CreateThread`, `WaitFor*`, critical sections and friends map onto the same scheduler.
- **Inline** (`threads_inline.rs`, wasm): all threads take turns on one host thread; a thread that must wait suspends with a `Suspended` trap and its frames save themselves.
- `compare_and_swap` carries the operand width as a fourth argument.

Known host semaphore waits give up the native baton before entering C, so a runnable peer can supply the signal without waiting for the short-call grace. The host SDK address, a core system-library binding and the actual loader provider must match (`native/waits.rs`): the Mach header belonging to the already loaded kernel-library handle on macOS, or the already loaded libc link map on glibc Linux. Other loader providers retain the conservative policy; proofs are prepared once before native guest execution, so classification does not alter a guest’s pending loader error. A user library with the same symbol spelling and opaque calls retain the existing timed-steal policy. Fork calls retain exclusive execution. Arguments and the caller’s execution state stay parked until it reacquires the baton, including on a native error return. A nested C callback reacquires normally; returning from it to C resumes the conservative short-call policy.

Thread groups notify the selected worker and stealing peers when work becomes available. Appending to a queue only publishes its entry: completed queues have no semaphore waiter, so they do not post notifications. `get_completed_work` polls those queues; `wait_for_all` uses the group’s outstanding-work condition. Keep these wake policies in `stdlib/internal/thread/group.jai` when changing queue ownership. The public counting `Semaphore` and available-work broadcasts retain their existing behavior.

### Fibers

`fiber_init`, `fiber_switch` and `fiber_free` start a procedure on its own stack and switch between stacks ([fibers](fibers.md)).

## How to change it

- New IR instruction: add it to `inst_vals` in `code.rs` (definitions and uses; a missing use lets a needed definition be dropped), then translate it to an `Op` or fall back to `Op::Ir`.
- New op: keep `Op` at 16 bytes. An op that folds a register away calls `fold`; one reading an uncounted register adds to `uses`.
- Native call shapes: `call_as` in `native.rs` and the matching thunk family in `native/callbacks.rs` must stay mirror images.
- Threads: anything touching `Interp` from a thread holds the baton; never hold the `Shared` lock across a native call. A new blocking primitive needs a `Block` variant and a case in `Sched::expire_waits`.
- Extend known native waits only with host SDK identity checks and a system-library binding; keep unknown calls conservative. Preserve caller identity before handing the baton on, and cover no-peer, cancellation, native return values and nested callbacks.
- New compiler operation: an `Operation` in `build/public_api/mod.rs`, a handler, the engine call in `build/engine.rs`, and a bodiless `#compiler` declaration in `stdlib/Compiler`.
- Writing a global from the compiler: use `initialize_global`; a plain write into a `read_only` global faults. New ways to write memory call `check_writable`.
- x86-64 on an arm64 Mac: `cargo build -p jaic-cli --no-default-features --target x86_64-apple-darwin`, run with `arch -x86_64`.

## Configuration

- `STACK_SIZE` (32 MiB) and `MAX_DEPTH` (20,000 frames) in `interp/mod.rs`; thread and fiber sizes in `threads.rs`, `fibers.rs`.
- `-profile` counts calls, blocks and instructions per procedure ([benchmarks](../tools/benchmarks.md)); `-coverage` writes line counts at exit ([coverage](../tools/coverage.md)).

## Dependencies

`ir`, `build.rs`, the platform's dynamic loader, `stdlib/Thread/`, `stdlib/Atomics/`. Tests: `tests/compiler/regressions/threads-*.jai`, `tests/compiler/native/fibers/`, `crates/jaic-cli/tests/{cli,native}.rs`.
