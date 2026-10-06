# jaic interpreter

## What it is

`crates/jaic/src/interp` runs IR: `#run` and other compile-time code, `jaic run` programs, and the browser playground. Memory is real host memory. Foreign procedures are called natively (`native.rs`), or through a `Host` shim where there is no dynamic linker (wasm).

## How it works

`Interp::call` is the entry from the compiler; `exec`, `run` and `step` interpret functions. Procedure values are tagged addresses (`FUNC_TAG`). Foreign procedures without a native address are tagged `FOREIGN_TAG` and trap with `foreign procedure '...' is not available here` when called.

`#compiler` procedures of the `Compiler` module are hooks (`Hook`, `run_hook`) handled by `MetaOp` in `build.rs`. `codes` mirrors the compiler's `Code` values so `compiler_get_nodes` can export them, and `made_codes` lists codes created by `compiler_get_code`; see [compiler records](../metaprogramming/compiler-records.md).

### Traps and checks

- `debug_break()` stops the program with a runtime error (exit 1), so a failed `assert` ends it as in Jai. Runtime_Support's `debug_break` is `#asm { int3; }` on x64 and `#bytes` `brk #0` on arm64; sema turns exactly those into the `DebugBreak` intrinsic, since raw machine code can't run here.
- Bounds checks are the `BoundsCheck` intrinsic; see [pointers and arrays](../language/pointers-and-arrays.md#bounds-checks). Pointer indexing is never checked.
- `block_budget`, when set, counts basic blocks and traps with `execution budget exhausted`. The language server and playground use it.

### Program arguments

`jaic run file.jai -- a b` calls the exported `main` with a C-style `argc`/`argv` (`Compiler::run_program_with_args`; the strings are leaked host memory), so `get_command_line_arguments()` returns `[file.jai, a, b]`. Without `--`, `argc` is 0. Arguments after `-` and before `--` go to the metaprogram (`compiler_get_command_line`). `-os wasm` runs in the sandbox, which has virtual memory, and passes none.

The program runs in the main file's directory, so relative paths among its arguments resolve there, not where `jaic` was started.

### Hosts

`Host::foreign` can implement a foreign symbol itself. `SandboxHost` (`interp/sandbox.rs`) is the browser's operating system: a virtual clock, an in-memory file system over a read-only base (`FileSystem::list_dir`; `FILE*`, descriptors, `DIR*`, `stat`, `getcwd`, `/tmp`; `chmod` succeeds on existing paths because the overlay keeps no permission bits), `localtime_r` in UTC, the `strtod`/`strtol` family, `sysconf` (one CPU), and the heap. `jaic run -os wasm file.jai` runs a program on it natively, which is the quick way to debug browser-only behaviour without a wasm build. Threads: see [threads under `jaic run`](interpreter-threads.md).

### fork

Compile-time code may `fork()`; the `Process` module does, to run commands. If the child comes back to the compiler instead of `exec`ing (exec failed, or it trapped on a foreign procedure the host lacks), `Interp::call` ends it with `_exit` rather than letting it carry on as a second compiler. `call_foreign` sets the flag when a native `fork` returns 0.

### Native foreign calls

Every call in `native.rs` goes through one C prototype with 8 integer and 8 `double` register arguments; the callee ignores extras. Arguments are classified with `jaic::abi`, shared with `jaic-llvm`:

- scalars and register-sized struct pieces fill the integer and float registers in order;
- larger structs are copied and passed by pointer (`Indirect`, arm64);
- the return shape comes from the classification (`II`, `IF`, `FI`, `FF`, `FFF`, `FFFF`, or a 512-byte `Sret` buffer) and is copied to the IR out-pointer.

The prototype ends with 16 eight-byte stack slots, filled in order with what doesn't fit in registers: arguments past the registers, aggregates that no longer fit (all or nothing; on arm64 that also closes their register class, per AAPCS64), x86-64 `byval` structs over 16 bytes, and on Apple arm64 the variadic arguments of a C variadic call (those after `Sig::c_fixed`). x86-64 has only six integer registers (five with a hidden result pointer), so there the prototype's last integer parameters double as its first stack slots (`Regs::prototype`), and the prototype is declared variadic so the caller sets `al`.

Limits: about 18 stack words. On Apple arm64 a stack argument smaller than 8 bytes is an error, because that ABI packs them.

### Callbacks from C

`native/callbacks.rs`: a `#c_call` procedure passed to a foreign procedure is replaced by a thunk, a real C function with the same prototype. It unpacks its arguments the way `call` packs them, runs the procedure through the suspended interpreter (`Reenter`, kept in a thread-local during the native call), and returns in the shape the C caller expects. There is one family of 64 thunks per return shape, assigned per procedure on first use. A trap inside a callback can't unwind through C, so it prints the error and exits.

Not covered: callbacks stored in memory before the call (only arguments are translated), calls from other native threads, variadic callbacks, and on arm64 callbacks returning a struct through a hidden pointer.

### macOS main thread

`native::main_thread`: `jaic` parks the process main thread in `serve` and runs the compiler on a 1 GiB worker. After the first Objective-C or AppKit call (`note_symbol`), the program's foreign calls go to the main thread, because AppKit and GL contexts need it. `fork` always runs on the worker. Calls into libSystem and libc++ (`needs_main_thread`, decided once per address with `dladdr`) stay on the worker: they're thread-safe, and the round trip made allocation-heavy programs like Focus twice as slow.

### Performance notes

- `run` takes value registers from `val_pool` instead of allocating per call; `Call` and `Intrinsic` gather up to 8 operands on the Rust stack (`gather`).
- Hooks, `Stack_Trace_Procedure_Info` addresses and foreign symbols are `Vec`s indexed by id, and results come back as `Rets` (up to four inline), so a call does no hashing or allocation.
- `frame` lays out a procedure's slots once, each at a multiple of its alignment (at least 8), and records the largest alignment; `exec` rounds the frame's absolute start up to it. Rounding offsets alone isn't enough because the stack itself is only 8-aligned, and an `#align 64` local would land on an arbitrary 16-byte boundary.

## How to change it

- New return shape or calling convention for native calls: `call_as` and the shape structs in `native.rs`, plus the matching `Ret` impl and thunk family in `native/callbacks.rs`. The two must stay mirror images.
- To test x86-64 paths on an arm64 Mac, build an interpreter-only `jaic` (`cargo build -p jaic-cli --no-default-features --target x86_64-apple-darwin`, no LLVM needed) and run it with `arch -x86_64`.
- New sandbox foreign procedures: a match arm in `sandbox.rs`. Arguments arrive as raw `u64`s (host addresses, doubles as bits); report errors through `set_errno` and return -1.
- New compiler primitives: a `MetaOp` in `build.rs` and a bodiless `#compiler` declaration in `stdlib/Compiler`.

## Configuration

- `STACK_SIZE` (32 MiB) and `MAX_DEPTH` (20,000 frames) in `interp/mod.rs`.
- `JAIC_PROFILE=1` makes `exec` count calls, blocks and instructions per procedure (`interp/profile.rs`), and the CLI prints the totals. See [benchmarks](../tools/benchmarks.md).

## Dependencies

`ir`, `build.rs` (compiler hooks), and the platform's dynamic loader for native foreign calls.
