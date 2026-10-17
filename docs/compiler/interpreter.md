# Interpreter

## What it is

`crates/jaic/src/interp` runs IR: `#run` and other compile-time code, `jaic -interp` programs, and the browser playground. Memory is real host memory. Foreign procedures are called natively (`native.rs`) or through a `Host` shim where there is no dynamic linker (wasm).

## How it works

`Interp::call` is the entry from the compiler; `exec` pushes a frame and `run_entered` runs the body with `run_code` (`interp/code.rs`). Procedure values are tagged addresses (`FUNC_TAG`), except `#c_call` procedures under native linking, which are native thunks. Foreign procedures without a native address are tagged `FOREIGN_TAG` and trap with ``foreign procedure `...` is not available here``. `#compiler` procedures are hooks (`Hook`, `run_hook`) handled by `MetaOp` in `build.rs` ([compiler records](../metaprogramming/compiler-module.md)).

### Code form

The IR is shaped for LLVM (every local in a stack slot, every constant its own instruction), which costs one dispatch per instruction. On a procedure's first call `code::build` turns each block into 16-byte `Op`s cached in its `Frame`: slot addresses feeding loads/stores become `LoadFrame`/`StoreFrame`, constants fold into `*Imm` ops, a compare feeding a branch becomes `BranchCmp`, named calls become `Call`, and anything else falls back to `Ir`, which runs the original instruction through `step`. Ops are one flat array run with a program counter (`run_ops`); there is no per-op source location (`Code::loc_at` looks it up for calls and `Ir` ops).

Registers live on the interpreter stack after the slots, so a call allocates nothing. `promote.rs` turns scalar slots whose address is never taken into registers and forwards stores, then dead pure ops are removed. Registers are accessed unchecked: `build` computes `Code::regs` and `enter` reserves exactly that many words. The `Frame` is rebuilt when a procedure body is replaced.

### Traps and limits

- `debug_break()` stops the program with a runtime error (exit 1), so a failed `assert` ends it. Sema turns Runtime_Support's `#asm { int3; }` / `brk #0` into the `DebugBreak` intrinsic, since raw machine code cannot run here.
- Bounds checks are the `BoundsCheck` intrinsic ([pointers and arrays](../language/pointers-and-arrays.md#bounds-checks)); pointer indexing is never checked.
- `block_budget`, when set, counts basic blocks and traps with `execution budget exhausted`. The language server, jailint and the playground use it. A workspace a metaprogram compiles runs on the budget of the interpreter that drives it (`build::step`, `build::finish_all` take `&mut Option<u64>`).
- A `fork()` child that comes back to the compiler instead of `exec`ing is ended with `_exit` (`call_foreign` sets the flag).

### Program arguments and paths

`jaic file.jai -interp -- a b` calls `main` with C-style `argc`/`argv` (`Compiler::run_program_with_args`), so `get_command_line_arguments()` returns `[file.jai, a, b]`. Arguments between `-` and `--` go to the metaprogram. The program runs in the main file's directory. `get_path_of_running_executable` answers with the executable `jaic` would have written (`Interp::run_executable`, `interp/executable_path.rs`) so data found relative to the program resolves correctly. On Windows `GetCommandLineW` returns the program's own arguments (`Interp::run_arguments`).

### Hosts

`Host::foreign` can implement a foreign symbol itself. `SandboxHost` (`interp/sandbox.rs`) is the browser's operating system: a virtual clock, an in-memory file system over a read-only base, UTC `localtime_r`, `strtod`/`strtol`, `sysconf` (one CPU) and the heap. `jaic -interp -os wasm file.jai` runs a program on it natively, the quick way to debug browser-only behaviour.

The sandbox also stands in for calls a metaprogram written for another target makes, because the editor compiles one program for every target:

- **Windows calls** (`sandbox/windows.rs`): last error, UTF-8 wide-string conversion, file and directory calls, handles as sandbox descriptors plus `HANDLE_BASE`.
- **Objective-C stand-ins** (`sandbox/objc.rs`): selectors and classes are interned names, `objc_msgSend` answers 0 except `class`/`superclass`, `dlopen` of system frameworks returns a handle. Real objects still see null.
- **Refused calls**: process creation and shared memory fail with `EPERM` / `ERROR_ACCESS_DENIED` instead of trapping. A trap in the middle of a call that holds a lock leaves the lock held forever.
- Relative `add_build_file` and `import_path` entries resolve against the host's working directory.
- An unknown call traps with "unavailable". To add one, add an arm to `windows_foreign` or the POSIX match in `sandbox.rs`; arguments arrive as raw `u64`s, report errors through `set_errno` and return -1.

### Compile-time code for another target

`#run` code runs on the machine compiling, whatever `-os` says. Stdlib code that chooses constants by target OS (Linux clock IDs, `struct` layouts) therefore disagrees with the host libc when the two differ. Where a call has a portable host equivalent, the stdlib asks the interpreter for it instead: `current_time_monotonic` ends in `compile_time_host_monotonic_nanoseconds` (`stdlib/internal/basic/host-clock.jai`, `Hook::HostMonotonicNanos`) whenever `OS != BUILD_OS` and the code runs at compile time. Anything else that needs the real operating system (forking, pipes, file locking) belongs in `#run,host` or on a host of the target's own OS.

### Native foreign calls

Every call in `native.rs` goes through one C prototype with 8 integer and 8 `double` register arguments plus 16 eight-byte stack slots. Arguments are classified with `jaic::abi` (shared with `jaic-llvm`): scalars fill registers in order, larger structs are copied and passed by pointer (arm64) or `byval` (x86-64), and the return shape (`II`, `IF`, `FF`, ..., or a hidden result pointer) comes from the same classification. On Apple arm64 variadic arguments go on the stack and a stack argument under 8 bytes is an error. Limit: about 18 stack words.

**Callbacks from C** (`native/callbacks.rs`): the value of a `#c_call` procedure is a thunk, a real C function with the same prototype; one family of 64 thunks per return shape, freed with the interpreter. `Interp::proc_value` decides a procedure's value once, from its signature alone, so a value taken before or after the body is lowered is the same thunk. `thunk_funcs` maps a thunk back to its procedure (`Interp::func_of`). A trap in a callback cannot unwind through C, so it prints `error: in a procedure called from C: ...` and exits with status 1. Windows x64 uses 256 assembly stubs and a shared routine (`callbacks/win64.rs`) because Microsoft x64 assigns argument positions. Not covered: variadic callbacks, and on arm64 callbacks returning a struct through a hidden pointer.

**macOS main thread** (`native::main_thread`): `jaic` parks the main thread and runs the compiler on a 1 GiB worker. After the first Objective-C or AppKit call, foreign calls go to the main thread; libSystem and libc++ calls stay on the worker (the round trip made allocation-heavy programs twice as slow).

**Crashes in native code**: see [diagnostics](diagnostics.md#crashes-in-native-code-interpcrashrs).

### Threads

Programs using `Thread` run on a cooperative scheduler, chosen by `Host::cooperative_threads()`.

**Native (`interp/threads.rs`).** Several OS threads exist but only the one holding the baton runs interpreted code. `pthread_create` spawns a thread (256 MiB native stack, 8 MiB value stack) that waits for the baton. Mutexes and condition variables are emulated in the scheduler (`Sched`, behind one mutex in `threads::Shared`) and never passed to libc, so a blocked thread cannot keep others out. The baton moves on join, blocking, sleep, yield, after `PREEMPT_TICKS` blocks (so atomic busy waits progress), or when a native call lasts `NATIVE_SLICE` (1 ms). A thread in C (`call_unlocked`) keeps the baton for short calls and gives it up after the slice. Per-thread state (`ExecState`) is taken out of `Interp` whenever the thread does not hold the baton. When nothing is runnable and no thread is in C, the last blocker gets `deadlock: every thread is blocked`; with thunks that C may call, the report waits `DEADLOCK_GRACE` (1 s).

**Callbacks on C threads.** A callback on a thread already inside `call_unlocked` runs as that thread; on any other thread (Core Audio's render thread) it is adopted into a scheduler slot (`Sched::adopt`), runs on a stack from `callback_stacks`, and may block like a Jai thread. Callbacks take the baton at once from a holder in a native call. Their Rust frames run on C's often small thread stack.

**Win32** (`threads/win32.rs`): `CreateThread`, `WaitFor*`, critical sections, SRW locks, condition variables, semaphores, events and `Sleep` map onto the same scheduler; handles are scheduler objects at `HANDLE_BASE`. Events are also real kernel events (so `OVERLAPPED` I/O works) and waits poll them about every millisecond. Thread ids and TLS are not intercepted.

**Inline threads (sandbox, browser)** (`threads_inline.rs`): wasm32 has no `std::thread`, so all threads take turns on one host thread, each with its own value stack. Because the interpreter is recursive Rust, a thread that must wait suspends: it returns a `TrapKind::Suspended` trap, each interpreted frame saves itself (`SuspFrame`), and the scheduler (`inline_schedule`, in the outermost `Interp::call`) later resumes the innermost frame from its op. Threads switch on blocking calls, yields, or after `POLLS_BEFORE_SWITCH` polls (a failing `compare_and_swap`, `pause`, a busy `trylock`); a busy wait on a plain non-atomic flag never lets others run. Time passes only when no thread can run (`Host::advance_clock`), so sleeping takes no real time and output order is deterministic. Switching works only in the outermost call (`call_nesting == 1`); a nested wait that is not over times out or reports a deadlock. A blocking call is re-executed when its thread resumes, so work it did before suspending (releasing a mutex, computing a deadline) goes in a `Resume` record. Host waits (a page promise, WebGPU) call `jai_sched_yield_for_wait` first so other threads can run.

The `compare_and_swap` intrinsic carries the operand width in bytes as a fourth argument; comparing a `bool` field as 8 bytes would never match.

### Fibers

`fiber_init`, `fiber_switch` and `fiber_free` start a procedure on its own stack and switch between stacks. A module declares them itself with `#intrinsic` and a four-pointer context (`stdlib/Extensions/Io`, [fibers](../extensions/io-engine.md)):

```jai
Fiber_Context :: struct { sp, stack_low, stack_high, stack_dealloc: *void; }
fiber_init   :: (fiber: *Fiber_Context, stack: *void, size: s64, entry: (data: *void) #c_call, data: *void) #intrinsic;
fiber_switch :: (from: *Fiber_Context, to: *Fiber_Context) #intrinsic;
```

`entry` must never return. Native: `crates/jaic/src/fiber.rs` has an assembly routine pair per ABI (`jaic_fiber_switch`, `jaic_fiber_prepare`) emitted by `jaic-llvm` as module-level asm; a switch pushes callee-saved registers (Win64 and arm64 Windows also the TIB stack fields), swaps `sp` and pops. Interpreter: `interp/fibers.rs` gives each fiber a mapping `[guard][native stack][value stack]` and its own `ExecState`; the stack passed to `fiber_init` is ignored so runaway recursion becomes the interpreter's "stack overflow" trap. A trap in a fiber is carried to the root fiber. Fibers trap in the sandbox host and fail to compile for wasm. A new target needs an `asm_*` macro in `fiber.rs` (plus `module_asm` and the `host` module); keep the context offsets (0, 8, 16, 24) in sync with the assembly.

## How to change it

- **New IR instruction**: add it to `inst_vals` in `code.rs` (definitions and uses; a missing use lets a needed definition be dropped), then translate it to an `Op` or let it fall back to `Op::Ir` (needs only an arm in `step`).
- **New op or fold**: keep `Op` at 16 bytes (compile-time assert). An op that folds a register away must call `fold`; one that reads a register `build` did not count must add to `uses`; an op that may be dropped when unread must have no side effect and list its operands in `op_srcs`.
- **Native call shapes**: `call_as` and the shape structs in `native.rs`, plus the matching `Ret` impl and thunk family in `native/callbacks.rs`. They must stay mirror images.
- **Threads**: anything reading or writing `Interp` from a thread must hold the baton; a new way of calling native code must go through `call_unlocked`; never hold the `Shared` lock across a native call or `exec`. A new blocking primitive needs a `Block` variant, a case in `Sched::expire_waits`, and must set `Sched::woke`. For the inline scheduler add a `Wait` variant, its `satisfied` rule and a case in `inline_thread_foreign`.
- **New compiler primitive**: a `MetaOp` in `build.rs` and a bodiless `#compiler` declaration in `stdlib/Compiler`.
- **x86-64 paths on an arm64 Mac**: `cargo build -p jaic-cli --no-default-features --target x86_64-apple-darwin` and run with `arch -x86_64`. Windows stubs: `cargo build -p jaic --target x86_64-pc-windows-msvc` and inspect with `llvm-objdump -d`.
- **Foreign effects**: add a C function that never releases memory to `KEEPS_MEMORY_SYMBOLS` so it stops flushing the memory probe cache.

## Configuration

- `STACK_SIZE` (32 MiB) and `MAX_DEPTH` (20,000 frames) in `interp/mod.rs`; thread constants `PREEMPT_TICKS`, `NATIVE_SLICE`, `DEADLOCK_GRACE`, `THREAD_STACK`, `NATIVE_STACK`; fiber sizes in `interp/fibers.rs`.
- `JAIC_PROFILE=1` counts calls, blocks and instructions per procedure ([benchmarks](../tools/benchmarks.md)).
- `JAIC_COVERAGE=FILE` records each procedure the first time it runs (`path:line name`) and appends the set to FILE at exit ([testing](../tools/testing.md)).

## Dependencies

`ir`, `build.rs`, the platform's dynamic loader, `stdlib/Thread/`, `stdlib/Atomics.jai`. Tests: `tests/compiler/regressions/threads-*.jai`, `tests/compiler/native/fibers/`, `tests/compiler/native/c-callback-threads`, and `crates/jaic-cli/tests/{cli,native}.rs`.
