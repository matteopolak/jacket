# jaic interpreter

## What it is

`crates/jaic/src/interp` runs IR: `#run` and other compile-time code, `jaic run` programs, and the browser
playground. Memory is real host memory; foreign procedures are called natively (`native.rs`) or through a
`Host` shim where there is no dynamic linker (wasm).

## How it works

- **Traps**: `debug_break()` stops the program with a runtime error (exit 1), so a failed `assert` ends it
  as in Jai. Runtime Support's `debug_break` is `#asm { int3; }` on x64 and `#bytes` `brk #0` on arm64; sema
  turns both (and only those `#bytes` encodings) into the `DebugBreak` intrinsic, since raw machine code
  cannot run in the interpreter.
- **Array bounds checks**: indexing a fixed array, view, dynamic array or string emits the `BoundsCheck`
  intrinsic (index, count), which traps with the index and count (`jaic-llvm` branches to `llvm.trap`).
  Sema skips it inside `#no_abc` procedures, `for ... #no_abc` loops, `#no_abc { }` blocks and the bodies of
  `while`/`if` headers flagged `#no_abc` (the parser marks those bodies `ast::Block::no_abc`; sema sets
  `FnCtx::no_abc`), and everywhere when a workspace sets `array_bounds_check = .OFF`
  (`Options::array_bounds_check`). Pointer indexing is never checked.
- `Interp::call` is the entry from the compiler; `exec` / `run` / `step` interpret functions. Procedure
  values are tagged addresses (`FUNC_TAG`); foreign procedures without a native address are tagged
  `FOREIGN_TAG` and trap with "foreign procedure '...' is not available here" when called.
- `Host::foreign` may implement a foreign symbol itself (libc shims in `SandboxHost`, used in the browser).
  `SandboxHost` (`interp/sandbox.rs`) is the browser's operating system: virtual clock, an in-memory file system
  over a read-only base (`FileSystem::list_dir`; `FILE*`, descriptors, `DIR*`, `stat`, `getcwd`, `/tmp`),
  `localtime_r` (UTC), `strtod`/`strtol` family, `sysconf` (one CPU), and the heap. `jaic run -os wasm file.jai`
  runs a program on it natively, which is the fast way to debug browser-only behavior without a wasm build.
  Threads: see [interpreter-threads.md](interpreter-threads.md).
- `#compiler` procedures of the `Compiler` module are hooks (`Hook`, `run_hook`) handled in `build.rs`
  (`MetaOp`).
- `codes` mirrors the compiler's `Code` values (AST and source text) so `compiler_get_nodes` can export them;
  `made_codes` lists codes compile-time code created (`compiler_get_code`), adopted by the compiler later.
- **fork**: compile-time code may `fork()` (the `Process` module does, to run commands). If the child comes
  back to the compiler instead of `exec`ing (exec failed, or it trapped, e.g. on a foreign procedure that the
  host lacks), `Interp::call` ends the child with `_exit` instead of letting it carry on as a second
  compiler. The flag is set when a native `fork` returns 0 in `call_foreign`.
- **Native foreign calls** (`native.rs`): every call goes through one C prototype with 8 integer and 8
  `double` register arguments (the extra registers are ignored by the callee). Arguments are classified with
  `jaic::abi` (shared with `jaic-llvm`): scalars and register-sized struct pieces fill the integer/float files
  in order, larger structs are copied and passed by pointer (`Indirect`, arm64). The return shape is chosen
  from the classification (`II`, `IF`, `FI`, `FF`, `FFF`, `FFFF`, or a 512-byte `Sret` buffer) and copied to
  the IR out-pointer. The prototype ends with 16 8-byte stack slots, filled in argument order with what
  does not fit the registers: arguments past the registers, aggregates that no longer fit (all-or-nothing;
  on arm64 their register class is then closed, as AAPCS64 says), x86-64 `byval` structs (larger than
  16 bytes), and on Apple arm64 the variadic arguments of a C variadic call (`Sig::c_varargs`, those after
  `Sig::c_fixed`). x86-64 has only six integer registers (five with a hidden result pointer), so there the
  prototype's last integer parameters are its first stack slots (`Regs::prototype`), and the prototype is
  declared variadic so the caller sets `al` for variadic callees. Limits: about 18 stack words, and on
  Apple arm64 a stack argument smaller than 8 bytes (that ABI packs them) is an error.
- **Callbacks from C** (`native/callbacks.rs`): a `#c_call` procedure value passed to a foreign
  procedure is replaced by a thunk, a real C function with the same prototype that unpacks its arguments
  the way `call` packs them, runs the procedure through the suspended interpreter (`Reenter`, kept in a
  thread-local for the duration of the native call) and returns its result in the shape the C caller
  expects. There is one family of 64 thunks per return shape; slots are assigned per procedure on first
  use. A trap inside a callback cannot unwind through C, so it prints the error and exits. Not covered:
  callbacks stored in memory before the call (only arguments are translated), calls from other native
  threads, variadic callbacks, and on arm64 callbacks that return a struct through a hidden pointer.

- **macOS main thread** (`native::main_thread`): `jaic` parks the process main thread in `serve` and runs the
  compiler on a 1 GiB worker. After the first Objective-C/AppKit call (`note_symbol`), the program's foreign
  calls are handed to the main thread (AppKit and GL contexts need it); `fork` always runs on the worker.
  Calls into libSystem and libc++ (`needs_main_thread`, decided once per address with `dladdr`) stay on the
  worker: they are thread-safe, and the round trip made allocation-heavy programs (Focus) twice as slow.
- **Speed**: `run` takes its value registers from `val_pool` instead of allocating per call, and `Call` /
  `Intrinsic` gather up to 8 operands on the Rust stack (`gather`). `block_budget` (when set) counts basic
  blocks and traps with "execution budget exhausted"; the language server and playground use it.
  Hooks are a `Vec` indexed by `FuncId`, checked on every call.
- **Profile** (`interp/profile.rs`): with `JAIC_PROFILE=1`, `exec` counts calls, blocks and instructions
  per procedure (self counts), and the CLI prints the totals. See [benchmarks](../tools/benchmarks.md).

## How to change it

- New return shape or calling convention for native calls: `call_as` and the shape structs in `native.rs`,
  and the matching `Ret` impl and thunk family in `native/callbacks.rs` (the two must stay mirror images).
- Testing the x86-64 paths on an arm64 Mac: build an interpreter-only `jaic` (`cargo build -p jaic-cli
  --no-default-features --target x86_64-apple-darwin`, no LLVM needed) and run it with `arch -x86_64`.
- New host-provided foreign procedures: `Host::foreign` implementations (`SandboxHost` for the browser). Add
  a match arm in `sandbox.rs`; arguments arrive as raw `u64`s (pointers are host addresses, doubles are bits)
  and errors are reported through `set_errno` (returns -1).
- New compiler primitives: a `MetaOp` in `build.rs` plus a bodiless `#compiler` declaration in
  `stdlib/Compiler/records.jai`.

## Configuration

`STACK_SIZE` (32 MiB interpreter stack) and `MAX_DEPTH` (20,000 frames) in `interp/mod.rs`. `JAIC_PROFILE`
enables the per-procedure profile.

## Dependencies

`ir` (programs), `build.rs` (compiler hooks), the platform's dynamic loader for native foreign calls.
