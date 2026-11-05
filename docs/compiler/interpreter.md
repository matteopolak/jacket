# Interpreter

`crates/jaic/src/interp` runs IR: `#run` and other compile-time code, `jaic -interp` programs, and the browser playground. Memory is real host memory. Foreign procedures are called natively (`native.rs`) or through a `Host` shim where there is no dynamic linker (wasm).

```sh
jaic file.jai -interp -- a b     # main gets argc/argv: [file.jai, a, b]
```

Arguments between `-` and `--` go to a metaprogram. The program runs in the main file's directory.

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
- `block_budget` counts loop back-edges, calls and `switch` jumps and traps with `execution budget exhausted`. The language server, jailint and the playground set it; `jaic` does too unless `-run_budget 0` ([limits](compile-time-limits.md)).
- String literals and other `read_only` globals live in one page-protected region, so a write traps instead of changing every equal literal.

### Hosts

`Host::foreign` can implement a foreign symbol itself. `SandboxHost` (`interp/sandbox.rs`) is the browser's operating system: a virtual clock, an in-memory file system over a read-only base, one CPU and the heap. `jaic -interp -os wasm file.jai` runs a program on it natively.

The editor compiles one program for every target, so the sandbox also stands in for Windows and Objective-C calls a metaprogram makes. Process creation and shared memory fail with `EPERM` instead of trapping. An unknown call traps with "unavailable".

To add a call: add a member to the API's enum in `sandbox/symbols.rs` (one place names every symbol; a symbol must not appear in two APIs) and an arm in the area's `*_call`. Arguments arrive as raw `u64`s; report errors through `set_errno` and return -1.

### Compile-time code for another target

`#run` runs on the compiling machine whatever `-os` says. In a cross build, plain `#run` is evaluated by a second compiler with the host `OS` and `CPU` (the host twin) and the results are converted for the target ([compile-time host execution](compile-time-host-execution.md)).

### Native foreign calls

Every call in `native.rs` goes through one C prototype with 8 integer and 8 `double` register arguments plus 16 stack slots. Arguments are classified with `jaic::abi` (shared with `jaic-llvm`).

- A `#c_call` procedure's value is a real C thunk (`native/callbacks.rs`): 64 per return shape, freed with the interpreter.
- On macOS `jaic` parks the main thread and runs the compiler on a 1 GiB worker. After the first AppKit call, foreign calls go to the main thread; libc calls stay on the worker.
- A fault in native code is caught and reported ([diagnostics](diagnostics.md)).

### Threads

Programs using `Thread` run on a cooperative scheduler, chosen by `Host::cooperative_threads()`. Mutexes and condition queues share one model (`interp/thread_sync.rs`); the schedulers differ only in how a blocked thread gives up its turn.

- **Native** (`threads.rs`): several OS threads exist but only the one holding the baton runs interpreted code. Callbacks from foreign threads are adopted into a scheduler slot and may block like a Jai thread.
- **Win32** (`threads/win32.rs`): `CreateThread`, `WaitFor*`, critical sections and friends map onto the same scheduler.
- **Inline** (`threads_inline.rs`, wasm): all threads take turns on one host thread; a thread that must wait suspends with a `Suspended` trap and its frames save themselves.
- `compare_and_swap` carries the operand width as a fourth argument.

Known host semaphore waits give up the native baton before entering C, so a runnable peer can supply the signal without waiting for the short-call grace. The host SDK address, a core system-library binding and the actual loader provider must match (`native/waits.rs`): the Mach header belonging to the already loaded kernel-library handle on macOS, or the already loaded libc link map on glibc Linux. Other loader providers retain the conservative policy; proofs are prepared once before native guest execution, so classification does not alter a guest’s pending loader error. A user library with the same symbol spelling and opaque calls retain the existing timed-steal policy. Fork calls retain exclusive execution. Arguments and the caller’s execution state stay parked until it reacquires the baton, including on a native error return. A nested C callback reacquires normally; returning from it to C resumes the conservative short-call policy.

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
