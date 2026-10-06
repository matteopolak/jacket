# jaic interpreter

## What it is

`crates/jaic/src/interp` runs IR: `#run` and other compile-time code, `jaic run` programs, and the browser playground. Memory is real host memory. Foreign procedures are called natively (`native.rs`), or through a `Host` shim where there is no dynamic linker (wasm).

## How it works

`Interp::call` is the entry from the compiler; `exec` sets up a frame and `run_code` (`interp/code.rs`) runs the body. Procedure values are tagged addresses (`FUNC_TAG`). Foreign procedures without a native address are tagged `FOREIGN_TAG` and trap with `foreign procedure '...' is not available here` when called.

`#compiler` procedures of the `Compiler` module are hooks (`Hook`, `run_hook`) handled by `MetaOp` in `build.rs`. `codes` mirrors the compiler's `Code` values so `compiler_get_nodes` can export them, and `made_codes` lists codes created by `compiler_get_code`; see [compiler records](../metaprogramming/compiler-records.md).

### The interpreter's code form

The IR is shaped for LLVM: every local lives in a stack slot, so a statement is mostly `SlotAddr` + `Load`/`Store`, and every constant is its own `IConst`. Interpreting that directly costs one dispatch per instruction. On a procedure's first call, `code::build` turns each block into a run of 16-byte `Op`s, cached in the procedure's `Frame`:

| IR | Op |
|---|---|
| `SlotAddr` (or a slot plus a constant) feeding a `Load`/`Store` | `LoadFrame`/`StoreFrame` with the frame offset; no null check, since frame memory is always valid |
| `PtrAdd p, base, IConst` feeding a `Load`/`Store` later in the same block | `Load`/`Store` of `base` + `off` |
| `IConst` that fits 32 bits as an operand | `AddImm`, `MulImm`, `BinImm`, `CmpImm`, `StoreImm`, `StoreFrameImm` |
| 64-bit `Add`/`Sub`/`Mul`, `PtrAdd` | `Add`/`Sub`/`Mul`, which need no masking |
| `Cmp` read only by the block's `Branch` | a `CmpBranch` terminator; a branch on a constant becomes a jump |
| two `Loc`s in a row | the second |
| `Call`, `Intrinsic`, oversized `Copy`/`Zero` | `Ir`, which runs the original instruction through `step` |

Definitions whose results nobody reads any more (the folded constants and slot addresses) are dropped. Folding moves a register read later than the IR has it, so it is only done for values with one definition (SSA, which the IR builder produces): constants and frame addresses anywhere, other values only within the block that defined them, where no definition can run again in between.

Registers are accessed unchecked: `build` asserts every register an op names is below `func.vals.len()`, and `run` sizes the register file to exactly that. The `Frame` records a fingerprint of the IR it was made from (blocks pointer and counts) and is rebuilt when a procedure body is replaced.

`JAIC_PROFILE` keeps counting IR instructions, so instruction counts stay comparable across interpreter changes; a second line counts the ops that actually ran, by kind.

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

Windows x64 (`native/callbacks/win64.rs`) can't use a Rust prototype: Microsoft x64 assigns argument *positions*, so argument 1 is in RDX or XMM1 depending on the callee's own declaration. Its thunks are 256 assembly stubs (16 bytes apart, each loading its index into EAX) that jump to one shared routine. That routine spills RCX/RDX/R8/R9 into the caller's 32-byte home area, which sits right below the stack arguments, so slot `n` of the call is simply `home[n]`, and spills XMM0-XMM3 into its own frame. It then calls `dispatch`, which reads each parameter from the slot or XMM register its position and type select, and returns words for both RAX and XMM0. Aggregates of 1, 2, 4 or 8 bytes arrive in a slot; others arrive as a pointer. A result that isn't 1, 2, 4 or 8 bytes is written through the hidden pointer in slot 0, which is also returned in RAX. The routine has SEH unwind information (`.seh_stackalloc`), so Windows stack walks through it work.

Not covered: callbacks stored in memory before the call (only arguments are translated), calls from other native threads, variadic callbacks, and on arm64 callbacks returning a struct through a hidden pointer.

### macOS main thread

`native::main_thread`: `jaic` parks the process main thread in `serve` and runs the compiler on a 1 GiB worker. After the first Objective-C or AppKit call (`note_symbol`), the program's foreign calls go to the main thread, because AppKit and GL contexts need it. `fork` always runs on the worker. Calls into libSystem and libc++ (`needs_main_thread`, decided once per address with `dladdr`) stay on the worker: they're thread-safe, and the round trip made allocation-heavy programs like Focus twice as slow.

### Performance notes

- `run` takes value registers from `val_pool` instead of allocating per call; `Call` and `Intrinsic` gather up to 8 operands on the Rust stack (`gather`).
- Hooks, `Stack_Trace_Procedure_Info` addresses and foreign symbols are `Vec`s indexed by id, and results come back as `Rets` (up to four inline), so a call does no hashing or allocation.
- `frame` lays out a procedure's slots once, each at a multiple of its alignment (at least 8), and records the largest alignment; `exec` rounds the frame's absolute start up to it. Rounding offsets alone isn't enough because the stack itself is only 8-aligned, and an `#align 64` local would land on an arbitrary 16-byte boundary.

**The program's own path under `jaic run`.** A program asking where its executable is (`get_path_of_running_executable`, through `_NSGetExecutablePath`, `readlink("/proc/self/exe")` or `GetModuleFileNameW(null)`) would get `jaic`'s path, so data it finds relative to itself (`../assets`) would be looked for next to the compiler. `jaic run` sets `Interp::run_executable` to the executable `jaic build` would write for the same file (`<main file's directory>/<stem>`, `.exe` on Windows), and `interp/executable_path.rs` answers those three calls with it at run time. Compile-time code still gets the compiler's path, and the wasm sandbox is unaffected. Test: `crates/jaic-cli/tests/run_executable.rs`.

## How to change it

- New IR instruction: add it to `inst_vals` in `code.rs` (definitions and uses; a missing use would let a definition it reads be dropped), then either translate it to an `Op` or let it fall back to `Op::Ir`, which needs only an arm in `step`.
- New op or fold: keep `Op` at 16 bytes (a compile-time assert checks it). An op that folds a register operand away must call `fold` on it, one that reads a register `build` did not count must add to `uses`, and an op that may be dropped when unread must have no side effect and list its operands in `op_srcs`.
- New return shape or calling convention for native calls: `call_as` and the shape structs in `native.rs`, plus the matching `Ret` impl and thunk family in `native/callbacks.rs`. The two must stay mirror images. On Windows x64 the pair is `windows::call` and `callbacks/win64.rs`. To check the stubs without a Windows machine, build the crate for `x86_64-pc-windows-msvc` (`cargo build -p jaic --target x86_64-pc-windows-msvc`; no linker needed for an rlib) and look at the object with `llvm-objdump -d` and `llvm-readobj --unwind`.
- To test x86-64 paths on an arm64 Mac, build an interpreter-only `jaic` (`cargo build -p jaic-cli --no-default-features --target x86_64-apple-darwin`, no LLVM needed) and run it with `arch -x86_64`.
- New sandbox foreign procedures: a match arm in `sandbox.rs`. Arguments arrive as raw `u64`s (host addresses, doubles as bits); report errors through `set_errno` and return -1.
- New compiler primitives: a `MetaOp` in `build.rs` and a bodiless `#compiler` declaration in `stdlib/Compiler`.

## Configuration

- `STACK_SIZE` (32 MiB) and `MAX_DEPTH` (20,000 frames) in `interp/mod.rs`.
- `JAIC_PROFILE=1` makes `exec` count calls, blocks and instructions per procedure (`interp/profile.rs`), and the CLI prints the totals. See [benchmarks](../tools/benchmarks.md).

## Dependencies

`ir`, `build.rs` (compiler hooks), and the platform's dynamic loader for native foreign calls.
