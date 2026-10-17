# WebAssembly target (wasm64)

## What it is

`jaic` can compile a program to a WebAssembly module: LLVM generates wasm64 (Memory64) object code and `wasm-ld` links it. With `-os wasm` the module is a WASI preview 1 command that runs under node, wasmtime, or a browser with a WASI shim. With a bare `wasm64-unknown-unknown` triple the host supplies imports and calls exports directly.

This is separate from the [browser compiler](../browser/playground.md), which is the compiler itself built to wasm32 and *interprets* programs whose target is `OS == .WASM`. `jaifmt.wasm` ([jaifmt](../tools/jaifmt.md#webassembly-build-jaifmtwasm)) is the main user of this target.

```sh
jaic hello.jai -os wasm -exe hello.wasm            # WASI command; Wasi_Runtime is added
node --no-warnings tools/wasi_run.mjs hello.wasm a b  # stdio/env/args, exits with main's status
jaic lib.jai -target wasm64-unknown-unknown -O2 -exe lib.wasm   # no runtime: your host's imports
```

From a metaprogram:

```jai
options.os_target = .WASM;
options.cpu_target = .CUSTOM;
options.llvm_options.target_system_triple = "wasm64-unknown-wasi";   // "wasi": add Wasi_Runtime
options.llvm_options.target_system_features = "+bulk-memory,+sign-ext";
options.additional_linker_arguments = .["-z", "stack-size=1048576"];  // passed to wasm-ld
```

`tests/native/wasm/build_wasi.jai` is a complete example.

## How it works

**Target selection.** `-os wasm` maps to `wasm64-unknown-wasi`; `-target <triple>` is taken as given. A workspace with `os_target = .WASM` uses `llvm_options.target_system_triple`, or `wasm64-unknown-unknown` when empty. wasm32 triples are refused: Jai's `*void`, `s64` counts and `Type_Info` layouts assume 8-byte pointers. `+bulk-memory` is always added, the relocation model is static, and objects go through `wasm-ld` (`crates/jaic-llvm/src/wasm.rs`, `link_wasm`): `wasm-ld -mwasm64 <objects> -o out.wasm [--no-entry] --stack-first -z stack-size=8388608 [--strip-debug] <libraries> <additional_linker_arguments>`. `--no-entry` is passed unless the program exports `_start`.

**Imports and exports** (`crates/jaic-llvm/src/lower.rs`):

- A `#foreign` procedure or `#elsewhere` global becomes a wasm import from the library's name (`#system_library "host_graphics"`), or `env` for no library or `libc`/`c`/`m`/`msvcrt`. If the program itself `#program_export`s a procedure with that symbol's name, the reference binds to it instead. This is how Wasi_Runtime supplies `malloc` or `write` to the stdlib.
- `#program_export` procedures are exported with `memory`; Wasi_Runtime's are the exception (`is_wasi_library`): only `_start` is exported so `wasm-ld` drops unused runtime functions and their WASI imports (hello world imports four WASI calls).
- `#intrinsic "llvm.<name>"` on a bodiless procedure calls that LLVM intrinsic ([intrinsics](../language/intrinsics.md)).
- Every function has `no-builtins` (otherwise LLVM turns the runtime's own loops into calls to `strlen`/`memset`, which recurse). A weak `__multi3` is emitted for 64-bit division by constants. Return addresses are null, the cycle counter reads 0.

**ABI** (`crates/jaic/src/abi.rs`, `Arch::Wasm64`): a struct whose single scalar field is an `s64`, pointer, `float64` or `float32` is passed directly; every other aggregate goes by pointer. Clang passes single small-integer structs as `i32`, so a Clang-built C library taking such a struct by value would disagree.

**Wasi_Runtime** (`stdlib/Extensions/Wasi_Runtime/`, written in Jai) implements what the stdlib's `OS == .WASM` code calls on top of WASI preview 1: `_start` (args, `main`, `proc_exit`), `write`/`read`/`getenv`/`clock_gettime`/`exit`, a size-class heap for `malloc` and friends, `memcpy`/`memset`/`strlen` loops, file and directory calls (`files.jai`), process stubs failing with `ENOSYS`/`ECHILD` (`module.jai`), green threads (`threads.jai`), libm (`math.jai`, `elementary.jai`), and binary128 helpers for `Long_Double` (`quad.jai`). jaic adds `#import "Extensions/Wasi_Runtime"` when the triple has a component starting with `wasi` (`jaic::build::wants_wasi_runtime`). With bare `wasm64-unknown-unknown` nothing is added; Runtime_Support's needs are `env` imports the host provides (`tests/native/wasm/exports.mjs`).

- WASI preview 1 is defined for 32-bit memories, so pointers are `u32` offsets; that works while the stack and heap stay under 4 GiB.
- **Files**: WASI has no global file system; the host pre-opens directories. `files.jai` makes each C path absolute against its working directory (`$PWD` when a pre-open covers it, else `/`), normalizes `.`/`..` lexically and hands it to the pre-open whose name is its longest prefix; otherwise `ENOENT`. Results use Linux x86-64 `stat`/`dirent` layouts. There are no modes or owners.
- **Math**: `fma`, `fmod` and the binary128 helpers round once from a wide exact accumulator; transcendentals work in float64 pairs and round once (`sin`/`cos`/`tan` reduce with 2/pi to 1344 bits).
- The sandbox's `OS == .WASM` code is unaffected: the same C names are answered by `SandboxHost` in the interpreter and browser, and bind to Wasi_Runtime's exports in a native wasm build, so no stdlib code distinguishes them.
- **Running**: `tools/wasi_run.mjs` runs a command with node's WASI, exits with the module's status (134 on a trap, 127 naming every missing import). `--unknown-imports-trap` resolves missing function imports to trapping functions so a WebGPU program runs and finds no WebGPU. On a Windows host `/` is the root of the current drive, so an argument or `PWD` naming a path on that drive (`D:\a\b`) reaches the module as `/a/b`; paths a program compiled in (`#filepath`) keep their drive letter and cannot be opened.

### Threads in WASI builds (green threads)

WASI preview 1 has no threads and wasm cannot switch stacks, so a program using the Thread module gets green threads: Wasi_Runtime runs them one at a time on the host thread, and the LLVM backend rewrites procedures that may block so a thread can unwind to the scheduler and resume later. Only for native WASI builds; `jaic -interp -os wasm` and the browser schedule threads themselves ([interpreter](../compiler/interpreter.md#threads)).

**Scheduling** (`threads.jai`): `pthread_create` adds a thread with a 1 MiB frame stack that first runs when the creator blocks. Mutexes are recursive (owner and depth in the first 16 bytes); condition waits release the mutex and block on the condition's address; `nanosleep`, `usleep` and `poll` with no descriptors are timed waits. `_start` calls `run_threads`, which runs `main` as the first thread, round robin over runnable threads, sleeping until the earliest timed wait when none are runnable. When every thread waits without a time limit, the one that blocked last wakes with `EDEADLK`. `main` returning exits the process.

**Unwinding** (`crates/jaic-llvm/src/green.rs`, the technique of Binaryen's Asyncify applied to jaic's IR). A blocking call ends in `block_running`, which calls `green_suspend`; that sets `green.state` to unwinding, and each rewritten procedure on the way back sees it after its call, stores which call it was in, and returns. To resume, the scheduler sets the state to rewinding and calls the entry again; each procedure takes the same frame back and jumps to the call it left. `green::instrument` runs before LLVM lowering on a copy of the program, only when the program has the `jaic_wasi_*` exports and reaches `pthread_create`:

1. Procedures that may block: `green_suspend` and, to a fixed point, every caller. An indirect call may block when an address-taken procedure with the same lowered signature may.
2. Each block and each may-block call starts a stretch; a value used in a different stretch than its definition gets an 8-byte place in the frame.
3. Slots stop being allocas: the entry block takes a frame from `green.top`, traps ("thread stack overflow") past `green.limit`, and lays out `[resume point][slots][stored values]`. A normal return frees it; an unwinding return keeps it.
4. After a may-block call, if the state is unwinding the procedure stores the call's index and returns zeros; on entry, if rewinding, a `Switch` on that index jumps to the call.

Debug info keeps lines and scopes for rewritten procedures but drops their variables.

## How to change it

- **New libc entry point** used by stdlib code under `OS == .WASM`: add a `#program_export` with the C name to Wasi_Runtime and keep `SandboxHost` answering it too. A symbol neither defines becomes an `env` import and the module fails to instantiate (`wasm link error`); this also happens for a libcall Wasi_Runtime lacks (`sinh`, `cbrt`, `__powitf2`). For libm or compiler-rt functions, `fp128` arguments arrive as two `u64`s (low first) and results go through a pointer passed first.
- **New WASI call**: declare it `#foreign wasi "<name>"` with `u32` offsets.
- **Linker flags**: `link_wasm` in `wasm.rs`; per-program flags in `additional_linker_arguments`. **wasm-ld discovery**: `find_llvm_tool`; update versioned names when LLVM's major version changes ([LLVM setup](../tools/llvm-setup.md)). **ABI**: `classify_arg`/`wasm_single_scalar` in `abi.rs`.
- **New blocking call in Wasi_Runtime**: block with `block_running(object, deadline)` and wake with `wake_one`/`wake_all`; the analysis finds it. Keep a path for `!green.enabled`. The scheduler and `_start` are not rewritten, so never block below an unrewritten frame; `Program::check_failed` must not block.
- `Green_Control`'s layout is shared: offsets are constants in `green.rs` (`TOP`, `LIMIT`, `ENABLED`, state at 0) and the struct in `threads.jai`. Frame stack sizes: `MAIN_FRAME_BYTES` (8 MiB) and `THREAD_FRAME_BYTES` (about 1 MiB).
- Gotchas: nothing preempts, so a busy wait never lets another thread run. A `#c_call` procedure that may block and takes a struct by value gets a pointer into the wasm stack that is gone after a resume. Rewritten procedures keep locals in memory, so they are slower. Under a runtime that pre-opens nothing (wasmtime without `--dir`) every path call fails with `ENOENT`. Sockets and processes are unavailable.
- **Tests**: `crates/jaic-cli/tests/wasm_target/` builds hello, `tests/native/wasm/program.jai` at `-O0` and `-O2`, the metaprogram build, jaifmt.wasm and every corpus case and runs them under node; `threads-*`, `atomics-contention` and `file-async` stdlib tests run in `wasm-native` mode. `python3 tools/jaic-diff.py --backends interp,wasm-native corpus gen:1:200` compares the interpreter with WASI builds ([testing](../tools/testing.md)).

## Configuration

| Setting | Effect |
| --- | --- |
| `-os wasm` | triple `wasm64-unknown-wasi`, Wasi_Runtime added |
| `-target wasm64-unknown-unknown` | no runtime; the host supplies imports |
| `-no_debug_info` | module linked with `--strip-debug` |
| `JAIC_WASM_LD` | path of `wasm-ld`; otherwise `$LLVM_SYS_231_PREFIX/bin`, `llvm-config --bindir`, Homebrew `lld`/`lld@23`, `/usr/lib/llvm-23/bin`, then `PATH` |
| `JAIC_REQUIRE_WASM_TESTS=1` | wasm tests fail instead of skipping when node or wasm-ld is missing (CI sets it) |

Install LLD with `brew install lld` or Debian's `lld-23`. Runtimes need Memory64: Node.js 24, Chrome/Edge 133, Firefox 134, Deno 2.2, Wasmtime 30; Safari only in Technology Preview 251 and later.

## Dependencies

LLVM 23 with the WebAssembly target, LLD's `wasm-ld`, node 24 for tests and `tools/wasi_run.mjs`. Internal: `crates/jaic-llvm` (`lower.rs`, `wasm.rs`, `green.rs`), `crates/jaic/src/abi.rs`, `crates/jaic/src/build.rs`, `stdlib/Extensions/Wasi_Runtime`, and the `Runtime_Support` entry points it calls.
